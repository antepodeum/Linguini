import assert from "node:assert/strict";
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
