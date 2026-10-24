import assert from 'node:assert/strict';
import test from 'node:test';
import {createWebLocaleI18n, createWebI18n, createLocaleRequestContext, pathLocale, cookieLocale, requestLanguage, localStorageLocale} from '../dist/index.js';
import {createLocaleSwitchHandler} from '../dist/server.js';

const runtime = {locales:['en','fr','ar'],baseLocale:'en',getTextDirection:locale => locale === 'ar' ? 'rtl' : 'ltr'};
const environment = {base:'/app',origin:'https://example.test'};

test('runtime sources are explicit, ordered capabilities independent of message generation', () => {
  const input = {url:'https://example.test/app/fr/shop',cookie:'LOCALE=ar',headers:new Headers({'accept-language':'en'})};
  assert.equal(createWebLocaleI18n(runtime, {}, environment).resolveLocaleSync(input), 'en');
  const web = createWebLocaleI18n(runtime, {locale:{sources:[pathLocale(),cookieLocale({name:'LOCALE'}),requestLanguage()]}}, environment);
  assert.equal(web.resolveLocaleSync(input), 'fr');
  assert.equal(web.resolveLocaleSync({...input,url:undefined}), 'ar');
  assert.equal(web.resolveLocaleSync({headers:new Headers({'accept-language':'fr;q=0.8,en;q=0.2'})}), 'fr');
  assert.equal(web.localizeHref('/app/en/shop?q=1#part','fr'), '/app/fr/shop?q=1#part');
  assert.equal(web.localizeHref('https://external.test/shop','fr'), 'https://external.test/shop');
  assert.equal(web.localizeHrefAttribute('javascript:alert(1)','fr'), 'javascript:alert(1)');
  assert.equal(web.shouldExclude('http://['), false);
  assert.equal(web.htmlAttrs('ar').dir, 'rtl');
  const globalCookie=createWebLocaleI18n(runtime,{cookie:{name:'GLOBAL',httpOnly:true},locale:{sources:[cookieLocale({secure:true})]}});
  assert.equal(globalCookie.resolveLocaleSync({cookie:'GLOBAL=fr'}),'fr');
  assert.match(globalCookie.serializeLocaleCookie('fr'),/HttpOnly/);
  assert.match(globalCookie.serializeLocaleCookie('fr'),/Secure/);
  assert.throws(() => createWebLocaleI18n(runtime,{locale:{sources:['path']}}), /capabilities/);
  assert.throws(() => createWebLocaleI18n(runtime,{locale:{sources:[pathLocale(),pathLocale()]}}), /Duplicate/);
  assert.throws(() => createWebLocaleI18n(runtime,{locale:{sources:[cookieLocale({name:'bad;name'})]}}), /cookie name/);
  assert.throws(() => createWebLocaleI18n(runtime,{locale:{sources:[cookieLocale({sameSite:'none'})]}}), /secure/);
  assert.equal(createWebLocaleI18n(runtime,{locale:{sources:[localStorageLocale()]}}).resolveLocaleSync({localStorage:{getItem(){throw Error('denied');}}}), 'en');
});

test('cookie and switch endpoint policy runs at application startup and rejects unsafe returns', () => {
  const web = createWebLocaleI18n(runtime, {routing:{localePrefix:'never'},locale:{sources:[cookieLocale({name:'LANG',httpOnly:true,secure:true})]}}, environment);
  const handle = createLocaleSwitchHandler(web,{path:'/locale/{locale}/set',returnQuery:'next',status:307});
  for (const target of ['//external.test','%2F%2Fexternal.test','https://external.test/path','%5C%5Cexternal.test']) {
    const url = new URL('/app/locale/fr/set?next='+encodeURIComponent(target), environment.origin);
    const response = handle({url,request:new Request(url)});
    assert.equal(response.headers.get('location'), '/app');
    assert.match(response.headers.get('set-cookie'), /^LANG=fr;/);
    assert.match(response.headers.get('set-cookie'), /HttpOnly/);
    assert.equal(response.status,307);
  }
  const url = new URL('/app/locale/ar/set?next='+encodeURIComponent('/app/shop?q=1#part'), environment.origin);
  assert.equal(handle({url,request:new Request(url)}).headers.get('location'), '/app/shop?q=1#part');
  for (const path of ['/app/locale/no/set','/app/locale/fr/wrong','/app/locale/%ZZ/set','/app/locale/fr/extra/set']) {
    const url = new URL(path,environment.origin);
    assert.equal(handle({url,request:new Request(url)}), undefined);
  }
  assert.throws(() => createLocaleSwitchHandler(web,{path:'/missing-marker'}), /Invalid/);
});

test('message-enabled request contexts preload before calls and isolate concurrent request input', async () => {
  const prepared = new Set();
  const web = createWebI18n({...runtime,
    async prepareLinguini(locale) {await new Promise(resolve => setTimeout(resolve,locale==='fr'?5:1));prepared.add(locale);},
    createLinguini(locale) {assert.ok(prepared.has(locale));return {greeting:name => `${locale}:${name}`};},
  },{locale:{sources:[pathLocale()]}},environment);
  const [fr,en] = await Promise.all(['fr','en'].map(locale => web.resolveRequest(new Request(`https://${locale}.test/app/${locale}/page`))));
  assert.equal(fr.messages.greeting('Ada'),'fr:Ada');
  assert.equal(en.messages.greeting('Ada'),'en:Ada');
  assert.equal(fr.localizeUrl('/next').origin,'https://fr.test');
  assert.equal(en.localizeUrl('/next').origin,'https://en.test');
});

test('combined link localization parses one environment snapshot and isolates exclusion mutations', context => {
  const web = createWebLocaleI18n(runtime, {}, environment);
  const NativeURL = globalThis.URL;
  let parses = 0;
  globalThis.URL = class extends NativeURL {
    constructor(...args) { super(...args); parses++; }
  };
  context.after(() => { globalThis.URL = NativeURL; });
  let originReads = 0;
  let currentReads = 0;
  const input = {
    get origin() { originReads++; return environment.origin; },
    get currentUrl() { currentReads++; return environment.origin + '/app/en/shop'; },
  };
  assert.equal(web.localizeLinkHref('/app/en/cart?q=1#part', 'fr', {}, input), '/app/fr/cart?q=1#part');
  assert.equal(parses, 3, 'resolve origin/current URL once, then parse the href once');
  assert.equal(originReads, 1);
  assert.equal(currentReads, 1);
  for (const attributes of [{download:true}, {ignored:true}, {rel:'nofollow EXTERNAL'}]) {
    assert.equal(web.localizeLinkHref('/cart', 'fr', attributes, input), '/cart');
  }
  assert.equal(parses, 3, 'ignored link attributes must short-circuit URL parsing');
  for (const href of ['#part', '//external.test/path', 'mailto:a@example.test', 'http://[', 'https://external.test/cart']) {
    assert.equal(web.localizeLinkHref(href, 'fr', {}, input), href);
  }
  const observations = [];
  const changing = createWebLocaleI18n(runtime, {routes:{exclude:[
    url => { observations.push(url.pathname); url.pathname = '/mutated'; return false; },
    url => { observations.push(url.pathname); return false; },
  ]}}, environment);
  assert.equal(changing.localizeLinkHref('/app/en/cart', 'fr'), '/app/fr/cart');
  assert.deepEqual(observations, ['/app/en/cart', '/mutated']);
  const failure = new Error('predicate failed');
  const throwing = createWebLocaleI18n(runtime, {routes:{exclude:[() => {throw failure;}]}}, environment);
  assert.throws(() => throwing.localizeLinkHref('/cart', 'fr'), error => error === failure);
  const manual = createWebLocaleI18n(runtime, {links:{mode:'manual'}}, environment);
  assert.equal(manual.localizeLinkHref('/cart', 'fr'), '/cart');
});

test('request metadata snapshots direction once per context without caching across requests', () => {
  let calls = 0;
  let reads = 0;
  const web = createWebLocaleI18n({...runtime, get getTextDirection() {
    reads++;
    return () => ++calls === 1 ? 'ltr' : 'rtl';
  }}, {}, environment);
  // Factory construction spreads runtime metadata once; only context reads count here.
  reads = 0;
  const first = createLocaleRequestContext(web, 'fr');
  const second = createLocaleRequestContext(web, 'ar');
  assert.equal(calls, 2);
  assert.equal(reads, 2);
  assert.deepEqual([first.direction, first.textDirection, first.htmlAttrs.dir], ['ltr', 'ltr', 'ltr']);
  assert.deepEqual([second.direction, second.textDirection, second.htmlAttrs.dir], ['rtl', 'rtl', 'rtl']);
  assert.equal(first.localizeLinkHref('/app/en/cart'), '/app/fr/cart');
  assert.equal(second.localizeLinkHref('/app/en/cart'), '/app/ar/cart');
});
