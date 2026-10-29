import assert from 'node:assert/strict';
import test from 'node:test';
import {createWebLocaleI18n,pathLocale,cookieLocale,localStorageLocale} from '@antepodeum/linguini-web';
import {createReroute} from '../dist/index.js';
import {createHandle,createLoad} from '../dist/server.js';

const runtime = {locales:['en','fr','ar'],baseLocale:'en',getTextDirection:locale=>locale==='ar'?'rtl':'ltr',
  createLinguini(){throw Error('Hooks must not load message catalogs');},
  prepareLinguini(){throw Error('Hooks must not prepare message catalogs');},
};
const environment = {base:'/app',origin:'https://example.test'};
function event(path, headers={}) {
  const url = new URL(path,environment.origin);
  return {url,request:new Request(url,{headers}),locals:{}};
}

test('hooks retain request-local metadata across concurrent rendering without message or browser access', async () => {
  const web = createWebLocaleI18n(runtime,{routing:{localePrefix:'always'},locale:{sources:[pathLocale()]}},environment);
  const handle = createHandle(web);
  const events = [event('/app/fr/page'),event('/app/ar/page')];
  const responses = await Promise.all(events.map(event => handle({event,async resolve(request,{transformPageChunk}) {
    await new Promise(resolve=>setTimeout(resolve,request.locals.linguini.locale==='fr'?5:1));
    const html = '<html lang="%linguini.lang%" dir="%linguini.dir%">%linguini.locale%</html>';
    let rendered='';
    for(let index=0;index<html.length;index++) rendered+=transformPageChunk({html:html[index],done:index===html.length-1});
    return new Response(rendered);
  }})));
  assert.equal(await responses[0].text(),'<html lang="fr" dir="ltr">fr</html>');
  assert.equal(await responses[1].text(),'<html lang="ar" dir="rtl">ar</html>');
  const load=createLoad();
  assert.equal(load(events[0]).linguini.locale,'fr');
  assert.equal(load(events[1]).linguini.locale,'ar');
  assert.deepEqual(Object.keys(JSON.parse(JSON.stringify(load(events[0]).linguini))).sort(),['baseLocale','direction','htmlAttrs','lang','locale','locales']);
  assert.equal(load({locals:{}}).linguini,undefined);
  const reroute=createReroute(web);
  assert.equal(reroute(events[0]),'/app/page');
});

test('application hook configuration controls redirects, exclusions, and cookie persistence', async () => {
  const web=createWebLocaleI18n(runtime,{routing:{localePrefix:'always'},locale:{sources:[cookieLocale({name:'LANG',httpOnly:true})]},routes:{exclude:['/api/**']}},environment);
  let resolved=0;
  const resolve=async()=>{resolved++;return new Response('normal');};
  const handle=createHandle(web);
  const redirect=await handle({event:event('/app/page',{cookie:'LANG=fr'}),resolve});
  assert.equal(redirect.status,307);
  assert.equal(redirect.headers.get('location'),'/app/fr/page');
  assert.match(redirect.headers.get('set-cookie'),/^LANG=fr;/);
  assert.match(redirect.headers.get('set-cookie'),/HttpOnly/);
  assert.equal(resolved,0);
  const excluded=event('/api/health');
  assert.equal(await (await handle({event:excluded,resolve})).text(),'normal');
  assert.equal(excluded.locals.linguini,undefined);
  const pathless=createWebLocaleI18n(runtime,{routing:{localePrefix:'never'},locale:{sources:[cookieLocale({name:'LANG'})]}},environment);
  const immutable=await fetch('data:text/plain,streamed');
  const response=await createHandle(pathless)({event:event('/app/page',{cookie:'LANG=ar'}),resolve:async()=>immutable});
  assert.equal(await response.text(),'streamed');
  assert.match(response.headers.get('set-cookie'),/^LANG=ar;/);
  const untouched=new Response('plain');
  assert.equal(await createHandle(pathless,{persistCookie:false})({event:event('/app/page'),resolve:async()=>untouched}),untouched);
});

test('server switching is optional runtime policy and storage-only resolution is rejected', async () => {
  assert.throws(()=>createHandle(createWebLocaleI18n(runtime,{locale:{sources:[localStorageLocale()]}})),/server-readable/);
  const web=createWebLocaleI18n(runtime,{routing:{localePrefix:'never'},locale:{sources:[cookieLocale({name:'LANG'})]}},environment);
  const handle=createHandle(web,{switchRoute:{path:'/locale/{locale}'}});
  const response=await handle({event:event('/app/locale/fr?return=/app/page'),resolve(){throw Error('switch must bypass rendering');}});
  assert.equal(response.status,303);
  assert.equal(response.headers.get('location'),'/app/page');
  assert.match(response.headers.get('set-cookie'),/^LANG=fr;/);
});

test('localized canonical URLs leave trailing-slash normalization to SvelteKit', async () => {
  const web = createWebLocaleI18n(runtime, {routing:{localePrefix:'always'}, locale:{sources:[pathLocale()]}}, environment);
  const handle = createHandle(web);
  for (const path of ['/app/fr/', '/app/fr/shop/', '/app/fr/shop', '/app/fr/shop/?q=1']) {
    let rendered = false;
    const response = await handle({event:event(path), resolve:async () => {rendered = true; return new Response('rendered');}});
    assert.equal(rendered, true, path);
    assert.equal(response.status, 200, path);
  }
  const response = await handle({event:event('/app/shop/?q=1'), resolve:async () => {throw Error('missing prefix must redirect');}});
  assert.equal(response.status, 307);
  assert.equal(response.headers.get('location'), '/app/en/shop?q=1');
});

test('page payload selection is request-local metadata and navigation data does not add HTML preloads', async () => {
  const web = createWebLocaleI18n(runtime,{routing:{localePrefix:'always'},locale:{sources:[pathLocale()]}},environment);
  const selected = [];
  const getPagePayload = async input => {
    selected.push(input);
    return {url:`/app/_app/immutable/assets/linguini/page/${input.locale}.js`};
  };
  const handle = createHandle(web,{getPagePayload});
  for (const isDataRequest of [false,true]) {
    const request = {...event('/app/fr/page'),route:{id:'/page'},isDataRequest};
    const response = await handle({event:request,resolve:async (event,{transformPageChunk}) => {
      const load = createLoad({getPagePayload});
      const data = await load(event);
      assert.deepEqual(data.linguini.payload,{url:'/app/_app/immutable/assets/linguini/page/fr.js'});
      return new Response(transformPageChunk({html:'<head><link rel="modulepreload" href="/app/_app/immutable/entry/start.js"></head>',done:true}));
    }});
    const html = await response.text();
    assert.equal(html.includes('/linguini/page/fr.js'),!isDataRequest);
    assert.doesNotMatch(html,/\/en\.js|\/ar\.js/);
  }
  assert.deepEqual(selected,[
    {routeId:'/page',locale:'fr'},
    {routeId:'/page',locale:'fr'},
  ], 'handle and load share one selection per request');
});
