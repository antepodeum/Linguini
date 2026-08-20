import { spawnSync } from "node:child_process";
import { mkdirSync, mkdtempSync, readFileSync, readdirSync, rmSync, writeFileSync } from "node:fs";
import { basename, dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const SCRIPT_ROOT = dirname(fileURLToPath(import.meta.url));
const REPOSITORY_ROOT = resolve(SCRIPT_ROOT, "..");
const OPENING_FENCE = /^```(lgs|lgl)(?:\s+(.+))?$/;
const FRAGMENT = /^fragment=([a-z0-9]+(?:-[a-z0-9]+)*)$/;

export function extractLinguiniFences(markdown, sourcePath) {
  const lines = markdown.split(/\r?\n/);
  const fences = [];
  let open;

  for (let index = 0; index < lines.length; index += 1) {
    const line = lines[index];
    if (!open) {
      const match = OPENING_FENCE.exec(line);
      if (match) {
        const metadata = match[2];
        const fragment = metadata === undefined ? undefined : FRAGMENT.exec(metadata)?.[1];
        if (metadata !== undefined && fragment === undefined) {
          throw new Error(`${sourcePath}:${index + 1}: unsupported Linguini fence metadata: ${metadata}`);
        }
        open = { language: match[1], line: index + 1, fragment, body: [] };
      }
      continue;
    }

    if (line === "```") {
      if (!open.body.some((bodyLine) => bodyLine.trim() !== "")) {
        throw new Error(`${sourcePath}:${open.line}: empty Linguini fence`);
      }
      fences.push(Object.freeze({
        language: open.language,
        line: open.line,
        fragment: open.fragment,
        source: `${open.body.join("\n")}\n`,
        sourcePath,
      }));
      open = undefined;
    } else {
      open.body.push(line);
    }
  }

  if (open) throw new Error(`${sourcePath}:${open.line}: unclosed Linguini fence`);
  return fences;
}

export function documentedLinguiniFences(root = REPOSITORY_ROOT) {
  const documents = [
    "README.md",
    ...readdirSync(resolve(root, "docs"), { recursive: true })
      .filter((sourcePath) => sourcePath.endsWith(".md"))
      .map((sourcePath) => join("docs", sourcePath))
      .sort(),
  ];
  return documents.flatMap((sourcePath) =>
    extractLinguiniFences(readFileSync(resolve(root, sourcePath), "utf8"), sourcePath),
  );
}

export function main({ root = REPOSITORY_ROOT, execute = spawnSync } = {}) {
  const fences = documentedLinguiniFences(root);
  const standalone = fences.filter((fence) => fence.fragment === undefined);
  const fragments = fences.filter((fence) => fence.fragment !== undefined);
  if (standalone.length === 0) throw new Error("documentation contains no standalone Linguini fences");

  const targetRoot = resolve(root, "target");
  mkdirSync(targetRoot, { recursive: true });
  const temporaryRoot = mkdtempSync(join(targetRoot, "docs-syntax-"));
  try {
    const paths = standalone.map((fence, index) => {
      const name = `${String(index + 1).padStart(3, "0")}-${basename(fence.sourcePath, ".md")}-${fence.line}.${fence.language}`;
      const path = join(temporaryRoot, name);
      writeFileSync(path, fence.source);
      return path;
    });
    const result = execute(
      "cargo",
      ["run", "--offline", "--locked", "--quiet", "-p", "linguini-cli", "--", "format", ...paths],
      { cwd: root, encoding: "utf8" },
    );
    if (result.error) throw result.error;
    if (result.status !== 0) {
      process.stdout.write(result.stdout ?? "");
      process.stderr.write(result.stderr ?? "");
      return result.status ?? 1;
    }
  } finally {
    rmSync(temporaryRoot, { recursive: true, force: true });
  }

  console.log(`Validated ${standalone.length} standalone Linguini documentation fences; registered ${fragments.length} fragments.`);
  return 0;
}

if (process.argv[1] && fileURLToPath(import.meta.url) === resolve(process.argv[1])) {
  process.exitCode = main();
}
