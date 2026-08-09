import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { mkdir, mkdtemp, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import path from "node:path";
import test from "node:test";
import { pathToFileURL } from "node:url";
import {
  discoverLinguiniFiles,
  isLinguiniSource,
  linguini,
  readProjectLayout
} from "../src/index.js";

async function fixture() {
  const root = await mkdtemp(path.join(tmpdir(), "linguini-vite-"));
  await mkdir(path.join(root, "src/schema/shop"), { recursive: true });
  await mkdir(path.join(root, "src/locale/shop"), { recursive: true });
  await writeFile(
    path.join(root, "linguini.toml"),
    [
      "# Standard TOML is parsed, not matched with regular expressions.",
      "[paths]",
      'schema = "src/schema"',
      'locale = "src/locale"',
      "",
      "[targets.ts]",
      'out = "build/custom-linguini"',
      ""
    ].join("\n")
  );
  await writeFile(path.join(root, "src/schema/shop/delivery.lgs"), "delivery()\n");
  await writeFile(path.join(root, "src/locale/shop/ru.lgl"), "delivery = OK\n");
  return root;
}

async function dynamicLocaleFixture({
  version = 4,
  configLocaleLoading = "eager",
  manifestLocaleLoading = configLocaleLoading,
  includeManifestLocaleLoading = version >= 4
} = {}) {
  const root = await mkdtemp(path.join(tmpdir(), "linguini-vite-locale-"));
  const generated = path.join(root, "build/custom-linguini");
  await mkdir(path.join(root, "src"), { recursive: true });
  await mkdir(path.join(generated, "bundler/messages/main/title"), { recursive: true });
  await writeFile(
    path.join(root, "linguini.toml"),
    [
      "[targets.ts]",
      'out = "build/custom-linguini"',
      "[targets.ts.bundler]",
      'sources = ["src"]',
      `locale_loading = "${configLocaleLoading}"`,
      ""
    ].join("\n")
  );
  await writeFile(
    path.join(generated, "svelte-locale.js"),
    [
      'let current = "en";',
      "const loaders = new Set();",
      "export function getCurrentLocale() { return current; }",
      "export function setCurrentLocale(locale) { current = locale; }",
      "export function registerLocaleLoader(loader) {",
      "  loaders.add(loader);",
      "  let disposed = false;",
      "  return () => {",
      "    if (disposed) return;",
      "    disposed = true;",
      "    loaders.delete(loader);",
      "    globalThis.__linguiniDisposed = (globalThis.__linguiniDisposed ?? 0) + 1;",
      "  };",
      "}",
      "export async function prepareLocale(locale) {",
      "  await Promise.all([...loaders].map((loader) => loader(locale)));",
      "}"
    ].join("\n")
  );
  await writeFile(
    path.join(generated, "bundler/messages/main/title/en.js"),
    'export function message() { return "EN"; }\n'
  );
  await writeFile(
    path.join(generated, "bundler/messages/main/title/fr.js"),
    'export function message() { return "FR"; }\n'
  );
  const manifest = {
    version,
    ...(includeManifestLocaleLoading ? { locale_loading: manifestLocaleLoading } : {}),
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
  };
  if (version >= 5) {
    manifest.message_runtimes = {
      en: { module: "locales/en/_runtime.js", source_ids: [] },
      fr: { module: "locales/fr/_runtime.js", source_ids: [] }
    };
    for (const locale of ["en", "fr"]) {
      await mkdir(path.join(generated, `locales/${locale}`), { recursive: true });
      await writeFile(path.join(generated, `locales/${locale}/_runtime.js`), "export {};\n");
    }
  }
  await mkdir(path.join(generated, "bundler"), { recursive: true });
  const manifestPath = path.join(generated, "bundler/manifest.json");
  await writeFile(manifestPath, JSON.stringify(manifest));
  return { root, generated, manifestPath, manifest };
}

function byteSpan(source, needle) {
  const bytes = Buffer.from(source, "utf8");
  const start = bytes.indexOf(Buffer.from(needle, "utf8"));
  assert.notEqual(start, -1, `missing fixture needle ${needle}`);
  return [start, start + Buffer.byteLength(needle)];
}

async function bundlerFixture({ version = 2, source, applicationName = "page.svelte" } = {}) {
  const root = await fixture();
  const generated = path.join(root, "build/custom-linguini");
  const applicationKey = `src/app/${applicationName}`;
  const application = path.join(root, applicationKey);
  const code =
    source ??
    '<script>\r\nimport { l as tr, helper } from "../../build/custom-linguini/svelte.ts";\r\nconst привет = tr.main.title;\r\n</script>\r\n';
  await mkdir(path.dirname(application), { recursive: true });
  await mkdir(path.join(generated, "bundler/messages/main/title"), { recursive: true });
  await writeFile(application, code);
  await writeFile(path.join(generated, "svelte.ts"), "export const l = {};\n");
  await writeFile(
    path.join(generated, "svelte-locale.svelte.ts"),
    'export function getCurrentLocale() { return "en"; }\n'
  );
  await writeFile(path.join(generated, "svelte-effects.svelte.ts"), "globalThis.effects = true;\n");
  await writeFile(
    path.join(generated, "bundler/messages/main/title/en.ts"),
    'export function message() { return "Title"; }\n'
  );
  const declarationText = code.includes("l as tr, helper")
    ? 'import { l as tr, helper } from "../../build/custom-linguini/svelte.ts";'
    : 'import { l as tr } from "../../build/custom-linguini/svelte.ts";';
  const declaration = byteSpan(code, declarationText);
  const item = byteSpan(code, "l as tr");
  const removal = code.includes("l as tr, ") ? byteSpan(code, "l as tr, ") : declaration;
  const reference = byteSpan(code, "tr.main.title");
  const manifest =
    version === 1
      ? {
          version: 1,
          base_locale: "en",
          configured_locales: ["en"],
          effective_locales: ["en"],
          sources: [
            { id: 1, path: "linguini/schema/main.lgs" },
            { id: 2, path: "linguini/locale/main/en.lgl" }
          ],
          messages: {}
        }
      : {
          version,
          base_locale: "en",
          configured_locales: ["en"],
          effective_locales: ["en"],
          sources: [
            { id: 1, path: "linguini/schema/main.lgs" },
            { id: 2, path: "linguini/locale/main/en.lgl" }
          ],
          runtime_helpers: {
            svelte_locale: {
              import: "./svelte-locale.svelte.js",
              file: "svelte-locale.svelte.ts"
            },
            svelte_effects: {
              import: "./svelte-effects.svelte.js",
              file: "svelte-effects.svelte.ts"
            }
          },
          messages: {
            "main.title": {
              arity: 0,
              locales: {
                en: {
                  module: "bundler/messages/main/title/en.ts",
                  source_ids: [1, 2]
                }
              }
            }
          },
          applications: {
            [applicationKey]: {
              source_id: 2147483648,
              sha256: createHash("sha256").update(Buffer.from(code)).digest("hex"),
              byte_length: Buffer.byteLength(code),
              unresolved: [],
              analysis_dynamic_prefixes: [],
              ...(version === 3 ? { dynamic_references: [] } : {}),
              references: [
                {
                  message: "main.title",
                  start: reference[0],
                  end: reference[1],
                  kind: "value",
                  local: "tr",
                  provenance: {
                    kind: "imported",
                    module_specifier: "../../build/custom-linguini/svelte.ts",
                    symbol: "l"
                  },
                  arity: 0,
                  binding_id: "binding"
                }
              ],
              imports: [
                {
                  binding_id: "binding",
                  module_specifier: "../../build/custom-linguini/svelte.ts",
                  imported: "l",
                  local: "tr",
                  declaration_start: declaration[0],
                  declaration_end: declaration[1],
                  item_start: item[0],
                  item_end: item[1],
                  removal_start: removal[0],
                  removal_end: removal[1],
                  module_specifier_start: declaration[0],
                  module_specifier_end: declaration[1],
                  imported_start: item[0],
                  imported_end: item[0] + 1,
                  local_start: item[1] - 2,
                  local_end: item[1],
                  analyzer_exact_uses_only: true,
                  ...(version === 3 ? { analyzer_tracked_uses_only: true } : {}),
                  transformable: true
                }
              ]
            }
          }
        };
  await mkdir(path.join(generated, "bundler"), { recursive: true });
  await writeFile(path.join(generated, "bundler/manifest.json"), JSON.stringify(manifest));
  return { root, generated, application, applicationKey, code, manifest };
}

async function v5RuntimeFixture() {
  const data = await bundlerFixture({ version: 3 });
  data.manifest.version = 5;
  data.manifest.locale_loading = "eager";
  data.manifest.sources[0].path = "src/schema/shop/delivery.lgs";
  data.manifest.sources[1].path = "src/locale/shop/ru.lgl";
  data.manifest.messages["main.title"].locales.en.source_ids = [1];
  data.manifest.message_runtimes = {
    en: { module: "locales/en/_runtime.ts", source_ids: [2] }
  };
  data.runtimeModule = path.join(data.generated, "locales/en/_runtime.ts");
  await mkdir(path.dirname(data.runtimeModule), { recursive: true });
  await writeFile(data.runtimeModule, "export {};\n");
  await writeFile(
    path.join(data.generated, "bundler/manifest.json"),
    JSON.stringify(data.manifest)
  );
  return data;
}

async function dynamicBundlerFixture({ applicationName = "dynamic.ts" } = {}) {
  const body = [
    'import { l as tr } from "../../build/custom-linguini/svelte.ts";',
    'import { messages as alt } from "../../build/custom-linguini/svelte.ts";',
    'const __linguini_dispatch_0 = "user-owned";',
    "export const fixed = tr.main.title;",
    "export const value = tr.main[ /* keep */ ключ ];",
    "export const called = tr.main[action](2);",
    "export const root = tr[rootKey];",
    "export const again = tr.main[keyAgain];",
    "export const proto = tr.main[protoKey];",
    "export const altValue = alt.main[altKey];",
    ""
  ].join("\n");
  const code = applicationName.endsWith(".svelte")
    ? `<script>\n${body}</script>\n`
    : body;
  const data = await bundlerFixture({ version: 3, source: code, applicationName });
  const declarationText =
    'import { l as tr } from "../../build/custom-linguini/svelte.ts";';
  const declaration = byteSpan(code, declarationText);
  const item = byteSpan(code, "l as tr");
  const binding = data.manifest.applications[data.applicationKey].imports[0];
  Object.assign(binding, {
    declaration_start: declaration[0],
    declaration_end: declaration[1],
    item_start: item[0],
    item_end: item[1],
    removal_start: declaration[0],
    removal_end: declaration[1],
    module_specifier_start: declaration[0],
    module_specifier_end: declaration[1],
    imported_start: item[0],
    imported_end: item[0] + 1,
    local_start: item[1] - 2,
    local_end: item[1],
    analyzer_exact_uses_only: false,
    analyzer_tracked_uses_only: true,
    transformable: true
  });
  const application = data.manifest.applications[data.applicationKey];
  const altDeclarationText =
    'import { messages as alt } from "../../build/custom-linguini/svelte.ts";';
  const altDeclaration = byteSpan(code, altDeclarationText);
  const altItem = byteSpan(code, "messages as alt");
  const altBinding = {
    ...binding,
    binding_id: "binding-alt",
    imported: "messages",
    local: "alt",
    declaration_start: altDeclaration[0],
    declaration_end: altDeclaration[1],
    item_start: altItem[0],
    item_end: altItem[1],
    removal_start: altDeclaration[0],
    removal_end: altDeclaration[1],
    module_specifier_start: altDeclaration[0],
    module_specifier_end: altDeclaration[1],
    imported_start: altItem[0],
    imported_end: altItem[0] + Buffer.byteLength("messages"),
    local_start: altItem[1] - Buffer.byteLength("alt"),
    local_end: altItem[1]
  };
  application.imports.push(altBinding);
  const staticSpan = byteSpan(code, "tr.main.title");
  application.references = [
    {
      message: "main.title",
      start: staticSpan[0],
      end: staticSpan[1],
      kind: "value",
      local: "tr",
      provenance: {
        kind: "imported",
        module_specifier: binding.module_specifier,
        symbol: "l"
      },
      arity: 0,
      binding_id: binding.binding_id
    }
  ];
  const dynamicReference = (
    needle,
    receiver,
    key,
    prefix,
    referenceKind,
    messages,
    local = "tr",
    importBinding = binding
  ) => {
    const span = byteSpan(code, needle);
    const receiverLength = Buffer.byteLength(receiver);
    const sourceBytes = Buffer.from(code, "utf8");
    const keyStart = sourceBytes.indexOf(Buffer.from(key), span[0] + receiverLength);
    assert.ok(keyStart >= 0 && keyStart < span[1]);
    return {
      kind: "computed",
      prefix,
      span: { start: span[0], end: span[1] },
      receiver_span: { start: span[0], end: span[0] + receiverLength },
      key_span: { start: keyStart, end: keyStart + Buffer.byteLength(key) },
      reference_kind: referenceKind,
      local,
      provenance: {
        kind: "imported",
        module_specifier: importBinding.module_specifier,
        symbol: importBinding.imported
      },
      binding_id: importBinding.binding_id,
      messages
    };
  };
  const mainMessages = [
    { message: "main.__proto__", key: "__proto__", arity: 0 },
    { message: "main.items", key: "items", arity: 1 },
    { message: "main.title", key: "title", arity: 0 }
  ];
  application.dynamic_references = [
    dynamicReference(
      "tr.main[ /* keep */ ключ ]",
      "tr.main",
      " /* keep */ ключ ",
      "main",
      "value",
      mainMessages
    ),
    dynamicReference(
      "tr.main[action]",
      "tr.main",
      "action",
      "main",
      "call",
      mainMessages
    ),
    dynamicReference(
      "tr[rootKey]",
      "tr",
      "rootKey",
      "",
      "value",
      [{ message: "notice", key: "notice", arity: 0 }]
    ),
    dynamicReference(
      "tr.main[keyAgain]",
      "tr.main",
      "keyAgain",
      "main",
      "value",
      mainMessages
    ),
    dynamicReference(
      "tr.main[protoKey]",
      "tr.main",
      "protoKey",
      "main",
      "value",
      mainMessages
    ),
    dynamicReference(
      "alt.main[altKey]",
      "alt.main",
      "altKey",
      "main",
      "value",
      mainMessages,
      "alt",
      altBinding
    )
  ];
  application.analysis_dynamic_prefixes = ["", "main"];
  application.sha256 = createHash("sha256").update(Buffer.from(code)).digest("hex");
  application.byte_length = Buffer.byteLength(code);
  data.manifest.messages["main.items"] = {
    arity: 1,
    locales: {
      en: {
        module: "bundler/messages/main/items/en.ts",
        source_ids: [1, 2]
      }
    }
  };
  data.manifest.messages["main.__proto__"] = {
    arity: 0,
    locales: {
      en: {
        module: "bundler/messages/main/__proto__/en.ts",
        source_ids: [1, 2]
      }
    }
  };
  data.manifest.messages.notice = {
    arity: 0,
    locales: {
      en: {
        module: "bundler/messages/notice/en.ts",
        source_ids: [1, 2]
      }
    }
  };
  for (const [relative, contents] of [
    ["bundler/messages/main/__proto__/en.ts", 'export function message() { return "Proto"; }\n'],
    ["bundler/messages/main/items/en.ts", 'export function message(value) { return `Items:${value}`; }\n'],
    ["bundler/messages/notice/en.ts", 'export function message() { return "Notice"; }\n']
  ]) {
    const file = path.join(data.generated, relative);
    await mkdir(path.dirname(file), { recursive: true });
    await writeFile(file, contents);
  }
  await writeFile(
    path.join(data.generated, "bundler/manifest.json"),
    JSON.stringify(data.manifest)
  );
  return data;
}

function resolvedMessageId(message) {
  return `\0virtual:linguini/message/${Buffer.from(message, "utf8").toString("hex")}`;
}

async function selectiveHmrFixture() {
  const data = await bundlerFixture();
  const files = {
    titleSchema: path.join(data.root, "src/schema/shop/delivery.lgs"),
    titleLocale: path.join(data.root, "src/locale/shop/ru.lgl"),
    itemsSchema: path.join(data.root, "src/schema/shop/items.lgs"),
    itemsLocale: path.join(data.root, "src/locale/shop/items.lgl"),
    sharedSchema: path.join(data.root, "src/schema/shop/shared.lgs"),
    unrelatedSchema: path.join(data.root, "src/schema/shop/unrelated.lgs")
  };
  await writeFile(files.itemsSchema, "items(count: Number)\n");
  await writeFile(files.itemsLocale, "items = Items\n");
  await writeFile(files.sharedSchema, "# shared dependency\n");
  await writeFile(files.unrelatedSchema, "unused\n");
  const itemsModule = path.join(data.generated, "bundler/messages/main/items/en.ts");
  await mkdir(path.dirname(itemsModule), { recursive: true });
  await writeFile(itemsModule, 'export function message(count) { return String(count); }\n');
  data.manifest.sources = [
    { id: 1, path: "src/schema/shop/delivery.lgs" },
    { id: 2, path: "src/locale/shop/ru.lgl" },
    { id: 3, path: "src/schema/shop/items.lgs" },
    { id: 4, path: "src/locale/shop/items.lgl" },
    { id: 5, path: "src/schema/shop/shared.lgs" },
    { id: 6, path: "src/schema/shop/unrelated.lgs" }
  ];
  data.manifest.messages["main.title"].locales.en.source_ids = [1, 2, 5];
  data.manifest.messages["main.items"] = {
    arity: 1,
    locales: {
      en: {
        module: "bundler/messages/main/items/en.ts",
        source_ids: [3, 4, 5]
      }
    }
  };
  await writeFile(
    path.join(data.generated, "bundler/manifest.json"),
    JSON.stringify(data.manifest)
  );
  return {
    ...data,
    files,
    titleModule: path.join(data.generated, "bundler/messages/main/title/en.ts"),
    itemsModule,
    titleVirtual: resolvedMessageId("main.title"),
    itemsVirtual: resolvedMessageId("main.items"),
    manifestPath: path.join(data.generated, "bundler/manifest.json")
  };
}

function mockServer(modules = []) {
  const handlers = new Map();
  const added = [];
  const removed = [];
  const invalidated = [];
  const reloaded = [];
  const events = [];
  return {
    handlers,
    added,
    removed,
    invalidated,
    reloaded,
    events,
    server: {
      watcher: {
        add(files) {
          added.push(...(Array.isArray(files) ? files : [files]));
        },
        async unwatch(files) {
          removed.push(...(Array.isArray(files) ? files : [files]));
        },
        on(event, handler) {
          handlers.set(event, handler);
        }
      },
      moduleGraph: {
        idToModuleMap: new Map(modules.map((module, index) => [String(index), module])),
        invalidateModule(module) {
          invalidated.push(module.id);
        }
      },
      async reloadModule(module) {
        reloaded.push(module.id);
      },
      ws: {
        send(event) {
          events.push(event);
        }
      }
    }
  };
}

test("discovers configured schema and locale files", async (context) => {
  const root = await fixture();
  context.after(() => rm(root, { recursive: true, force: true }));

  const files = (await discoverLinguiniFiles(root)).map((file) => path.relative(root, file));

  assert.deepEqual(files.sort(), [
    "linguini.toml",
    "src/locale/shop/ru.lgl",
    "src/schema/shop/delivery.lgs"
  ]);
});

test("does not add a missing config to the watch set", async (context) => {
  const root = await mkdtemp(path.join(tmpdir(), "linguini-vite-"));
  context.after(() => rm(root, { recursive: true, force: true }));
  assert.deepEqual(await discoverLinguiniFiles(root), []);
});

test("ignores generated output and transaction trees without hiding the manifest", async (context) => {
  const root = await fixture();
  context.after(() => rm(root, { recursive: true, force: true }));

  const plugin = linguini();
  const config = await plugin.config({ root });
  assert.ok(Array.isArray(config.server.watch.ignored));
  assert.equal(config.server.watch.ignored.length, 1);
  const ignored = config.server.watch.ignored[0];
  assert.equal(typeof ignored, "function");

  const generated = path.join(root, "build/custom-linguini");
  const manifest = path.join(generated, "bundler/manifest.json");
  const transaction = path.join(
    root,
    "build/.linguini-transaction-123-456-0/new/bundler/messages/title.js"
  );
  const cases = [
    [path.join(root, "src/routes/page.svelte"), false, "outside generated output"],
    [generated, false, "generated root"],
    [path.join(generated, "bundler"), false, "manifest ancestor"],
    [manifest, false, "manifest"],
    [path.join(generated, "bundler/messages/main/title/en.js"), true, "physical module"],
    [transaction, true, "transaction staging path"]
  ];
  for (const [file, expected, label] of cases) {
    assert.equal(ignored(file), expected, label);
  }
});

test("rejects configured paths outside the project", async (context) => {
  const root = await mkdtemp(path.join(tmpdir(), "linguini-vite-"));
  context.after(() => rm(root, { recursive: true, force: true }));
  await writeFile(path.join(root, "linguini.toml"), '[paths]\nschema = "../outside"\n');
  await assert.rejects(readProjectLayout(root), /must stay inside/);
});

test("validates finite dynamic bundler config with CLI parity", async (context) => {
  const root = await mkdtemp(path.join(tmpdir(), "linguini-vite-config-"));
  context.after(() => rm(root, { recursive: true, force: true }));
  const writeConfig = (dynamic) =>
    writeFile(
      path.join(root, "linguini.toml"),
      [
        "[targets.ts]",
        'out = "generated/linguini"',
        "[targets.ts.bundler]",
        'sources = ["src"]',
        "[targets.ts.bundler.dynamic]",
        dynamic,
        ""
      ].join("\n")
    );

  await writeConfig('mode = "bundle"\nallow = ["main.title", "main.items"]');
  await readProjectLayout(root);
  for (const [dynamic, pattern] of [
    ['mode = "bundle"\nallow = []', /allow must be non-empty/],
    ['mode = "error"\nallow = ["main.title"]', /requires mode = "bundle"/],
    ['mode = "unknown"', /mode must be "error" or "bundle"/],
    ['mode = "bundle"\nallow = ["main..title"]', /invalid path/],
    ['mode = "bundle"\nallow = ["main\/title"]', /invalid path/],
    [
      'mode = "bundle"\nallow = ["main.title", "MAIN.TITLE"]',
      /duplicate path/
    ],
    ['mode = "bundle"\nallow = ["main.title"]\nunknown = true', /is unknown/]
  ]) {
    await writeConfig(dynamic);
    await assert.rejects(readProjectLayout(root), pattern);
  }
});

test("v4 locale-loading policy validates config parity and preserves eager/SSR boundaries", async (context) => {
  const dynamic = await dynamicLocaleFixture({
    configLocaleLoading: "dynamic",
    manifestLocaleLoading: "dynamic"
  });
  context.after(() => rm(dynamic.root, { recursive: true, force: true }));
  const dynamicPlugin = linguini({ root: dynamic.root, buildOnStart: false });
  await dynamicPlugin.configResolved({ root: dynamic.root });
  await dynamicPlugin.buildStart.call({ addWatchFile() {} });
  const virtualId = await dynamicPlugin.resolveId(
    "virtual:linguini/message/6d61696e2e7469746c65"
  );
  const dynamicClient = dynamicPlugin.load.call(
    { environment: { config: { consumer: "client" } } },
    virtualId
  );
  assert.match(dynamicClient, /registerLocaleLoader/);
  assert.match(dynamicClient, /import\(".*bundler\/messages\/main\/title\/en\.js"\)/);
  assert.match(dynamicClient, /import\(".*bundler\/messages\/main\/title\/fr\.js"\)/);
  assert.match(dynamicClient, /await __linguini_load\(__linguini_initial_locale\)/);
  assert.doesNotMatch(dynamicClient, /import\.meta\.hot\.accept/);

  const dynamicSsr = dynamicPlugin.load.call(
    { environment: { config: { consumer: "server" } } },
    virtualId
  );
  assert.doesNotMatch(dynamicSsr, /registerLocaleLoader|import\(".*title\/fr\.js"\)/);
  const dynamicSsrV5 = dynamicPlugin.load.call({}, virtualId, { ssr: true });
  assert.doesNotMatch(dynamicSsrV5, /registerLocaleLoader|import\(".*title\/fr\.js"\)/);

  const eagerV3 = await dynamicLocaleFixture({ version: 3, configLocaleLoading: "eager" });
  context.after(() => rm(eagerV3.root, { recursive: true, force: true }));
  const eagerPlugin = linguini({ root: eagerV3.root, buildOnStart: false });
  await eagerPlugin.configResolved({ root: eagerV3.root });
  await eagerPlugin.buildStart.call({ addWatchFile() {} });
  const eagerId = await eagerPlugin.resolveId(
    "virtual:linguini/message/6d61696e2e7469746c65"
  );
  assert.doesNotMatch(
    eagerPlugin.load.call({ environment: { config: { consumer: "client" } } }, eagerId),
    /registerLocaleLoader|import\(/
  );

  const configMismatch = await dynamicLocaleFixture({
    configLocaleLoading: "eager",
    manifestLocaleLoading: "dynamic"
  });
  context.after(() => rm(configMismatch.root, { recursive: true, force: true }));
  const mismatchPlugin = linguini({ root: configMismatch.root, buildOnStart: false });
  await mismatchPlugin.configResolved({ root: configMismatch.root });
  await assert.rejects(
    mismatchPlugin.buildStart.call({ addWatchFile() {} }),
    /does not match config targets\.ts\.bundler\.locale_loading/
  );

  const dynamicLegacy = await dynamicLocaleFixture({ version: 3, configLocaleLoading: "dynamic" });
  context.after(() => rm(dynamicLegacy.root, { recursive: true, force: true }));
  const dynamicLegacyPlugin = linguini({ root: dynamicLegacy.root, buildOnStart: false });
  await dynamicLegacyPlugin.configResolved({ root: dynamicLegacy.root });
  await assert.rejects(
    dynamicLegacyPlugin.buildStart.call({ addWatchFile() {} }),
    /dynamic requires manifest version 4/
  );

  const malformedConfig = await dynamicLocaleFixture({ configLocaleLoading: "lazy" });
  context.after(() => rm(malformedConfig.root, { recursive: true, force: true }));
  await assert.rejects(
    readProjectLayout(malformedConfig.root),
    /locale_loading.*must be "eager" or "dynamic"/
  );
});

test("dynamic v4 virtual modules switch locales, retry failures, and dispose on HMR", async (context) => {
  const data = await dynamicLocaleFixture({
    configLocaleLoading: "dynamic",
    manifestLocaleLoading: "dynamic"
  });
  context.after(() => rm(data.root, { recursive: true, force: true }));
  const plugin = linguini({ root: data.root, buildOnStart: false });
  await plugin.configResolved({ root: data.root });
  await plugin.buildStart.call({ addWatchFile() {} });
  const virtualId = await plugin.resolveId(
    "virtual:linguini/message/6d61696e2e7469746c65"
  );
  const helperFile = path.join(data.generated, "svelte-locale.js");
  const frFile = path.join(data.generated, "bundler/messages/main/title/fr.js");
  let source = plugin.load.call(
    { environment: { config: { consumer: "client" } } },
    virtualId
  );
  const frImport = `() => import(${JSON.stringify(frFile)})`;
  assert.ok(source.includes(frImport));
  source = source.replace(
    frImport,
    '() => { if (globalThis.__linguiniFrAttempts++ === 0) throw new Error("transient locale failure"); return Promise.resolve({ message() { return "FR"; } }); }'
  );
  source = source.replace(
    "if (import.meta.hot) {",
    "if (globalThis.__linguiniHot) {"
  );
  source = source.replaceAll("import.meta.hot", "globalThis.__linguiniHot");
  const moduleFile = path.join(data.root, "dynamic-message.mjs");
  await writeFile(moduleFile, source);
  globalThis.__linguiniFrAttempts = 0;
  globalThis.__linguiniDisposed = 0;
  const hotDisposers = [];
  globalThis.__linguiniHot = {
    dispose(callback) {
      hotDisposers.push(callback);
    }
  };
  const runtime = await import(`${pathToFileURL(moduleFile).href}?runtime-test`);
  const helper = await import(pathToFileURL(helperFile).href);
  assert.equal(runtime.message(), "EN");
  helper.setCurrentLocale("fr");
  await assert.rejects(helper.prepareLocale("fr"), /transient locale failure/);
  assert.equal(runtime.message(), "EN");
  await helper.prepareLocale("fr");
  assert.equal(runtime.message(), "FR");
  assert.equal(hotDisposers.length, 1);
  hotDisposers[0]();
  hotDisposers[0]();
  assert.equal(globalThis.__linguiniDisposed, 1);
  delete globalThis.__linguiniHot;
  delete globalThis.__linguiniFrAttempts;
  delete globalThis.__linguiniDisposed;
});

test("recognizes only configured Linguini source roots", async (context) => {
  const root = await fixture();
  context.after(() => rm(root, { recursive: true, force: true }));
  const layout = await readProjectLayout(root);

  assert.equal(isLinguiniSource(layout.configPath, root, undefined, layout), true);
  assert.equal(
    isLinguiniSource(path.join(root, "src/schema/shop.lgs"), root, undefined, layout),
    true
  );
  assert.equal(
    isLinguiniSource(path.join(root, "unrelated/shop.lgs"), root, undefined, layout),
    false
  );
  assert.equal(isLinguiniSource(path.join(root, "src/app.ts"), root, undefined, layout), false);
});

test("hot update rebuilds and invalidates only configured output", async (context) => {
  const root = await fixture();
  context.after(() => rm(root, { recursive: true, force: true }));
  const changedFile = path.join(root, "src/schema/shop/delivery.lgs");
  const generated = path.join(root, "build/custom-linguini/index.js");
  const legacySubstring = path.join(root, "other/generated/linguini/user.js");
  const builds = [];
  const harness = mockServer([
    { id: generated },
    { id: legacySubstring },
    { id: path.join(root, "src/app.js") }
  ]);
  const plugin = linguini({
    root,
    buildOnStart: false,
    debounceMs: 0,
    build: (buildContext) => builds.push(buildContext)
  });

  await plugin.configResolved({ root });
  const result = await plugin.handleHotUpdate({
    file: changedFile,
    server: harness.server,
    timestamp: 42
  });

  assert.deepEqual(result, []);
  assert.equal(builds.length, 1);
  assert.equal(builds[0].reason, "hot-update");
  assert.deepEqual(harness.invalidated, [generated]);
  assert.equal(harness.events[0].event, "linguini:update");
});

test("change during a build schedules a dirty follow-up", async (context) => {
  const root = await fixture();
  context.after(() => rm(root, { recursive: true, force: true }));
  const firstFile = path.join(root, "src/schema/shop/delivery.lgs");
  const secondFile = path.join(root, "src/locale/shop/ru.lgl");
  let releaseFirst;
  let notifyStarted;
  const started = new Promise((resolve) => {
    notifyStarted = resolve;
  });
  const firstPending = new Promise((resolve) => {
    releaseFirst = resolve;
  });
  let builds = 0;
  const harness = mockServer();
  const plugin = linguini({
    root,
    buildOnStart: false,
    debounceMs: 0,
    async build() {
      builds += 1;
      if (builds === 1) {
        notifyStarted();
        await firstPending;
      }
    }
  });

  await plugin.configResolved({ root });
  const first = plugin.handleHotUpdate({
    file: firstFile,
    server: harness.server,
    timestamp: 1
  });
  await started;
  const second = plugin.handleHotUpdate({
    file: secondFile,
    server: harness.server,
    timestamp: 2
  });
  releaseFirst();
  await Promise.all([first, second]);

  assert.equal(builds, 2);
});

test("build errors become Vite overlay errors", async (context) => {
  const root = await fixture();
  context.after(() => rm(root, { recursive: true, force: true }));
  const harness = mockServer();
  const plugin = linguini({
    root,
    buildOnStart: false,
    debounceMs: 0,
    build() {
      throw new Error("schema build failed");
    }
  });

  await plugin.configResolved({ root });
  const result = await plugin.handleHotUpdate({
    file: path.join(root, "src/schema/shop/delivery.lgs"),
    server: harness.server,
    timestamp: 1
  });

  assert.deepEqual(result, []);
  assert.equal(harness.events.at(-1).type, "error");
  assert.match(harness.events.at(-1).err.message, /schema build failed/);
});

test("unlink rebuilds and removes obsolete watches", async (context) => {
  const root = await fixture();
  context.after(() => rm(root, { recursive: true, force: true }));
  const removedFile = path.join(root, "src/locale/shop/ru.lgl");
  const harness = mockServer();
  let builds = 0;
  const plugin = linguini({
    root,
    buildOnStart: false,
    debounceMs: 0,
    build() {
      builds += 1;
    }
  });

  await plugin.configResolved({ root });
  await plugin.configureServer(harness.server);
  await rm(removedFile);
  await harness.handlers.get("unlink")(removedFile);

  assert.equal(builds, 1);
  assert.ok(harness.removed.includes(removedFile));
});

test("v2 transforms Unicode Svelte source and loads exact virtual module", async (context) => {
  const fixtureData = await bundlerFixture();
  context.after(() => rm(fixtureData.root, { recursive: true, force: true }));
  const plugin = linguini({ root: fixtureData.root, buildOnStart: false });
  await plugin.configResolved({ root: fixtureData.root });
  const watched = [];
  await plugin.buildStart.call({ addWatchFile: (file) => watched.push(file) });

  const result = await plugin.transform.call(
    {
      async resolve() {
        return { id: path.join(fixtureData.generated, "svelte.ts") };
      }
    },
    fixtureData.code,
    fixtureData.application
  );

  assert.match(result.code, /import \{ message as __linguini_message_0 \} from "virtual:linguini\/message\/6d61696e2e7469746c65"/);
  assert.match(result.code, /import \{ helper \}/);
  assert.doesNotMatch(result.code, /l as tr/);
  assert.match(result.code, /const привет = __linguini_message_0\(\)/);
  assert.equal(result.map.sourcesContent[0], fixtureData.code);
  assert.ok(watched.includes(fixtureData.application));

  const publicId = "virtual:linguini/message/6d61696e2e7469746c65";
  const resolved = await plugin.resolveId(publicId);
  assert.equal(resolved, `\0${publicId}`);
  const virtual = plugin.load(resolved);
  assert.match(virtual, /getCurrentLocale/);
  assert.match(virtual, /svelte-effects\.svelte\.ts/);
  assert.match(virtual, /bundler\/messages\/main\/title\/en\.ts/);
  assert.doesNotMatch(virtual, /(?:^|\/)index(?:\.|["'])|\/svelte\.ts["']/m);
  assert.match(virtual, /__linguini_messages\[getCurrentLocale\(\)\]/);
});

test("v2 rejects stale bytes and skips unsafe bindings", async (context) => {
  const fixtureData = await bundlerFixture();
  context.after(() => rm(fixtureData.root, { recursive: true, force: true }));
  const plugin = linguini({ root: fixtureData.root, buildOnStart: false });
  await plugin.configResolved({ root: fixtureData.root });
  await plugin.buildStart.call({ addWatchFile() {} });
  await assert.rejects(
    plugin.transform.call(
      { resolve: async () => ({ id: path.join(fixtureData.generated, "svelte.ts") }) },
      fixtureData.code.replace("привет", "hello"),
      fixtureData.application
    ),
    /manifest is stale/
  );
  const unchanged = await plugin.transform.call(
    { resolve: async () => ({ id: path.join(fixtureData.root, "wrong.ts") }) },
    fixtureData.code,
    fixtureData.application
  );
  assert.equal(unchanged, undefined);
  const generatedEntry = path.join(fixtureData.generated, "svelte.ts");
  for (const suffix of ["?raw", "?url", "#fragment", "?raw#fragment"]) {
    assert.equal(
      await plugin.transform.call(
        { resolve: async () => ({ id: `${generatedEntry}${suffix}` }) },
        fixtureData.code,
        fixtureData.application
      ),
      undefined
    );
    assert.equal(
      await plugin.transform.call(
        { resolve: async () => ({ id: `${pathToFileURL(generatedEntry).href}${suffix}` }) },
        fixtureData.code,
        fixtureData.application
      ),
      undefined
    );
  }
  assert.ok(
    await plugin.transform.call(
      { resolve: async () => ({ id: pathToFileURL(generatedEntry).href }) },
      fixtureData.code,
      fixtureData.application
    )
  );
  assert.equal(
    await plugin.transform.call(
      { resolve: async () => ({ id: path.join(fixtureData.generated, "svelte.ts") }) },
      fixtureData.code,
      `${fixtureData.application}?svelte&type=script`
    ),
    undefined
  );
  const queried = await bundlerFixture({ applicationName: "page.ts" });
  context.after(() => rm(queried.root, { recursive: true, force: true }));
  const queriedPlugin = linguini({ root: queried.root, buildOnStart: false });
  await queriedPlugin.configResolved({ root: queried.root });
  await queriedPlugin.buildStart.call({ addWatchFile() {} });
  assert.ok(
    await queriedPlugin.transform.call(
      { resolve: async () => ({ id: path.join(queried.generated, "svelte.ts") }) },
      queried.code,
      queried.application
    )
  );
  for (const suffix of ["?raw", "?url", "?worker", "#fragment", "?raw#fragment"]) {
    assert.equal(
      await queriedPlugin.transform.call(
        { resolve: async () => ({ id: path.join(queried.generated, "svelte.ts") }) },
        "transformed payload that must not be hashed",
        `${queried.application}${suffix}`
      ),
      undefined
    );
  }
  for (const unsupported of ["page.vue", "page.astro", "page.css", "page.svelte?raw"]) {
    assert.equal(
      await plugin.transform.call(
        { resolve: async () => ({ id: path.join(fixtureData.generated, "svelte.ts") }) },
        fixtureData.code,
        path.join(path.dirname(fixtureData.application), unsupported)
      ),
      undefined
    );
  }
});

test("v3 transforms static and dynamic refs with finite frozen dispatches", async (context) => {
  const data = await dynamicBundlerFixture();
  context.after(() => rm(data.root, { recursive: true, force: true }));
  const plugin = linguini({ root: data.root, buildOnStart: false });
  await plugin.configResolved({ root: data.root });
  await plugin.buildStart.call({ addWatchFile() {} });
  const result = await plugin.transform.call(
    { resolve: async () => ({ id: path.join(data.generated, "svelte.ts") }) },
    data.code,
    data.application
  );

  const encoded = (message) =>
    `virtual:linguini/message/${Buffer.from(message, "utf8").toString("hex")}`;
  for (const message of ["main.__proto__", "main.items", "main.title", "notice"]) {
    assert.equal(result.code.split(encoded(message)).length - 1, 1);
  }
  assert.doesNotMatch(result.code, /from "\.\.\/\.\.\/build\/custom-linguini\/svelte\.ts"/);
  const titleImport = result.code.match(
    new RegExp(`message as (__linguini_message_\\d+) \\} from "${encoded("main.title")}"`)
  );
  const itemsImport = result.code.match(
    new RegExp(`message as (__linguini_message_\\d+) \\} from "${encoded("main.items")}"`)
  );
  assert.ok(titleImport && itemsImport);
  assert.match(result.code, new RegExp(`fixed = ${titleImport[1]}\\(\\)`));
  const mainDispatch = result.code.match(
    /value = (__linguini_dispatch_\d+)\[ \/\* keep \*\/ ключ \]/
  );
  assert.ok(mainDispatch);
  assert.match(result.code, new RegExp(`called = ${mainDispatch[1]}\\[action\\]\\(2\\)`));
  assert.match(result.code, new RegExp(`again = ${mainDispatch[1]}\\[keyAgain\\]`));
  const rootDispatch = result.code.match(/root = (__linguini_dispatch_\d+)\[rootKey\]/);
  assert.ok(rootDispatch);
  assert.notEqual(rootDispatch[1], mainDispatch[1]);
  const altDispatch = result.code.match(
    /altValue = (__linguini_dispatch_\d+)\[altKey\]/
  );
  assert.ok(altDispatch);
  assert.notEqual(altDispatch[1], mainDispatch[1]);
  assert.notEqual(mainDispatch[1], "__linguini_dispatch_0");
  assert.match(result.code, /Object\.freeze\(Object\.defineProperties\(Object\.create\(null\)/);
  assert.match(
    result.code,
    new RegExp(`\\["title"\\]: \\{ enumerable: true, get: \\(\\) => ${titleImport[1]}\\(\\) \\}`)
  );
  assert.match(
    result.code,
    new RegExp(`\\["items"\\]: \\{ enumerable: true, value: ${itemsImport[1]} \\}`)
  );
  assert.equal(result.map.sourcesContent[0], data.code);

  let runtime = result.code.replace(
    /import \{ message as ([A-Za-z0-9_$]+) \} from "virtual:linguini\/message\/([0-9a-f]+)";\n/g,
    (_match, alias, encodedMessage) => {
      const message = Buffer.from(encodedMessage, "hex").toString("utf8");
      if (message === "main.items") {
        return `const ${alias} = (value) => \`Items:\${value}\`;\n`;
      }
      if (message === "main.title") {
        return `const ${alias} = () => ++titleReads;\n`;
      }
      if (message === "main.__proto__") {
        return `const ${alias} = () => "Proto";\n`;
      }
      return `const ${alias} = () => "Notice";\n`;
    }
  );
  runtime = runtime.replaceAll("export const", "const");
  const execute = new Function(
    "ключ",
    "action",
    "rootKey",
    "keyAgain",
    "protoKey",
    "altKey",
    `let titleReads = 0;\n${runtime}\nreturn { fixed, value, called, root, again, proto, altValue, titleReads };`
  );
  const values = execute("missing", "items", "missing", "title", "__proto__", "items");
  assert.equal(values.value, undefined);
  assert.equal(values.root, undefined);
  assert.equal(values.called, "Items:2");
  assert.equal(values.fixed, 1);
  assert.equal(values.again, 2);
  assert.equal(values.proto, "Proto");
  assert.equal(values.titleReads, 2);
  assert.equal(values.altValue(3), "Items:3");
});

test("v3 uses only explicit dynamic references and rejects resolver mismatch", async (context) => {
  const staticOnly = await bundlerFixture({ version: 3, applicationName: "static.ts" });
  context.after(() => rm(staticOnly.root, { recursive: true, force: true }));
  staticOnly.manifest.applications[staticOnly.applicationKey].analysis_dynamic_prefixes = ["main"];
  await writeFile(
    path.join(staticOnly.generated, "bundler/manifest.json"),
    JSON.stringify(staticOnly.manifest)
  );
  const staticPlugin = linguini({ root: staticOnly.root, buildOnStart: false });
  await staticPlugin.configResolved({ root: staticOnly.root });
  await staticPlugin.buildStart.call({ addWatchFile() {} });
  const staticResult = await staticPlugin.transform.call(
    { resolve: async () => ({ id: path.join(staticOnly.generated, "svelte.ts") }) },
    staticOnly.code,
    staticOnly.application
  );
  assert.doesNotMatch(staticResult.code, /__linguini_dispatch_/);
  assert.match(staticResult.code, /__linguini_message_0\(\)/);

  const dynamic = await dynamicBundlerFixture({ applicationName: "resolver.ts" });
  context.after(() => rm(dynamic.root, { recursive: true, force: true }));
  const dynamicPlugin = linguini({ root: dynamic.root, buildOnStart: false });
  await dynamicPlugin.configResolved({ root: dynamic.root });
  await dynamicPlugin.buildStart.call({ addWatchFile() {} });
  await assert.rejects(
    dynamicPlugin.transform.call(
      { resolve: async () => ({ id: path.join(dynamic.generated, "svelte.ts") }) },
      dynamic.code.replace("user-owned", "changed"),
      dynamic.application
    ),
    /manifest is stale/
  );
  await assert.rejects(
    dynamicPlugin.transform.call(
      { resolve: async () => null },
      dynamic.code,
      dynamic.application
    ),
    /must resolve to generated svelte\.ts/
  );
  await assert.rejects(
    dynamicPlugin.transform.call(
      {
        resolve: async () => {
          throw new Error("resolver failed");
        }
      },
      dynamic.code,
      dynamic.application
    ),
    /could not resolve/
  );
  await assert.rejects(
    dynamicPlugin.transform.call(
      { resolve: async () => ({ id: path.join(dynamic.generated, "index.ts") }) },
      dynamic.code,
      dynamic.application
    ),
    /must resolve to generated svelte\.ts/
  );
});

test("v3 keeps deduped dynamic helpers inside Svelte script scope", async (context) => {
  const data = await dynamicBundlerFixture({ applicationName: "dynamic.svelte" });
  context.after(() => rm(data.root, { recursive: true, force: true }));
  const plugin = linguini({ root: data.root, buildOnStart: false });
  await plugin.configResolved({ root: data.root });
  await plugin.buildStart.call({ addWatchFile() {} });
  const result = await plugin.transform.call(
    { resolve: async () => ({ id: path.join(data.generated, "svelte.ts") }) },
    data.code,
    data.application
  );
  const [beforeClose, afterClose] = result.code.split("</script>");
  assert.match(beforeClose, /^<script>import \{ message as __linguini_message_/);
  assert.match(beforeClose, /Object\.create\(null\)/);
  assert.doesNotMatch(afterClose, /virtual:linguini|__linguini_dispatch_/);
  const titleId = `virtual:linguini/message/${Buffer.from("main.title").toString("hex")}`;
  assert.equal(beforeClose.split(titleId).length - 1, 1);
});

test("v3 strictly validates dynamic manifest contracts", async (context) => {
  const data = await dynamicBundlerFixture({ applicationName: "malformed.ts" });
  context.after(() => rm(data.root, { recursive: true, force: true }));
  const original = structuredClone(data.manifest);
  const manifestPath = path.join(data.generated, "bundler/manifest.json");
  const reject = async (mutate, pattern) => {
    const manifest = structuredClone(original);
    mutate(manifest.applications[data.applicationKey], manifest);
    await writeFile(manifestPath, JSON.stringify(manifest));
    const plugin = linguini({ root: data.root, buildOnStart: false });
    await plugin.configResolved({ root: data.root });
    await assert.rejects(plugin.buildStart.call({ addWatchFile() {} }), pattern);
  };

  await reject(
    (application) => delete application.imports[0].analyzer_tracked_uses_only,
    /analyzer_tracked_uses_only must be boolean/
  );
  await reject(
    (application) => application.dynamic_references[0].messages.reverse(),
    /messages must be sorted and unique/
  );
  await reject(
    (application) => {
      application.dynamic_references[0].key_span.start =
        application.dynamic_references[0].receiver_span.end;
    },
    /invalid nested dynamic spans/
  );
  await reject(
    (application) => {
      application.dynamic_references[0].messages[0].arity = 1;
    },
    /invalid message, key, or arity/
  );
  await reject(
    (application) => {
      application.dynamic_references[0].messages[0].key = "nested.items";
    },
    /invalid message, key, or arity/
  );
  await reject(
    (application) => {
      application.dynamic_references[0].provenance.symbol = "messages";
    },
    /does not match a tracked transformable import binding/
  );
  await reject(
    (application) => {
      application.dynamic_references[1] = structuredClone(
        application.dynamic_references[0]
      );
    },
    /overlapping static or dynamic spans/
  );
  await reject(
    (application) => {
      application.dynamic_references[0].messages[0].message = "main.unknown";
    },
    /invalid message, key, or arity/
  );
});

test("v5 accepts locale runtimes and dynamic locale loading", async (context) => {
  const data = await dynamicLocaleFixture({
    version: 5,
    configLocaleLoading: "dynamic",
    manifestLocaleLoading: "dynamic"
  });
  context.after(() => rm(data.root, { recursive: true, force: true }));
  const plugin = linguini({ root: data.root, buildOnStart: false });
  await plugin.configResolved({ root: data.root });
  await plugin.buildStart.call({ addWatchFile() {} });
  const virtualId = await plugin.resolveId(
    "virtual:linguini/message/6d61696e2e7469746c65"
  );
  const source = plugin.load.call(
    { environment: { config: { consumer: "client" } } },
    virtualId
  );
  assert.match(source, /registerLocaleLoader/);
  assert.match(source, /title\/en\.js/);
  assert.match(source, /title\/fr\.js/);
});

test("v5 validates exact locale runtime descriptors", async (context) => {
  const data = await v5RuntimeFixture();
  context.after(() => rm(data.root, { recursive: true, force: true }));
  const original = structuredClone(data.manifest);
  const reject = async (mutate, pattern) => {
    const manifest = structuredClone(original);
    mutate(manifest);
    await writeFile(data.generated + "/bundler/manifest.json", JSON.stringify(manifest));
    const plugin = linguini({ root: data.root, buildOnStart: false });
    await plugin.configResolved({ root: data.root });
    await assert.rejects(plugin.buildStart.call({ addWatchFile() {} }), pattern);
  };

  await reject((manifest) => delete manifest.message_runtimes, /message_runtimes must be an object/);
  await reject(
    (manifest) => delete manifest.message_runtimes.en,
    /must contain exactly one entry for every effective locale/
  );
  await reject(
    (manifest) => {
      manifest.message_runtimes.en.module = "../escape.ts";
    },
    /clean project-relative POSIX path/
  );
  await reject(
    (manifest) => {
      manifest.message_runtimes.en.source_ids = [999];
    },
    /source_ids.*integer id from sources/
  );
  await reject(
    (manifest) => {
      manifest.message_runtimes.en.source_ids = [2, 2];
    },
    /source_ids.*unique/
  );
});

test("allocates generated aliases around existing JavaScript bindings", async (context) => {
  const source = [
    'import { l as tr, helper } from "../../build/custom-linguini/svelte.ts";',
    'const __linguini_message_0 = "user-owned";',
    "const title = tr.main.title;",
    ""
  ].join("\r\n");
  const fixtureData = await bundlerFixture({ applicationName: "collision.ts", source });
  context.after(() => rm(fixtureData.root, { recursive: true, force: true }));
  const plugin = linguini({ root: fixtureData.root, buildOnStart: false });
  await plugin.configResolved({ root: fixtureData.root });
  await plugin.buildStart.call({ addWatchFile() {} });
  const result = await plugin.transform.call(
    { resolve: async () => ({ id: path.join(fixtureData.generated, "svelte.ts") }) },
    source,
    fixtureData.application
  );
  assert.match(result.code, /message as __linguini_message_1/);
  assert.match(result.code, /const __linguini_message_0 = "user-owned"/);
  assert.match(result.code, /const title = __linguini_message_1\(\)/);
});

test("keeps virtual imports inside each Svelte script scope and collapses sole import", async (context) => {
  const fixtureData = await bundlerFixture();
  context.after(() => rm(fixtureData.root, { recursive: true, force: true }));
  const source = [
    '<script context="module">',
    'const emoji = "😀";',
    'const __linguini_message_0 = "module-owned";',
    'import { l as moduleL } from "../../build/custom-linguini/svelte.ts";',
    "const moduleTitle = moduleL.main.title;",
    "</script>",
    "<script>",
    'const __linguini_message_1 = "instance-owned";',
    'import { messages as instanceL, helper } from "../../build/custom-linguini/svelte.ts";',
    "const instanceTitle = instanceL.main.title;",
    "const implicitTitle = implicit.main.title;",
    "</script>",
    ""
  ].join("\r\n");
  const binding = (id, declarationText, itemText, removalText, imported, local) => {
    const declaration = byteSpan(source, declarationText);
    const item = byteSpan(source, itemText);
    const removal = byteSpan(source, removalText);
    return {
      binding_id: id,
      module_specifier: "../../build/custom-linguini/svelte.ts",
      imported,
      local,
      declaration_start: declaration[0],
      declaration_end: declaration[1],
      item_start: item[0],
      item_end: item[1],
      removal_start: removal[0],
      removal_end: removal[1],
      module_specifier_start: declaration[0],
      module_specifier_end: declaration[1],
      imported_start: item[0],
      imported_end: item[0] + imported.length,
      local_start: item[1] - local.length,
      local_end: item[1],
      analyzer_exact_uses_only: true,
      transformable: true
    };
  };
  const reference = (needle, bindingId, local) => {
    const span = byteSpan(source, needle);
    return {
      message: "main.title",
      start: span[0],
      end: span[1],
      kind: "value",
      local,
      provenance:
        bindingId === null
          ? { kind: "implicit" }
          : {
              kind: "imported",
              module_specifier: "../../build/custom-linguini/svelte.ts",
              symbol: local === "moduleL" ? "l" : "messages"
            },
      arity: 0,
      binding_id: bindingId
    };
  };
  const moduleDeclaration = 'import { l as moduleL } from "../../build/custom-linguini/svelte.ts";';
  const instanceDeclaration =
    'import { messages as instanceL, helper } from "../../build/custom-linguini/svelte.ts";';
  const application = fixtureData.manifest.applications[fixtureData.applicationKey];
  application.sha256 = createHash("sha256").update(Buffer.from(source)).digest("hex");
  application.byte_length = Buffer.byteLength(source);
  application.imports = [
    binding("module", moduleDeclaration, "l as moduleL", moduleDeclaration, "l", "moduleL"),
    binding(
      "instance",
      instanceDeclaration,
      "messages as instanceL",
      "messages as instanceL, ",
      "messages",
      "instanceL"
    )
  ];
  application.references = [
    reference("moduleL.main.title", "module", "moduleL"),
    reference("instanceL.main.title", "instance", "instanceL"),
    reference("implicit.main.title", null, "implicit")
  ];
  await writeFile(fixtureData.application, source);
  await writeFile(
    path.join(fixtureData.generated, "bundler/manifest.json"),
    JSON.stringify(fixtureData.manifest)
  );
  const plugin = linguini({ root: fixtureData.root, buildOnStart: false });
  await plugin.configResolved({ root: fixtureData.root });
  await plugin.buildStart.call({ addWatchFile() {} });
  const result = await plugin.transform.call(
    { resolve: async () => ({ id: path.join(fixtureData.generated, "svelte.ts") }) },
    source,
    fixtureData.application
  );
  const [moduleScript, instanceScript] = result.code.split("</script>");
  assert.match(moduleScript, /message as __linguini_message_2/);
  assert.doesNotMatch(moduleScript, /message as __linguini_message_3/);
  assert.doesNotMatch(moduleScript, /l as moduleL/);
  assert.match(instanceScript, /message as __linguini_message_3/);
  assert.match(instanceScript, /import \{ helper \}/);
  assert.match(instanceScript, /implicit\.main\.title/);
});

test("missing and v1 manifests stay legacy; unknown versions fail", async (context) => {
  const missingRoot = await fixture();
  context.after(() => rm(missingRoot, { recursive: true, force: true }));
  const missingPlugin = linguini({ root: missingRoot, buildOnStart: false });
  await missingPlugin.configResolved({ root: missingRoot });
  await missingPlugin.buildStart.call({ addWatchFile() {} });
  assert.equal(
    await missingPlugin.transform.call(
      { resolve: async () => null },
      "export const untouched = true;\n",
      path.join(missingRoot, "src/app.ts")
    ),
    undefined
  );

  const legacy = await bundlerFixture({ version: 1 });
  context.after(() => rm(legacy.root, { recursive: true, force: true }));
  const legacyPlugin = linguini({ root: legacy.root, buildOnStart: false });
  await legacyPlugin.configResolved({ root: legacy.root });
  await legacyPlugin.buildStart.call({ addWatchFile() {} });
  assert.equal(
    await legacyPlugin.transform.call({ resolve: async () => null }, legacy.code, legacy.application),
    undefined
  );

  const unknown = await bundlerFixture({ version: 4 });
  context.after(() => rm(unknown.root, { recursive: true, force: true }));
  const unknownPlugin = linguini({ root: unknown.root, buildOnStart: false });
  await unknownPlugin.configResolved({ root: unknown.root });
  await assert.rejects(
    unknownPlugin.buildStart.call({ addWatchFile() {} }),
    /unsupported Linguini bundler manifest version 4/
  );

  const escaping = await bundlerFixture();
  context.after(() => rm(escaping.root, { recursive: true, force: true }));
  escaping.manifest.messages["main.title"].locales.en.module = "../escape.ts";
  await writeFile(
    path.join(escaping.generated, "bundler/manifest.json"),
    JSON.stringify(escaping.manifest)
  );
  const escapingPlugin = linguini({ root: escaping.root, buildOnStart: false });
  await escapingPlugin.configResolved({ root: escaping.root });
  await assert.rejects(
    escapingPlugin.buildStart.call({ addWatchFile() {} }),
    /clean project-relative POSIX path/
  );
});

test("application hot update rebuilds without suppressing Vite module update", async (context) => {
  const fixtureData = await bundlerFixture();
  context.after(() => rm(fixtureData.root, { recursive: true, force: true }));
  const harness = mockServer([{ id: fixtureData.application }]);
  let builds = 0;
  const plugin = linguini({
    root: fixtureData.root,
    buildOnStart: false,
    debounceMs: 0,
    build() {
      builds += 1;
    }
  });
  await plugin.configResolved({ root: fixtureData.root });
  await plugin.buildStart.call({ addWatchFile() {} });
  await plugin.configureServer(harness.server);
  const result = await plugin.handleHotUpdate({
    file: fixtureData.application,
    server: harness.server,
    timestamp: 8
  });
  assert.equal(result, undefined);
  assert.equal(builds, 1);
  assert.deepEqual(harness.invalidated, []);
});

test("watches configured application roots so newly added modules rebuild manifest", async (context) => {
  const fixtureData = await bundlerFixture();
  context.after(() => rm(fixtureData.root, { recursive: true, force: true }));
  const configPath = path.join(fixtureData.root, "linguini.toml");
  await writeFile(
    configPath,
    [
      "[paths]",
      'schema = "src/schema"',
      'locale = "src/locale"',
      "[targets.ts]",
      'out = "build/custom-linguini"',
      'framework = "svelte"',
      "[targets.ts.bundler]",
      'sources = ["src/app"]',
      ""
    ].join("\n")
  );
  const harness = mockServer();
  let builds = 0;
  const plugin = linguini({
    root: fixtureData.root,
    buildOnStart: false,
    debounceMs: 0,
    build() {
      builds += 1;
    }
  });
  await plugin.configResolved({ root: fixtureData.root });
  await plugin.buildStart.call({ addWatchFile() {} });
  await plugin.configureServer(harness.server);
  const newApplication = path.join(fixtureData.root, "src/app/new.ts");
  await writeFile(newApplication, "export const fresh = true;\n");
  await harness.handlers.get("add")(newApplication);
  assert.equal(builds, 1);
  assert.ok(harness.added.includes(path.join(fixtureData.root, "src/app")));
});

test("selective HMR follows exact message source dependencies", async (context) => {
  const data = await selectiveHmrFixture();
  context.after(() => rm(data.root, { recursive: true, force: true }));
  let nextManifest = structuredClone(data.manifest);
  const unrelatedGenerated = path.join(data.generated, "index.ts");
  const addedSource = path.join(data.root, "src/schema/shop/added.lgs");
  const addedModule = path.join(data.generated, "bundler/messages/main/added/en.ts");
  const addedVirtual = resolvedMessageId("main.added");
  const harness = mockServer([
    { id: data.titleVirtual },
    { id: data.itemsVirtual },
    { id: data.titleModule },
    { id: data.itemsModule },
    { id: addedModule },
    { id: addedVirtual },
    { id: data.application },
    { id: unrelatedGenerated }
  ]);
  const plugin = linguini({
    root: data.root,
    buildOnStart: false,
    debounceMs: 0,
    async build() {
      await writeFile(data.manifestPath, JSON.stringify(nextManifest));
    }
  });
  await plugin.configResolved({ root: data.root });
  await plugin.buildStart.call({ addWatchFile() {} });
  await plugin.configureServer(harness.server);

  const update = async (file) => {
    harness.invalidated.length = 0;
    const result = await plugin.handleHotUpdate({
      file,
      server: harness.server,
      timestamp: 20
    });
    return {
      invalidated: [...harness.invalidated].sort(),
      returned: (result ?? []).map((module) => module.id).sort()
    };
  };

  assert.deepEqual(await update(data.files.titleSchema), {
    invalidated: [data.titleModule, data.titleVirtual].sort(),
    returned: [data.titleModule, data.titleVirtual].sort()
  });
  assert.deepEqual(harness.reloaded, []);
  assert.deepEqual((await update(data.files.itemsLocale)).invalidated, [
    data.itemsModule,
    data.itemsVirtual
  ].sort());
  assert.deepEqual((await update(data.files.sharedSchema)).invalidated, [
    data.itemsModule,
    data.itemsVirtual,
    data.titleModule,
    data.titleVirtual
  ].sort());
  assert.deepEqual((await update(data.files.unrelatedSchema)).invalidated, []);
  assert.ok(!harness.invalidated.includes(unrelatedGenerated));
  harness.reloaded.length = 0;
  await harness.handlers.get("add")(data.files.unrelatedSchema);
  assert.deepEqual(harness.reloaded, []);

  await mkdir(path.dirname(addedModule), { recursive: true });
  await writeFile(addedModule, 'export function message() { return "added"; }\n');
  await writeFile(addedSource, "added\n");
  nextManifest = structuredClone(nextManifest);
  nextManifest.sources.push({ id: 7, path: "src/schema/shop/added.lgs" });
  nextManifest.messages["main.added"] = {
    arity: 0,
    locales: {
      en: {
        module: "bundler/messages/main/added/en.ts",
        source_ids: [7]
      }
    }
  };
  harness.invalidated.length = 0;
  harness.reloaded.length = 0;
  await harness.handlers.get("add")(addedSource);
  assert.deepEqual([...harness.invalidated].sort(), [addedModule, addedVirtual].sort());
  assert.deepEqual([...harness.reloaded].sort(), [addedModule, addedVirtual].sort());

  await rm(addedSource);
  nextManifest = structuredClone(nextManifest);
  nextManifest.sources = nextManifest.sources.filter((source) => source.id !== 7);
  delete nextManifest.messages["main.added"];
  harness.invalidated.length = 0;
  harness.reloaded.length = 0;
  await harness.handlers.get("unlink")(addedSource);
  assert.deepEqual([...harness.invalidated].sort(), [addedModule, addedVirtual].sort());
  assert.deepEqual([...harness.reloaded].sort(), [addedModule, addedVirtual].sort());

  nextManifest = structuredClone(nextManifest);
  nextManifest.messages["main.title"].locales.en.source_ids.push(6);
  await writeFile(data.manifestPath, JSON.stringify(nextManifest));
  harness.invalidated.length = 0;
  harness.reloaded.length = 0;
  await harness.handlers.get("add")(data.manifestPath);
  assert.deepEqual([...harness.invalidated].sort(), [
    data.titleModule,
    data.titleVirtual
  ].sort());
  assert.deepEqual([...harness.reloaded].sort(), [
    data.titleModule,
    data.titleVirtual
  ].sort());

  const configBroad = await update(path.join(data.root, "linguini.toml"));
  assert.ok(configBroad.invalidated.includes(unrelatedGenerated));
  assert.ok(configBroad.invalidated.includes(data.application));
  assert.ok(configBroad.invalidated.includes(data.titleVirtual));

  delete harness.server.reloadModule;
  harness.events.length = 0;
  await harness.handlers.get("add")(data.files.sharedSchema);
  assert.equal(harness.events.at(-1).type, "full-reload");
  assert.equal(harness.events.at(-1).path, "*");
});

test("manifest deltas target applications, renames, and broad version transitions", async (context) => {
  const data = await selectiveHmrFixture();
  context.after(() => rm(data.root, { recursive: true, force: true }));
  const unrelatedGenerated = path.join(data.generated, "index.ts");
  const harness = mockServer([
    { id: data.titleVirtual },
    { id: data.itemsVirtual },
    { id: resolvedMessageId("main.list") },
    { id: data.titleModule },
    { id: data.itemsModule },
    { id: data.application },
    { id: unrelatedGenerated }
  ]);
  const plugin = linguini({ root: data.root, buildOnStart: false });
  await plugin.configResolved({ root: data.root });
  await plugin.buildStart.call({ addWatchFile() {} });
  await plugin.configureServer(harness.server);

  const reload = async (manifest) => {
    harness.invalidated.length = 0;
    await writeFile(data.manifestPath, JSON.stringify(manifest));
    const result = await plugin.handleHotUpdate({
      file: data.manifestPath,
      server: harness.server,
      timestamp: 30
    });
    return {
      invalidated: [...harness.invalidated].sort(),
      returned: (result ?? []).map((module) => module.id).sort()
    };
  };

  let manifest = structuredClone(data.manifest);
  manifest.applications[data.applicationKey].sha256 = "a".repeat(64);
  assert.deepEqual((await reload(manifest)).invalidated, [data.application]);
  assert.deepEqual((await reload(manifest)).invalidated, []);

  manifest = structuredClone(manifest);
  manifest.messages["main.list"] = manifest.messages["main.items"];
  delete manifest.messages["main.items"];
  const renamed = await reload(manifest);
  assert.deepEqual(renamed.invalidated, [
    data.itemsModule,
    data.itemsVirtual,
    resolvedMessageId("main.list")
  ].sort());
  assert.deepEqual(renamed.returned, renamed.invalidated);

  manifest = structuredClone(manifest);
  manifest.runtime_helpers.svelte_locale = {
    import: "./svelte-locale-alt.svelte.js",
    file: "svelte-locale-alt.svelte.ts"
  };
  await writeFile(
    path.join(data.generated, "svelte-locale-alt.svelte.ts"),
    'export function getCurrentLocale() { return "en"; }\n'
  );
  const helperBroad = await reload(manifest);
  assert.ok(helperBroad.invalidated.includes(unrelatedGenerated));
  assert.ok(helperBroad.invalidated.includes(data.application));
  assert.ok(helperBroad.invalidated.includes(data.titleVirtual));

  const legacy = {
    version: 1,
    base_locale: "en",
    configured_locales: ["en"],
    effective_locales: ["en"],
    sources: [],
    messages: {}
  };
  const broad = await reload(legacy);
  assert.ok(broad.invalidated.includes(unrelatedGenerated));
  assert.ok(broad.invalidated.includes(data.application));
  assert.ok(broad.invalidated.includes(data.titleVirtual));
});

test("v3 HMR fingerprints dynamic app contracts and message descriptors", async (context) => {
  const data = await dynamicBundlerFixture({ applicationName: "hmr.ts" });
  context.after(() => rm(data.root, { recursive: true, force: true }));
  const manifestPath = path.join(data.generated, "bundler/manifest.json");
  const otherModule = path.join(
    data.generated,
    "bundler/messages/main/other/en.ts"
  );
  await mkdir(path.dirname(otherModule), { recursive: true });
  await writeFile(otherModule, 'export function message() { return "Other"; }\n');
  data.manifest.messages["main.other"] = {
    arity: 0,
    locales: {
      en: {
        module: "bundler/messages/main/other/en.ts",
        source_ids: [1, 2]
      }
    }
  };
  await writeFile(manifestPath, JSON.stringify(data.manifest));
  const itemsModule = path.join(
    data.generated,
    "bundler/messages/main/items/en.ts"
  );
  const itemsVirtual = resolvedMessageId("main.items");
  const otherVirtual = resolvedMessageId("main.other");
  const harness = mockServer([
    { id: data.application },
    { id: itemsModule },
    { id: itemsVirtual },
    { id: otherModule },
    { id: otherVirtual }
  ]);
  const plugin = linguini({ root: data.root, buildOnStart: false });
  await plugin.configResolved({ root: data.root });
  await plugin.buildStart.call({ addWatchFile() {} });
  await plugin.configureServer(harness.server);

  const contract = structuredClone(data.manifest);
  for (const reference of contract.applications[data.applicationKey].dynamic_references) {
    if (reference.prefix === "main") {
      reference.messages = [
        { message: "main.items", key: "items", arity: 1 },
        { message: "main.other", key: "other", arity: 0 },
        { message: "main.title", key: "title", arity: 0 }
      ];
    }
  }
  await writeFile(manifestPath, JSON.stringify(contract));
  await plugin.handleHotUpdate({
    file: manifestPath,
    server: harness.server,
    timestamp: 70
  });
  assert.deepEqual(harness.invalidated, [data.application]);
  assert.ok(!harness.invalidated.includes(otherVirtual));

  harness.invalidated.length = 0;
  const descriptor = structuredClone(contract);
  descriptor.sources.push({ id: 3, path: "src/schema/extra.lgs" });
  descriptor.messages["main.items"].locales.en.source_ids.push(3);
  await writeFile(manifestPath, JSON.stringify(descriptor));
  await plugin.handleHotUpdate({
    file: manifestPath,
    server: harness.server,
    timestamp: 71
  });
  assert.deepEqual([...harness.invalidated].sort(), [itemsModule, itemsVirtual].sort());
  assert.ok(!harness.invalidated.includes(data.application));
});

test("message descriptor deltas invalidate only changed locale modules", async (context) => {
  const data = await selectiveHmrFixture();
  context.after(() => rm(data.root, { recursive: true, force: true }));
  const frModule = path.join(data.generated, "bundler/messages/main/title/fr.ts");
  const movedFrModule = path.join(data.generated, "bundler/messages/main/title/fr-next.ts");
  await writeFile(frModule, 'export function message() { return "Titre"; }\n');
  await writeFile(movedFrModule, 'export function message() { return "Titre next"; }\n');
  data.manifest.configured_locales.push("fr");
  data.manifest.effective_locales.push("fr");
  data.manifest.messages["main.title"].locales.fr = {
    module: "bundler/messages/main/title/fr.ts",
    source_ids: [2]
  };
  await writeFile(data.manifestPath, JSON.stringify(data.manifest));
  const harness = mockServer([
    { id: data.titleVirtual },
    { id: data.titleModule },
    { id: frModule },
    { id: movedFrModule }
  ]);
  const plugin = linguini({ root: data.root, buildOnStart: false });
  await plugin.configResolved({ root: data.root });
  await plugin.buildStart.call({ addWatchFile() {} });
  await plugin.configureServer(harness.server);

  let next = structuredClone(data.manifest);
  next.messages["main.title"].locales.fr.source_ids.push(6);
  await writeFile(data.manifestPath, JSON.stringify(next));
  await plugin.handleHotUpdate({
    file: data.manifestPath,
    server: harness.server,
    timestamp: 40
  });
  assert.deepEqual([...harness.invalidated].sort(), [data.titleVirtual, frModule].sort());
  assert.ok(!harness.invalidated.includes(data.titleModule));

  harness.invalidated.length = 0;
  next = structuredClone(next);
  next.messages["main.title"].locales.fr.module =
    "bundler/messages/main/title/fr-next.ts";
  await writeFile(data.manifestPath, JSON.stringify(next));
  await plugin.handleHotUpdate({
    file: data.manifestPath,
    server: harness.server,
    timestamp: 41
  });
  assert.deepEqual([...harness.invalidated].sort(), [
    data.titleVirtual,
    frModule,
    movedFrModule
  ].sort());
  assert.ok(!harness.invalidated.includes(data.titleModule));
});

test("v5 runtime deltas invalidate source-specific and old/new runtime modules", async (context) => {
  const data = await v5RuntimeFixture();
  context.after(() => rm(data.root, { recursive: true, force: true }));
  const movedRuntime = path.join(data.generated, "locales/en/runtime-next.ts");
  await writeFile(movedRuntime, "export {};\n");
  const sourceChange = path.join(data.root, "src/locale/shop/ru.lgl");
  let nextManifest = structuredClone(data.manifest);
  const harness = mockServer([
    { id: data.runtimeModule },
    { id: movedRuntime },
    { id: data.titleModule },
    { id: data.titleVirtual }
  ]);
  const plugin = linguini({
    root: data.root,
    buildOnStart: false,
    debounceMs: 0,
    async build() {
      await writeFile(data.generated + "/bundler/manifest.json", JSON.stringify(nextManifest));
    }
  });
  await plugin.configResolved({ root: data.root });
  await plugin.buildStart.call({ addWatchFile() {} });
  await plugin.configureServer(harness.server);

  harness.invalidated.length = 0;
  await plugin.handleHotUpdate({ file: sourceChange, server: harness.server, timestamp: 90 });
  assert.deepEqual(harness.invalidated, [data.runtimeModule]);

  nextManifest = structuredClone(nextManifest);
  nextManifest.message_runtimes.en = {
    module: "locales/en/runtime-next.ts",
    source_ids: [1]
  };
  await writeFile(data.generated + "/bundler/manifest.json", JSON.stringify(nextManifest));
  harness.invalidated.length = 0;
  await plugin.handleHotUpdate({
    file: data.generated + "/bundler/manifest.json",
    server: harness.server,
    timestamp: 91
  });
  assert.deepEqual([...harness.invalidated].sort(), [data.runtimeModule, movedRuntime].sort());
});
