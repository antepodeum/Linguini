import assert from "node:assert/strict";
import { readFile, readdir } from "node:fs/promises";
import test from "node:test";
import { linguini } from "@antepodeum/linguini-vite";

test("published framework boundary is explicit and validates complete adapter contracts", async () => {
  assert.equal(linguini().name, "vite-plugin-linguini");
  const manifest = JSON.parse(await readFile(new URL("../package.json", import.meta.url), "utf8"));
  assert.deepEqual(Object.keys(manifest.exports), ["."]);
  assert.deepEqual(Object.keys(manifest.peerDependencies), ["vite"]);
  const sources = await readdir(new URL("../src/", import.meta.url));
  assert.ok(!sources.some(name => /svelte|vue|react/.test(name)));
  const core = await readFile(new URL("../src/index.js", import.meta.url), "utf8");
  assert.doesNotMatch(core, /svelte|from ["'](?:vue|react)(?:\/|["'])/);
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
