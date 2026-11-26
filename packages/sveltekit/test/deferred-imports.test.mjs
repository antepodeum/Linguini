import assert from 'node:assert/strict';
import {mkdtemp,writeFile,mkdir,rm} from 'node:fs/promises';
import {tmpdir} from 'node:os';
import path from 'node:path';
import {pathToFileURL} from 'node:url';
import {createRequire} from 'node:module';
import test from 'node:test';
import {build} from 'vite';
import {clientEntry} from '../dist/build/client-entry.js';
import {deferredImports} from '../dist/build/deferred-imports.js';

test('late deferred preparation preserves shaking, evaluation order, cached imports, and live namespaces', async context => {
  const root = await mkdtemp(path.join(tmpdir(),'linguini-kit-deferred-'));
  context.after(() => rm(root,{recursive:true,force:true}));
  context.after(() => {delete globalThis.__linguiniDeferredTest;});
  const directory = path.join(root,'generated/client-optimized');
  await mkdir(directory,{recursive:true});
  await writeFile(path.join(root,'package.json'),' {"type":"module"}');
  const application = path.join(directory,'app.js');
  const target = path.join(directory,'lazy.js');
  await writeFile(target,`globalThis.__linguiniDeferredTest.events.push('evaluate:' + globalThis.__linguiniDeferredTest.message);
    export const render = () => globalThis.__linguiniDeferredTest.message;
    export const debug = () => 'FORBIDDEN';
    export let counter = 0; export function bump(){ counter++; }`);
  for (const namespace of [false,true]) {
    const state = {locale:'fr',events:[],message:undefined,wait:Promise.resolve()};
    globalThis.__linguiniDeferredTest = state;
    let plan;
    await writeFile(application,namespace
      ? 'export function action(){return import("./lazy.js");}'
      : 'export async function action(){const {render}=await import("./lazy.js");return render();}');
    const result = await build({
      configFile:false,root,logLevel:'silent',
      plugins:[clientEntry(async () => {
        plan = {pages:{},deferred:{[target]:{'/':{fr:{file:'assets/messages/group/fr.js',messages:['title']}}}}};
        return `export const __linguiniLocale = () => globalThis.__linguiniDeferredTest.locale;
          export async function __linguiniPrepare(routes,getLocale){
            const state = globalThis.__linguiniDeferredTest;
            state.events.push('prepare:' + getLocale());
            state.directory = routes['/'];
            await state.wait;
            state.message = getLocale().toUpperCase();
          }`;
      }),deferredImports(() => plan)],
      build:{write:true,minify:true,sourcemap:true,manifest:true,
        rollupOptions:{input:{app:application},preserveEntrySignatures:'strict'}},
    });
    const chunks = result.output.filter(item => item.type === 'chunk');
    assert.equal(chunks.length,2,'preparation must use the app entry without an additional helper chunk');
    if (!namespace) for (const chunk of chunks) assert.doesNotMatch(chunk.code,/FORBIDDEN/);
    const entry = chunks.find(chunk => chunk.isEntry);
    const module = await import(pathToFileURL(path.join(root,'dist',entry.fileName)).href);
    assert.deepEqual(state.events,[],'initial module loading must not prepare or evaluate a lazy action');
    let release;
    state.wait = new Promise(resolve => {release=resolve;});
    const loading = module.action();
    await new Promise(resolve => setImmediate(resolve));
    assert.deepEqual(state.events,['prepare:fr']);
    release();
    const first = await loading;
    assert.deepEqual(state.events,['prepare:fr','evaluate:FR']);
    assert.equal(state.directory,pathToFileURL(path.join(root,'dist/assets/messages/group/')).href);
    state.locale = 'en';
    state.wait = Promise.resolve();
    const second = await module.action();
    assert.deepEqual(state.events,['prepare:fr','evaluate:FR','prepare:en']);
    if (namespace) {
      assert.equal(first,second,'native import namespace identity must survive preparation');
      first.bump();
      assert.equal(second.counter,1,'prepared imports must preserve live export bindings');
      assert.equal(second.render(),'EN');
    } else {
      assert.equal(first,'FR');
      assert.equal(second,'EN');
    }
  }
});

test('nested imports prepare before evaluation through message-bearing or transit-only parents without crossing framework routes', async context => {
  const root = await mkdtemp(path.join(tmpdir(),'linguini-kit-nested-'));
  context.after(() => rm(root,{recursive:true,force:true}));
  context.after(() => {delete globalThis.__linguiniNestedTest;});
  const directory = path.join(root,'generated/client-optimized');
  await mkdir(directory,{recursive:true});
  await writeFile(path.join(root,'package.json'),'{"type":"module"}');
  const application = path.join(directory,'app.js');
  const owner = path.join(directory,'owner.js');
  const unrelated = path.join(directory,'unrelated.js');
  const parent = path.join(directory,'parent.js');
  const child = path.join(directory,'child.js');
  await writeFile(application,'export {route} from "./router.js";');
  await writeFile(path.join(directory,'router.js'),`export const tag = () => globalThis.__linguiniNestedTest.locale;
    export const route = () => import('./owner.js');`);
  await writeFile(owner,`import {tag} from './router.js';
    export async function action(){tag();const {next}=await import('./parent.js');return next;}`);
  await writeFile(unrelated,'export {tag} from "./router.js";');
  await writeFile(parent,`globalThis.__linguiniNestedTest.events.push('evaluate-parent:' + globalThis.__linguiniNestedTest.parent);
    export async function next(){const {render}=await import('./child.js');return render();}`);
  await writeFile(child,`globalThis.__linguiniNestedTest.events.push('evaluate-child:' + globalThis.__linguiniNestedTest.child);
    export const render = () => globalThis.__linguiniNestedTest.parent + '/' + globalThis.__linguiniNestedTest.child;`);
  for (const hasParentMessages of [false,true]) {
    const outDir = `dist/${hasParentMessages ? 1 : 0}`;
    const state = {locale:'fr',events:[],parent:hasParentMessages ? undefined : 'INITIAL',child:undefined};
    globalThis.__linguiniNestedTest = state;
    const payload = group => ({'/owner':{fr:{file:`assets/messages/${group}/fr.js`,messages:[group]}}});
    const plan = {pages:{},deferred:{[child]:payload('child'),...(hasParentMessages ? {[parent]:payload('parent')} : {})},
      deferredImports:{'/owner':[parent,child],'/unrelated':[]}};
    const result = await build({
      configFile:false,root,logLevel:'silent',
      plugins:[clientEntry(async () => `export const __linguiniLocale = () => globalThis.__linguiniNestedTest.locale;
        export async function __linguiniPrepare(routes,getLocale){
          const state=globalThis.__linguiniNestedTest;
          const group=routes['/owner'].includes('/parent/') ? 'parent' : 'child';
          state.events.push('prepare-' + group + ':' + getLocale());
          state[group]=(group + '_' + getLocale()).toUpperCase();
        }`),deferredImports(() => plan,undefined,() => [
          {id:'/owner',entries:[owner]},{id:'/unrelated',entries:[unrelated]},
        ])],
      build:{write:true,outDir,minify:true,manifest:true,sourcemap:true,modulePreload:false,
        rollupOptions:{input:{app:application,owner,unrelated},preserveEntrySignatures:'strict'}},
    });
    const chunk = result.output.find(item => item.type === 'chunk' && item.name === 'owner');
    assert.ok(chunk);
    const module = await import(pathToFileURL(path.join(root,outDir,chunk.fileName)).href);
    assert.deepEqual(state.events,[]);
    const next = await module.action();
    const prefix = hasParentMessages ? ['prepare-parent:fr','evaluate-parent:PARENT_FR'] : ['evaluate-parent:INITIAL'];
    assert.deepEqual(state.events,prefix,'loading the parent must not prepare or evaluate the child');
    assert.equal(await next(),(hasParentMessages ? 'PARENT_FR' : 'INITIAL') + '/CHILD_FR');
    assert.deepEqual(state.events,[...prefix,'prepare-child:fr','evaluate-child:CHILD_FR']);
    state.locale = 'en';
    const again = await module.action();
    assert.equal(again,next,'preparation must preserve the cached native function binding');
    assert.equal(await again(),(hasParentMessages ? 'PARENT_EN' : 'INITIAL') + '/CHILD_EN');
    assert.equal(state.events.filter(event => event.startsWith('evaluate-child')).length,1);
    assert.equal(state.events.filter(event => event.startsWith('evaluate-parent')).length,1);
  }
});

test('unsupported deferred ownership fails instead of widening page delivery', async context => {
  const root = await mkdtemp(path.join(tmpdir(),'linguini-kit-deferred-guard-'));
  context.after(() => rm(root,{recursive:true,force:true}));
  const directory = path.join(root,'generated/client-optimized');
  await mkdir(directory,{recursive:true});
  const application = path.join(directory,'app.js');
  const target = path.join(directory,'lazy.js');
  await writeFile(target,'export const render = () => "lazy message";');
  const plan = {pages:{},deferred:{[target]:{'/owner':{en:{file:'assets/messages/group/en.js',messages:['title']}}}}};
  const capability = async () => 'export const __linguiniLocale = () => "en"; export async function __linguiniPrepare() {}';
  await writeFile(application,'export {render} from "./lazy.js"; export async function action(){const {render}=await import("./lazy.js");return render();}');
  await assert.rejects(build({
    configFile:false,root,logLevel:'silent',
    plugins:[clientEntry(capability),deferredImports(() => plan)],
    build:{write:false,rollupOptions:{input:{app:application},preserveEntrySignatures:'strict'}},
  }),/cannot yet prepare deferred messages through an inlined import/);

  const owner = path.join(directory,'owner.js');
  const unrelated = path.join(directory,'unrelated.js');
  await writeFile(application,'export const application = true;');
  await writeFile(owner,'export {action} from "./shared.js";');
  await writeFile(unrelated,'export {action} from "./shared.js";');
  await writeFile(path.join(directory,'shared.js'),'export async function action(){const {render}=await import("./lazy.js");return render();}');
  await assert.rejects(build({
    configFile:false,root,logLevel:'silent',
    plugins:[clientEntry(capability),deferredImports(() => plan,undefined,() => [
      {id:'/owner',entries:[owner]}, {id:'/unrelated',entries:[unrelated]},
    ])],
    build:{write:false,rollupOptions:{input:{app:application,owner,unrelated},preserveEntrySignatures:'strict'}},
  }),/caller shared with an unrelated page/);
});

test('export variants share only their new payload after parent preparation', async context=>{
  const root=await mkdtemp(path.join(tmpdir(),'linguini-kit-parent-variants-'));
  context.after(()=>rm(root,{recursive:true,force:true}));
  context.after(()=>{delete globalThis.__linguiniVariantsTest;});
  await writeFile(path.join(root,'package.json'),'{"type":"module"}');
  const directory=path.join(root,'generated/client-optimized');
  await mkdir(directory,{recursive:true});
  const application=path.join(directory,'app.js');
  const owner=path.join(directory,'owner.js');
  const parent=path.join(directory,'parent.js');
  const child=path.join(directory,'child.js');
  const virtual=name=>'virtual:linguini/message/'+Buffer.from(name).toString('hex');
  await writeFile(application,"export const route=()=>import('./owner.js');");
  await writeFile(owner,"export async function parent(){const {render,getOne,getTwo}=await import('./parent.js');return {render,getOne,getTwo};}");
  await writeFile(parent,`import {message as a} from ${JSON.stringify(virtual('a'))}; import {message as b} from ${JSON.stringify(virtual('b'))};
    globalThis.__linguiniVariantsTest.events.push('parent:'+a()+'/'+b());
    export const render=()=>a()+'/'+b();
    export async function getOne(){const {one}=await import('./child.js');return one;}
    export async function getTwo(){const {two}=await import('./child.js');return two;}`);
  await writeFile(child,`import {message as a} from ${JSON.stringify(virtual('a'))}; import {message as b} from ${JSON.stringify(virtual('b'))}; import {message as c} from ${JSON.stringify(virtual('c'))};
    globalThis.__linguiniVariantsTest.events.push('child:'+c());
    let calls=0;
    export const one=()=>a()+'/'+c()+':'+(++calls);
    export const two=()=>b()+'/'+c()+':'+(++calls);
    export {message as debug} from ${JSON.stringify(virtual('unused'))};`);
  const messages=new Map();
  for (const name of ['a','b','c','unused']) {
    const locales=new Map();
    for (const locale of ['en','fr']) {
      const file=path.join(root,`${name}-${locale}.js`);
      await writeFile(file,`export const message=()=>${JSON.stringify(name==='unused'?'FORBIDDEN_PAYLOAD':name+':'+locale)};`);
      locales.set(locale,{module:file});
    }
    messages.set(name,{locales});
  }
  const ids=new Map([...messages.keys()].sort().map((name,index)=>[name,index]));
  const viteEntry=pathToFileURL(createRequire(import.meta.url).resolve('@antepodeum/linguini-vite'));
  const {compilePageBundles}=await import(new URL('./page-payloads.js',viteEntry));
  const pages=[{id:'/owner',entries:[owner]}];
  let plan;
  const result=await build({configFile:false,root,logLevel:'silent',plugins:[{
    name:'variant-message-facades',
    resolveId(id){if(id.startsWith('virtual:'))return '\0'+id;},
    load(id){
      if(!id.startsWith('\0virtual:linguini/message/'))return;
      const name=Buffer.from(id.split('/').at(-1),'hex').toString('utf8');
      if(name==='unused')return 'export function message(){throw Error("FORBIDDEN_FACADE");}';
      return `export function message(){const state=globalThis.__linguiniVariantsTest;const value=state.messages.get(state.locale)?.[${ids.get(name)}];if(!value)throw Error('unprepared ${name}');return value();}`;
    },
  },clientEntry(async compiler=>{
    plan=await compilePageBundles(compiler,pages,{messages,baseLocale:'en',effectiveLocales:['en','fr']},
      {root,build:{target:'es2022',minify:true,assetsDir:'assets'}});
    return `export const __linguiniLocale=()=>globalThis.__linguiniVariantsTest.locale;
      const pending=new Map();
      export async function __linguiniPrepare(routes,getLocale){
        const locale=getLocale(),href=routes['/owner']+locale+'.js';
        if(!pending.has(href)){
          const state=globalThis.__linguiniVariantsTest;
          state.requests.push(href);
          pending.set(href,import(/* @vite-ignore */href).then(({messages})=>{
            const current=state.messages.get(locale)??{};
            Object.assign(current,messages);state.messages.set(locale,current);
          }));
        }
        await pending.get(href);
      }`;
  }),deferredImports(()=>plan,undefined,()=>pages)],
    build:{write:true,minify:true,manifest:true,sourcemap:true,modulePreload:false,
      rollupOptions:{input:{app:application,owner},preserveEntrySignatures:'strict'}},
  });
  assert.deepEqual(Object.keys(plan.pages),[],'no initial message demand means no initial locale payload');
  for(const locale of ['fr','en']){
    assert.deepEqual(plan.deferred[parent]['/owner'][locale].messages,['a','b']);
    assert.deepEqual(plan.deferred[child]['/owner'][locale].messages,['c']);
  }
  assert.equal(result.output.filter(item=>item.type==='asset'&&/^assets\/linguini\/.*\.js$/.test(item.fileName)).length,4);
  for(const chunk of result.output.filter(item=>item.type==='chunk'))assert.doesNotMatch(chunk.code,/FORBIDDEN_FACADE|FORBIDDEN_PAYLOAD/);
  const state={locale:'fr',events:[],requests:[],messages:new Map()};
  globalThis.__linguiniVariantsTest=state;
  const entry=result.output.find(item=>item.type==='chunk'&&item.facadeModuleId===owner);
  const module=await import(pathToFileURL(path.join(root,'dist',entry.fileName)).href);
  assert.deepEqual(state.requests,[]);
  assert.deepEqual(state.events,[]);
  const first=await module.parent();
  assert.equal(first.render(),'a:fr/b:fr');
  assert.deepEqual(state.events,['parent:a:fr/b:fr']);
  assert.equal(state.messages.get('fr')[ids.get('c')],undefined,'parent preparation must not include the child demand');
  const one=await first.getOne(),two=await first.getTwo();
  assert.equal(one(),'a:fr/c:fr:1');
  assert.equal(two(),'b:fr/c:fr:2');
  assert.equal(await first.getOne(),one,'repeated imports retain the native binding');
  state.locale='en';
  const next=await module.parent();
  assert.equal(next.render,first.render);
  assert.equal(next.render(),'a:en/b:en');
  assert.equal(state.messages.get('en')[ids.get('c')],undefined);
  assert.equal(await next.getOne(),one);
  assert.equal(one(),'a:en/c:en:3','export variants retain shared module state after locale changes');
  assert.deepEqual(state.events,['parent:a:fr/b:fr','child:c:fr'],'module initialization runs once');
  const expected=[];
  for(const locale of ['fr','en'])for(const target of [parent,child]){
    expected.push(pathToFileURL(path.join(root,'dist',plan.deferred[target]['/owner'][locale].file)).href);
  }
  assert.deepEqual(state.requests,expected,'one parent and one additional payload per selected locale, with cached repeated variants');
});
