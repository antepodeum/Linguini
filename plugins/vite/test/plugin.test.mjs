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
  const declaration = byteSpan(code, 'import { l as tr, helper } from "../../build/custom-linguini/svelte.ts";');
  const item = byteSpan(code, "l as tr");
  const removal = byteSpan(code, "l as tr, ");
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

function mockServer(modules = []) {
  const handlers = new Map();
  const added = [];
  const removed = [];
  const invalidated = [];
  const events = [];
  return {
    handlers,
    added,
    removed,
    invalidated,
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

test("rejects configured paths outside the project", async (context) => {
  const root = await mkdtemp(path.join(tmpdir(), "linguini-vite-"));
  context.after(() => rm(root, { recursive: true, force: true }));
  await writeFile(path.join(root, "linguini.toml"), '[paths]\nschema = "../outside"\n');
  await assert.rejects(readProjectLayout(root), /must stay inside/);
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

  const unknown = await bundlerFixture({ version: 3 });
  context.after(() => rm(unknown.root, { recursive: true, force: true }));
  const unknownPlugin = linguini({ root: unknown.root, buildOnStart: false });
  await unknownPlugin.configResolved({ root: unknown.root });
  await assert.rejects(
    unknownPlugin.buildStart.call({ addWatchFile() {} }),
    /unsupported Linguini bundler manifest version 3/
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
  const result = await plugin.handleHotUpdate({
    file: fixtureData.application,
    server: harness.server,
    timestamp: 8
  });
  assert.equal(result, undefined);
  assert.equal(builds, 1);
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
