#!/usr/bin/env node

import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { existsSync, readFileSync } from "node:fs";
import path from "node:path";
import { createRequire } from "node:module";
import { platformPackage } from "../lib/platform.js";

const require = createRequire(import.meta.url);
const packageName = platformPackage(process.platform, process.arch);
let packageRoot;
try {
  packageRoot = path.dirname(require.resolve(`${packageName}/package.json`));
} catch (error) {
  const hint = `Reinstall @antepodeum/linguini for ${process.platform}-${process.arch}; optional package ${packageName} is missing.`;
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

const args = process.argv.slice(2);
let input;
if (args[0] === "build" && !args.some(arg => arg === "--module-plan" || arg.startsWith("--module-plan=")) && !args.includes("--module-plan-stdin") &&
    ["linguini.config.mjs", "linguini.modules.json"].some(file => existsSync(path.resolve(file)))) {
  const { readModuleConfigurationSync } = await import("@antepodeum/linguini-project/modules");
  input = JSON.stringify(readModuleConfigurationSync(process.cwd()).plan);
  args.push("--module-plan-stdin");
}
const result = spawnSync(executable, args, {
  input,
  stdio: input === undefined ? "inherit" : ["pipe", "inherit", "inherit"],
  windowsHide: false
});
if (result.error) {
  throw result.error;
}
process.exitCode = result.status ?? 1;
