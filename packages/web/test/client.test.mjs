import assert from 'node:assert/strict';
import test from 'node:test';
import {createWebLocaleI18n} from '../dist/index.js';
import {createLocaleControl} from '../dist/client.js';

const runtime = {
  locales: ['en', 'fr', 'ar'],
  baseLocale: 'en',
  getTextDirection: locale => locale === 'ar' ? 'rtl' : 'ltr',
};

function fixture(context, {browser = true, httpOnly = false, persistLocale, rejectPrepare = false, denied = false, getRequestIdentity} = {}) {
  const events = [];
  let locale = 'en';
  let release;
  const gate = new Promise(resolve => { release = resolve; });
  const descriptors = ['window', 'document'].map(key => [key, Object.getOwnPropertyDescriptor(globalThis, key)]);
  Object.defineProperty(globalThis, 'window', {configurable: true, value: {
    location: {
      href: 'https://example.test/app/en/shop?q=1#part',
      assign: href => events.push(['assign', href]),
      replace: href => events.push(['replace', href]),
    },
    localStorage: {setItem(key, value) {
      events.push(['storage', key, value]);
      if (denied) throw Error('denied storage');
    }},
  }});
  Object.defineProperty(globalThis, 'document', {configurable: true, value: {
    set cookie(value) {
      events.push(['cookie', value]);
      if (denied) throw Error('denied cookie');
    },
  }});
  context.after(() => {
    for (const [key, descriptor] of descriptors) {
      if (descriptor) Object.defineProperty(globalThis, key, descriptor);
      else delete globalThis[key];
    }
  });
  const state = {
    getCurrentLocale: () => locale,
    async prepareLocale(value) {
      events.push(['prepare', value]);
      await gate;
      if (rejectPrepare) throw Error('preload failed');
      return runtime.locales.includes(value) ? value : 'en';
    },
    setCurrentLocale(value) { events.push(['publish', value]); locale = value; return locale; },
    clearCurrentLocaleOverride() { events.push(['clear']); },
  };
  const web = createWebLocaleI18n(runtime, {
    routing: {localePrefix: 'always'},
    cookie: {name: 'LANG', httpOnly},
    localStorage: {key: 'LANG'},
    locale: {switch: {writesPath: true, writesCookie: true, writesLocalStorage: true}},
  }, {base: '/app', origin: 'https://example.test'});
  const controls = createLocaleControl(web, state, {
    browser,
    getRequestIdentity,
    persistLocale: persistLocale && (value => persistLocale(value, events)),
    navigate: async (href, options) => { events.push(['navigate', href, options]); },
    refresh: () => events.push(['refresh']),
    destroy: () => events.push(['destroy']),
  });
  return {events, release, controls, web, state};
}

test('client controls preload before publishing, persist, navigate, and forward options', async context => {
  const {events, release, controls} = fixture(context);
  const options = {replaceState: true, invalidateAll: false, state: {tab: 'shop'}};
  const switching = controls.setLocale('ar', options);
  assert.equal(controls.locale, 'en');
  assert.deepEqual(events, [['prepare', 'ar']]);
  release();
  assert.equal(await switching, 'ar');
  assert.deepEqual(events.map(event => event[0]), ['prepare', 'publish', 'storage', 'cookie', 'navigate', 'clear', 'refresh']);
  assert.deepEqual(events[2], ['storage', 'LANG', 'ar']);
  assert.match(events[3][1], /^LANG=ar;/);
  assert.deepEqual(events[4], ['navigate', 'https://example.test/app/ar/shop?q=1#part', options]);
  assert.equal(controls.direction, 'rtl');
  assert.deepEqual(controls.htmlAttrs, {lang: 'ar', dir: 'rtl'});
  assert.equal(controls.localizeHref('/app/en/cart'), '/app/ar/cart');
  controls.destroy();
  assert.deepEqual(events.at(-1), ['destroy']);
});

test('server controls prepare without publishing, persistence, navigation, or effects', async context => {
  const {events, release, controls} = fixture(context, {browser: false, httpOnly: true,
    getRequestIdentity() {throw Error('request identity must not be read on the server');},
  });
  const switching = controls.setLocale('fr');
  release();
  assert.equal(await switching, 'fr');
  assert.equal(controls.locale, 'en');
  assert.deepEqual(events, [['prepare', 'fr']]);
});

test('superseded, navigated, and destroyed locale changes cannot publish stale browser state', async context => {
  const {events,web,state} = fixture(context);
  const gates = new Map();
  state.prepareLocale = locale => new Promise(resolve => {gates.set(locale,() => resolve(locale));});
  let request = {};
  const controls = createLocaleControl(web,state,{
    browser:true,getRequestIdentity:() => request,
    navigate:async href => {events.push(['navigate',href]);},
    refresh:() => events.push(['refresh']),destroy:() => events.push(['destroy']),
  });
  const first = controls.setLocale('fr');
  const second = controls.setLocale('ar');
  gates.get('ar')();
  assert.equal(await second,'ar');
  const checkpoint = events.length;
  gates.get('fr')();
  assert.equal(await first,'ar');
  assert.equal(controls.locale,'ar');
  assert.deepEqual(events.slice(checkpoint),[],'a slower earlier choice must have no browser side effects');
  const navigated = controls.setLocale('fr');
  request = {};
  state.setCurrentLocale('en');
  const navigationCheckpoint = events.length;
  gates.get('fr')();
  assert.equal(await navigated,'en');
  assert.deepEqual(events.slice(navigationCheckpoint),[],'a new request context must invalidate pending changes');
  const destroyed = controls.setLocale('fr');
  controls.destroy();
  const destroyCheckpoint = events.length;
  gates.get('fr')();
  assert.equal(await destroyed,'en');
  assert.deepEqual(events.slice(destroyCheckpoint),[],'disposed controls must not publish or navigate');
});

test('request replacement during asynchronous persistence prevents stale publication and navigation', async context => {
  const {events,web,state,release} = fixture(context,{httpOnly:true});
  let request = {},releasePersistence,started;
  const persistenceStarted = new Promise(resolve => {started=resolve;});
  const persistence = new Promise(resolve => {releasePersistence=resolve;});
  const controls = createLocaleControl(web,state,{
    browser:true,getRequestIdentity:() => request,
    async persistLocale(locale) {events.push(['persist',locale]);started();await persistence;},
    navigate:async href => {events.push(['navigate',href]);},
  });
  const switching = controls.setLocale('fr');
  release();
  await persistenceStarted;
  assert.deepEqual(events,[['prepare','fr'],['persist','fr']]);
  request = {};
  state.setCurrentLocale('ar');
  const checkpoint = events.length;
  releasePersistence();
  assert.equal(await switching,'ar');
  assert.deepEqual(events.slice(checkpoint),[]);
});

test('an earlier navigation completion cannot clear or refresh a newer locale change', async context => {
  const {events,web,state,release} = fixture(context);
  const completions = [];
  let firstStarted,secondStarted;
  const enteredFirst = new Promise(resolve => {firstStarted=resolve;});
  const enteredSecond = new Promise(resolve => {secondStarted=resolve;});
  const controls = createLocaleControl(web,state,{
    browser:true,
    navigate:() => new Promise(resolve => {
      completions.push(resolve);
      if (completions.length === 1) firstStarted(); else secondStarted();
    }),
    refresh:() => events.push(['refresh']),
  });
  const first = controls.setLocale('fr');
  release();
  await enteredFirst;
  const second = controls.setLocale('ar');
  await enteredSecond;
  const checkpoint = events.length;
  completions[0]();
  assert.equal(await first,'ar');
  assert.deepEqual(events.slice(checkpoint),[]);
  completions[1]();
  assert.equal(await second,'ar');
  assert.deepEqual(events.slice(checkpoint),[['clear'],['refresh']]);
});

test('failed preload leaves locale and browser side effects untouched', async context => {
  const {events, release, controls} = fixture(context, {rejectPrepare: true});
  const switching = controls.setLocale('fr');
  release();
  await assert.rejects(switching, /preload failed/);
  assert.equal(controls.locale, 'en');
  assert.deepEqual(events, [['prepare', 'fr']]);
});

test('HttpOnly persistence is required and must succeed before publishing', async context => {
  const {events, release, controls, web, state} = fixture(context, {httpOnly: true});
  const switching = controls.setLocale('fr');
  release();
  await assert.rejects(switching, /server persistence callback/);
  assert.deepEqual(events, [['prepare', 'fr']]);
  const failed = createLocaleControl(web, state, {browser: true, persistLocale: async () => {throw Error('server rejected');}});
  await assert.rejects(failed.setLocale('fr'), /server rejected/);
  assert.equal(controls.locale, 'en');
});

test('explicit cookie/navigation opt-outs skip their transports and retain storage', async context => {
  const {events, release, controls} = fixture(context, {
    httpOnly: true,
    persistLocale: async (locale, events) => { events.push(['persist', locale]); },
  });
  const switching = controls.setLocale('fr', {cookie: false, navigate: false});
  release();
  await switching;
  assert.deepEqual(events.map(event => event[0]), ['prepare', 'publish', 'storage', 'refresh']);
  events.length = 0;
  await controls.setLocale('ar', {navigate: false});
  assert.deepEqual(events.map(event => event[0]), ['prepare', 'persist', 'publish', 'storage', 'refresh']);
});

test('denied browser writes preserve a prepared switch and native navigation modes', async context => {
  const {events, release, controls, web, state} = fixture(context, {denied: true});
  const switching = controls.setLocale('fr');
  release();
  await switching;
  assert.equal(controls.locale, 'fr');
  assert.deepEqual(events.map(event => event[0]), ['prepare', 'publish', 'storage', 'cookie', 'navigate', 'clear', 'refresh']);
  const native = createLocaleControl(web, state, {browser: true});
  await native.setLocale('en', {replaceState: true});
  assert.deepEqual(events.at(-1), ['replace', 'https://example.test/app/en/shop?q=1#part']);
  await native.setLocale('fr');
  assert.deepEqual(events.at(-1), ['assign', 'https://example.test/app/fr/shop?q=1#part']);
});
