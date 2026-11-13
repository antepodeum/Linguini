import assert from 'node:assert/strict';
import {fileURLToPath} from 'node:url';
import test from 'node:test';
import {build} from 'vite';

test('the Kit message provider automatically scopes direct locale changes to its incoming request', async context => {
  const entry = '\0kit-provider-test';
  const client = fileURLToPath(new URL('../dist/client.js',import.meta.url));
  const result = await build({
    configFile:false,logLevel:'silent',ssr:{noExternal:true},
    plugins:[{
      name:'kit-provider-host',enforce:'pre',
      resolveId(id) {if (id === entry || id.startsWith('$app/')) return id;},
      load(id) {
        if (id === entry) return `export {createLinguiniRune} from ${JSON.stringify(client)}; export {page} from '$app/state';`;
        if (id === '$app/state') return 'export const page = {data:{linguini:{locale:"en"}}};';
        if (id === '$app/environment') return 'export const browser = true;';
        if (id === '$app/navigation') return 'export async function goto() {}';
      },
    }],
    build:{ssr:true,write:false,target:'es2022',rollupOptions:{input:entry,preserveEntrySignatures:'strict'}},
  });
  const chunks = result.output.filter(item => item.type === 'chunk');
  assert.equal(chunks.length,1);
  const module = await import('data:text/javascript;base64,' + Buffer.from(chunks[0].code).toString('base64'));
  const previous = Object.getOwnPropertyDescriptor(globalThis,'window');
  Object.defineProperty(globalThis,'window',{configurable:true,value:{}});
  context.after(() => {if (previous) Object.defineProperty(globalThis,'window',previous); else delete globalThis.window;});
  let release;
  const published = [];
  const state = {
    getCurrentLocale:()=>module.page.data.linguini.locale,
    prepareLocale:locale=>new Promise(resolve=>{release=()=>resolve(locale);}),
    setCurrentLocale(locale) {published.push(locale);return locale;},
  };
  const runtime = {createLinguiniProvider:()=>({}),getTextDirection:()=> 'ltr'};
  const provider = module.createLinguiniRune(runtime,state);
  const change = provider.setLocale('fr');
  module.page.data.linguini = {locale:'en'};
  release();
  assert.equal(await change,'en');
  assert.deepEqual(published,[],'even a same-locale request replacement must invalidate older preparation');
});
