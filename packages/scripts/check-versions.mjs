import { readdir, readFile } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..", "..");
const launcherPath = "packages/cli/package.json";
const launcher = JSON.parse(await readFile(path.join(root, launcherPath), "utf8"));
const expected = launcher.version;
const manifests = [
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

for (const entry of await readdir(path.join(root, "crates"), { withFileTypes: true })) {
  if (!entry.isDirectory()) {
    continue;
  }
  const relative = path.join("crates", entry.name, "Cargo.toml");
  const cargo = await readFile(path.join(root, relative), "utf8");
  const version = cargo.match(/^version\s*=\s*"([^"]+)"/m)?.[1];
  if (version !== expected) {
    throw new Error(`${relative}: expected ${expected}, found ${version ?? "no package version"}`);
  }
}

const nativeDependencies = Object.entries(launcher.optionalDependencies ?? {});
if (nativeDependencies.length !== 6) {
  throw new Error(`${launcherPath}: expected exactly 6 native optional dependencies`);
}
for (const [name, version] of nativeDependencies) {
  if (!name.startsWith("@linguini/cli-") || version !== expected) {
    throw new Error(`${launcherPath}: ${name} must be pinned exactly to ${expected}`);
  }
}

const documentedInstallCommands = [
  ["README.md", `@linguini/cli@${expected}`, `linguini-cli --version ${expected}`],
  ["docs/getting-started.md", `@linguini/cli@${expected}`, `linguini-cli --version ${expected}`]
];
for (const [relative, ...required] of documentedInstallCommands) {
  const source = await readFile(path.join(root, relative), "utf8");
  for (const value of required) {
    if (!source.includes(value)) {
      throw new Error(`${relative}: missing release-synchronized text ${JSON.stringify(value)}`);
    }
  }
}

const extensionLockPath = "editors/vscode/package-lock.json";
const extensionLock = JSON.parse(await readFile(path.join(root, extensionLockPath), "utf8"));
if (extensionLock.version !== expected || extensionLock.packages?.[""]?.version !== expected) {
  throw new Error(`${extensionLockPath}: root package versions must both equal ${expected}`);
}

const readme = await readFile(path.join(root, "README.md"), "utf8");
if (!readme.includes("status-preview") || !readme.includes("very early stage of development")) {
  throw new Error("README.md: public release status must remain explicitly preview/early-stage");
}

if (
  process.env.GITHUB_REF_TYPE === "tag" &&
  process.env.GITHUB_REF_NAME !== `v${expected}`
) {
  throw new Error(
    `release tag ${process.env.GITHUB_REF_NAME} does not match component version v${expected}`
  );
}
