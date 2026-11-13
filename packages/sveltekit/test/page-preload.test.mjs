import assert from 'node:assert/strict';
import test from 'node:test';
import {createPagePreload,pagePreloadHeader} from '../dist/page-preload.js';

test('selected page preload follows Kit hydration, preserves streaming bytes, and omits CSR-disabled pages', () => {
  const url = '/app/_app/immutable/assets/linguini/page/fr.js?quote="&value=<test>';
  const link = '<link rel="modulepreload" media="(scripting: enabled)" href="/app/_app/immutable/assets/linguini/page/fr.js?quote=&quot;&amp;value=&lt;test&gt;">';
  const kit = '<link href="../../_app/immutable/entry/start.hash.js" rel="modulepreload">';
  const prefix = '<!doctype html><head><link rel="stylesheet" href="app.css">';
  const suffix = '<link rel="modulepreload" href="../../_app/immutable/entry/app.hash.js"></head><body>Texte français</body>';
  for (const length of [1,2,7,128,10000]) {
    const input = prefix + kit + suffix;
    const transform = createPagePreload({url});
    let output = '';
    for (let start = 0; start < input.length; start += length) {
      output += transform({html:input.slice(start,start+length),done:start+length >= input.length});
    }
    assert.equal(output,prefix + kit + link + suffix, `chunk length ${length}`);
  }
  for (const input of [
    '<html><head><link rel="stylesheet" href="app.css"></head><body>SSR only</body></html>',
    '<link rel="modulepreload" href="/_app/immutable/chunks/shared.js"><body>SSR only</body>',
    '<link', '<lin', '<', '',
  ]) {
    const transform = createPagePreload({url});
    assert.equal(transform({html:input,done:false}) + transform({html:'',done:true}),input);
  }
});

test('live SSR preload headers require a Kit hydration hint and escape URL delimiters', () => {
  const request = new URL('https://site.example/app/fr/');
  const payload = {url:'/assets/fr.js?test=<probe>"'};
  assert.equal(pagePreloadHeader(payload,'</_app/immutable/entry/start.js>; rel="modulepreload"; nopush',request),
    '</assets/fr.js?test=%3Cprobe%3E%22>; rel="modulepreload"; media="(scripting: enabled)"');
  assert.equal(pagePreloadHeader(payload,'</app.css>; rel="preload"; as="style", </_app/immutable/entry/app.js>; rel=modulepreload',request),
    '</assets/fr.js?test=%3Cprobe%3E%22>; rel="modulepreload"; media="(scripting: enabled)"');
  for (const header of [null,'','</app.css>; rel="preload"; as="style"','</_app/immutable/chunks/shared.js>; rel="modulepreload"']) {
    assert.equal(pagePreloadHeader(payload,header,request),undefined);
  }
  const hint = '</_app/immutable/entry/start.js>; rel="modulepreload"';
  assert.match(pagePreloadHeader({url:'../перевод/%E2%82%AC.js'},hint,request),
    /^<\/app\/%D0%BF%D0%B5%D1%80%D0%B5%D0%B2%D0%BE%D0%B4\/%E2%82%AC\.js>;/);
  assert.match(pagePreloadHeader({url:'https://cdn.example/assets/fr.js'},hint,request),
    /^<https:\/\/cdn\.example\/assets\/fr\.js>;/);
});
