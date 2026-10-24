import assert from "node:assert/strict";
import { mkdtempSync, mkdirSync, rmSync, symlinkSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import test from "node:test";
import { readProjectConfig, readProjectConfigSync, resolveProjectPath } from "../src/index.js";
import { readEcmascriptProject, readEcmascriptProjectSync } from "../src/ecmascript.js";

function fixture(context) {
  const root = mkdtempSync(path.join(tmpdir(), "linguini-project-"));
  context.after(() => rmSync(root, { recursive: true, force: true }));
  return root;
}

test("sync and async hosts share defaults, configured paths, and closed layout policy", async (context) => {
  const root = fixture(context);
  const missing = readEcmascriptProjectSync(root);
  assert.deepEqual(await readEcmascriptProject(root), missing);
  assert.equal(missing.configExists, false);
  assert.equal(missing.targetConfigured, false);
  assert.equal(missing.schemaRoot, path.join(root, "linguini/schema"));
  assert.equal(missing.localeRoot, path.join(root, "linguini/locale"));
  assert.equal(missing.generatedRoot, path.join(root, "generated/linguini"));
  assert.equal(missing.localeLoading, "eager");
  writeFileSync(path.join(root, "custom.toml"), `
[paths]
schema = "src/schema"
locale = "src/locale"
[targets.ts]
out = "src/generated"
[targets.ts.bundler]
sources = ["src", "components"]
exclude = ["src/tests"]
locale_loading = "dynamic"
[targets.ts.bundler.dynamic]
mode = "bundle"
allow = ["main.title"]
`);
  writeFileSync(path.join(root, "linguini.config.mjs"), 'throw new Error("runtime config must never be evaluated by native build integrations");');
  const selected = readEcmascriptProjectSync(root, "custom.toml");
  assert.deepEqual(await readEcmascriptProject(root, "custom.toml"), selected);
  assert.equal(selected.targetConfigured, true);
  assert.equal(selected.configExists, true);
  assert.equal(selected.schemaRoot, path.join(root, "src/schema"));
  assert.equal(selected.localeRoot, path.join(root, "src/locale"));
  assert.equal(selected.generatedRoot, path.join(root, "src/generated"));
  assert.deepEqual(selected.bundlerSourceRoots, [path.join(root, "src"), path.join(root, "components")]);
  assert.deepEqual(selected.bundlerExcludeRoots, [path.join(root, "src/generated"), path.join(root, "src/tests")]);
  assert.equal(selected.localeLoading, "dynamic");
  assert(Object.isFrozen(selected));
  assert(Object.isFrozen(selected.bundlerSourceRoots));
  assert(Object.isFrozen(selected.bundlerExcludeRoots));
  assert.deepEqual(await readProjectConfig(root, "custom.toml"), readProjectConfigSync(root, "custom.toml"));
});

test("both hosts reject malformed documents and invalid layout values consistently", async (context) => {
  const root = fixture(context);
  for (const [source, expected] of [
    ["[paths", /cannot parse Linguini config/],
    ['[paths]\nschema = "../outside"', /must stay inside/],
    ['[paths]\nlocale = 12', /paths.locale must be a non-empty string/],
    ['[targets.ts]\nout = ""', /targets.ts.out must be a non-empty string/],
    ['[targets.ts]\nbundler = false', /bundler must be a table/],
    ['[targets.ts.bundler]\nsources = []', /non-empty string array/],
    ['[targets.ts.bundler]\nsources = ["src"]\nlocale_loading = "lazy"', /locale_loading.*eager/],
    ['[targets.ts.bundler]\nsources = ["src"]\n[targets.ts.bundler.dynamic]\nmode = "bundle"\nallow = ["a", "A"]', /duplicate path/],
    ['[web.links]\nmode = "unsafe"', /TOML web\/framework policy was removed/],
  ]) {
    writeFileSync(path.join(root, "linguini.toml"), source);
    assert.throws(() => readEcmascriptProjectSync(root), expected);
    await assert.rejects(readEcmascriptProject(root), expected);
  }
  assert.throws(() => resolveProjectPath(root, path.resolve(root, "outside"), "test"), /relative path/);
  assert.throws(() => readProjectConfigSync(root, "../outside.toml"), /must stay inside/);
  await assert.rejects(readProjectConfig(root, "../outside.toml"), /must stay inside/);
  mkdirSync(path.join(root, "directory"));
  assert.throws(() => readProjectConfigSync(root, "directory"), /cannot read Linguini config/);
  await assert.rejects(readProjectConfig(root, "directory"), /cannot read Linguini config/);
});

test("project-root aliases keep the selected path and resolve the same document", async (context) => {
  const root = fixture(context);
  const actual = path.join(root, "actual"), alias = path.join(root, "alias");
  mkdirSync(actual);
  symlinkSync(actual, alias, process.platform === "win32" ? "junction" : "dir");
  writeFileSync(path.join(actual, "linguini.toml"), '[targets.ts]\nout = "generated/messages"');
  const layout = await readEcmascriptProject(alias);
  assert.deepEqual(layout, readEcmascriptProjectSync(alias));
  assert.equal(layout.projectRoot, alias);
  assert.equal(layout.generatedRoot, path.join(alias, "generated/messages"));
  assert.equal(layout.configExists, true);
});
