import assert from 'node:assert/strict';
import {readFile,readdir} from 'node:fs/promises';
import {join} from 'node:path';
import {fileURLToPath,pathToFileURL} from 'node:url';
import test from 'node:test';
import {chromium} from 'playwright';
import {serve,createSsrServer} from './browser-host.mjs';

const output = fileURLToPath(new URL('../build/',import.meta.url));
const isPayload = url => url.includes('/assets/linguini/') && url.endsWith('.js');
const nativeCopy = async locale => (await import(`../src/lib/generated/linguini/locales/${locale}/deferred_lab.js`)).deferred_lab;

async function payloadFiles() {
  const {payloads} = await import('../.svelte-kit/output/server/linguini-page-manifest.js');
  const initial = new Set(Object.values(payloads).flatMap(locales => Object.values(locales)));
  const file = payloads['/chunk-lab'].fr;
  const directory = file.slice(0,file.indexOf('/linguini/') + '/linguini/'.length);
  const files = (await readdir(join(output,directory),{recursive:true})).filter(file => file.endsWith('.js'))
    .map(file => directory + file.replaceAll('\\','/')).filter(file => !initial.has(file));
  const manifest = JSON.parse(await readFile(new URL('../src/lib/generated/linguini/bundler/manifest.json',import.meta.url),'utf8'));
  const names = Object.keys(manifest.messages).sort();
  const entries = await Promise.all(files.map(async file => ({file,messages:(await import(pathToFileURL(join(output,file)).href)).messages})));
  const deferred = (locale,nested = false) => {
    const id = names.indexOf('deferred_lab.' + (nested ? 'detail' : 'title'));
    assert.notEqual(id,-1);
    const matches = entries.filter(entry => entry.file.endsWith('/' + encodeURIComponent(locale) + '.js') && Object.hasOwn(entry.messages,id));
    assert.equal(matches.length,1,`expected one ${nested ? 'nested' : 'parent'} payload for ${locale}`);
    return '/' + matches[0].file;
  };
  return {initial:(locale,route='/chunk-lab') => '/' + payloads[route][locale],deferred};
}

test('new lazy messages use one selected payload on action and retain preparation across same-page locale navigation', {timeout:30_000}, async context => {
  const files = await payloadFiles();
  const origin = await serve(context,{preload:files.initial('fr')});
  const manifest = JSON.parse(await readFile(new URL('../.svelte-kit/output/client/.vite/manifest.json',import.meta.url),'utf8'));
  const lazy = '/' + manifest['src/lib/deferred-lab-copy.ts'].file;
  const browser = await chromium.launch();
  context.after(() => browser.close());
  const session = await browser.newContext();
  const page = await session.newPage();
  page.setDefaultTimeout(5000);
  const requests = [];
  const errors = [];
  page.on('request',request => requests.push(new URL(request.url()).pathname));
  page.on('pageerror',error => errors.push(error.message));
  page.on('console',message => {if (/hydration_/i.test(message.text())) errors.push(message.text());});
  const payloads = () => requests.filter(isPayload);
  await page.goto(origin + '/fr/chunk-lab/',{waitUntil:'networkidle'});
  assert.deepEqual(payloads(),[files.initial('fr')]);
  assert.equal(requests.includes(lazy),false,'the lazy application module must not be requested before its action');
  const before = requests.length;
  await page.getByRole('button',{name:'Load additional messages',exact:true}).click();
  const french = await nativeCopy('fr');
  await page.locator('[data-additional-title]').filter({hasText:french.title}).waitFor();
  assert.equal(await page.locator('[data-additional-summary]').textContent(),french.summary({value:1299.5}));
  assert.deepEqual(payloads(),[files.initial('fr'),files.deferred('fr')]);
  assert.deepEqual(requests.slice(before).filter(url => url.endsWith('.js')).sort(),[lazy,files.deferred('fr')].sort(),
    'an action may fetch its application module and one locale file, without a helper request');
  const loaded = requests.length;
  await page.getByRole('button',{name:'Load additional messages',exact:true}).click();
  await page.waitForLoadState('networkidle');
  assert.equal(requests.slice(loaded).filter(url => url.endsWith('.js')).length,0);

  const english = await nativeCopy('en');
  await page.getByRole('link',{name:'EN',exact:true}).click();
  await page.waitForURL('**/en/chunk-lab/');
  await page.locator('[data-additional-title]').filter({hasText:english.title}).waitFor();
  assert.equal(await page.locator('[data-additional-summary]').textContent(),english.summary({value:1299.5}));
  assert.deepEqual(payloads(),[files.initial('fr'),files.deferred('fr'),files.initial('en'),files.deferred('en')]);
  assert.deepEqual(errors,[]);

  async function navigate(href) {
    await page.evaluate(href => {
      const anchor = document.createElement('a');
      anchor.href = href;anchor.setAttribute('data-linguini-no-localize','');
      document.body.append(anchor);anchor.click();anchor.remove();
    },href);
    await page.waitForURL('**' + href);
    await page.waitForLoadState('networkidle');
  }
  await navigate('/ru/');
  assert.equal(payloads().includes(files.deferred('ru')),false,'another page must not inherit activated lazy message sets');
  await navigate('/fr/chunk-lab/');
  assert.equal(await page.locator('[data-additional-title]').count(),0);
  await page.getByRole('link',{name:'RU',exact:true}).click();
  await page.waitForURL('**/ru/chunk-lab/');
  await page.waitForLoadState('networkidle');
  assert.equal(payloads().includes(files.deferred('ru')),false,'a new page instance must wait for its own lazy action');
  const russian = await nativeCopy('ru');
  await page.getByRole('button',{name:'Load additional messages',exact:true}).click();
  await page.locator('[data-additional-title]').filter({hasText:russian.title}).waitFor();
  assert.equal(await page.locator('[data-additional-summary]').textContent(),russian.summary({value:1299.5}));
  assert.equal(payloads().filter(url => url === files.deferred('ru')).length,1);
  assert.equal(requests.filter(url => url === lazy).length,1,'revisiting must reuse the native module cache');
  await navigate('/ru/message-lab/');
  assert.equal(await page.locator('[data-additional-title]').count(),0);
  const beforeShared = requests.length;
  await page.getByRole('button',{name:'Load additional messages',exact:true}).click();
  await page.locator('[data-additional-title]').filter({hasText:russian.title}).waitFor();
  assert.equal(requests.slice(beforeShared).filter(url => url.endsWith('.js')).length,0,
    'another owning page must reuse the same native module, runtime, and prepared locale file');
  assert.deepEqual(errors,[]);
});

test('nested actions fetch only their additional message and reuse parent values and functions through locale changes', {timeout:30_000}, async context => {
  const files = await payloadFiles();
  const origin = await serve(context,{preload:files.initial('fr')});
  const manifest = JSON.parse(await readFile(new URL('../.svelte-kit/output/client/.vite/manifest.json',import.meta.url),'utf8'));
  const child = '/' + manifest['src/lib/deferred-lab-detail.ts'].file;
  const browser = await chromium.launch();
  context.after(() => browser.close());
  const session = await browser.newContext();
  const page = await session.newPage();
  page.setDefaultTimeout(5000);
  const requests = [];
  const errors = [];
  page.on('request',request => requests.push(new URL(request.url()).pathname));
  page.on('pageerror',error => errors.push(error.message));
  page.on('console',message => {if (/hydration_/i.test(message.text())) errors.push(message.text());});
  const payloads = () => requests.filter(isPayload);
  await page.goto(origin + '/fr/chunk-lab/',{waitUntil:'networkidle'});
  await page.getByRole('button',{name:'Load additional messages',exact:true}).click();
  await page.getByRole('button',{name:'Load nested messages',exact:true}).waitFor();
  await page.waitForLoadState('networkidle');
  assert.deepEqual(payloads(),[files.initial('fr'),files.deferred('fr')]);
  assert.equal(requests.includes(child),false,'parent preparation must not fetch the nested application module');
  const before = requests.length;
  await page.getByRole('button',{name:'Load nested messages',exact:true}).click();
  const french = await nativeCopy('fr');
  await page.locator('[data-nested-detail]').filter({hasText:french.detail}).waitFor();
  assert.equal(await page.locator('[data-nested-parent]').textContent(),french.title);
  assert.equal(await page.locator('[data-nested-summary]').textContent(),french.summary({value:1299.5}));
  assert.deepEqual(requests.slice(before).filter(url => url.endsWith('.js')).sort(),[child,files.deferred('fr',true)].sort(),
    'the nested action fetches only its code and one additional locale file');
  const loaded = requests.length;
  await page.getByRole('button',{name:'Load nested messages',exact:true}).click();
  await page.waitForLoadState('networkidle');
  assert.equal(requests.slice(loaded).filter(url => url.endsWith('.js')).length,0);
  await page.getByRole('link',{name:'EN',exact:true}).click();
  await page.waitForURL('**/en/chunk-lab/');
  const english = await nativeCopy('en');
  await page.locator('[data-nested-detail]').filter({hasText:english.detail}).waitFor();
  assert.equal(await page.locator('[data-nested-parent]').textContent(),english.title);
  assert.equal(await page.locator('[data-nested-summary]').textContent(),english.summary({value:1299.5}));
  assert.deepEqual(payloads().sort(),[files.initial('fr'),files.deferred('fr'),files.deferred('fr',true),
    files.initial('en'),files.deferred('en'),files.deferred('en',true)].sort());
  await page.evaluate(() => {
    const anchor = document.createElement('a');
    anchor.href = '/en/message-lab/';anchor.setAttribute('data-linguini-no-localize','');
    document.body.append(anchor);anchor.click();anchor.remove();
  });
  await page.waitForURL('**/en/message-lab/');
  await page.waitForLoadState('networkidle');
  assert.equal(await page.locator('[data-nested-detail]').count(),0,'another page starts without an activated child');
  const beforeShared = requests.length;
  await page.getByRole('button',{name:'Load additional messages',exact:true}).click();
  await page.getByRole('button',{name:'Load nested messages',exact:true}).click();
  await page.locator('[data-nested-detail]').filter({hasText:english.detail}).waitFor();
  assert.equal(await page.locator('[data-nested-summary]').textContent(),english.summary({value:1299.5}));
  assert.equal(requests.slice(beforeShared).filter(url => url.endsWith('.js')).length,0,
    'another owning page reuses both native modules and both prepared locale files');
  assert.deepEqual(errors,[]);
});

test('lazy values and named formatter calls render in concurrent server loads and hydrate without a deferred locale fetch', {timeout:30_000}, async context => {
  const server = await createSsrServer();
  await Promise.all(['fr','ru','en','fr','en','ru'].map(async locale => {
    const response = await server.respond(new Request(`http://localhost/${locale}/chunk-lab/?deferred-ssr&nested-ssr`),{getClientAddress:()=> '127.0.0.1'});
    assert.equal(response.status,200);
    const html = await response.text();
    const native = await nativeCopy(locale);
    for (const value of [native.title,native.summary({value:1299.5}),native.detail]) {
      assert.ok(html.includes(value.replaceAll('&','&amp;').replaceAll('<','&lt;').replaceAll('>','&gt;')),`${locale} SSR omitted ${value}`);
    }
  }));
  const files = await payloadFiles();
  const origin = await serve(context,{ssr:server});
  const browser = await chromium.launch();
  context.after(() => browser.close());
  const french = await nativeCopy('fr');
  for (const javaScriptEnabled of [true,false]) {
    const session = await browser.newContext({javaScriptEnabled});
    const page = await session.newPage();
    const requests = [];
    const errors = [];
    page.on('request',request => requests.push(new URL(request.url()).pathname));
    page.on('pageerror',error => errors.push(error.message));
    page.on('console',message => {if (/hydration_/i.test(message.text())) errors.push(message.text());});
    await page.goto(origin + '/fr/chunk-lab/?deferred-ssr&nested-ssr',{waitUntil:'networkidle'});
    assert.deepEqual(await page.getByRole('region',{name:'Deferred SSR copy'}).locator('p').allTextContents(),[
      french.title,french.summary({value:1299.5}),
    ]);
    assert.deepEqual(await page.getByRole('region',{name:'Nested SSR copy'}).locator('p').allTextContents(),[
      french.detail,french.title,french.summary({value:1299.5}),
    ]);
    assert.deepEqual(requests.filter(isPayload),javaScriptEnabled ? [files.initial('fr')] : []);
    assert.deepEqual(errors,[]);
    await session.close();
  }
});
