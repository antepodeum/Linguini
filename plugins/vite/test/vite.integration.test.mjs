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

test("real Vite build bundles finite v3 dynamic refs without eager barrel", async (context) => {
  const root = await mkdtemp(path.join(tmpdir(), "linguini-vite-dynamic-"));
  context.after(() => rm(root, { recursive: true, force: true }));
  const generated = path.join(root, "generated/linguini");
  const entry = path.join(root, "entry.js");
  const source = [
    'import { l } from "./generated/linguini/svelte.ts";',
    "export const fixed = l.main.title;",
    "export const selected = l.main[key];",
    "export const called = l.main[action](2);",
    ""
  ].join("\n");
  await mkdir(path.join(generated, "bundler/messages/main/title"), { recursive: true });
  await mkdir(path.join(generated, "bundler/messages/main/items"), { recursive: true });
  await writeFile(
    path.join(root, "linguini.toml"),
    [
      "[targets.ts]",
      'out = "generated/linguini"',
      'framework = "svelte"',
      "[targets.ts.bundler]",
      'sources = ["entry.js"]',
      "[targets.ts.bundler.dynamic]",
      'mode = "bundle"',
      'allow = ["main.items", "main.title"]',
      ""
    ].join("\n")
  );
  await writeFile(entry, source);
  await writeFile(
    path.join(generated, "svelte.ts"),
    'import "./index.ts"; throw new Error("FORBIDDEN_DYNAMIC_SVELTE");\n'
  );
  await writeFile(
    path.join(generated, "index.ts"),
    'throw new Error("FORBIDDEN_DYNAMIC_INDEX");\n'
  );
  await writeFile(
    path.join(generated, "svelte-locale.svelte.ts"),
    'export function getCurrentLocale() { return "en"; }\n'
  );
  const titleModule = path.join(generated, "bundler/messages/main/title/en.ts");
  const itemsModule = path.join(generated, "bundler/messages/main/items/en.ts");
  await writeFile(
    titleModule,
    'export function message() { return "EXACT_DYNAMIC_TITLE"; }\n'
  );
  await writeFile(
    itemsModule,
    'export function message(count) { return `EXACT_DYNAMIC_ITEMS:${count}`; }\n'
  );
  const declarationText = 'import { l } from "./generated/linguini/svelte.ts";';
  const declarationStart = 0;
  const declarationEnd = Buffer.byteLength(declarationText);
  const itemStart = Buffer.from(source).indexOf(Buffer.from("l"));
  const bindingId = "entry:binding";
  const reference = (needle, message, kind, arity) => {
    const start = Buffer.from(source).indexOf(Buffer.from(needle));
    return {
      message,
      start,
      end: start + Buffer.byteLength(needle),
      kind,
      local: "l",
      provenance: {
        kind: "imported",
        module_specifier: "./generated/linguini/svelte.ts",
        symbol: "l"
      },
      arity,
      binding_id: bindingId
    };
  };
  const dynamicReference = (needle, key, referenceKind) => {
    const bytes = Buffer.from(source);
    const start = bytes.indexOf(Buffer.from(needle));
    const end = start + Buffer.byteLength(needle);
    const receiverEnd = start + Buffer.byteLength("l.main");
    const keyStart = bytes.indexOf(Buffer.from(key), receiverEnd);
    return {
      kind: "computed",
      prefix: "main",
      span: { start, end },
      receiver_span: { start, end: receiverEnd },
      key_span: { start: keyStart, end: keyStart + Buffer.byteLength(key) },
      reference_kind: referenceKind,
      local: "l",
      provenance: {
        kind: "imported",
        module_specifier: "./generated/linguini/svelte.ts",
        symbol: "l"
      },
      binding_id: bindingId,
      messages: [
        { message: "main.items", key: "items", arity: 1 },
        { message: "main.title", key: "title", arity: 0 }
      ]
    };
  };
  const manifest = {
    version: 3,
    base_locale: "en",
    configured_locales: ["en"],
    effective_locales: ["en"],
    sources: [{ id: 1, path: "linguini/schema/main.lgs" }],
    runtime_helpers: {
      svelte_locale: {
        import: "./svelte-locale.svelte.js",
        file: "svelte-locale.svelte.ts"
      }
    },
    messages: {
      "main.items": {
        arity: 1,
        locales: {
          en: { module: "bundler/messages/main/items/en.ts", source_ids: [1] }
        }
      },
      "main.title": {
        arity: 0,
        locales: {
          en: { module: "bundler/messages/main/title/en.ts", source_ids: [1] }
        }
      }
    },
    applications: {
      "entry.js": {
        source_id: 2147483648,
        sha256: createHash("sha256").update(Buffer.from(source)).digest("hex"),
        byte_length: Buffer.byteLength(source),
        unresolved: [],
        analysis_dynamic_prefixes: ["main"],
        references: [reference("l.main.title", "main.title", "value", 0)],
        dynamic_references: [
          dynamicReference("l.main[key]", "key", "value"),
          dynamicReference("l.main[action]", "action", "call")
        ],
        imports: [
          {
            binding_id: bindingId,
            module_specifier: "./generated/linguini/svelte.ts",
            imported: "l",
            local: "l",
            declaration_start: declarationStart,
            declaration_end: declarationEnd,
            item_start: itemStart,
            item_end: itemStart + 1,
            removal_start: declarationStart,
            removal_end: declarationEnd,
            module_specifier_start: declarationStart,
            module_specifier_end: declarationEnd,
            imported_start: itemStart,
            imported_end: itemStart + 1,
            local_start: itemStart,
            local_end: itemStart + 1,
            analyzer_exact_uses_only: false,
            analyzer_tracked_uses_only: true,
            transformable: true
          }
        ]
      }
    }
  };
  await mkdir(path.join(generated, "bundler"), { recursive: true });
  await writeFile(
    path.join(generated, "bundler/manifest.json"),
    JSON.stringify(manifest)
  );

  const result = await build({
    root,
    logLevel: "silent",
    plugins: [linguini({ root, buildOnStart: false })],
    build: {
      write: false,
      rollupOptions: { input: entry }
    }
  });
  const chunks = result.output.filter((output) => output.type === "chunk");
  const combined = chunks.map((chunk) => chunk.code).join("\n");
  assert.match(combined, /EXACT_DYNAMIC_TITLE/);
  assert.match(combined, /EXACT_DYNAMIC_ITEMS/);
  assert.match(combined, /Object\.create\(null\)/);
  assert.doesNotMatch(
    combined,
    /FORBIDDEN_DYNAMIC_SVELTE|FORBIDDEN_DYNAMIC_INDEX|virtual:linguini/
  );
  const modules = chunks.flatMap((chunk) => Object.keys(chunk.modules));
  assert.ok(modules.includes(titleModule));
  assert.ok(modules.includes(itemsModule));
  assert.ok(modules.filter((id) => id.includes("\0virtual:linguini/message/")).length >= 2);
});

test("real v4 dynamic client boundaries keep inactive locales out of entry and SSR stays static", async (context) => {
  const root = await mkdtemp(path.join(tmpdir(), "linguini-vite-v4-dynamic-"));
  context.after(() => rm(root, { recursive: true, force: true }));
  const generated = path.join(root, "generated/linguini");
  await mkdir(path.join(generated, "bundler/messages/main/title"), { recursive: true });
  await mkdir(path.join(generated, "bundler"), { recursive: true });
  await writeFile(
    path.join(root, "linguini.toml"),
    [
      "[targets.ts]",
      'out = "generated/linguini"',
      'framework = "svelte"',
      "[targets.ts.bundler]",
      'sources = ["entry.js"]',
      'locale_loading = "dynamic"',
      ""
    ].join("\n")
  );
  const virtualMessage = "virtual:linguini/message/6d61696e2e7469746c65";
  await writeFile(
    path.join(root, "entry.js"),
    `import { message } from ${JSON.stringify(virtualMessage)};\nexport const rendered = message();\n`
  );
  await writeFile(
    path.join(generated, "svelte-locale.js"),
    [
      'export function getCurrentLocale() { return "en"; }',
      "export function registerLocaleLoader() { return () => {}; }",
      ""
    ].join("\n")
  );
  await writeFile(
    path.join(generated, "bundler/messages/main/title/en.js"),
    'export function message() { return "V4_EN"; }\n'
  );
  await writeFile(
    path.join(generated, "bundler/messages/main/title/fr.js"),
    'export function message() { return "V4_FR"; }\n'
  );
  await writeFile(
    path.join(generated, "bundler/manifest.json"),
    JSON.stringify({
      version: 4,
      locale_loading: "dynamic",
      base_locale: "en",
      configured_locales: ["en", "fr"],
      effective_locales: ["en", "fr"],
      sources: [],
      runtime_helpers: {
        svelte_locale: {
          import: "./svelte-locale.js",
          file: "svelte-locale.js"
        }
      },
      messages: {
        "main.title": {
          arity: 0,
          locales: {
            en: {
              module: "bundler/messages/main/title/en.js",
              source_ids: []
            },
            fr: {
              module: "bundler/messages/main/title/fr.js",
              source_ids: []
            }
          }
        }
      },
      applications: {}
    })
  );

  const clientResult = await build({
    root,
    logLevel: "silent",
    plugins: [linguini({ root, buildOnStart: false })],
    build: {
      write: false,
      rollupOptions: { input: path.join(root, "entry.js") }
    }
  });
  const clientChunks = clientResult.output.filter((output) => output.type === "chunk");
  const clientEntry = clientChunks.find((chunk) => chunk.isEntry);
  assert.ok(clientEntry);
  assert.match(clientChunks.map((chunk) => chunk.code).join("\n"), /V4_EN|V4_FR/);
  assert.ok(clientChunks.some((chunk) => chunk.code.includes("V4_FR")));
  assert.doesNotMatch(clientEntry.code, /V4_FR/);
  assert.equal(
    clientEntry.imports.some((file) => file.includes("fr-")),
    false,
    "inactive locale must not be a static entry import"
  );

  const ssrResult = await build({
    root,
    logLevel: "silent",
    plugins: [linguini({ root, buildOnStart: false })],
    build: {
      write: false,
      ssr: path.join(root, "entry.js"),
      rollupOptions: { input: path.join(root, "entry.js") }
    }
  });
  const ssrCode = ssrResult.output
    .filter((output) => output.type === "chunk")
    .map((chunk) => chunk.code)
    .join("\n");
  assert.match(ssrCode, /V4_EN/);
  assert.match(ssrCode, /V4_FR/);
  assert.doesNotMatch(ssrCode, /registerLocaleLoader|import\(/);
  assert.match(ssrCode, /rendered/);
});
