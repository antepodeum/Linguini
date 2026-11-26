import assert from 'node:assert/strict';
import {mkdtemp,writeFile,rm} from 'node:fs/promises';
import {tmpdir} from 'node:os';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
import test from 'node:test';
import {build} from 'vite';

async function compileWebConsumer(context,source) {
  const root = await mkdtemp(path.join(tmpdir(),'linguini-web-graph-'));
  context.after(() => rm(root,{recursive:true,force:true}));
  const entry = path.join(root,'entry.js');
  await writeFile(entry,source);
  const result = await build({configFile:false,root,logLevel:'silent',
    build:{write:false,minify:true,lib:{entry,formats:['es']},rollupOptions:{preserveEntrySignatures:'strict'}},
  });
  const chunks=(Array.isArray(result)?result:[result]).flatMap(bundle=>bundle.output).filter(item=>item.type==='chunk');
  assert.equal(chunks.length,1);
  assert.deepEqual(chunks[0].imports,[]);
  assert.deepEqual(chunks[0].dynamicImports,[]);
  return chunks[0];
}

test('routing-only web metadata drops optional links, SEO, persistence and transports', async context => {
  const web = fileURLToPath(new URL('../../web/dist/index.js',import.meta.url));
  const chunk = await compileWebConsumer(context, `
import {createWebLocaleI18n,pathLocale} from ${JSON.stringify(web)};
const web=createWebLocaleI18n({baseLocale:'en',locales:['en','fr']},{routing:{localePrefix:'always'},locale:{sources:[pathLocale()]}},{origin:'https://example.test'});
export function resolve(url){return web.resolveLocaleSync({url});}
export function localize(href,locale){return web.localizeHref(href,locale);}
`);
  assert.doesNotMatch(chunk.code,/Max-Age=|HttpOnly|set-cookie|serializeLocaleCookie|setLocaleCookie|document\.cookie|localStorage\.setItem|hreflang/);
  for (const [id,module] of Object.entries(chunk.modules)) {
    if (/(?:\/(?:cookie|client|server|alternate-links|links)\.js|\/sources\/(?:cookie|local-storage|accept-language)\.js)$/.test(id)) {
      assert.equal(module.renderedLength,0,`unexpected routing-only dependency: ${id}`);
    }
  }
  const output=await import('data:text/javascript;base64,'+Buffer.from(chunk.code).toString('base64'));
  assert.equal(output.resolve('https://example.test/fr/page'),'fr');
  assert.equal(output.resolve('https://example.test/unknown/page'),'en');
  assert.equal(output.localize('https://example.test/en/cart?q=1#part','fr'),'https://example.test/fr/cart?q=1#part');
});

test('browser controls retain browser persistence without server cookie sinks', async context => {
  const directory=fileURLToPath(new URL('../../web/dist/',import.meta.url));
  const chunk=await compileWebConsumer(context, `
import {createWebLocaleI18n} from ${JSON.stringify(path.join(directory,'index.js'))};
import {createLocaleControl} from ${JSON.stringify(path.join(directory,'client.js'))};
const web=createWebLocaleI18n({baseLocale:'en',locales:['en','fr']},{routing:{localePrefix:'always'},cookie:{secure:true},locale:{switch:{writesPath:true,writesCookie:true,writesLocalStorage:true}}},{origin:'https://example.test'});
export function controls(state,options){return createLocaleControl(web,state,options);}
`);
  assert.match(chunk.code,/Max-Age=/);
  assert.doesNotMatch(chunk.code,/set-cookie|setLocaleCookie|hreflang/);
  for (const [id,module] of Object.entries(chunk.modules)) {
    if (/(?:server|alternate-links|links)\.js$/.test(id)) assert.equal(module.renderedLength,0,`unexpected browser dependency: ${id}`);
  }
  const output=await import('data:text/javascript;base64,'+Buffer.from(chunk.code).toString('base64'));
  const descriptors=['window','document'].map(key=>[key,Object.getOwnPropertyDescriptor(globalThis,key)]);
  context.after(()=>{for(const [key,descriptor] of descriptors) {
    if(descriptor) Object.defineProperty(globalThis,key,descriptor);else delete globalThis[key];
  }});
  const events=[];
  Object.defineProperty(globalThis,'window',{configurable:true,value:{
    location:{href:'https://example.test/en/page'},
    localStorage:{setItem(key,value){events.push(['storage',key,value]);}},
  }});
  Object.defineProperty(globalThis,'document',{configurable:true,value:{set cookie(value){events.push(['cookie',value]);}}});
  let locale='en';
  const controls=output.controls({
    getCurrentLocale:()=>locale,
    async prepareLocale(value){events.push(['prepare',value]);return value;},
    setCurrentLocale(value){events.push(['publish',value]);locale=value;return value;},
    clearCurrentLocaleOverride(){events.push(['clear']);},
  },{browser:true,navigate:href=>{events.push(['navigate',href]);}});
  assert.equal('alternateLinks' in controls,false);
  for (const name of ['shouldLocalizeHref','shouldLocalizeLink','localizeHrefAttribute','localizeLinkHref']) assert.equal(name in controls,false,name);
  assert.equal(await controls.setLocale('fr'),'fr');
  assert.deepEqual(events.map(event=>event[0]),['prepare','publish','storage','cookie','navigate','clear']);
  assert.deepEqual(events[2],['storage','LINGUINI_LOCALE','fr']);
  assert.match(events[3][1],/^LINGUINI_LOCALE=fr;.*; Secure$/);
  assert.deepEqual(events[4],['navigate','https://example.test/fr/page']);
});

test('metadata-only request hooks shake catalog barrels and unselected resolver modules', async context => {
  const root = await mkdtemp(path.join(tmpdir(),'linguini-metadata-graph-'));
  context.after(() => rm(root,{recursive:true,force:true}));
  const web = fileURLToPath(new URL('../../web/dist/',import.meta.url));
  const server = fileURLToPath(new URL('../dist/server.js',import.meta.url));
  await writeFile(path.join(root,'catalog.js'), 'export const messages={title:"FORBIDDEN_CATALOG_DATA"}; export const createLinguini=()=>messages;');
  await writeFile(path.join(root,'runtime.js'), 'export const locales=["en","fr"]; export const baseLocale="en"; export {messages,createLinguini} from "./catalog.js";');
  await writeFile(path.join(root,'entry.js'), `
import {baseLocale,locales} from './runtime.js';
import {createWebLocaleI18n,pathLocale} from '@antepodeum/linguini-web';
import {createHandle,createLoad} from ${JSON.stringify(server)};
const web=createWebLocaleI18n({baseLocale,locales},{routing:{localePrefix:'always'},locale:{sources:[pathLocale()]}});
export const handle=createHandle(web);
export const load=createLoad();
`);
  const result = await build({configFile:false,root,logLevel:'silent',
    resolve:{alias:[{find:'@antepodeum/linguini-web/server',replacement:path.join(web,'server.js')},
      {find:'@antepodeum/linguini-web',replacement:path.join(web,'index.js')}]},
    ssr:{noExternal:true},
    build:{ssr:path.join(root,'entry.js'),write:false,minify:true,sourcemap:true,
      rollupOptions:{external:[/^virtual:linguini\//,'$app/paths'],preserveEntrySignatures:'strict'}},
  });
  const chunks = result.output.filter(item => item.type === 'chunk');
  assert.equal(chunks.length,1);
  const chunk = chunks[0];
  assert.deepEqual(chunk.imports,[]);
  assert.deepEqual(chunk.dynamicImports,[]);
  assert.doesNotMatch(chunk.code,/FORBIDDEN_CATALOG_DATA|createLinguini|prepareLinguini|hreflang/);
  for (const [id, module] of Object.entries(chunk.modules)) {
    if (/(?:\/catalog\.js|\/alternate-links\.js|\/links\.js|\/sources\/(?:cookie|local-storage|accept-language)\.js)$/.test(id)) {
      assert.equal(module.renderedLength,0,`unexpected metadata-hook dependency: ${id}`);
    }
  }
  const output = await import('data:text/javascript;base64,' + Buffer.from(chunk.code).toString('base64'));
  for (const locale of ['fr','en']) {
    const event={url:new URL(`https://example.test/${locale}/page`),request:new Request(`https://example.test/${locale}/page`),locals:{}};
    const response=await output.handle({event,resolve:async (_event,{transformPageChunk}) => new Response(transformPageChunk({html:'<html lang="%linguini.lang%">%linguini.locale%</html>',done:true}))});
    assert.equal(await response.text(),`<html lang="${locale}">${locale}</html>`);
    assert.equal(output.load(event).linguini.locale,locale);
    assert.equal(Object.hasOwn(event.locals.linguini,'messages'),false);
    assert.equal(Object.hasOwn(event.locals.linguini,'alternateLinks'),false);
    assert.equal(Object.hasOwn(event.locals.linguini,'localizeLinkHref'),false);
  }
});

test('explicit alternate links retain only their selected SEO operation', async context => {
  const web=fileURLToPath(new URL('../../web/dist/index.js',import.meta.url));
  const chunk=await compileWebConsumer(context, `
import {createWebLocaleI18n,alternateLinks} from ${JSON.stringify(web)};
const web=createWebLocaleI18n({baseLocale:'en',locales:['en','fr']},{},{origin:'https://example.test'});
export function links(href){return alternateLinks(web,href);}
`);
  const owner=Object.entries(chunk.modules).find(([id])=>id.endsWith('/alternate-links.js'));
  assert.ok(owner?.[1].renderedLength>0,'selected SEO helper must own the implementation');
  assert.match(chunk.code,/hreflang/);
  assert.doesNotMatch(chunk.code,/Max-Age=|HttpOnly|set-cookie|document\.cookie|MutationObserver/);
  const output=await import('data:text/javascript;base64,'+Buffer.from(chunk.code).toString('base64'));
  assert.deepEqual(output.links('/en/shop?q=1#part'),[
    {rel:'alternate',hreflang:'en',href:'https://example.test/shop?q=1#part'},
    {rel:'alternate',hreflang:'fr',href:'https://example.test/fr/shop?q=1#part'},
    {rel:'alternate',hreflang:'x-default',href:'https://example.test/shop?q=1#part'},
  ]);
});

test('selected link localization retains its shared helper without browser traversal', async context => {
  const web=fileURLToPath(new URL('../../web/dist/index.js',import.meta.url));
  const chunk=await compileWebConsumer(context, `
import {createWebLocaleI18n,localizeLinkHref} from ${JSON.stringify(web)};
const web=createWebLocaleI18n({baseLocale:'en',locales:['en','fr']},{routing:{localePrefix:'always'},routes:{exclude:['/private/**']}},{origin:'https://example.test'});
export function localize(href,attributes){return localizeLinkHref(web,href,'fr',attributes);}
`);
  const owner=Object.entries(chunk.modules).find(([id])=>id.endsWith('/links.js'));
  assert.ok(owner?.[1].renderedLength>0,'selected links must retain their shared implementation');
  assert.doesNotMatch(chunk.code,/MutationObserver|document\.cookie|Max-Age=|hreflang/);
  const output=await import('data:text/javascript;base64,'+Buffer.from(chunk.code).toString('base64'));
  assert.equal(output.localize('/en/shop?q=1#part',{}),'/fr/shop?q=1#part');
  for (const href of ['/private/page','https://external.test/page','mailto:a@example.test']) {
    assert.equal(output.localize(href,{}),href);
  }
  assert.equal(output.localize('/en/shop',{download:true}),'/en/shop');
});
