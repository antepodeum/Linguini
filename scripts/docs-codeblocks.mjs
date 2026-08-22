import { readFileSync, readdirSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const SCRIPT_ROOT = dirname(fileURLToPath(import.meta.url));
const REPOSITORY_ROOT = resolve(SCRIPT_ROOT, "..");
const OPENING_FENCE = /^```([a-z0-9-]*)(?:\s+(.+))?$/;
const FRAGMENT = /^fragment=([a-z0-9]+(?:-[a-z0-9]+)*)$/;
const LANGUAGES = new Set(["", "bash", "html", "lgl", "lgs", "sh", "svelte", "text", "toml", "ts"]);

export function extractCodeblocks(markdown, sourcePath) {
  const lines = markdown.split(/\r?\n/);
  const blocks = [];
  let open;

  for (let index = 0; index < lines.length; index += 1) {
    const line = lines[index];
    if (!open) {
      const match = OPENING_FENCE.exec(line);
      if (!match) continue;
      const language = match[1];
      if (!LANGUAGES.has(language)) {
        throw new Error(`${sourcePath}:${index + 1}: unsupported documentation fence language: ${language}`);
      }
      const metadata = match[2];
      const fragment = metadata === undefined ? undefined : FRAGMENT.exec(metadata)?.[1];
      if (metadata !== undefined && fragment === undefined) {
        throw new Error(`${sourcePath}:${index + 1}: unsupported documentation fence metadata: ${metadata}`);
      }
      open = { language: language || "text", line: index + 1, fragment, body: [] };
      continue;
    }

    if (line === "```") {
      if (!open.body.some((bodyLine) => bodyLine.trim() !== "")) {
        throw new Error(`${sourcePath}:${open.line}: empty documentation fence`);
      }
      blocks.push(Object.freeze({
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

  if (open) throw new Error(`${sourcePath}:${open.line}: unclosed documentation fence`);
  return blocks;
}

export function documentationFiles(root = REPOSITORY_ROOT) {
  return [
    "README.md",
    ...readdirSync(resolve(root, "docs"), { recursive: true })
      .filter((sourcePath) => sourcePath.endsWith(".md"))
      .map((sourcePath) => join("docs", sourcePath))
      .sort(),
  ];
}

export function documentedCodeblocks(root = REPOSITORY_ROOT) {
  return documentationFiles(root).flatMap((sourcePath) =>
    extractCodeblocks(readFileSync(resolve(root, sourcePath), "utf8"), sourcePath),
  );
}

export function main({ root = REPOSITORY_ROOT } = {}) {
  const blocks = documentedCodeblocks(root);
  if (blocks.length === 0) throw new Error("documentation contains no code blocks");
  const counts = new Map();
  for (const block of blocks) counts.set(block.language, (counts.get(block.language) ?? 0) + 1);
  const summary = [...counts.entries()]
    .sort(([left], [right]) => left.localeCompare(right))
    .map(([language, count]) => `${language}:${count}`)
    .join(", ");
  console.log(`Registered ${blocks.length} documentation code blocks (${summary}).`);
  return 0;
}

if (process.argv[1] && fileURLToPath(import.meta.url) === resolve(process.argv[1])) {
  process.exitCode = main();
}
