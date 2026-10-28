import assert from 'node:assert/strict';
import {AsyncLocalStorage} from 'node:async_hooks';
import {readFileSync} from 'node:fs';
import {createRequire} from 'node:module';
import {pathToFileURL} from 'node:url';
import test from 'node:test';
import {compileModule} from 'svelte/compiler';
import {createLinguiniRune} from '../dist/index.js';

const require=createRequire(import.meta.url);
const runtime={locales:['en','fr'],baseLocale:'en',normalizeLocale(value){const locale=typeof value==='string'?value.toLowerCase():'';return this.locales.includes(locale)?locale:undefined;},getTextDirection:()=> 'ltr'};
let nonce=0;
async function compiledState(generate, fixture = '') {
  const source=readFileSync(new URL('../dist/state.svelte.js',import.meta.url),'utf8') + '\n' + fixture;
  const result=compileModule(source,{filename:'state.svelte.js',generate,dev:false});
  assert.deepEqual(result.warnings,[]);
  const code=result.js.code.replace(/from (["'])svelte\/internal\/(client|server)\1/g,(_all,_quote,target)=>`from ${JSON.stringify(pathToFileURL(require.resolve('svelte/internal/'+target)).href)}`);
  return import('data:text/javascript;base64,'+Buffer.from(code+'\n// instance '+nonce++).toString('base64'));
}

test('real Svelte server compilation reads each request and never publishes client overrides', async()=>{
  const {createLocaleState}=await compiledState('server');
  const requests=new AsyncLocalStorage();
  const state=createLocaleState(runtime,{getRequestLocale:()=>requests.getStore(),getRequestIdentity(){throw Error('request identity is client-only');}});
  const values=await Promise.all(['fr','en'].map(locale=>requests.run(locale,async()=>{
    state.setCurrentLocale(locale==='fr'?'en':'fr');
    await new Promise(resolve=>setTimeout(resolve,locale==='fr'?5:1));
    return state.getCurrentLocale();
  })));
  assert.deepEqual(values,['fr','en']);
  assert.equal(state.getCurrentLocale(),'en');
});

test('real Svelte client compilation preserves preparation, loader snapshots, disposal, and failure', async context=>{
  const {createLocaleState}=await compiledState('client');
  const previous=Object.getOwnPropertyDescriptor(globalThis,'window');
  Object.defineProperty(globalThis,'window',{configurable:true,value:{}});
  context.after(()=>{if(previous)Object.defineProperty(globalThis,'window',previous);else delete globalThis.window;});
  let requestLocale='fr';
  const state=createLocaleState(runtime,{getRequestLocale:()=>requestLocale});
  assert.equal(state.getCurrentLocale(),'fr');
  state.setCurrentLocale('en');
  assert.equal(state.getCurrentLocale(),'en');
  requestLocale='fr';
  state.clearCurrentLocaleOverride();
  assert.equal(state.getCurrentLocale(),'fr');
  state.initializeCurrentLocale('en');
  const seen=[];
  let second=()=>{},late=()=>{};
  const first=state.registerLocaleLoader(locale=>{seen.push('first:'+locale);second();late=state.registerLocaleLoader(next=>seen.push('late:'+next));});
  second=state.registerLocaleLoader(locale=>{seen.push('second:'+locale);});
  assert.equal(await state.prepareLocale('FR'),'fr');
  assert.deepEqual(seen,['first:fr','second:fr']);
  first();first();second();second();
  assert.equal(await state.prepareLocale('unknown'),'en');
  assert.deepEqual(seen,['first:fr','second:fr','late:en']);
  late();
  const failed=state.registerLocaleLoader(()=>{throw Error('failed preload');});
  await assert.rejects(state.prepareLocale('fr'),/failed preload/);
  failed();
});

test('message providers retain callable messages and wait before publishing locale state',async()=>{
  let locale='en',release;
  const gate=new Promise(resolve=>{release=resolve;});
  const messages={en:{title:'EN',greet:name=>'Hello '+name},fr:{title:'FR',greet:name=>'Bonjour '+name}};
  const state={getCurrentLocale:()=>locale,async prepareLocale(value){await gate;return runtime.normalizeLocale(value)??'en';},setCurrentLocale(value){locale=value;return locale;}};
  const provider=createLinguiniRune({...runtime,createLinguiniProvider({getLocale}){return new Proxy({},{get(_target,key){return messages[getLocale()][key];}});}},state);
  assert.equal(provider.l,provider.messages);
  assert.equal(provider.l.greet('Ada'),'Hello Ada');
  const switching=provider.setLocale('FR');
  assert.equal(provider.locale,'en');
  release();
  assert.equal(await switching,'fr');
  assert.equal(provider.l.title,'FR');
  assert.equal(provider.messages.greet('Ada'),'Bonjour Ada');
});

test('request publication invalidates overrides reactively and the same client locale can be selected again', async context => {
  const {createLocaleState,requestState,observe} = await compiledState('client', `
export function requestState(initial) {
  let value = $state.raw(initial);
  return {get:() => value,set(next) {value=next;}};
}
export function observe(read,report) {
  return $effect.root(() => {$effect(() => report(read()));});
}
`);
  const previous = Object.getOwnPropertyDescriptor(globalThis,'window');
  Object.defineProperty(globalThis,'window',{configurable:true,value:{}});
  context.after(() => {if (previous) Object.defineProperty(globalThis,'window',previous); else delete globalThis.window;});
  const {flush} = await import(pathToFileURL(require.resolve('svelte/internal/client')).href);
  const request = requestState({locale:'fr'});
  let identityReads = 0;
  const state = createLocaleState(runtime,{getRequestLocale:() => request.get().locale,getRequestIdentity:() => {identityReads++;return request.get();}});
  const seen = [];
  let stop;
  flush(() => {stop = observe(state.getCurrentLocale,value => seen.push(value));});
  context.after(() => stop());
  assert.deepEqual(seen,['fr']);
  assert.equal(identityReads,0,'inactive overrides must not read request identity');
  flush(() => state.setCurrentLocale('en'));
  assert.deepEqual(seen,['fr','en']);
  flush(() => request.set({locale:'fr'}));
  assert.deepEqual(seen,['fr','en','fr']);
  flush(() => state.setCurrentLocale('en'));
  assert.deepEqual(seen,['fr','en','fr','en']);
  flush(() => request.set({locale:'en'}));
  assert.equal(state.getCurrentLocale(),'en');
  flush(() => request.set({locale:'fr'}));
  assert.equal(seen.at(-1),'fr');
  flush(() => state.setCurrentLocale('en'));
  const beforeClear = identityReads;
  flush(() => state.clearCurrentLocaleOverride());
  assert.equal(seen.at(-1),'fr');
  assert.equal(identityReads,beforeClear);
});
