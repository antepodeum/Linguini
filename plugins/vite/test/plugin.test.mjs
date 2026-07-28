import assert from "node:assert/strict";
import { mkdir, mkdtemp, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import path from "node:path";
import test from "node:test";
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
