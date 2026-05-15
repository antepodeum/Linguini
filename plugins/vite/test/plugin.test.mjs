import assert from "node:assert/strict";
import { mkdir, mkdtemp, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import path from "node:path";
import test from "node:test";
import { discoverLinguiniFiles, isLinguiniSource, linguini } from "../src/index.js";

test("discovers config, schema, and locale files", async () => {
  const root = await mkdtemp(path.join(tmpdir(), "linguini-vite-"));
  await mkdir(path.join(root, "src/schema/shop"), { recursive: true });
  await mkdir(path.join(root, "src/locale/shop"), { recursive: true });
  await writeFile(
    path.join(root, "linguini.toml"),
    '[paths]\nschema = "src/schema"\nlocale = "src/locale"\n'
  );
  await writeFile(path.join(root, "src/schema/shop/delivery.lgs"), "delivery()\n");
  await writeFile(path.join(root, "src/locale/shop/ru.lgl"), "delivery = OK\n");

  const files = (await discoverLinguiniFiles(root)).map((file) => path.relative(root, file));

  assert.deepEqual(files.sort(), [
    "linguini.toml",
    "src/locale/shop/ru.lgl",
    "src/schema/shop/delivery.lgs"
  ]);
});

test("hot update rebuilds and invalidates generated modules", async () => {
  const root = await mkdtemp(path.join(tmpdir(), "linguini-vite-"));
  const changedFile = path.join(root, "linguini/schema/shop.lgs");
  const builds = [];
  const invalidated = [];
  const events = [];
  const plugin = linguini({
    root,
    buildOnStart: false,
    build: (context) => builds.push(context)
  });
  const server = {
    watcher: {
      add() {},
      on() {}
    },
    moduleGraph: {
      idToModuleMap: new Map([
        ["a", { id: path.join(root, "src/generated/linguini/index.js") }],
        ["b", { id: path.join(root, "src/app.js") }]
      ]),
      invalidateModule(module) {
        invalidated.push(module.id);
      }
    },
    ws: {
      send(event) {
        events.push(event);
      }
    }
  };

  plugin.configResolved({ root });
  const result = await plugin.handleHotUpdate({ file: changedFile, server });

  assert.deepEqual(result, []);
  assert.equal(builds.length, 1);
  assert.equal(builds[0].reason, "hot-update");
  assert.deepEqual(invalidated, [path.join(root, "src/generated/linguini/index.js")]);
  assert.equal(events[0].event, "linguini:update");
});

test("recognizes only linguini source files", () => {
  const root = "/project";

  assert.equal(isLinguiniSource("/project/linguini.toml", root), true);
  assert.equal(isLinguiniSource("/project/linguini/schema/shop.lgs", root), true);
  assert.equal(isLinguiniSource("/project/src/app.ts", root), false);
});
