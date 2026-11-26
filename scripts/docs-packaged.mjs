import { spawnSync } from "node:child_process";
import {
  copyFileSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { dirname, join, relative, resolve, sep } from "node:path";
import { fileURLToPath } from "node:url";

import { main as runDocumentedExamples } from "./docs-examples.mjs";

const SCRIPT_ROOT = dirname(fileURLToPath(import.meta.url));
const REPOSITORY_ROOT = resolve(SCRIPT_ROOT, "..");

export const PACKAGED_RUNTIME_CRATES = Object.freeze([
  "linguini-analyzer",
  "linguini-cldr",
  "linguini-cli",
  "linguini-codegen",
  "linguini-codegen-ecmascript",
  "linguini-codegen-svelte",
  "linguini-codegen-vite",
  "linguini-config",
  "linguini-core",
  "linguini-format",
  "linguini-ir",
  "linguini-lsp",
  "linguini-schema",
  "linguini-syntax",
]);

function run(command, args, options) {
  const result = spawnSync(command, args, { encoding: "utf8", ...options });
  if (result.error) throw result.error;
  if (result.status !== 0) {
    process.stdout.write(result.stdout ?? "");
    process.stderr.write(result.stderr ?? "");
    throw new Error(`${command} exited with status ${result.status}`);
  }
  return result.stdout;
}

function assertContained(root, path) {
  const pathFromRoot = relative(root, path);
  if (pathFromRoot === "" || pathFromRoot === ".." || pathFromRoot.startsWith(`..${sep}`)) {
    throw new Error(`packaged path escapes crate root: ${path}`);
  }
}

function stageCratePackage(root, stageRoot, crateName) {
  const crateRoot = resolve(root, "crates", crateName);
  const destinationRoot = resolve(stageRoot, "crates", crateName);
  const packageFiles = run(
    "cargo",
    ["package", "--offline", "--locked", "--allow-dirty", "--list", "-p", crateName],
    { cwd: root },
  )
    .trimEnd()
    .split("\n")
    .filter(Boolean);

  for (const packagedPath of packageFiles) {
    if ([".cargo_vcs_info.json", "Cargo.lock", "Cargo.toml.orig"].includes(packagedPath)) continue;
    const source = resolve(crateRoot, packagedPath);
    assertContained(crateRoot, source);
    const destination = resolve(destinationRoot, packagedPath);
    assertContained(destinationRoot, destination);
    mkdirSync(dirname(destination), { recursive: true });
    copyFileSync(source, destination);
  }

  if (!packageFiles.includes("Cargo.toml")) {
    throw new Error(`${crateName}: package does not contain Cargo.toml`);
  }
}

export function workspacePackageMetadata(manifest) {
  const section = /^\[workspace\.package\][\s\S]*?(?=^\[|$(?![\s\S]))/m.exec(manifest)?.[0];
  if (!section) throw new Error("workspace package metadata is required for packaged documentation");
  return section.trimEnd();
}

export function packagedWorkspaceCrates(packages, roots = PACKAGED_RUNTIME_CRATES) {
  const byName = new Map(packages.map(pkg => [pkg.name, pkg]));
  const byPath = new Map(packages.map(pkg => [resolve(dirname(pkg.manifest_path)), pkg.name]));
  const selected = new Set();
  const pending = [...roots];
  while (pending.length) {
    const name = pending.pop();
    if (selected.has(name)) continue;
    const pkg = byName.get(name);
    if (!pkg) throw new Error(`missing packaged workspace crate: ${name}`);
    selected.add(name);
    // Cargo loads path dependencies from original manifests even when they are dev-only.
    for (const dependency of pkg.dependencies) {
      if (!dependency.path) continue;
      const target = byPath.get(resolve(dependency.path));
      if (!target) throw new Error(`packaged dependency is outside the workspace: ${dependency.path}`);
      pending.push(target);
    }
  }
  return [...selected].sort();
}

export function verifyPackagedDependencies(original, staged) {
  const locked = new Set(original.filter(pkg => pkg.source).map(pkg => pkg.id));
  for (const pkg of staged) {
    if (pkg.source && !locked.has(pkg.id)) {
      throw new Error(`packaged dependency is absent from the workspace lock: ${pkg.id}`);
    }
  }
}

export function stagePackagedWorkspace(root, stageRoot) {
  const metadata = JSON.parse(run("cargo", ["metadata", "--offline", "--locked", "--format-version", "1"], {cwd: root}));
  const crates = packagedWorkspaceCrates(metadata.packages);
  for (const crateName of crates) {
    stageCratePackage(root, stageRoot, crateName);
  }
  const members = crates.map((name) => `  "crates/${name}",`).join("\n");
  const packageMetadata = workspacePackageMetadata(readFileSync(join(root, "Cargo.toml"), "utf8"));
  writeFileSync(
    join(stageRoot, "Cargo.toml"),
    `[workspace]\nresolver = "2"\nmembers = [\n${members}\n]\n\n${packageMetadata}\n`,
  );
  copyFileSync(join(root, "Cargo.lock"), join(stageRoot, "Cargo.lock"));
  // Prune unrelated workspace members (such as the WASM transport) in the isolated lock.
  // Every retained external version must still come from the original locked resolution.
  const staged = JSON.parse(run("cargo", ["metadata", "--offline", "--format-version", "1"], {cwd: stageRoot}));
  verifyPackagedDependencies(metadata.packages, staged.packages);
}

export async function main({ root = REPOSITORY_ROOT } = {}) {
  const targetRoot = resolve(root, "target");
  mkdirSync(targetRoot, { recursive: true });
  const stageRoot = mkdtempSync(join(targetRoot, "docs-packaged-"));
  try {
    stagePackagedWorkspace(root, stageRoot);
    await runDocumentedExamples({
      root,
      cliManifestPath: join(stageRoot, "Cargo.toml"),
      cargoTargetDir: join(targetRoot, "docs-packaged-build"),
    });
  } finally {
    rmSync(stageRoot, { recursive: true, force: true });
  }
  console.log("Built documentation with the packaged CLI and codegen source artifacts.");
  return 0;
}

if (process.argv[1] && fileURLToPath(import.meta.url) === resolve(process.argv[1])) {
  main().then(
    (code) => {
      process.exitCode = code;
    },
    (error) => {
      console.error(error);
      process.exitCode = 1;
    },
  );
}
