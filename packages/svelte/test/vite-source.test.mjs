import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";
import MagicString from "magic-string";
import { svelteAdapter } from "../dist/vite.js";

test("Svelte insertions respect quoted generics and ignore commented script tags", async () => {
  const source = await readFile(new URL('./fixtures/generic-component.svelte',import.meta.url),'utf8');
  const name = 'Generic.svelte';
  const opening = `<script lang="ts" generics="T extends { text: string; nested: Array<' module '> }">`;
  const insertion = source.indexOf(opening) + opening.length;
  const scope = svelteAdapter.applicationScope(source,source.indexOf("import { l }"),name);
  assert.equal(scope.insertionByte,Buffer.byteLength(source.slice(0,insertion)));
  const magic = new MagicString(source);
  const operations = svelteAdapter.findStaticLinks(source,name);
  assert.deepEqual(operations.map(item => item.href),['/account']);
  svelteAdapter.applyStaticLinks(magic,source,name,'generated',operations,'./links.js');
  svelteAdapter.injectLocaleLoaderLifecycle(magic,source,{relative:name},{
    prelude:'const release = () => {};\n',release:'release',allocateAlias:prefix => prefix + '0',
  });
  const result = magic.toString();
  const inserted = result.slice(insertion,result.indexOf("import { l }"));
  assert.match(inserted,/import \{ localizeTransformedHref/);
  assert.match(inserted,/import \{ onDestroy/);
  assert.ok(result.includes(opening));
  assert.equal(result.match(/import \{ onDestroy/g)?.length,1);
  assert.ok(result.indexOf('import { onDestroy') > result.indexOf(opening));
  assert.match(result,/<!-- 😀 <script>not a script scope<\/script> -->/);
});
