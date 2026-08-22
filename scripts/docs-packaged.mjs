import { spawnSync } from "node:child_process";
import {
  copyFileSync,
  mkdirSync,
  mkdtempSync,
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
  "linguini-codegen-ts",
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
    ["package", "--offline", "--locked", "--list", "-p", crateName],
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

export function stagePackagedWorkspace(root, stageRoot) {
  for (const crateName of PACKAGED_RUNTIME_CRATES) {
    stageCratePackage(root, stageRoot, crateName);
  }
  const members = PACKAGED_RUNTIME_CRATES.map((name) => `  "crates/${name}",`).join("\n");
  writeFileSync(
    join(stageRoot, "Cargo.toml"),
    `[workspace]\nresolver = "2"\nmembers = [\n${members}\n]\n\n[workspace.package]\nedition = "2021"\nlicense = "Apache-2.0"\nrepository = "https://github.com/antepodeum/Linguini"\nrust-version = "1.76"\n`,
  );
  copyFileSync(join(root, "Cargo.lock"), join(stageRoot, "Cargo.lock"));
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
