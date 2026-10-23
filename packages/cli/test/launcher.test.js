import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { chmodSync, copyFileSync, cpSync, mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";
import { platformPackage } from "../lib/platform.js";

function copyPublishedPackage(source, destination) {
  const manifest = JSON.parse(readFileSync(path.join(source, "package.json"), "utf8"));
  mkdirSync(destination, {recursive:true});
  copyFileSync(path.join(source, "package.json"), path.join(destination, "package.json"));
  for (const file of manifest.files) cpSync(path.join(source, file), path.join(destination, file), {recursive:true});
  return manifest;
}

test("installed launcher loads typed module config and preserves explicit native plan input", context => {
  const root = mkdtempSync(path.join(tmpdir(), "linguini-launcher-modules-"));
  context.after(() => rmSync(root, {recursive:true, force:true}));
  const cliRoot = fileURLToPath(new URL("../", import.meta.url));
  const commonRoot = fileURLToPath(new URL("../../project/", import.meta.url));
  const launcher = path.join(root, "node_modules/@antepodeum/linguini");
  const manifest = copyPublishedPackage(cliRoot, launcher);
  assert.equal(manifest.dependencies["@antepodeum/linguini-project"], manifest.version);
  copyPublishedPackage(commonRoot, path.join(launcher, "node_modules/@antepodeum/linguini-project"));
  const native = path.join(root, "node_modules", platformPackage(process.platform, process.arch));
  mkdirSync(path.join(native, "bin"), {recursive:true});
  writeFileSync(path.join(native, "package.json"), JSON.stringify({name:platformPackage(process.platform, process.arch), version:manifest.version}));
  // A real executable keeps the launcher/checksum/stdin test portable across all native targets.
  const executable = path.join(native, "bin", process.platform === "win32" ? "linguini.exe" : "linguini");
  copyFileSync(process.execPath, executable);
  if (process.platform !== "win32") chmodSync(executable, 0o755);
  writeFileSync(path.join(native, "bin/manifest.json"), JSON.stringify({sha256:createHash("sha256").update(readFileSync(executable)).digest("hex")}));
  writeFileSync(path.join(root, "package.json"), '{"type":"module"}');
  writeFileSync(path.join(root, "build"), `import {readFileSync} from 'node:fs';
const args = process.argv.slice(2);
const file = args.indexOf('--module-plan');
const inline = args.find(arg => arg.startsWith('--module-plan='));
const plan = JSON.parse(readFileSync(inline ? inline.slice('--module-plan='.length) : file < 0 ? 0 : args[file+1], 'utf8'));
console.log(JSON.stringify({args,plan}));`);
  writeFileSync(path.join(root, "linguini.config.mjs"), `
import {defineModules,web,svelteKit} from '@antepodeum/linguini/modules';
console.log('config logging stays off native JSON stdout');
export default defineModules([web({links:{mode:'manual'}}),svelteKit()]);`);
  const invoke = (args, input) => {
    const result = spawnSync(process.execPath, [path.join(launcher, "bin/linguini.js"), ...args], {cwd:root, encoding:"utf8", input});
    assert.equal(result.status, 0, result.stderr + result.stdout);
    return result;
  };
  const loaded = invoke(["build"]);
  assert.match(loaded.stderr, /config logging/);
  const actual = JSON.parse(loaded.stdout);
  assert.deepEqual(actual.args, ["--module-plan-stdin"]);
  assert.deepEqual(actual.plan.modules.map(module => module.id), ["ecmascript.web", "sveltekit"]);
  const explicit = {version:1, modules:[]};
  writeFileSync(path.join(root, "linguini.config.mjs"), "throw new Error('explicit transport must not reload config');");
  writeFileSync(path.join(root, "plan.json"), JSON.stringify(explicit));
  assert.deepEqual(JSON.parse(invoke(["build", "--module-plan", "plan.json"]).stdout), {args:["--module-plan","plan.json"],plan:explicit});
  assert.deepEqual(JSON.parse(invoke(["build", "--module-plan-stdin"], JSON.stringify(explicit)).stdout), {args:["--module-plan-stdin"],plan:explicit});
  assert.deepEqual(JSON.parse(invoke(["build", "--module-plan=plan.json"]).stdout), {args:["--module-plan=plan.json"],plan:explicit});
});
