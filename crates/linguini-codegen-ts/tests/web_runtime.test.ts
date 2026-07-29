import assert from "node:assert/strict";
import test from "node:test";

import {
  createWebI18n,
  type LinguiniRuntime,
  type LinguiniWebOptions,
} from "../src/module/templates/web.runtime.ts";

type Locale = "en" | "fr";

const runtime: LinguiniRuntime<Locale, { locale: Locale }> = {
  locales: ["en", "fr"],
  baseLocale: "en",
  createLinguini: (locale) => ({ locale }),
};

function createWeb(options: LinguiniWebOptions = {}) {
  return createWebI18n(runtime, {
    origin: "https://app.example",
    ...options,
  });
}

test("localization changes only same-origin HTTP URLs", () => {
  const web = createWeb();
  const external = "https://outside.example:443/en/%7Euser?next=%2Fhome#Part";

  assert.equal(web.localizeHref(external, "fr"), external);
  assert.equal(
    web.localizeHref("//outside.example/en/account", "fr"),
    "//outside.example/en/account",
  );
  assert.equal(
    web.localizeHref("mailto:hello@example.com", "fr"),
    "mailto:hello@example.com",
  );
  assert.equal(
    web.localizeUrl(external, "fr").toString(),
    "https://outside.example/en/%7Euser?next=%2Fhome#Part",
  );

  assert.equal(
    web.localizeHref("https://app.example/en/account?tab=profile#name", "fr"),
    "https://app.example/fr/account?tab=profile#name",
  );
  assert.equal(web.localizeHref("/en/account", "fr"), "/fr/account");
  assert.equal(
    web.localizeUrl("/en/account", "fr", { currentUrl: "/en/current" }).toString(),
    "https://app.example/fr/account",
  );
});

test("locale path segments require an exact case-insensitive match", () => {
  const web = createWeb();

  assert.equal(web.localizeHref("/en-products", "fr"), "/fr/en-products");
  assert.equal(web.localizeHref("/en/products", "fr"), "/fr/products");
  assert.equal(web.localizeHref("/EN/products", "fr"), "/fr/products");
  assert.equal(web.delocalizeUrl("/en-products").pathname, "/en-products");
  assert.equal(web.delocalizeUrl("/en/products").pathname, "/products");

  const urlStrategy = createWebI18n(
    {
      locales: ["fr", "en"] as const,
      baseLocale: "fr" as const,
      createLinguini: (locale) => ({ locale }),
    },
    {
      origin: "https://app.example",
      strategy: ["url", "baseLocale"],
    },
  );
  assert.equal(
    urlStrategy.resolveLocaleSync({ url: "https://app.example/en-products" }),
    "fr",
  );
  assert.equal(
    urlStrategy.resolveLocaleSync({ url: "https://app.example/en/products" }),
    "en",
  );
});

test("recursive exclusion globs stop at path-segment boundaries", () => {
  const web = createWeb({ exclude: ["/api/**"] });

  assert.equal(web.shouldExclude("/api"), true);
  assert.equal(web.shouldExclude("/api/"), true);
  assert.equal(web.shouldExclude("/api/users"), true);
  assert.equal(web.shouldExclude("/apix"), false);
  assert.equal(web.shouldExclude("/apiary/users"), false);
});

test("global and sticky exclusion regular expressions are deterministic", () => {
  for (const matcher of [/^\/private/g, /^\/private/y]) {
    matcher.lastIndex = 4;
    const web = createWeb({ exclude: [matcher] });

    assert.equal(web.shouldExclude("/private/account"), true);
    assert.equal(matcher.lastIndex, 0);
    assert.equal(web.shouldExclude("/private/account"), true);
    assert.equal(matcher.lastIndex, 0);
  }
});

test("URL transforms share one parse error and boolean detection fails closed", () => {
  const web = createWeb();
  const invalid = "http://[";
  const expected = { name: "TypeError", message: "Linguini: invalid URL" };

  for (const operation of [
    () => web.localizeUrl(invalid, "fr"),
    () => web.localizeHref(invalid, "fr"),
    () => web.delocalizeUrl(invalid),
    () => web.alternateLinks(invalid),
    () => web.getCanonicalRedirect(invalid, "fr"),
    () => web.shouldExclude(invalid),
    () => web.localizeUrl("/account", "fr", { origin: invalid }),
  ]) {
    assert.throws(operation, expected);
  }

  assert.equal(web.shouldLocalizeHref(invalid), false);
  assert.equal(web.shouldLocalizeHref("/account", { origin: invalid }), false);
  assert.equal(web.localizeHrefAttribute(invalid, "fr"), invalid);
  assert.equal(web.resolveLocaleSync({ url: invalid }), "en");
});
