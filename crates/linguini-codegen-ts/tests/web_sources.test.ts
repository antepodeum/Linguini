import { test, expect } from "bun:test";
import { resolvePathLocale } from "../src/module/templates/web.path.runtime.ts";
import { resolveCookieLocale } from "../src/module/templates/web.cookie.runtime.ts";
import { resolveLocalStorageLocale } from "../src/module/templates/web.local-storage.runtime.ts";
import { resolveAcceptLanguageLocale } from "../src/module/templates/web.accept-language.runtime.ts";
import { createWebLocaleI18n } from "../src/module/templates/web.runtime.ts";
import { matchesRoute } from "../src/module/templates/web.routes.runtime.ts";

const locales = ["en", "de", "fr"] as const;
const match = (value: unknown) => locales.find((locale) => locale.toLowerCase() === String(value).toLowerCase());

test("selected route matcher shares exact and recursive exclusion semantics", () => {
  expect(matchesRoute("/api/**", new URL("https://example.test/api/orders"))).toBe(true);
  expect(matchesRoute("/api/**", new URL("https://example.test/apiculture"))).toBe(false);
  expect(matchesRoute("/health", new URL("https://example.test/health"))).toBe(true);
  const global = /^\/admin/g;
  expect(matchesRoute(global, new URL("https://example.test/admin"))).toBe(true);
  expect(matchesRoute(global, new URL("https://example.test/admin"))).toBe(true);
});

test("selected source modules preserve validated source semantics", () => {
  expect(resolvePathLocale({ url: "https://example.test/de/shop" }, { environment: { base: "" } }, locales)).toBe("de");
  expect(resolvePathLocale({ url: "https://example.test/shop" }, { environment: { base: "" } }, locales)).toBeUndefined();
  expect(resolveCookieLocale({ cookie: "LINGUINI_LOCALE=fr" }, { cookie: { name: "LINGUINI_LOCALE" } }, match)).toBe("fr");
  expect(resolveCookieLocale({ cookie: "LINGUINI_LOCALE=%ZZ" }, { cookie: { name: "LINGUINI_LOCALE" } }, match)).toBeUndefined();
  const storage = { getItem: (key: string) => key === "locale" ? "de" : null } as Storage;
  expect(resolveLocalStorageLocale({ localStorage: storage }, { localStorage: { key: "locale" } }, match)).toBe("de");
  expect(resolveAcceptLanguageLocale(
    { headers: new Headers({ "accept-language": "fr-CA, de;q=0.8" }) },
    locales,
    "en",
    match,
  )).toBe("fr");
});

test("physical source modules preserve structured runtime resolver edge behavior", () => {
  const pathCases = [
    { input: { url: "https://example.test/shop/de/orders" }, basePath: "/shop", expected: "de" },
    { input: { url: "https://example.test/shopkeeper/de" }, basePath: "/shop", expected: undefined },
    { input: { url: "not a URL" }, basePath: "", expected: undefined },
    { input: { url: new URL("https://example.test/FR") }, basePath: "", expected: "fr" },
  ];
  for (const { input, basePath, expected } of pathCases) {
    const runtimePath = createWebLocaleI18n(
      { locales, baseLocale: "en" },
      { locale: { sources: ["path"] } },
      { base: basePath },
    );
    const physical = resolvePathLocale(input, { environment: { base: basePath } }, locales);
    expect(physical).toBe(expected);
    expect(runtimePath.resolveLocaleSync(input)).toBe(physical ?? "en");
  }

  const cookieCases = [
    ["other=fr; LINGUINI_LOCALE=de", "de"],
    ["LINGUINI_LOCALE=de=ignored", undefined],
    ["LINGUINI_LOCALE=%E0%A4%A", undefined],
    [undefined, undefined],
  ];
  for (const [cookie, expected] of cookieCases) {
    const runtimeCookie = createWebLocaleI18n({ locales, baseLocale: "en" }, {
      locale: { sources: ["cookie"] },
    });
    const physical = resolveCookieLocale({ cookie }, { cookie: { name: "LINGUINI_LOCALE" } }, match);
    expect(physical).toBe(expected);
    expect(runtimeCookie.resolveLocaleSync({ cookie })).toBe(physical ?? "en");
  }

  const storageCases = [
    { getItem: () => "fr", expected: "fr" },
    { getItem: () => null, expected: undefined },
    { getItem: () => { throw new Error("storage denied"); }, expected: undefined },
  ];
  for (const { expected, ...localStorage } of storageCases) {
    const physical = resolveLocalStorageLocale({ localStorage }, { localStorage: { key: "locale" } }, match);
    const runtimeStorage = createWebLocaleI18n({ locales, baseLocale: "en" }, {
      locale: { sources: ["local-storage"] },
      localStorage: { key: "locale" },
    });
    expect(physical).toBe(expected);
    expect(runtimeStorage.resolveLocaleSync({ localStorage })).toBe(physical ?? "en");
  }

  const acceptCases = [
    { input: { headers: new Headers({ "accept-language": "de;q=1, fr;q=0.7" }) }, expected: "de" },
    { input: { headers: new Headers({ "accept-language": "fr;q=0, *;q=0.5" }) }, expected: "en" },
    { input: { headers: new Headers({ "accept-language": "fr;q=invalid, en;q=0.5" }) }, expected: "en" },
    { input: { navigator: { languages: ["fr-CA", "en-US"], language: "en-US" } }, expected: "fr" },
    { input: { navigator: { languages: "fr", language: "en" } }, expected: "en" },
  ];
  const runtimeAccept = createWebLocaleI18n({ locales, baseLocale: "en" }, {
    locale: { sources: ["accept-language"] },
  });
  for (const { input, expected } of acceptCases) {
    const physical = resolveAcceptLanguageLocale(input, locales, "en", match);
    expect(physical).toBe(expected);
    expect(runtimeAccept.resolveLocaleSync(input)).toBe(physical ?? "en");
  }
});
