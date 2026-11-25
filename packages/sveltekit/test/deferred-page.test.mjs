import assert from 'node:assert/strict';
import {mkdtemp,mkdir,writeFile,rm} from 'node:fs/promises';
import {tmpdir} from 'node:os';
import path from 'node:path';
import {fileURLToPath,pathToFileURL} from 'node:url';
import test from 'node:test';
import {build} from 'vite';

test('deferred records deduplicate preparation, follow locale changes, and stay scoped to their request', async context => {
  const root = await mkdtemp(path.join(tmpdir(),'linguini-deferred-records-'));
  context.after(() => rm(root,{recursive:true,force:true}));
  context.after(() => {delete globalThis.__linguiniPayloadTest;});
  await writeFile(path.join(root,'package.json'),'{"type":"module"}');
  const source = fileURLToPath(new URL('../dist/deferred-page.svelte.js',import.meta.url));
  const loaderSource = fileURLToPath(new URL('../dist/load.js',import.meta.url));
  const entry = '\0deferred-page-test';
  const result = await build({
    configFile:false,logLevel:'silent',ssr:{noExternal:true},
    plugins:[{
      name:'deferred-page-host',enforce:'pre',
      resolveId(id) {if (id === entry || id.startsWith('$app/')) return id;},
      load(id) {
        if (id === entry) return `export * from ${JSON.stringify(source)}; export {createClientLoad,registerPagePreparation} from ${JSON.stringify(loaderSource)}; export {page} from '$app/state';`;
        if (id === '$app/state') return 'export const page = {data:{},route:{id:"/lab"}};';
        if (id === '$app/environment') return 'export const browser = true;';
      },
    }],
    build:{ssr:true,write:false,target:'es2022',rollupOptions:{input:entry,preserveEntrySignatures:'strict'}},
  });
  const chunks = result.output.filter(item => item.type === 'chunk');
  assert.equal(chunks.length,1);
  const module = await import('data:text/javascript;base64,' + Buffer.from(chunks[0].code).toString('base64'));
  const control = {events:[],gates:new Map(),notify:undefined};
  globalThis.__linguiniPayloadTest = control;
  let locale = 'fr';
  const getLocale = () => locale;
  function newRequest(route = '/lab') {
    locale = 'fr';
    module.page.data = {linguini:{locale},linguiniMessages:{0:'INITIAL'}};
    module.page.route = {id:route};
  }
  async function payload(directory,language,text) {
    await mkdir(directory,{recursive:true});
    const tag = path.basename(directory) + ':' + language;
    await writeFile(path.join(directory,language + '.js'),
      `const control = globalThis.__linguiniPayloadTest; control.events.push(${JSON.stringify(tag)});
       control.notify?.(${JSON.stringify(tag)}); await control.gates.get(${JSON.stringify(tag)});
       export const messages = {1:${JSON.stringify(text)},2:()=>${JSON.stringify(text)}};`);
    return tag;
  }
  function started(tag) {
    return new Promise(resolve => {control.notify = value => {if (value === tag) resolve();};});
  }
  const routes = directory => ({'/lab':pathToFileURL(directory + path.sep).href});

  newRequest();
  const first = path.join(root,'first');
  const firstTag = await payload(first,'fr','FIRST_FR');
  await payload(first,'en','FIRST_EN');
  let release;
  control.gates.set(firstTag,new Promise(resolve => {release=resolve;}));
  const evaluating = started(firstTag);
  const one = module.prepareDeferredPageMessages(routes(first),getLocale);
  const two = module.prepareDeferredPageMessages(routes(first),getLocale);
  await evaluating;
  assert.throws(() => module.getDeferredPageMessage(1,'fr'),/not prepared/);
  release();
  await Promise.all([one,two]);
  assert.equal(control.events.filter(tag => tag === firstTag).length,1);
  assert.equal(module.getDeferredPageMessage(0,'fr'),'INITIAL');
  assert.equal(module.getDeferredPageMessage(1,'fr'),'FIRST_FR');
  const captured = module.getDeferredPageMessage(2,'fr');
  await module.prepareDeferredPageMessages(routes(first),getLocale);
  assert.equal(module.getDeferredPageMessage(2,'fr'),captured);
  await module.prepareDeferredCurrentPageMessages('en');
  locale = 'en';
  assert.equal(module.getDeferredPageMessage(1,'en'),'FIRST_EN');
  assert.deepEqual(control.events,['first:fr','first:en']);

  module.registerPagePreparation(module.prepareDeferredNavigation);
  const load = module.createClientLoad();
  const incoming = {linguini:{locale:'en'}};
  const prepared = await load({data:incoming,route:{id:'/lab'}});
  assert.notEqual(prepared.linguini,incoming.linguini,'every publication needs a fresh identity, even for cached input data');
  assert.equal(module.getDeferredPageMessage(1,'fr'),'FIRST_FR','preparation must leave the current page unchanged');
  module.page.data = prepared;
  assert.equal(module.getDeferredPageMessage(1,'en'),'FIRST_EN');
  await module.prepareDeferredCurrentPageMessages('fr');
  assert.equal(module.getDeferredPageMessage(2,'fr'),captured);
  assert.deepEqual(control.events,['first:fr','first:en']);

  const thirdTag = await payload(first,'de','FIRST_DE');
  control.gates.set(thirdTag,new Promise(resolve => {release=resolve;}));
  const thirdStarted = started(thirdTag);
  const slower = load({data:{linguini:{locale:'de'}},route:{id:'/lab'}});
  await thirdStarted;
  const faster = await load({data:incoming,route:{id:'/lab'}});
  module.page.data = faster;
  release();
  await slower;
  assert.equal(module.page.data,faster,'a late incoming preparation must not publish over another navigation');
  assert.equal(module.getDeferredPageMessage(1,'en'),'FIRST_EN');
  const other = await load({data:{linguini:{locale:'xx'}},route:{id:'/other'}});
  assert.deepEqual(control.events,['first:fr','first:en','first:de'],'another route must not inherit activated lazy payloads');
  module.page.data = other;
  module.page.route = {id:'/other'};
  assert.throws(() => module.getDeferredPageMessage(1,'xx'),/not prepared/);

  newRequest('/other');
  assert.throws(() => module.getDeferredPageMessage(1,'fr'),/not prepared/);
  await module.prepareDeferredCurrentPageMessages('en');
  await assert.rejects(module.prepareDeferredPageMessages(routes(first),getLocale),{name:'AbortError'});
  assert.deepEqual(control.events,['first:fr','first:en','first:de']);

  newRequest();
  const old = path.join(root,'old');
  const oldTag = await payload(old,'fr','OLD_REQUEST');
  control.gates.set(oldTag,new Promise(resolve => {release=resolve;}));
  const oldStarted = started(oldTag);
  const outdated = module.prepareDeferredPageMessages(routes(old),getLocale);
  const rejected = assert.rejects(outdated,{name:'AbortError'});
  await oldStarted;
  newRequest();
  release();
  await rejected;
  assert.throws(() => module.getDeferredPageMessage(1,'fr'),/not prepared/);

  const retry = path.join(root,'retry');
  await assert.rejects(module.prepareDeferredPageMessages(routes(retry),getLocale),/Cannot find module/);
  await module.prepareDeferredCurrentPageMessages('en');
  await payload(retry,'fr','RETRIED');
  await module.prepareDeferredPageMessages(routes(retry),getLocale);
  assert.equal(module.getDeferredPageMessage(1,'fr'),'RETRIED');

  newRequest();
  const changing = path.join(root,'changing');
  const changingTag = await payload(changing,'fr','CHANGING_FR');
  await payload(changing,'en','CHANGING_EN');
  control.gates.set(changingTag,new Promise(resolve => {release=resolve;}));
  const changingStarted = started(changingTag);
  const changed = module.prepareDeferredPageMessages(routes(changing),getLocale);
  await changingStarted;
  locale = 'en';
  release();
  await changed;
  assert.equal(module.getDeferredPageMessage(1,'en'),'CHANGING_EN');

  newRequest();
  const loader = path.join(root,'loader');
  await payload(loader,'fr','LOADER_FR');
  const loaderTag = await payload(loader,'en','LOADER_EN');
  await module.prepareDeferredPageMessages(routes(loader),getLocale);
  control.gates.set(loaderTag,new Promise(resolve => {release=resolve;}));
  const loaderStarted = started(loaderTag);
  const loading = module.prepareDeferredCurrentPageMessages('en');
  await loaderStarted;
  newRequest();
  release();
  await loading;
  assert.throws(() => module.getDeferredPageMessage(1,'en'),/not prepared/);
});
