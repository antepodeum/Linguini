import assert from 'node:assert/strict';
import {createServer} from 'node:http';
import {readFile, stat} from 'node:fs/promises';
import {basename, extname, join, resolve, sep} from 'node:path';
import test from 'node:test';
import {fileURLToPath} from 'node:url';
import {Readable} from 'node:stream';
import {chromium} from 'playwright';

const site = fileURLToPath(new URL('../', import.meta.url));
const output = join(site, 'build');

async function serve(context, {preload,ssr} = {}) {
  let releaseScripts;
  const scriptsReady = new Promise(resolve => {releaseScripts=resolve;});
  if (!preload) releaseScripts();
  const types = {'.html':'text/html', '.js':'text/javascript', '.json':'application/json', '.css':'text/css', '.svg':'image/svg+xml', '.woff2':'font/woff2'};
  const server = createServer(async (request, response) => {
    try {
      const pathname = decodeURIComponent(new URL(request.url, 'http://localhost').pathname);
      if (pathname === preload) releaseScripts();
      else if (pathname.includes('/immutable/entry/') && pathname.endsWith('.js')) await scriptsReady;
      if (ssr && (pathname.endsWith('/') || pathname.endsWith('/__data.json'))) {
        const headers = new Headers();
        for (const [name,value] of Object.entries(request.headers)) {
          if (value !== undefined) headers.set(name,Array.isArray(value) ? value.join(', ') : value);
        }
        const rendered = await ssr.respond(new Request(`http://${request.headers.host}${request.url}`,{headers}),{getClientAddress:()=> '127.0.0.1'});
        response.writeHead(rendered.status,Object.fromEntries(rendered.headers));
        if (rendered.body) Readable.fromWeb(rendered.body).pipe(response);
        else response.end();
        return;
      }
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
  // Hold both framework entry modules until the SSR-declared locale preload arrives.
  // Other preload responses can finish, avoiding HTTP/1 connection starvation.
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
  await page.getByRole('button',{name:'Read deferred copy'}).click();
  await page.getByRole('status',{name:'Deferred copy'}).filter({hasText:fr}).waitFor();
  assert.deepEqual(requestedPayloads(), [payload('fr')], 'a deferred read of prepared messages must not fetch another payload');
  assert.deepEqual(errors, []);

  await page.getByRole('link', {name:'EN', exact:true}).click();
  await page.waitForURL('**/en/chunk-lab/');
  await page.waitForFunction(expected => document.querySelectorAll('main p')[1]?.textContent === expected, en);
  assert.deepEqual(requestedPayloads(), [payload('fr'), payload('en')]);
  await page.getByRole('button',{name:'Read deferred copy'}).click();
  await page.getByRole('status',{name:'Deferred copy'}).filter({hasText:en}).waitFor();
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

  for (const locale of ['fr','en']) {
    const native = (await import(`../src/lib/generated/linguini/locales/${locale}/main.js`)).main;
    await navigate(`/${locale}/message-lab/`,native.hero.tagline);
    await page.getByRole('heading',{name:'Message lab',exact:true}).waitFor();
    const formatting = page.getByRole('region',{name:'Computed message functions'});
    assert.deepEqual(await formatting.locator('p').allTextContents(),[
      native.playground.number_format(1299.5),native.playground.currency_format(1299.5),native.playground.number_format({value:1299.5}),
    ]);
    assert.equal(await page.locator('[data-fallback]').textContent(),'Base-only message fallback');
  }
  assert.deepEqual(requestedPayloads(), [payload('fr'),payload('en'),payload('ru','/'),payload('ru'),payload('fr','/message-lab'),payload('en','/message-lab')]);
  assert.deepEqual(errors, []);

  const noJavaScript = await browser.newContext({javaScriptEnabled:false});
  const staticPage = await noJavaScript.newPage();
  const staticRequests = [];
  staticPage.on('request', request => staticRequests.push(new URL(request.url()).pathname));
  await staticPage.goto(origin + '/fr/chunk-lab/');
  assert.equal(await staticPage.locator('main p').nth(1).textContent(), fr);
  await staticPage.goto(origin + '/fr/message-lab/');
  assert.equal(await staticPage.locator('[data-fallback]').textContent(),'Base-only message fallback');
  assert.deepEqual(staticRequests.filter(file => files.has(file)), []);
});

test('finite computed values, callable references, named calls, and sparse fallback render in concurrent SSR', async () => {
  const server = await createSsrServer();
  await Promise.all(['fr','ru','en','fr','en','ru'].map(async locale => {
    const response = await server.respond(new Request(`http://localhost/${locale}/message-lab/`),{getClientAddress:()=>'127.0.0.1'});
    assert.equal(response.status,200);
    const html = await response.text();
    const native = (await import(`../src/lib/generated/linguini/locales/${locale}/main.js`)).main;
    for (const value of [native.hero.title,native.hero.tagline,native.playground.number_format(1299.5),native.playground.currency_format(1299.5),'Base-only message fallback']) {
      assert.ok(html.includes(value.replaceAll('&','&amp;').replaceAll('<','&lt;').replaceAll('>','&gt;')), `${locale} finite SSR omitted ${JSON.stringify(value)}`);
    }
    const header = response.headers.get('link');
    const {payloads} = await import('../.svelte-kit/output/server/linguini-page-manifest.js');
    assert.ok(header?.includes(`</${payloads['/message-lab'][locale]}>; rel="modulepreload"; media="(scripting: enabled)"`));
  }));
});

async function createSsrServer() {
  const {Server} = await import('../.svelte-kit/output/server/index.js');
  const {manifest} = await import('../.svelte-kit/output/server/manifest-full.js');
  const server = new Server(manifest);
  await server.init({env:process.env});
  return server;
}

test('live SSR preloads one selected payload through headers and scripting-disabled clients fetch none', {timeout:30_000}, async context => {
  const {payloads} = await import('../.svelte-kit/output/server/linguini-page-manifest.js');
  const files = new Set(Object.values(payloads).flatMap(locales => Object.values(locales).map(file => '/' + file)));
  const selected = '/' + payloads['/message-lab'].fr;
  const origin = await serve(context,{preload:selected,ssr:await createSsrServer()});
  const browser = await chromium.launch();
  context.after(() => browser.close());
  for (const enabled of [true,false]) {
    const session = await browser.newContext({javaScriptEnabled:enabled});
    const page = await session.newPage();
    const requests = [], errors = [];
    page.on('request',request => requests.push(new URL(request.url()).pathname));
    page.on('pageerror',error => errors.push(error.message));
    page.on('console',message => {if (/hydration_/i.test(message.text())) errors.push(message.text());});
    await page.goto(origin + '/fr/message-lab/',{waitUntil:'networkidle',timeout:5000}).catch(error => {
      throw new Error(`${error.message}; scripting=${enabled}; requests=${JSON.stringify(requests)}; errors=${JSON.stringify(errors)}`,{cause:error});
    });
    assert.equal(await page.locator('[data-fallback]').textContent(),'Base-only message fallback');
    const native = (await import('../src/lib/generated/linguini/locales/fr/main.js')).main;
    assert.deepEqual(await page.getByRole('region',{name:'Computed message functions'}).locator('p').allTextContents(),[
      native.playground.number_format(1299.5),native.playground.currency_format(1299.5),native.playground.number_format({value:1299.5}),
    ]);
    assert.deepEqual(requests.filter(file => files.has(file)),enabled ? [selected] : []);
    assert.deepEqual(errors,[]);
    await session.close();
  }
});

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

test('a locale preload completed after cross-page navigation cannot overwrite the incoming page', {timeout:30_000}, async context => {
  const {payloads} = await import('../.svelte-kit/output/server/linguini-page-manifest.js');
  const files = new Set(Object.values(payloads).flatMap(locales => Object.values(locales).map(file => '/' + file)));
  const initial = '/' + payloads['/chunk-lab'].fr;
  const delayed = '/' + payloads['/chunk-lab'].en;
  const incoming = '/' + payloads['/'].ru;
  const origin = await serve(context);
  const browser = await chromium.launch();
  let release,started;
  const gate = new Promise(resolve => {release=resolve;});
  const requested = new Promise(resolve => {started=resolve;});
  context.after(async () => {release();await browser.close();});
  const session = await browser.newContext();
  const requests = [],errors = [];
  const page = await session.newPage();
  page.on('request',request => requests.push(new URL(request.url()).pathname));
  page.on('pageerror',error => errors.push(error.message));
  await session.route('**' + delayed,async route => {
    started();await gate;await route.continue();
  });
  await page.goto(origin + '/fr/chunk-lab/',{waitUntil:'networkidle'});
  const completed = page.waitForResponse(response => new URL(response.url()).pathname === delayed).then(response => response.finished());
  await page.getByRole('link',{name:'EN',exact:true}).click();
  await requested;
  await page.evaluate(() => {
    const anchor = document.createElement('a');
    anchor.href = '/ru/';anchor.id = 'pending-locale-navigation';
    anchor.setAttribute('data-linguini-no-localize','');
    anchor.textContent = 'Continue navigation';document.body.append(anchor);
  });
  await page.locator('#pending-locale-navigation').click();
  await page.waitForURL('**/ru/');
  const native = (await import('../src/lib/generated/linguini/locales/ru/main.js')).main;
  await page.getByText(native.hero.tagline,{exact:true}).first().waitFor();
  release();await completed;
  // Join the existing module evaluation and let its preparation continuations finish.
  await page.evaluate(async url => {
    await import(url);
    await new Promise(resolve => requestAnimationFrame(() => resolve()));
  },origin + delayed);
  await page.waitForLoadState('networkidle');
  assert.equal(new URL(page.url()).pathname,'/ru/');
  assert.deepEqual(requests.filter(file => files.has(file)),[initial,delayed,incoming]);
  assert.deepEqual(errors,[]);
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
