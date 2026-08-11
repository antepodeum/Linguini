import assert from "node:assert/strict";
import { readFile, unlink, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { pathToFileURL } from "node:url";
import test from "node:test";

const transpiler = new Bun.Transpiler({ loader: "ts", target: "browser" });
let moduleNonce = 0;

async function importTypeScript(source: string) {
  const javascript = transpiler.transformSync(source);
  const path = join(tmpdir(), `linguini-svelte-browser-guard-${process.pid}-${moduleNonce += 1}.mjs`);
  await writeFile(path, javascript);
  try {
    return await import(pathToFileURL(path).href);
  } finally {
    await unlink(path);
  }
}

async function readTemplate(name: string) {
  return readFile(new URL(`../src/module/templates/${name}`, import.meta.url), "utf8");
}

test("effects initialization tolerates denied browser capability getters", async () => {
  const source = (await readTemplate("svelte-effects.runtime.ts"))
    .replace(
      "{{BROWSER_RUNTIME}}",
      "const browser = typeof window !== \"undefined\" && typeof document !== \"undefined\";",
    )
    .replace("{{OPTIONS}}", "{ localizeLinks: false }")
    .replace(
      'import * as locale from "./locale";',
      'const locale = { locales: ["en"], baseLocale: "en" };',
    )
    .replace('import type { Locale } from "./locale";', 'type Locale = "en";')
    .replace(
      'import { createWebLocaleI18n } from "./web";',
      `const createWebLocaleI18n = (runtime: typeof locale, options: Record<string, unknown>) => ({
        ...runtime,
        options,
        resolveLocaleSync(input: Record<string, unknown>) {
          globalThis.__linguiniProbeInput = input;
          return runtime.baseLocale;
        },
      });`,
    )
    .replace(
      `import {
  getCurrentLocale,
  initializeCurrentLocale,
} from "./svelte-locale.svelte.js";`,
      `const getCurrentLocale = () => "en" as const;
const initializeCurrentLocale = (locale: unknown) => {
  globalThis.__linguiniProbeLocale = locale;
};`,
    );

  Object.defineProperty(globalThis, "window", {
    configurable: true,
    value: {
      get location() { throw new Error("location denied"); },
      get localStorage() { throw new Error("storage denied"); },
      get navigator() { throw new Error("navigator denied"); },
    },
  });
  Object.defineProperty(globalThis, "document", {
    configurable: true,
    value: {
      get cookie() { throw new Error("cookie denied"); },
    },
  });

  try {
    await importTypeScript(source);
    assert.equal(globalThis.__linguiniProbeLocale, "en");
    assert.deepEqual(globalThis.__linguiniProbeInput, {
      url: undefined,
      cookie: undefined,
      localStorage: undefined,
      navigator: undefined,
    });
  } finally {
    delete globalThis.window;
    delete globalThis.document;
    delete globalThis.__linguiniProbeInput;
    delete globalThis.__linguiniProbeLocale;
  }
});

test("effects initialization forwards navigator preferences to locale resolver", async () => {
  const source = (await readTemplate("svelte-effects.runtime.ts"))
    .replace(
      "{{BROWSER_RUNTIME}}",
      "const browser = typeof window !== \"undefined\" && typeof document !== \"undefined\";",
    )
    .replace("{{OPTIONS}}", "{ localizeLinks: false, sources: [\"accept-language\"] }")
    .replace(
      'import * as locale from "./locale";',
      'const locale = { locales: ["en", "fr"], baseLocale: "en" };',
    )
    .replace('import type { Locale } from "./locale";', 'type Locale = "en" | "fr";')
    .replace(
      'import { createWebLocaleI18n } from "./web";',
      `const createWebLocaleI18n = (runtime: typeof locale, options: Record<string, unknown>) => ({
        ...runtime,
        options,
        resolveLocaleSync(input: Record<string, unknown>) {
          globalThis.__linguiniProbeInput = input;
          const navigator = input.navigator as { languages?: unknown[] } | undefined;
          return navigator?.languages?.includes("fr") ? "fr" : runtime.baseLocale;
        },
      });`,
    )
    .replace(
      `import {
  getCurrentLocale,
  initializeCurrentLocale,
} from "./svelte-locale.svelte.js";`,
      `const getCurrentLocale = () => "en" as const;
const initializeCurrentLocale = (locale: unknown) => {
  globalThis.__linguiniProbeLocale = locale;
};`,
    );

  Object.defineProperty(globalThis, "window", {
    configurable: true,
    value: {
      location: { href: "https://app.example/" },
      localStorage: undefined,
      navigator: { languages: ["fr", "en"], language: "fr" },
    },
  });
  Object.defineProperty(globalThis, "document", {
    configurable: true,
    value: { cookie: "" },
  });

  try {
    await importTypeScript(source);
    assert.deepEqual(globalThis.__linguiniProbeInput?.navigator, {
      languages: ["fr", "en"],
      language: "fr",
    });
    assert.equal(globalThis.__linguiniProbeLocale, "fr");
  } finally {
    delete globalThis.window;
    delete globalThis.document;
    delete globalThis.__linguiniProbeInput;
    delete globalThis.__linguiniProbeLocale;
  }
});

test("setLocale prepares before mutation and ignores persistence failures", async () => {
  const source = (await readTemplate("svelte-control.runtime.ts"))
    .replace("{{NAVIGATION_RUNTIME}}", "const browser = true;")
    .replace(
      `import {
  destroyLinguiniEffects,
  refreshLinguiniEffects,
  web,
} from "./svelte-effects.svelte.js";`,
      `const web = {
  baseLocale: "en",
  options: { localStorageKey: "locale" },
  matchLocale: (locale: unknown) => locale === "fr" ? "fr" : undefined,
  serializeLocaleCookie: () => "locale=fr",
};
const destroyLinguiniEffects = () => {};
const refreshLinguiniEffects = () => {};`,
    )
    .replace(
      "{{LOCALE_RUNTIME}}",
      `const getCurrentLocale = () => globalThis.__linguiniProbeCurrent ?? "en";
const prepareLocale = async (locale: string) => locale;
const setCurrentLocale = (locale: string) => {
  globalThis.__linguiniProbeCurrent = locale;
  return locale;
};`,
    )
    .replace("{{NAVIGATION}}", "        throw new Error(\"unexpected navigation\");");

  Object.defineProperty(globalThis, "window", {
    configurable: true,
    value: {
      localStorage: {
        setItem() { throw new Error("storage denied"); },
      },
    },
  });
  Object.defineProperty(globalThis, "document", {
    configurable: true,
    value: {
      set cookie(_value: string) { throw new Error("cookie denied"); },
    },
  });

  try {
    const generated = await importTypeScript(source);
    const pending = generated.setLocale("fr", { navigate: false, cookie: true });
    assert.equal(globalThis.__linguiniProbeCurrent, undefined);
    assert.equal(await pending, "fr");
    assert.equal(globalThis.__linguiniProbeCurrent, "fr");
  } finally {
    delete globalThis.window;
    delete globalThis.document;
    delete globalThis.__linguiniProbeCurrent;
  }
});

test("lightweight controls and SvelteKit hooks have no eager message imports", async () => {
  const controls = await readTemplate("svelte-control.runtime.ts");
  const svelte = await readTemplate("svelte.runtime.ts");
  const sveltekit = await readTemplate("sveltekit-control.runtime.ts");
  const legacySveltekit = await readTemplate("sveltekit.runtime.ts");

  for (const source of [controls, sveltekit]) {
    for (const forbidden of [
      'from "./index"',
      'import("./index")',
      'from "./locales/',
      'from "./messages',
    ]) {
      assert.ok(!source.includes(forbidden), `unexpected eager import ${forbidden}`);
    }
  }
  assert.match(sveltekit, /import \* as locale from "\.\/locale"/);
  assert.match(sveltekit, /createWebLocaleI18n/);
  assert.match(legacySveltekit, /from "\.\/index"/);
  assert.match(legacySveltekit, /createWebI18n/);
  assert.match(svelte, /get locale\(\) \{\s+return controls\.locale;/);
  assert.ok(!svelte.includes("...controls"));
});

declare global {
  var __linguiniProbeCurrent: string | undefined;
  var __linguiniProbeInput: Record<string, unknown> | undefined;
  var __linguiniProbeLocale: unknown;
}
