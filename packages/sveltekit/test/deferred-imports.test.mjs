import assert from 'node:assert/strict';
import {mkdtemp,writeFile,mkdir,rm} from 'node:fs/promises';
import {tmpdir} from 'node:os';
import path from 'node:path';
import {pathToFileURL} from 'node:url';
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
