import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";
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
  assert.equal(linguini({ adapter: { entry: "catalog.js", messageExports: ["catalog"] } }).name, "vite-plugin-linguini");
  for (const invalid of [
    null,
    { ...adapter, entry: "../escape.js" },
    { ...adapter, entry: "C:/escape.js" },
    { ...adapter, messageExports: [] },
    { ...adapter, localeHelper: "" },
    { ...adapter, extensions: ["vue"] },
    { ...adapter, applicationScope: true },
    { ...adapter, findStaticLinks: () => [] },
  ]) {
    assert.throws(() => linguini({ adapter: invalid }), /Linguini adapter/);
  }
});
