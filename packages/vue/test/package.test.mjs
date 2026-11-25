import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";
import { runPnpm } from "../../scripts/pnpm-command.mjs";

function pnpm(args, cwd) {
  const result = runPnpm(args, cwd);
  assert.equal(result.status, 0, result.stderr + result.stdout);
  return result.stdout;
}

test("packed framework adapters preserve strict types and framework ownership", (context) => {
  const root = mkdtempSync(path.join(tmpdir(), "linguini-framework-packages-"));
  context.after(() => rmSync(root, { recursive: true, force: true }));
  const overrides = {};
  for (const name of ["project", "vite", "svelte", "vue"]) {
    const archive = path.join(root, `${name}.tgz`);
    pnpm(["pack", "--out", archive], fileURLToPath(new URL(`../../${name}/`, import.meta.url)));
    overrides[`@antepodeum/linguini-${name}`] = `file:${archive.replaceAll("\\", "/")}`;
  }
  const consumer = path.join(root, "consumer");
  mkdirSync(consumer);
  writeFileSync(path.join(consumer, "package.json"), JSON.stringify({
    private: true, type: "module", dependencies: {
      ...overrides, vue: "3.5.46", svelte: "5.55.8", vite: "7.3.3",
      typescript: "6.0.3", "@types/node": "24.12.4",
    },
  }));
  writeFileSync(path.join(consumer, "pnpm-workspace.yaml"), `overrides:\n${Object.entries(overrides).map(([name, value]) => `  '${name}': ${JSON.stringify(value)}`).join("\n")}\n`);
  pnpm(["install", "--prefer-offline", "--ignore-scripts"], consumer);
  writeFileSync(path.join(consumer, "tsconfig.json"), JSON.stringify({ compilerOptions: {
    strict: true, allowJs: false, skipLibCheck: false, target: "ES2022",
    module: "NodeNext", moduleResolution: "NodeNext", outDir: "output", types: ["node"],
  }, include: ["consumer.ts"] }));
  writeFileSync(path.join(consumer, "consumer.ts"), `
import assert from 'node:assert/strict';
import {createRequire} from 'node:module';
import core from '@antepodeum/linguini-vite';
import svelte, {svelteAdapter} from '@antepodeum/linguini-svelte/vite';
import vue, {vueAdapter} from '@antepodeum/linguini-vue/vite';
assert.equal(core().name, 'vite-plugin-linguini');
assert.equal(svelte().name, 'vite-plugin-linguini');
assert.equal(vue().name, 'vite-plugin-linguini');
assert.equal(typeof svelte().api.compilePageBundles, 'function');
assert.equal(typeof vue().api.compilePages, 'function');
assert.ok(Object.isFrozen(svelteAdapter) && Object.isFrozen(vueAdapter));
const rejectTypes = () => {
  // @ts-expect-error Framework adapters own their descriptor.
  vue({adapter:{}});
  // @ts-expect-error Static-link modules are paths.
  svelte({linkModule:false});
};
void rejectTypes;
const retired = '@antepodeum/linguini-vite/svelte';
await assert.rejects(import(retired), {code:'ERR_PACKAGE_PATH_NOT_EXPORTED'});
const require = createRequire(import.meta.url);
assert.throws(() => require.resolve('react'), {code:'MODULE_NOT_FOUND'});
console.log('ok');
`);
  pnpm(["exec", "tsc", "--project", "tsconfig.json"], consumer);
  const result = spawnSync(process.execPath, [path.join(consumer, "output/consumer.js")], { cwd: consumer, encoding: "utf8" });
  assert.equal(result.status, 0, result.stderr);
  assert.equal(result.stdout.trim(), "ok");
  for (const name of ["svelte", "vue"]) {
    const installed = path.join(consumer, "node_modules/@antepodeum", `linguini-${name}`);
    const map = JSON.parse(readFileSync(path.join(installed, "dist/vite.d.ts.map"), "utf8"));
    for (const source of map.sources) readFileSync(path.resolve(installed, "dist", source), "utf8");
  }
});
