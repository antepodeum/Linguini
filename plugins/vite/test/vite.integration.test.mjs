import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { mkdir, mkdtemp, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import path from "node:path";
import test from "node:test";
import { build } from "vite";
import { linguini } from "../src/index.js";

test("runs through a real Vite build", async (context) => {
  const root = await mkdtemp(path.join(tmpdir(), "linguini-vite-integration-"));
  context.after(() => rm(root, { recursive: true, force: true }));
  await mkdir(path.join(root, "linguini/schema"), { recursive: true });
  await mkdir(path.join(root, "linguini/locale"), { recursive: true });
  await writeFile(path.join(root, "linguini.toml"), '[project]\nname = "integration"\n');
  await writeFile(path.join(root, "linguini/schema/main.lgs"), "hello\n");
  await writeFile(path.join(root, "linguini/locale/en.lgl"), "hello = Hello\n");
  await writeFile(
    path.join(root, "index.html"),
    '<!doctype html><script type="module" src="/main.js"></script>'
  );
  await writeFile(path.join(root, "main.js"), 'document.body.dataset.ready = "true";\n');

  const reasons = [];
  await build({
    root,
    logLevel: "silent",
    plugins: [
      linguini({
        debounceMs: 0,
        build({ reason }) {
          reasons.push(reason);
        }
      })
    ],
    build: {
      outDir: "dist"
    }
  });

  assert.deepEqual(reasons, ["build-start"]);
});

test("real multi-entry build owns exact messages in shared and route chunks", async (context) => {
  const root = await mkdtemp(path.join(tmpdir(), "linguini-vite-bundler-"));
  context.after(() => rm(root, { recursive: true, force: true }));
  const generated = path.join(root, "generated/linguini");
  await mkdir(path.join(generated, "bundler/messages/main/title"), { recursive: true });
  await mkdir(path.join(generated, "bundler/messages/main/items"), { recursive: true });
  await writeFile(
    path.join(root, "linguini.toml"),
    [
      "[project]",
      'name = "bundler-integration"',
      'default_locale = "en"',
      'locales = ["en"]',
      "[targets.ts]",
      'out = "generated/linguini"',
      'framework = "svelte"',
      "[targets.ts.bundler]",
      'sources = ["entry-one.js", "entry-two.js"]',
      ""
    ].join("\n")
  );
  await writeFile(
    path.join(generated, "svelte.ts"),
    'import "./index.ts"; throw new Error("FORBIDDEN_SVELTE");\n'
  );
  await writeFile(path.join(generated, "index.ts"), 'throw new Error("FORBIDDEN_INDEX");\n');
  await writeFile(
    path.join(generated, "svelte-locale.svelte.ts"),
    'export function getCurrentLocale() { return "en"; }\n'
  );
  await writeFile(
    path.join(generated, "svelte-effects.svelte.ts"),
    "globalThis.__linguiniEffects = true;\n"
  );
  const titleModule = path.join(generated, "bundler/messages/main/title/en.ts");
  const itemsModule = path.join(generated, "bundler/messages/main/items/en.ts");
  await writeFile(titleModule, 'export function message() { return "EXACT_TITLE"; }\n');
  await writeFile(itemsModule, 'export function message(count) { return `EXACT_ITEMS:${count}`; }\n');
  const sources = {
    "entry-one.js": [
      'import { l } from "./generated/linguini/svelte.ts";',
      "export const title = l.main.title;",
      "export const items = l.main.items(2);",
      ""
    ].join("\n"),
    "entry-two.js": [
      'import { messages as l } from "./generated/linguini/svelte.ts";',
      "export const title = l.main.title;",
      ""
    ].join("\n")
  };
  const applications = {};
  for (const [file, source] of Object.entries(sources)) {
    await writeFile(path.join(root, file), source);
    const declarationText = source.slice(0, source.indexOf("\n"));
    const declarationStart = 0;
    const declarationEnd = Buffer.byteLength(declarationText);
    const imported = file === "entry-one.js" ? "l" : "messages";
    const itemText = file === "entry-one.js" ? "l" : "messages as l";
    const itemStart = Buffer.from(source).indexOf(Buffer.from(itemText));
    const refs = [];
    for (const [needle, message, kind, arity] of [
      ["l.main.title", "main.title", "value", 0],
      ["l.main.items", "main.items", "call", 1]
    ]) {
      const start = Buffer.from(source).indexOf(Buffer.from(needle));
      if (start >= 0) {
        refs.push({
          message,
          start,
          end: start + Buffer.byteLength(needle),
          kind,
          local: "l",
          provenance: {
            kind: "imported",
            module_specifier: "./generated/linguini/svelte.ts",
            symbol: imported
          },
          arity,
          binding_id: `${file}:binding`
        });
      }
    }
    applications[file] = {
      source_id: 2147483648 + Object.keys(applications).length,
      sha256: createHash("sha256").update(Buffer.from(source)).digest("hex"),
      byte_length: Buffer.byteLength(source),
      unresolved: [],
      analysis_dynamic_prefixes: [],
      references: refs,
      imports: [
        {
          binding_id: `${file}:binding`,
          module_specifier: "./generated/linguini/svelte.ts",
          imported,
          local: "l",
          declaration_start: declarationStart,
          declaration_end: declarationEnd,
          item_start: itemStart,
          item_end: itemStart + Buffer.byteLength(itemText),
          removal_start: declarationStart,
          removal_end: declarationEnd,
          module_specifier_start: declarationStart,
          module_specifier_end: declarationEnd,
          imported_start: itemStart,
          imported_end: itemStart + imported.length,
          local_start: itemStart + itemText.length - 1,
          local_end: itemStart + itemText.length,
          analyzer_exact_uses_only: true,
          transformable: true
        }
      ]
    };
  }
  const manifest = {
    version: 2,
    base_locale: "en",
    configured_locales: ["en"],
    effective_locales: ["en"],
    sources: [{ id: 1, path: "linguini/schema/main.lgs" }],
    runtime_helpers: {
      svelte_locale: { import: "./svelte-locale.svelte.js", file: "svelte-locale.svelte.ts" },
      svelte_effects: { import: "./svelte-effects.svelte.js", file: "svelte-effects.svelte.ts" }
    },
    messages: {
      "main.title": {
        arity: 0,
        locales: {
          en: { module: "bundler/messages/main/title/en.ts", source_ids: [1] }
        }
      },
      "main.items": {
        arity: 1,
        locales: {
          en: { module: "bundler/messages/main/items/en.ts", source_ids: [1] }
        }
      }
    },
    applications
  };
  await writeFile(path.join(generated, "bundler/manifest.json"), JSON.stringify(manifest));

  const result = await build({
    root,
    logLevel: "silent",
    plugins: [linguini({ root, buildOnStart: false })],
    build: {
      write: false,
      rollupOptions: {
        input: {
          one: path.join(root, "entry-one.js"),
          two: path.join(root, "entry-two.js")
        }
      }
    }
  });
  const chunks = result.output.filter((output) => output.type === "chunk");
  const combined = chunks.map((chunk) => chunk.code).join("\n");
  assert.match(combined, /EXACT_TITLE/);
  assert.match(combined, /EXACT_ITEMS/);
  assert.match(combined, /__linguiniEffects/);
  assert.doesNotMatch(combined, /FORBIDDEN_SVELTE|FORBIDDEN_INDEX|virtual:linguini/);
  const titleOwner = chunks.find((chunk) => Object.keys(chunk.modules).includes(titleModule));
  const itemsOwner = chunks.find((chunk) => Object.keys(chunk.modules).includes(itemsModule));
  assert.ok(titleOwner && !titleOwner.isEntry, "shared title message must own a shared chunk");
  assert.ok(itemsOwner?.isEntry, "route-only items message must stay with one entry");
  assert.ok(Object.keys(titleOwner.modules).some((id) => id.includes("\0virtual:linguini/message/")));
});
