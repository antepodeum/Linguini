import { readFile } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..", "..");
const expected = "0.1.0-alpha.4";
const manifests = [
  "packages/cli/package.json",
  "packages/cli-darwin-arm64/package.json",
  "packages/cli-darwin-x64/package.json",
  "packages/cli-linux-arm64/package.json",
  "packages/cli-linux-x64/package.json",
  "packages/cli-win32-arm64/package.json",
  "packages/cli-win32-x64/package.json",
  "plugins/vite/package.json",
  "editors/vscode/package.json",
  "site/package.json"
];

for (const relative of manifests) {
  const manifest = JSON.parse(await readFile(path.join(root, relative), "utf8"));
  if (manifest.version !== expected) {
    throw new Error(`${relative}: expected ${expected}, found ${manifest.version}`);
  }
}

const cargo = await readFile(path.join(root, "crates/linguini-cli/Cargo.toml"), "utf8");
if (!cargo.includes(`version = "${expected}"`)) {
  throw new Error(`crates/linguini-cli/Cargo.toml: expected ${expected}`);
}
