import assert from "node:assert/strict";
import { mkdtempSync, rmSync, symlinkSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import test from "node:test";
import { defineModule, defineModules, normalizeModulePlan, readModuleConfiguration, readModuleConfigurationSync } from "../src/modules.js";
import { web, svelte, svelteKit } from "../src/ecmascript/modules.js";

function fixture(context) {
  const root = mkdtempSync(path.join(tmpdir(), "linguini-modules-test-"));
  context.after(() => rmSync(root, { recursive: true, force: true }));
  return root;
}

test("typed module factories preserve one JSON-only immutable plan", () => {
  const options = { locale: {sources: ["path", "cookie"]}, cookie: {maxAge: "2d"} };
  const module = web(options);
  options.locale.sources.push("local-storage");
  const plan = defineModules([module, svelteKit()]);
  assert.equal(plan.version, 1);
  assert.deepEqual(plan.modules.map(module => module.id), ["ecmascript.web", "sveltekit"]);
  assert.deepEqual(plan.modules[0].options.locale.sources, ["path", "cookie"]);
  assert.ok(Object.isFrozen(plan.modules[0].options.locale.sources));
  assert.throws(() => defineModules([svelte(), svelte()]), /Duplicate module/);
  assert.throws(() => svelte({}), /no options/);
  assert.throws(() => svelteKit({}), /no options/);
  for (const value of [undefined, () => 1, Symbol(), 1n, NaN, new Date()]) {
    assert.throws(() => defineModule("future.language", { value }), /JSON data|plain objects/);
  }
  const cyclic = {}; cyclic.self = cyclic;
  assert.throws(() => defineModule("future.language", cyclic), /cycles/);
  for (const value of [
    {version:2, modules:[]}, {version:1, modules:[], extra:true},
    {version:1, modules:[{id:"x", options:null}]}, {version:1, modules:[{id:"x", extra:true}]}
  ]) assert.throws(() => normalizeModulePlan(value));
});

test("sync and async hosts load the complete config import graph afresh", async context => {
  const root = fixture(context);
  assert.deepEqual(await readModuleConfiguration(root), readModuleConfigurationSync(root));
  const factories = new URL("../src/ecmascript/modules.js", import.meta.url).href;
  const modules = new URL("../src/modules.js", import.meta.url).href;
  writeFileSync(path.join(root, "policy.mjs"), 'export const mode = "manual";');
  writeFileSync(path.join(root, "linguini.config.mjs"), `
import { defineModules } from ${JSON.stringify(modules)};
import { web, svelteKit } from ${JSON.stringify(factories)};
import { mode } from "./policy.mjs";
if ((await import("node:fs")).realpathSync(process.cwd()) !== (await import("node:fs")).realpathSync(${JSON.stringify(root)})) throw new Error("wrong project cwd");
export default defineModules([web({links:{mode}}), svelteKit()]);
`);
  assert.equal((await readModuleConfiguration(root)).plan.modules[0].options.links.mode, "manual");
  assert.ok((await readModuleConfiguration(root)).dependencies.includes(path.join(root, "policy.mjs")));
  assert.deepEqual(await readModuleConfiguration(root), readModuleConfigurationSync(root));
  writeFileSync(path.join(root, "policy.mjs"), 'export const mode = "runtime";');
  assert.equal((await readModuleConfiguration(root)).plan.modules[0].options.links.mode, "runtime");
  assert.equal(readModuleConfigurationSync(root).plan.modules[0].options.links.mode, "runtime");
  writeFileSync(path.join(root, "nested.cjs"), 'module.exports = "manual";');
  writeFileSync(path.join(root, "policy.cjs"), 'module.exports = require("./nested.cjs");');
  writeFileSync(path.join(root, "linguini.config.mjs"), `
import { defineModules } from ${JSON.stringify(modules)};
import { web } from ${JSON.stringify(factories)};
import mode from './policy.cjs';
export default defineModules([web({links:{mode}})]);`);
  const alias = `${root}-alias`;
  symlinkSync(root, alias, "junction");
  context.after(() => rmSync(alias, {recursive:true, force:true}));
  let aliased = await readModuleConfiguration(alias);
  assert.equal(aliased.configPath, path.join(alias, "linguini.config.mjs"));
  assert.ok(aliased.dependencies.includes(path.join(alias, "nested.cjs")));
  assert.equal(aliased.plan.modules[0].options.links.mode, "manual");
  writeFileSync(path.join(root, "nested.cjs"), 'module.exports = "runtime";');
  aliased = readModuleConfigurationSync(alias);
  assert.equal(aliased.plan.modules[0].options.links.mode, "runtime");
  assert.deepEqual(await readModuleConfiguration(alias), aliased);
  writeFileSync(path.join(root, "linguini.config.mjs"), "import './missing.mjs';");
  for (const load of [readModuleConfiguration, readModuleConfigurationSync]) {
    await assert.rejects(async () => load(root), error => {
      assert.ok(error.moduleDependencies.includes(path.join(root, "missing.mjs")));
      return true;
    });
  }
  writeFileSync(path.join(root, "linguini.config.mjs"), 'throw new Error("author configuration error");');
  await assert.rejects(readModuleConfiguration(root), /author configuration error/);
  assert.throws(() => readModuleConfigurationSync(root), /author configuration error/);
});

test("static plans have the same shape and ambiguous configuration is rejected", async context => {
  const root = fixture(context);
  const plan = defineModules([defineModule("future.language", {output:"native"})]);
  writeFileSync(path.join(root, "linguini.modules.json"), JSON.stringify(plan));
  assert.deepEqual((await readModuleConfiguration(root)).plan, plan);
  assert.deepEqual(readModuleConfigurationSync(root).plan, plan);
  writeFileSync(path.join(root, "linguini.config.mjs"), "export default {};");
  await assert.rejects(readModuleConfiguration(root), /one module configuration source/);
  assert.throws(() => readModuleConfigurationSync(root), /one module configuration source/);
});
