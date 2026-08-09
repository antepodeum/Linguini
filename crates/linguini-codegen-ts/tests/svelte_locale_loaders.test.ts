import assert from "node:assert/strict";
import { readFile, unlink, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { pathToFileURL } from "node:url";
import test from "node:test";

const transpiler = new Bun.Transpiler({ loader: "ts", target: "browser" });
let moduleNonce = 0;

async function readTemplate(name: string) {
  return readFile(new URL(`../src/module/templates/${name}`, import.meta.url), "utf8");
}

async function importLocaleTemplate(name: string) {
  const source = (await readTemplate(name))
    .replace(
      'import { baseLocale, normalizeLocale, type Locale } from "./locale";',
      `const locales = ["en", "fr"] as const;
const baseLocale = "en" as const;
const normalizeLocale = (value: unknown) => {
  if (typeof value !== "string") return undefined;
  return locales.find((locale) => locale.toLowerCase() === value.split("-")[0].toLowerCase());
};`,
    )
    .replace('import { page } from "$app/state";', 'const page = { data: {} };')
    .replace(/\$state<Locale>\(baseLocale\)/g, "baseLocale")
    .replace(/\$state\(false\)/g, "false");
  const javascript = transpiler.transformSync(source);
  const path = join(tmpdir(), `linguini-svelte-locale-loader-${process.pid}-${moduleNonce += 1}.mjs`);
  await writeFile(path, javascript);
  try {
    return await import(`${pathToFileURL(path).href}?v=${moduleNonce}`);
  } finally {
    await unlink(path);
  }
}

for (const template of [
  "svelte-locale.context.runtime.ts",
  "svelte-locale.standalone.runtime.ts",
  "svelte-locale.runtime.ts",
]) {
  test(`${template} prepares through an isolated loader registry`, async () => {
    const generated = await importLocaleTemplate(template);
    assert.equal(await generated.prepareLocale("unknown"), "en");

    const seen: string[] = [];
    let release!: () => void;
    const pending = new Promise<void>((resolve) => { release = resolve; });
    const dispose = generated.registerLocaleLoader(async (locale: string) => {
      seen.push(`first:${locale}`);
      await pending;
    });
    const secondDispose = generated.registerLocaleLoader((locale: string) => {
      seen.push(`second:${locale}`);
    });

    const preparing = generated.prepareLocale("fr-CA");
    secondDispose();
    release();
    assert.equal(await preparing, "fr");
    assert.deepEqual(seen, ["first:fr", "second:fr"]);

    dispose();
    assert.equal(await generated.prepareLocale("fr"), "fr");
    assert.deepEqual(seen, ["first:fr", "second:fr"]);
  });

  test(`${template} rejects loader failures without changing locale state`, async () => {
    const generated = await importLocaleTemplate(template);
    generated.setCurrentLocale("en");
    const dispose = generated.registerLocaleLoader(async () => {
      throw new Error("loader failed");
    });
    await assert.rejects(generated.prepareLocale("fr"), /loader failed/);
    assert.equal(generated.getCurrentLocale(), "en");
    dispose();
  });
}

test("locale loader declarations expose typed registration and preparation", async () => {
  for (const name of [
    "svelte-locale.context.runtime.d.ts",
    "svelte-locale.standalone.runtime.d.ts",
    "svelte-locale.runtime.d.ts",
  ]) {
    const source = await readTemplate(name);
    assert.match(source, /export type LinguiniLocaleLoader/);
    assert.match(source, /registerLocaleLoader\(loader: LinguiniLocaleLoader\): \(\) => void/);
    assert.match(source, /prepareLocale\(locale: unknown\): Promise<Locale>/);
  }
  assert.match(
    await readTemplate("svelte.context.runtime.ts"),
    /setCurrentLocale\(await prepareLocale\(nextLocale\)\)/,
  );
});
