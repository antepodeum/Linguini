import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import test from "node:test";

function pnpm(args, cwd) {
  const result = process.env.npm_execpath
    ? spawnSync(process.execPath, [process.env.npm_execpath, ...args], { cwd, encoding: "utf8" })
    : spawnSync("pnpm", args, { cwd, encoding: "utf8" });
  assert.equal(result.status, 0, result.stderr + result.stdout);
  return result;
}

test("packed project APIs execute and preserve strict TypeScript contracts", (context) => {
  const root = mkdtempSync(path.join(tmpdir(), "linguini-project-package-"));
  context.after(() => rmSync(root, { recursive: true, force: true }));
  const archive = path.join(root, "project.tgz");
  const packageRoot = fileURLToPath(new URL("../", import.meta.url));
  const store = pnpm(["store", "path"], packageRoot).stdout.trim();
  pnpm(["pack", "--out", archive], packageRoot);
  writeFileSync(path.join(root, "package.json"), JSON.stringify({ type: "module", private: true, dependencies: {
    "@antepod/linguini-project": `file:${archive.replaceAll("\\", "/")}`,
    typescript: "6.0.3",
  } }));
  pnpm(["install", "--offline", "--ignore-scripts", "--store-dir", store], root);
  writeFileSync(path.join(root, "linguini.toml"), '[targets.ts]\nout = "generated/messages"');
  writeFileSync(path.join(root, "consumer.ts"), `
import { readProjectConfig, readProjectConfigSync } from "@antepod/linguini-project";
import { readEcmascriptProject, readEcmascriptProjectSync } from "@antepod/linguini-project/ecmascript";
const project = await readProjectConfig(".");
const sync = readProjectConfigSync(".");
const layout = await readEcmascriptProject(".");
const again = readEcmascriptProjectSync(".");
const mode: "eager" | "dynamic" = layout.localeLoading;
// @ts-expect-error project roots must be strings
void (() => readProjectConfig(42));
// @ts-expect-error the selected layout is immutable
void (() => { layout.generatedRoot = "unsafe"; });
console.log(JSON.stringify([project.configExists, sync.configExists, layout.targetConfigured, mode, again.generatedRoot.endsWith("messages")]));
`);
  writeFileSync(path.join(root, "tsconfig.json"), JSON.stringify({ compilerOptions: {
    strict: true, module: "NodeNext", moduleResolution: "NodeNext", target: "ES2022", outDir: "dist",
  }, include: ["consumer.ts"] }));
  pnpm(["exec", "tsc", "--project", "tsconfig.json"], root);
  const result = spawnSync(process.execPath, [path.join(root, "dist/consumer.js")], { cwd: root, encoding: "utf8" });
  assert.equal(result.status, 0, result.stderr);
  assert.deepEqual(JSON.parse(result.stdout), [true, true, true, "eager", true]);
  const manifest = JSON.parse(readFileSync(path.join(root, "node_modules/@antepod/linguini-project/package.json"), "utf8"));
  assert.equal(manifest.dependencies["smol-toml"], "1.7.1");
});
