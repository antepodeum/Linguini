import { test, expect } from "bun:test";
import { createWebI18n as legacy, createWebLocaleI18n as legacyLocale } from "./legacy/web.runtime";

import * as webPackage from "../../../../../packages/web/src/index.ts";

const resolverFactories = {
  path: webPackage.pathLocale,
  cookie: webPackage.cookieLocale,
  "local-storage": webPackage.localStorageLocale,
  "accept-language": webPackage.requestLanguage,
};
function runtimeOptions(options: any = {}) {
  const names: (keyof typeof resolverFactories)[] = options.locale?.sources ?? ["path", "cookie", "accept-language"];
  return {...options, locale:{...options.locale, sources:names.map(name => resolverFactories[name]())}};
}
const npm = {
  createWebI18n: (runtime: any, options?: any, environment?: any) => webPackage.createWebI18n(runtime, runtimeOptions(options), environment),
  createWebLocaleI18n: (runtime: any, options?: any, environment?: any) => webPackage.createWebLocaleI18n(runtime, runtimeOptions(options), environment),
};

const cases = [
  ["default", ["path", "cookie", "accept-language"]],
  ["all", ["path", "cookie", "local-storage", "accept-language"]],
  ["empty", []], ["path", ["path"]], ["cookie", ["cookie"]],
  ["storage", ["local-storage"]], ["language", ["accept-language"]],
  ["language-cookie", ["accept-language", "cookie"]],
] as const;
const runtime = { locales: ["en", "fr"] as const, baseLocale: "en" as const, createLinguini: (locale: "en" | "fr") => ({ locale }) };
const environment = { base: "/app", origin: "https://example.test" };

for (const [name, sources] of cases) {
  test(`${name}: closed web sources, URL/link safety, cookies and context match both targets`, async () => {
    const js = await import(`./${name}/web.js`);
    const ts = await import(`./${name}/typescript/web.ts`);
    const options = { locale: { sources }, routes: { exclude: name === "all" ? ["/api/**", /^\/private/g, (url: URL) => url.pathname === "/predicate"] : [] } };
    const oracle = legacy(runtime, options, environment);
    for (const compiler of [js, ts, npm]) {
      const web = compiler.createWebI18n(runtime, options, environment);
      expect(web.options).toEqual(oracle.options);
      const inputs = [
        {}, { url: "https://example.test/app/fr/shop", cookie: "LINGUINI_LOCALE=en", headers: new Headers({ "accept-language": "en" }) },
        { cookie: "LINGUINI_LOCALE=fr", headers: new Headers({ "accept-language": "en" }) },
        { localStorage: { getItem: () => "fr" } }, { localStorage: { getItem: () => { throw Error("denied"); } } },
        { headers: new Headers({ "accept-language": "fr;q=0, *;q=0.5" }) },
        { navigator: { languages: ["fr-CA"], language: "en" } }, { cookie: "LINGUINI_LOCALE=%ZZ", url: "http://[" },
      ];
      for (const input of inputs) {
        expect(web.resolveLocaleSync(input)).toBe(oracle.resolveLocaleSync(input));
        expect(await web.resolveLocale(input)).toBe(oracle.resolveLocaleSync(input));
      }
      for (const prefix of ["always", "except-default", "never"] as const) {
        const opts = { ...options, routing: { localePrefix: prefix } };
        const actual = compiler.createWebI18n(runtime, opts, environment), expected = legacy(runtime, opts, environment);
        for (const href of ["/app/en/shop?x=1#part", "/app/en-products", "/shop", "#anchor", "//external.test/a", "mailto:a@example.test", "https://external.test/app/en/shop", "https://example.test/app/fr/shop"]) {
          expect(actual.localizeHref(href, "fr")).toBe(expected.localizeHref(href, "fr"));
          expect(actual.delocalizeUrl(href).href).toBe(expected.delocalizeUrl(href).href);
          expect(actual.alternateLinks(href)).toEqual(expected.alternateLinks(href));
          expect(actual.getCanonicalRedirect(href, "fr")).toBe(expected.getCanonicalRedirect(href, "fr"));
          for (const attributes of [{}, { download: true }, { ignored: true }, { rel: "help EXTERNAL" }]) {
            expect(actual.shouldLocalizeLink(href, attributes)).toBe(expected.shouldLocalizeLink(href, attributes));
            if (compiler === npm) {
              expect(actual.localizeLinkHref(href, 'fr', attributes)).toBe(
                expected.shouldLocalizeLink(href, attributes) ? expected.localizeHref(href, 'fr') : href,
              );
            }
          }
        }
      }
      expect(web.shouldLocalizeHref("http://[")).toBe(false);
      expect(() => web.localizeUrl("http://[", "fr")).toThrow("Linguini: invalid URL");
      // Closed generation omits the parser when no exclusion capability exists.
      if (name !== "all") expect(web.shouldExclude("http://[")).toBe(false);
      else {
        expect(() => web.shouldExclude("http://[")).toThrow("Linguini: invalid URL");
        for (const path of ["/api", "/api/users", "/apix", "/private", "/predicate"]) {
          expect(web.shouldExclude(path)).toBe(oracle.shouldExclude(path));
          expect(web.shouldExclude(path)).toBe(oracle.shouldExclude(path));
        }
      }
      for (const input of [{}, { origin: "https://example.test", httpOnly: true }, { origin: "http://example.test", secure: true, maxAge: 5 }]) {
        expect(web.serializeLocaleCookie("fr", input)).toBe(oracle.serializeLocaleCookie("fr", input));
        const headers = new Headers({ "set-cookie": "theme=dark" });
        web.setLocaleCookie({ headers }, "fr", input);
        expect(headers.get("set-cookie")).toContain("theme=dark");
        expect(headers.get("set-cookie")).toContain(oracle.serializeLocaleCookie("fr", input));
      }
      const context = web.createRequestContext("fr", { origin: "https://request.test", currentUrl: "https://request.test/app/shop" });
      expect(context.messages).toEqual({ locale: "fr" });
      expect(context.l).toBe(context.messages);
      expect(context.localizeUrl("/shop").origin).toBe("https://request.test");
      expect(context.localizeHref("/shop")).toBe("/app/fr/shop");
      expect(compiler.createWebLocaleI18n(runtime, options, environment).createRequestContext).toBeUndefined();
      expect(legacyLocale(runtime, options, environment).createRequestContext).toBeUndefined();
    }
  });
}

test("request preparation precedes factories and concurrent requests retain their own input", async () => {
  for (const target of ["web.js", "typescript/web.ts", "npm"]) {
    const compiler = target === "npm" ? npm : await import(`./all/${target}`);
    const prepared = new Set<string>(), calls: string[] = [];
    const web = compiler.createWebI18n({ ...runtime,
      async prepareLinguini(locale: string) { await Bun.sleep(locale === "fr" ? 5 : 1); prepared.add(locale); calls.push(`prepare:${locale}`); },
      createLinguini(locale: string) { expect(prepared.has(locale)).toBe(true); calls.push(`create:${locale}`); return { locale }; },
    }, {}, environment);
    const [fr, en] = await Promise.all([
      web.resolveRequest(new Request("https://fr.test/app/fr/shop")),
      web.resolveRequest(new Request("https://en.test/app/en/shop")),
    ]);
    expect(fr.messages.locale).toBe("fr"); expect(en.messages.locale).toBe("en");
    expect(fr.localizeUrl("/next").origin).toBe("https://fr.test");
    expect(en.localizeUrl("/next").origin).toBe("https://en.test");
    expect(calls).toEqual(["prepare:en", "create:en", "prepare:fr", "create:fr"]);
    const failing = compiler.createWebI18n({ ...runtime, prepareLinguini: async () => { throw Error("load failed"); }, createLinguini: () => { throw Error("factory ran early"); } });
    await expect(failing.resolveRequest(new Request("https://example.test/app/fr"))).rejects.toThrow("load failed");
  }
});
