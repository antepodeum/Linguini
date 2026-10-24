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

test("installed launcher verifies native bytes and forwards arguments without runtime configuration", context => {
  const root = mkdtempSync(path.join(tmpdir(), "linguini-launcher-modules-"));
  context.after(() => rmSync(root, {recursive:true, force:true}));
  const cliRoot = fileURLToPath(new URL("../", import.meta.url));
  const launcher = path.join(root, "node_modules/@antepodeum/linguini");
  const manifest = copyPublishedPackage(cliRoot, launcher);
  assert.equal(manifest.dependencies, undefined);
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
console.log(JSON.stringify({args,input:args.includes('--read-stdin')?readFileSync(0,'utf8'):undefined}));`);
  writeFileSync(path.join(root, "linguini.config.mjs"), "throw new Error('runtime configuration must not enter the launcher');");
  const invoke = (args, input) => {
    const result = spawnSync(process.execPath, [path.join(launcher, "bin/linguini.js"), ...args], {cwd:root, encoding:"utf8", input});
    assert.equal(result.status, 0, result.stderr + result.stdout);
    return result;
  };
  assert.deepEqual(JSON.parse(invoke(["build", "--format", "json"]).stdout), {args:["--format","json"]});
  assert.deepEqual(JSON.parse(invoke(["build", "--read-stdin"], "native input").stdout), {args:["--read-stdin"],input:"native input"});
  writeFileSync(path.join(native, "bin/manifest.json"), JSON.stringify({sha256:"mismatch"}));
  const rejected = spawnSync(process.execPath,[path.join(launcher,"bin/linguini.js"),"build"],{cwd:root,encoding:"utf8"});
  assert.notEqual(rejected.status,0);
  assert.match(rejected.stderr,/checksum mismatch/);
});
