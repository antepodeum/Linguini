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
    .replace("{{LINK_RUNTIME_IMPORT}}", "")
    .replace(
      "{{LINK_RUNTIME_START}}",
      "const linkEffects: { refresh(): void; destroy(): void } | undefined = undefined;",
    )
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
    .replace("{{LINK_RUNTIME_IMPORT}}", "")
    .replace(
      "{{LINK_RUNTIME_START}}",
      "const linkEffects: { refresh(): void; destroy(): void } | undefined = undefined;",
    )
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
  options: {
    localStorageKey: "locale",
    localeSwitch: { writesPath: true, writesCookie: true, writesLocalStorage: true },
  },
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

  let storageAttempts = 0;
  let cookieAttempts = 0;
  Object.defineProperty(globalThis, "window", {
    configurable: true,
    value: {
      localStorage: {
        setItem() {
          storageAttempts += 1;
          throw new Error("storage denied");
        },
      },
    },
  });
  Object.defineProperty(globalThis, "document", {
    configurable: true,
    value: {
      set cookie(_value: string) {
        cookieAttempts += 1;
        throw new Error("cookie denied");
      },
    },
  });

  try {
    const generated = await importTypeScript(source);
    const pending = generated.setLocale("fr", { navigate: false, cookie: true });
    assert.equal(globalThis.__linguiniProbeCurrent, undefined);
    assert.equal(await pending, "fr");
    assert.equal(globalThis.__linguiniProbeCurrent, "fr");
    assert.equal(storageAttempts, 1);
    assert.equal(cookieAttempts, 1);
  } finally {
    delete globalThis.window;
    delete globalThis.document;
    delete globalThis.__linguiniProbeCurrent;
  }
});

test("setLocale obeys generated locale switch transports", async () => {
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
  options: {
    localStorageKey: "locale",
    localeSwitch: { writesPath: false, writesCookie: false, writesLocalStorage: false },
  },
  serializeLocaleCookie: () => "locale=fr",
  localizeHref: () => "/fr",
  getTextDirection: () => "ltr",
  htmlAttrs: () => ({ lang: "fr", dir: "ltr" }),
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

  let storageWrites = 0;
  let cookieWrites = 0;
  Object.defineProperty(globalThis, "window", {
    configurable: true,
    value: { localStorage: { setItem() { storageWrites += 1; } } },
  });
  Object.defineProperty(globalThis, "document", {
    configurable: true,
    value: { set cookie(_value: string) { cookieWrites += 1; } },
  });

  try {
    const generated = await importTypeScript(source);
    assert.equal(await generated.setLocale("fr", { cookie: true }), "fr");
    assert.equal(storageWrites, 0);
    assert.equal(cookieWrites, 0);
  } finally {
    delete globalThis.window;
    delete globalThis.document;
    delete globalThis.__linguiniProbeCurrent;
  }
});

test("SvelteKit adapters gate server cookie persistence through locale switch plan", async () => {
  for (const template of ["sveltekit.runtime.ts", "sveltekit-control.runtime.ts"]) {
    for (const writesCookie of [false, true]) {
      const source = (await readTemplate(template))
        .replace(
          'import type { Handle, Reroute, ServerLoad } from "@sveltejs/kit";',
          "",
        )
        .replace(
          'import type { Locale } from "./locale";',
          'type Locale = "en";',
        )
        .replace(
          'import * as locale from "./locale";',
          'const locale = { locales: ["en"], baseLocale: "en" };',
        )
        .replace(
          'import * as runtime from "./index";',
          'const runtime = { locales: ["en"], baseLocale: "en" };',
        )
        .replace(
          'import { createWebI18n } from "./web";',
          "const createWebI18n = (runtime: any, options: any) => createWeb(runtime, options);",
        )
        .replace(
          'import { createWebLocaleI18n } from "./web";',
          "const createWebLocaleI18n = (runtime: any, options: any) => createWeb(runtime, options);",
        )
        .replace(
          "const options = {{OPTIONS}};",
          `const options = {
  localeSwitch: { writesPath: false, writesCookie: ${writesCookie}, writesLocalStorage: false },
};`,
        )
        .replace(
          "{{SERVER_COOKIE_IMPORT}}",
          "const persistLocaleCookie = (web: any, target: unknown, locale: string) => web.setLocaleCookie(target, locale);",
        )
        .replace(
          "{{PERSIST_COOKIE_DECLARATION}}",
          "  const persistCookie = options.persistCookie !== false && web.options.localeSwitch.writesCookie;",
        )
        .replace(
          "{{COOKIE_INPUT}}",
          '      cookie: event.request.headers.get("cookie") ?? undefined,',
        )
        .replace(
          "{{PERSIST_REDIRECT_COOKIE}}",
          "      if (persistCookie) persistLocaleCookie(web, response, context.locale);",
        )
        .replace(
          "{{PERSIST_RESPONSE_COOKIE}}",
          "    if (persistCookie) persistLocaleCookie(web, response, context.locale);",
        );
      const sourceWithWeb = `${source}
function createWeb(_runtime: unknown, options: any) {
  const context = {
    locale: "en",
    baseLocale: "en",
    locales: ["en"],
    direction: "ltr",
    textDirection: "ltr",
    lang: "en",
    htmlAttrs: { lang: "en", dir: "ltr" },
  };
  return {
    options: { localeSwitch: options.localeSwitch },
    baseLocale: "en",
    locales: ["en"],
    shouldExclude: () => false,
    resolveRequest: async () => context,
    resolveLocale: async () => "en",
    matchLocale: () => "en",
    getTextDirection: () => "ltr",
    htmlAttrs: () => ({ lang: "en", dir: "ltr" }),
    localizeHref: (href: string) => href,
    localizeUrl: (url: string | URL) => new URL(url),
    shouldLocalizeHref: () => false,
    shouldLocalizeLink: () => false,
    localizeHrefAttribute: (href: string) => href,
    delocalizeUrl: (url: string | URL) => new URL(url),
    alternateLinks: () => [],
    delocalizePathname: (pathname: string) => pathname,
    getCanonicalRedirect: () => undefined,
    setLocaleCookie: () => { globalThis.__linguiniProbeCookieWrites += 1; },
  };
}
`;

      globalThis.__linguiniProbeCookieWrites = 0;
      const generated = await importTypeScript(sourceWithWeb);
      await generated.handle({
        event: {
          url: new URL("https://app.example/"),
          request: { headers: new Headers() },
          locals: {},
        },
        resolve: async () => new Response("ok"),
      });
      assert.equal(globalThis.__linguiniProbeCookieWrites, writesCookie ? 1 : 0);
    }
  }
  delete globalThis.__linguiniProbeCookieWrites;
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
  var __linguiniProbeCookieWrites: number;
  var __linguiniProbeInput: Record<string, unknown> | undefined;
  var __linguiniProbeLocale: unknown;
}
