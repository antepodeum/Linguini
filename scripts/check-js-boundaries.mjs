import { existsSync, readFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const expectedManager = "pnpm@11.1.0";
const projects = [".", "plugins/vite", "site", "editors/vscode"];

for (const project of projects) {
  const manifestPath = join(root, project, "package.json");
  const lockPath = join(root, project, "pnpm-lock.yaml");
  const manifest = JSON.parse(readFileSync(manifestPath, "utf8"));
  if (manifest.packageManager !== expectedManager) {
    throw new Error(`${manifestPath}: packageManager must be ${expectedManager}`);
  }
  const lock = readFileSync(lockPath, "utf8");
  if (!/^lockfileVersion: ['"]9\.0['"]$/m.test(lock)) {
    throw new Error(`${lockPath}: expected pnpm lockfile version 9.0`);
  }
  for (const incompatible of ["package-lock.json", "npm-shrinkwrap.json", "yarn.lock"]) {
    const path = join(root, project, incompatible);
    if (existsSync(path)) throw new Error(`${path}: mixed package-manager lockfile is forbidden`);
  }
}

const launcher = JSON.parse(readFileSync(join(root, "packages/cli/package.json"), "utf8"));
if (launcher.packageManager !== expectedManager) {
  throw new Error(`packages/cli/package.json: packageManager must be ${expectedManager}`);
}

const extension = JSON.parse(readFileSync(join(root, "editors/vscode/package.json"), "utf8"));
for (const [name, command] of Object.entries(extension.scripts ?? {})) {
  if (/\bnpm (run|test|install|ci|audit)\b|\bnpx\b/.test(command)) {
    throw new Error(`editors/vscode package script ${name} bypasses pnpm: ${command}`);
  }
}
const extensionBuildPolicy = readFileSync(join(root, "editors/vscode/pnpm-workspace.yaml"), "utf8");
for (const dependency of ["'@vscode/vsce-sign': true", "esbuild: true", "keytar: true"]) {
  if (!extensionBuildPolicy.includes(dependency)) {
    throw new Error(`editors/vscode/pnpm-workspace.yaml: missing reviewed build policy ${dependency}`);
  }
}

for (const relative of [".github/workflows/ci.yml", ".github/workflows/vscode-extension.yml"]) {
  const workflow = readFileSync(join(root, relative), "utf8");
  if (/\bnpm (ci|install|test|run)\b|\bnpx\b|cache:\s*npm|package-lock\.json/.test(workflow)) {
    throw new Error(`${relative}: dependency installation and scripts must use pinned pnpm`);
  }
}

console.log(`Verified ${projects.length} lockfiles and the native launcher use ${expectedManager}.`);
