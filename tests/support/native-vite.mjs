import assert from "node:assert/strict";
import { execFile } from "node:child_process";
import { mkdir, mkdtemp, readFile, rm, symlink, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { promisify } from "node:util";

const run = promisify(execFile);
const repository = fileURLToPath(new URL("../../", import.meta.url));
const executable = path.join(repository, "target/debug", process.platform === "win32" ? "linguini.exe" : "linguini");

export async function buildNativeCompiler() {
  await run("cargo", ["build", "--locked", "-p", "linguini-cli"], {
    cwd: repository, maxBuffer: 4 * 1024 * 1024,
  });
}

export async function nativeFrameworkProject(context, { label, packageRoot, extension, components, references = ["main.title"] }) {
  const root = await mkdtemp(path.join(tmpdir(), `linguini-${label}-`));
  context.after(() => rm(root, { recursive: true, force: true }));
  await symlink(path.join(packageRoot, "node_modules"), path.join(root, "node_modules"), "junction");
  await mkdir(path.join(root, "schema"));
  await mkdir(path.join(root, "locales/main"), { recursive: true });
  await mkdir(path.join(root, "src"));
  await writeFile(path.join(root, "package.json"), '{"type":"module"}');
  await writeFile(path.join(root, "linguini.toml"), `
[project]
name = "framework"
default_locale = "en"
locales = ["en", "fr"]
[paths]
schema = "schema"
locale = "locales"
[targets.ts]
out = "generated"
[targets.ts.bundler]
sources = ["src"]
locale_loading = "dynamic"
`);
  await writeFile(path.join(root, "schema/main.lgs"), "title\nhello(name: String)\nunused\n");
  await writeFile(path.join(root, "locales/main/en.lgl"), `
fn Greeting(name: String) { _ => Hello {name} }
title = PAGE_TITLE_SENTINEL
hello = {Greeting(name)}
unused = UNUSED_PAGE_SENTINEL
`);
  await writeFile(path.join(root, "locales/main/fr.lgl"), `
title = INACTIVE_FRENCH_SENTINEL
hello = Bonjour {name}
unused = INACTIVE_UNUSED_SENTINEL
`);
  for (const [name, source] of Object.entries(components)) {
    await writeFile(path.join(root, "src", `${name}.${extension}`), source);
  }
  await run(executable, ["build"], { cwd: root });
  const manifest = await readFile(path.join(root, "generated/bundler/manifest.json"));
  await run(executable, ["build"], { cwd: root });
  assert.deepEqual(await readFile(path.join(root, "generated/bundler/manifest.json")), manifest);
  const { applications, messages } = JSON.parse(manifest);
  assert.deepEqual(Object.keys(applications).sort(), [`src/Child.${extension}`, `src/Other.${extension}`, `src/Page.${extension}`]);
  assert.deepEqual(applications[`src/Page.${extension}`].references.map((item) => item.message), references);
  return { root, messages, entry: path.join(root, "src", `Page.${extension}`) };
}

export function assertExactPage(result, messages) {
  const chunks = result.output.filter((item) => item.type === "chunk");
  assert.equal(chunks.length, 1, "one composed page must not require separate message requests");
  const chunk = chunks[0];
  assert.match(chunk.code, /PAGE_TITLE_SENTINEL/);
  assert.match(chunk.code, /Hello/);
  assert.doesNotMatch(chunk.code, /UNUSED_PAGE_SENTINEL|INACTIVE_|locale-registry|svelte|import\(/);
  assert.deepEqual(chunk.dynamicImports, []);
  const ids = Object.keys(chunk.modules).map((id) => id.replaceAll("\\", "/"));
  for (const canonical of ["main.title", "main.hello"]) {
    const module = messages[canonical].locales.en.module;
    assert.ok(ids.some((id) => id.endsWith(module)), `missing selected native leaf ${module}: ${ids}`);
  }
  for (const [canonical, message] of Object.entries(messages)) {
    for (const [locale, entry] of Object.entries(message.locales)) {
      if (locale !== "en" || canonical === "main.unused") {
        assert.ok(!ids.some((id) => id.endsWith(entry.module)), `unselected native leaf ${entry.module}`);
      }
    }
  }
  assert.ok(!ids.some((id) => id.endsWith("/generated/index.js")));
  return chunk.code;
}
