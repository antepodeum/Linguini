import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";
import MagicString from "magic-string";
import { linguini } from "@antepodeum/linguini-vite";
import { linguini as svelte, svelteAdapter } from "@antepodeum/linguini-vite/svelte";

test("published framework boundary is explicit and validates complete adapter contracts", async () => {
  assert.equal(linguini().name, "vite-plugin-linguini");
  assert.equal(svelte().name, "vite-plugin-linguini");
  assert.ok(Object.isFrozen(svelteAdapter));
  const core = await readFile(new URL("../src/index.js", import.meta.url), "utf8");
  assert.doesNotMatch(core, /svelte/i, "the Vite core must not import or encode a framework");
  const adapter = {
    entry: "catalog.js", messageExports: ["catalog"], localeHelper: "state",
  };
  assert.equal(linguini({ adapter }).name, "vite-plugin-linguini");
  assert.equal(linguini({ adapter: {...adapter,pageMessages:{module:"host",export:"getMessage"}} }).name, "vite-plugin-linguini");
  assert.equal(linguini({ adapter: { entry: "catalog.js", messageExports: ["catalog"] } }).name, "vite-plugin-linguini");
  for (const invalid of [
    null,
    { ...adapter, entry: "../escape.js" },
    { ...adapter, entry: "C:/escape.js" },
    { ...adapter, messageExports: [] },
    { ...adapter, localeHelper: "" },
    { ...adapter, entrySideEffects: 'false' },
    { ...adapter, extensions: ["vue"] },
    { ...adapter, applicationScope: true },
    { ...adapter, findStaticLinks: () => [] },
    { ...adapter, pageMessages: {} },
    { ...adapter, pageMessages: {module:"host",export:"not-a-function"} },
    { ...adapter, pageMessages: {module:"host",export:"getMessage",resolve:true} },
    { ...adapter, pageMessages: {module:"host",export:"getMessage",analysisModule:true} },
    { ...adapter, localeHelper:undefined,pageMessages:{module:"host",export:"getMessage"} },
  ]) {
    assert.throws(() => linguini({ adapter: invalid }), /Linguini adapter/);
  }
});

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
