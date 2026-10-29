import assert from 'node:assert/strict';
import test from 'node:test';
import {createPagePreload} from '../dist/page-preload.js';

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
