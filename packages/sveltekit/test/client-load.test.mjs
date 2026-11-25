import assert from 'node:assert/strict';
import {mkdtemp,mkdir,writeFile,rm} from 'node:fs/promises';
import {tmpdir} from 'node:os';
import path from 'node:path';
import {fileURLToPath,pathToFileURL} from 'node:url';
import test from 'node:test';
import {build} from 'vite';
import {createRequire} from 'node:module';
import {clientEntry} from '../dist/build/client-entry.js';
import {clientLoad,CLIENT_LOAD,CLIENT_LOAD_MODEL} from '../dist/build/client-load.js';

test('build-selected loaders omit unused deferred work and preserve initial, enhanced, and server behavior', {timeout:30_000}, async context => {
  const root = await mkdtemp(path.join(tmpdir(),'linguini-kit-load-'));
  context.after(() => rm(root,{recursive:true,force:true}));
  const directory = path.join(root,'generated/client-optimized');
  await mkdir(directory,{recursive:true});
  await writeFile(path.join(root,'package.json'),'{"type":"module"}');
  const application = path.join(directory,'app.js');
  await writeFile(application,'export {createClientLoad} from "@antepodeum/linguini-sveltekit/load";');
  const payload = path.join(root,'locale.js');
  await writeFile(payload,'export const messages = {1:"PREPARED"};');
  const initial = fileURLToPath(new URL('../dist/load.js',import.meta.url));
  const enhanced = fileURLToPath(new URL('../dist/deferred-load.js',import.meta.url));
  async function compile(enabled,ssr,select) {
    let release;
    const planned = new Promise(resolve => {release=resolve;});
    const result = await build({
      configFile:false,root,logLevel:'silent',
      plugins:[...(select ? [clientLoad(() => planned),clientEntry(async () => {
        release(enabled);
        return enabled ? `import {registerPagePreparation} from ${JSON.stringify(enhanced)};
          registerPagePreparation(async (context,route) => ({...context,preparedRoute:route}));` : 'export {};';
      })] : []),{
        name:'loader-host',
        resolveId(id) {
          if (id === '$app/environment') return '\0' + id;
          if (id === '@antepodeum/linguini-sveltekit/load') return initial;
        },
        load(id) {if (id === '\0$app/environment') return `export const browser=${!ssr};`;},
      }],
      build:{ssr,write:false,minify:false,manifest:true,rollupOptions:{input:{app:application},preserveEntrySignatures:'strict'}},
    });
    const chunks = result.output.filter(item => item.type === 'chunk');
    assert.equal(chunks.length,1);
    assert.deepEqual(chunks[0].imports,[]);
    return chunks[0];
  }
  const reference = await compile(false,false,false);
  for (const [enabled,ssr] of [[false,false],[true,false],[true,true]]) {
    const chunk = await compile(enabled,ssr,true);
    const capability = enabled && !ssr;
    assert.equal(Object.hasOwn(chunk.modules,enhanced),capability);
    if (!enabled) assert.equal(chunk.code,reference.code,'a project without additional messages must emit the unchanged base loader');
    if (!capability) assert.doesNotMatch(chunk.code,/preparation|preparedRoute|registerPagePreparation/);
    const module = await import('data:text/javascript;base64,' + Buffer.from(chunk.code).toString('base64'));
    const load = module.createClientLoad();
    let reads = 0;
    const metadata = {locale:'en',payload:{url:pathToFileURL(payload).href}};
    const data = {get linguini(){reads++;return metadata;},other:42};
    const prepared = await load({data,route:{id:'/lab'}});
    assert.equal(reads,ssr ? 1 : 2,'preparation must read request metadata once, plus the returned data snapshot');
    assert.equal(prepared.other,42);
    assert.equal(prepared.linguini.preparedRoute,capability ? '/lab' : undefined);
    assert.equal(prepared.linguiniMessages?.[1],ssr ? undefined : 'PREPARED');
    if (!capability) assert.equal(prepared.linguini,metadata);
    const empty = {other:7};
    assert.equal(await load({data:empty}),empty);
  }
});

test('pending loader analysis preserves message demand in application load wrappers', {timeout:30_000}, async context => {
  const root = await mkdtemp(path.join(tmpdir(),'linguini-kit-load-demand-'));
  context.after(() => rm(root,{recursive:true,force:true}));
  const directory = path.join(root,'generated/client-optimized');
  await mkdir(directory,{recursive:true});
  const application = path.join(directory,'app.js');
  await writeFile(application,`import {createClientLoad} from '@antepodeum/linguini-sveltekit/load';
    import {message} from 'virtual:linguini/message/7265717569726564';
    const initial = createClientLoad();
    export async function load(event) {
      const pending = initial(event);
      if (!pending) return {};
      await pending;
      return {title:message()};
    }`);
  const core = createRequire(import.meta.url).resolve('@antepodeum/linguini-vite');
  const {pageMessageUsage} = await import(pathToFileURL(path.join(path.dirname(core),'page-demand.js')).href);
  let release;
  const ready = new Promise(resolve => {release=resolve;});
  let usage;
  await build({
    configFile:false,root,logLevel:'silent',
    plugins:[clientLoad(() => ready),clientEntry(async context => {
      usage = await pageMessageUsage(context,[{id:application,exports:[['load']]}],{
        root,build:{target:'es2022'},analysisModule:id => id === CLIENT_LOAD ? CLIENT_LOAD_MODEL : undefined,
      });
      release(false);
      return 'export {};';
    }),{
      name:'loader-demand-host',
      resolveId(id) {if (id.startsWith('virtual:') || id === '$app/environment') return '\0' + id;},
      load(id) {
        if (id.startsWith('\0virtual:')) return 'export function message(){return globalThis.message();}';
        if (id === '\0$app/environment') return 'export const browser = true;';
      },
    }],
    build:{write:false,manifest:true,rollupOptions:{input:{app:application},preserveEntrySignatures:'strict'}},
  });
  assert.deepEqual(usage.messages,['required']);
});
