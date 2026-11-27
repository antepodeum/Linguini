import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import test from 'node:test';
import ts from 'typescript';
import {createWebLocaleI18n} from '../dist/index.js';
import {startRuntimeLinkLocalization} from '../dist/runtime-links.js';

// Reuse the browser fixture exercised by the frozen JS/TS comparison corpus.
const fixture = ts.transpileModule(readFileSync(new URL(
  '../../../tests/fixtures/golden/snapshots/js-web-runtime-links/browser-harness.ts', import.meta.url,
), 'utf8'), {
  fileName: 'browser-harness.ts',
  compilerOptions: {target: ts.ScriptTarget.ES2022, module: ts.ModuleKind.ESNext, moduleResolution: ts.ModuleResolutionKind.Bundler},
  reportDiagnostics: true,
});
assert.deepEqual(fixture.diagnostics, []);
const fixtureSource = fixture.outputText + '\n//# sourceURL=linguini-link-browser-fixture.mjs\n';
const {withLinkBrowser} = await import('data:text/javascript;base64,' + Buffer.from(fixtureSource).toString('base64'));

const web = createWebLocaleI18n({locales: ['en', 'fr'], baseLocale: 'en'}, {
  routing: {localePrefix: 'always'}, links: {mode: 'runtime'},
}, {origin: 'https://example.test'});

function resetVisits(browser) {
  for (const element of [browser.root, ...browser.anchors]) element.visits = 0;
}

for (const animation of [true, false]) {
  const scheduler = animation ? 'animation' : 'timer';
  test(`${scheduler}: text-only mutations schedule no link traversal`, () => withLinkBrowser(animation, browser => {
    const effects = startRuntimeLinkLocalization(web, () => 'fr');
    try {
      browser.flushAll();
      resetVisits(browser);
      browser.mutate([{type: 'childList', addedNodes: Array.from({length: 300}, () => ({nodeType: 3}))}]);
      assert.equal(browser.frames.size + browser.timers.size, 0);
      browser.flushAll();
      assert.ok([browser.root, ...browser.anchors].every(element => element.visits === 0));
    } finally {effects.destroy();}
  }));

  test(`${scheduler}: duplicate mutations and text nodes retain only the changed link`, () => withLinkBrowser(animation, browser => {
    const effects = startRuntimeLinkLocalization(web, () => 'fr');
    try {
      browser.flushAll();
      resetVisits(browser);
      const anchor = browser.anchors[0];
      anchor.attributes.set('href', '/changed');
      browser.mutate([
        {type: 'childList', addedNodes: Array.from({length: 300}, () => ({nodeType: 3}))},
        ...Array.from({length: 300}, () => ({type: 'attributes', target: anchor})),
      ]);
      assert.equal(browser.frames.size + browser.timers.size, 1);
      browser.flushAll();
      assert.equal(anchor.getAttribute('href'), '/fr/changed');
      assert.equal(anchor.visits, 1);
      assert.equal(browser.root.visits, 0);
      assert.ok(browser.anchors.slice(1).every(element => element.visits === 0));
    } finally {effects.destroy();}
  }));

  test(`${scheduler}: eligible-root overflow keeps traversal budgets and cleanup`, () => withLinkBrowser(animation, browser => {
    let locale = 'en';
    const effects = startRuntimeLinkLocalization(web, () => locale);
    browser.flushAll();
    resetVisits(browser);
    locale = 'fr';
    browser.mutate([{type: 'childList', addedNodes: browser.anchors.slice(0, 200)}]);
    browser.flushOne();
    assert.equal(browser.anchors.reduce((sum, anchor) => sum + anchor.visits, 0), 255);
    assert.equal(browser.frames.size + browser.timers.size, 1);
    browser.flushAll();
    assert.equal(browser.anchors[599].getAttribute('href'), '/fr/item/599');
    assert.equal(browser.anchors[1].getAttribute('href'), '/item/1');
    assert.equal(browser.anchors[2].getAttribute('href'), '/item/2');
    resetVisits(browser);
    locale = 'en';
    for (const [start, end] of [[0, 64], [64, 128], [128, 129]]) {
      browser.mutate([{type: 'childList', addedNodes: browser.anchors.slice(start, end)}]);
    }
    browser.flushOne();
    assert.equal(browser.anchors.reduce((sum, anchor) => sum + anchor.visits, 0), 255);
    assert.equal(browser.frames.size + browser.timers.size, 1);
    browser.flushAll();
    assert.equal(browser.anchors[599].getAttribute('href'), '/en/item/599');
    effects.refresh();
    assert.equal(browser.frames.size + browser.timers.size, 1);
    effects.destroy();
    assert.equal(browser.frames.size + browser.timers.size, 0);
    assert.equal(browser.listeners.size, 0);
    assert.equal(browser.disconnected(), 1);
    resetVisits(browser);
    browser.mutate(Array.from({length: 300}, () => ({type: 'attributes', target: browser.anchors[0]})));
    browser.flushAll();
    assert.equal(browser.frames.size + browser.timers.size, 0);
    assert.ok([browser.root, ...browser.anchors].every(element => element.visits === 0));
  }));
}
