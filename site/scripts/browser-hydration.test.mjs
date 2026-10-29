import assert from 'node:assert/strict';
import {createServer} from 'node:http';
import {readFile, stat} from 'node:fs/promises';
import {basename, extname, join, resolve, sep} from 'node:path';
import test from 'node:test';
import {fileURLToPath} from 'node:url';
import {chromium} from 'playwright';

const site = fileURLToPath(new URL('../', import.meta.url));
const output = join(site, 'build');

async function serve(context, {preload} = {}) {
  let releaseScripts;
  const scriptsReady = new Promise(resolve => {releaseScripts=resolve;});
  if (!preload) releaseScripts();
  const types = {'.html':'text/html', '.js':'text/javascript', '.json':'application/json', '.css':'text/css', '.svg':'image/svg+xml', '.woff2':'font/woff2'};
  const server = createServer(async (request, response) => {
    try {
      const pathname = decodeURIComponent(new URL(request.url, 'http://localhost').pathname);
      if (pathname === preload) releaseScripts();
      else if (pathname.endsWith('.js')) await scriptsReady;
      let file = resolve(output, '.' + pathname);
      if (file !== output && !file.startsWith(output + sep)) throw Error('outside output');
      if ((await stat(file)).isDirectory()) file = join(file, 'index.html');
      const body = await readFile(file);
      response.writeHead(200, {'content-type':types[extname(file)] ?? 'application/octet-stream'});
      response.end(body);
    } catch {
      response.writeHead(404);
      response.end('Not found');
    }
  });
  await new Promise(resolve => server.listen(0, '127.0.0.1', resolve));
  context.after(() => new Promise((resolve, reject) => {
    releaseScripts();
    server.close(error => error ? reject(error) : resolve());
    server.closeAllConnections();
  }));
  return `http://127.0.0.1:${server.address().port}`;
}

test('French prerender hydrates its selected locale and loads only its page payload before explicit switching', {timeout:30_000}, async context => {
  const {payloads} = await import('../.svelte-kit/output/server/linguini-page-manifest.js');
  const payload = (locale, route = '/chunk-lab') => '/' + payloads[route][locale];
  // Hold every framework module until the SSR-declared locale preload arrives.
  // An import discovered by client execution cannot satisfy this gate.
  const origin = await serve(context,{preload:payload('fr')});
  const browser = await chromium.launch();
  context.after(() => browser.close());
  const session = await browser.newContext();
  const page = await session.newPage();
  page.setDefaultTimeout(5000);
  page.setDefaultNavigationTimeout(5000);
  const requests = [];
  const errors = [];
  page.on('request', request => requests.push(new URL(request.url()).pathname));
  page.on('pageerror', error => errors.push(error.message));
  page.on('console', message => {
    if (/hydration_/i.test(message.text())) errors.push(message.text());
  });
  const files = new Set(Object.values(payloads).flatMap(locales => Object.values(locales).map(file => '/' + file)));
  const requestedPayloads = () => requests.filter(url => files.has(url));
  const fr = (await import('../src/lib/generated/linguini/locales/fr/main.js')).main.hero.tagline;
  const en = (await import('../src/lib/generated/linguini/locales/en/main.js')).main.hero.tagline;
  assert.notEqual(fr, en, 'the hydration probe must distinguish locales');

  const html = await (await fetch(origin + '/fr/chunk-lab/')).text();
  assert.ok(html.includes(fr), 'French text must be present before any client execution');
  assert.ok(html.includes(`media="(scripting: enabled)" href="${payload('fr')}"`), 'SSR must preload only its selected page/locale for script-enabled clients');
  await page.goto(origin + '/fr/chunk-lab/', {waitUntil:'networkidle'});
  assert.deepEqual(requestedPayloads(), [payload('fr')]);
  assert.equal(await page.locator('main p').nth(1).textContent(), fr);
  assert.deepEqual(errors, []);

  await page.getByRole('link', {name:'EN', exact:true}).click();
  await page.waitForURL('**/en/chunk-lab/');
  await page.waitForFunction(expected => document.querySelectorAll('main p')[1]?.textContent === expected, en);
  assert.deepEqual(requestedPayloads(), [payload('fr'), payload('en')]);
  assert.deepEqual(errors, []);

  const ru = (await import('../src/lib/generated/linguini/locales/ru/main.js')).main.hero.tagline;
  async function navigate(href, expected) {
    await page.evaluate(href => {
      const anchor = document.createElement('a');
      anchor.href = href;
      anchor.setAttribute('data-linguini-no-localize','');
      anchor.textContent = 'Navigate payload probe';
      anchor.id = 'payload-navigation';
      document.body.append(anchor);
    }, href);
    await page.locator('#payload-navigation').click();
    await page.waitForURL('**' + href);
    await page.getByText(expected, {exact:true}).first().waitFor();
    await page.locator('#payload-navigation').evaluate(element => element.remove()).catch(() => {});
  }
  await navigate('/ru/', ru);
  assert.deepEqual(requestedPayloads(), [payload('fr'), payload('en'), payload('ru', '/')]);
  await navigate('/ru/chunk-lab/', ru);
  assert.deepEqual(requestedPayloads(), [payload('fr'), payload('en'), payload('ru', '/'), payload('ru')]);
  await navigate('/fr/chunk-lab/', fr);
  assert.deepEqual(requestedPayloads(), [payload('fr'), payload('en'), payload('ru', '/'), payload('ru')], 'a prepared page/locale must not be requested again');
  assert.deepEqual(errors, []);

  const noJavaScript = await browser.newContext({javaScriptEnabled:false});
  const staticPage = await noJavaScript.newPage();
  const staticRequests = [];
  staticPage.on('request', request => staticRequests.push(new URL(request.url()).pathname));
  await staticPage.goto(origin + '/fr/chunk-lab/');
  assert.equal(await staticPage.locator('main p').nth(1).textContent(), fr);
  assert.deepEqual(staticRequests.filter(file => files.has(file)), []);
});

async function createSsrServer() {
  const {Server} = await import('../.svelte-kit/output/server/index.js');
  const {manifest} = await import('../.svelte-kit/output/server/manifest-full.js');
  const server = new Server(manifest);
  await server.init({env:process.env});
  return server;
}

test('shared locale metadata declares its original URL dependency to SvelteKit navigation', async () => {
  const server = await createSsrServer();
  for (const locale of ['fr','ru']) {
    const response = await server.respond(new Request(`http://localhost/${locale}/chunk-lab/__data.json`), {getClientAddress:() => '127.0.0.1'});
    assert.equal(response.status, 200);
    const data = await response.json();
    assert.equal(data.type, 'data');
    assert.ok(data.nodes[0].data, 'the shared root must serialize locale metadata');
    assert.equal(data.nodes[0].uses.url, 1, 'the root load must rerun when the localized URL changes');
  }
});

test('built SvelteKit SSR isolates concurrent locales and renders values, calls, forms, and formatters', async () => {
  const server = await createSsrServer();
  const locales = ['fr', 'ru', 'en', 'ru', 'en', 'fr'];
  await Promise.all(locales.map(async locale => {
    const response = await server.respond(new Request(`http://localhost/${locale}/`), {getClientAddress:() => '127.0.0.1'});
    assert.equal(response.status, 200);
    const html = await response.text();
    const messages = (await import(`../src/lib/generated/linguini/locales/${locale}/main.js`)).main;
    assert.ok(html.includes(`<html lang="${locale}" dir="ltr">`));
    const values = [
      messages.hero.tagline,
      messages.playground.sentence({fruit:'apple', size:'small', count:3, amount:1299.5, date:'2026-05-19'}),
      messages.playground.cart_summary(3, 'apple'),
      messages.playground.number_format({value:1299.5}),
      messages.playground.currency_format(1299.5),
      messages.playground.date_format({date:'2026-05-19'}),
      messages.playground.override_format({date:'2026-05-19', amount:1299.5}),
      messages.playground.size_line('small'),
    ];
    for (const value of values) {
      const escaped = value.replaceAll('&', '&amp;').replaceAll('<', '&lt;').replaceAll('>', '&gt;');
      assert.ok(html.includes(escaped), `${locale} SSR did not render ${JSON.stringify(value)}`);
    }
  }));
});
