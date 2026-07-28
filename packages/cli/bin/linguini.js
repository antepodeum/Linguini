#!/usr/bin/env node

import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import path from "node:path";
import { createRequire } from "node:module";
import { platformPackage } from "../lib/platform.js";

const require = createRequire(import.meta.url);
const packageName = platformPackage(process.platform, process.arch);
let packageRoot;
try {
  packageRoot = path.dirname(require.resolve(`${packageName}/package.json`));
} catch (error) {
  const hint = `Reinstall @linguini/cli for ${process.platform}-${process.arch}; optional package ${packageName} is missing.`;
  throw new Error(hint, { cause: error });
}

const executable = path.join(
  packageRoot,
  "bin",
  process.platform === "win32" ? "linguini.exe" : "linguini"
);
const manifest = JSON.parse(readFileSync(path.join(packageRoot, "bin", "manifest.json"), "utf8"));
const digest = createHash("sha256").update(readFileSync(executable)).digest("hex");
if (digest !== manifest.sha256) {
  throw new Error(`Refusing to execute ${packageName}: binary checksum mismatch`);
}

const result = spawnSync(executable, process.argv.slice(2), {
  stdio: "inherit",
  windowsHide: false
});
if (result.error) {
  throw result.error;
}
process.exitCode = result.status ?? 1;
