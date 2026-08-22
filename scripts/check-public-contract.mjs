import { spawnSync } from "node:child_process";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const SCRIPT_ROOT = dirname(fileURLToPath(import.meta.url));
const REPOSITORY_ROOT = resolve(SCRIPT_ROOT, "..");

const PUBLIC_CONTRACT_PATTERNS = [
  /^crates\/linguini-cli\/src\/lib\.rs$/,
  /^crates\/linguini-codegen-ts\/src\/(?:lib\.rs|module\/(?:decl|project|signature)\.rs|module\/templates\/)/,
  /^crates\/linguini-config\/src\/(?:lib|model|parser)\.rs$/,
  /^crates\/linguini-core\/src\/lib\.rs$/,
  /^crates\/linguini-lsp\/src\/(?:lib|server)\.rs$/,
  /^crates\/linguini-syntax\/src\/(?:ast|lib|parser)\.rs$/,
  /^editors\/vscode\/src\//,
  /^packages\/cli\/(?:bin|lib)\//,
  /^plugins\/vite\/src\//,
];

export function contractEvidence(paths) {
  const normalized = [...new Set(paths.map((path) => path.replaceAll("\\", "/")))];
  return Object.freeze({
    contracts: normalized.filter((path) => PUBLIC_CONTRACT_PATTERNS.some((pattern) => pattern.test(path))),
    docs: normalized.filter((path) => path === "README.md" || /^docs\/.*\.md$/.test(path)),
    migrations: normalized.filter((path) => path === "docs/migrations.md"),
    tests: normalized.filter(
      (path) =>
        /(?:^|\/)(?:tests?|fixtures)(?:\/|\.|$)/.test(path) ||
        /(?:\.test\.[cm]?[jt]s|snapshots?\/|\.snap$)/.test(path),
    ),
  });
}

export function validateContractEvidence(paths) {
  const evidence = contractEvidence(paths);
  if (evidence.contracts.length === 0) return evidence;
  const missing = [];
  if (evidence.docs.length === 0) missing.push("public documentation");
  if (evidence.migrations.length === 0) missing.push("docs/migrations.md");
  if (evidence.tests.length === 0) missing.push("tests or fixtures");
  if (missing.length > 0) {
    throw new Error(
      `public contract changes require ${missing.join(", ")}; changed contracts: ${evidence.contracts.join(", ")}`,
    );
  }
  return evidence;
}

function changedPaths(root, base) {
  const range = base ? [`${base}...HEAD`] : ["HEAD^", "HEAD"];
  const result = spawnSync("git", ["diff", "--name-only", "--diff-filter=ACMRT", ...range], {
    cwd: root,
    encoding: "utf8",
  });
  if (result.error) throw result.error;
  if (result.status !== 0) {
    process.stderr.write(result.stderr ?? "");
    throw new Error(`git diff exited with status ${result.status}`);
  }
  return result.stdout.split(/\r?\n/).filter(Boolean);
}

export function main({ root = REPOSITORY_ROOT, base = process.env.PUBLIC_CONTRACT_BASE } = {}) {
  const paths = changedPaths(root, base);
  const evidence = validateContractEvidence(paths);
  console.log(
    evidence.contracts.length === 0
      ? "No public contract changes require companion evidence."
      : `Validated ${evidence.contracts.length} public contract files with docs, migration notes, and tests.`,
  );
  return 0;
}

if (process.argv[1] && fileURLToPath(import.meta.url) === resolve(process.argv[1])) {
  try {
    process.exitCode = main();
  } catch (error) {
    console.error(error.message);
    process.exitCode = 1;
  }
}
