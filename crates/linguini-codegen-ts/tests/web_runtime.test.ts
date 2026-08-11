import assert from "node:assert/strict";
import test from "node:test";

import {
  createWebI18n,
  createWebLocaleI18n,
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

test("metadata-only web runtime localizes without a message factory", () => {
  const web = createWebLocaleI18n(
    {
      locales: ["en", "fr"] as const,
      baseLocale: "en" as const,
    },
    { origin: "https://app.example" },
  );

  assert.equal(web.localizeHref("/en/account", "fr"), "/fr/account");
  assert.equal("createLinguini" in web, false);
  assert.equal("createRequestContext" in web, false);
});

test("locale prefix modes preserve their exact URL policy", () => {
  const always = createWeb({ localePrefix: "always" });
  assert.equal(always.localizeHref("/account", "en"), "/en/account");
  assert.equal(always.localizeHref("/account", "fr"), "/fr/account");
  assert.equal(always.localizeHref("/fr/account", "en"), "/en/account");
  assert.equal(always.options.localePrefix, "always");
  assert.equal(always.options.prefixDefaultLocale, true);

  const exceptDefault = createWeb({ localePrefix: "except-default" });
  assert.equal(exceptDefault.localizeHref("/account", "en"), "/account");
  assert.equal(exceptDefault.localizeHref("/account", "fr"), "/fr/account");
  assert.equal(exceptDefault.localizeHref("/en/account", "en"), "/account");
  assert.equal(exceptDefault.localizeHref("/fr/account", "en"), "/account");
  assert.equal(exceptDefault.localizeHref("/en/account", "fr"), "/fr/account");
  assert.equal(exceptDefault.options.localePrefix, "except-default");
  assert.equal(exceptDefault.options.prefixDefaultLocale, false);

  const never = createWeb({ localePrefix: "never" });
  assert.equal(never.localizeHref("/account", "en"), "/account");
  assert.equal(never.localizeHref("/account", "fr"), "/account");
  for (const path of ["/en/account", "/fr/account", "/en-products"]) {
    assert.equal(never.localizeHref(path, "en"), path);
    assert.equal(never.localizeHref(path, "fr"), path);
    assert.equal(never.localizeUrl(path, "fr").pathname, path);
    assert.equal(never.delocalizePathname(path), path);
  }
  assert.equal(
    never.delocalizeUrl("/fr/account?tab=profile#name").toString(),
    "https://app.example/fr/account?tab=profile#name",
  );
  assert.equal(never.getCanonicalRedirect("/fr/account", "fr"), undefined);
  assert.equal(never.getCanonicalRedirect("/account", "fr"), undefined);
  assert.deepEqual(
    never.alternateLinks("/fr/account").map((link) => link.href),
    [
      "https://app.example/fr/account",
      "https://app.example/fr/account",
      "https://app.example/fr/account",
    ],
  );
  assert.equal(never.options.localePrefix, "never");
});

test("legacy prefixDefaultLocale remains compatible when localePrefix is omitted", () => {
  const legacyAlways = createWeb({ prefixDefaultLocale: true });
  assert.equal(legacyAlways.localizeHref("/account", "en"), "/en/account");
  assert.equal(legacyAlways.options.localePrefix, "always");
  const legacyExceptDefault = createWeb({ prefixDefaultLocale: false });
  assert.equal(legacyExceptDefault.localizeHref("/account", "en"), "/account");
  assert.equal(legacyExceptDefault.options.localePrefix, "except-default");
  assert.equal(
    createWeb({ localePrefix: "always", prefixDefaultLocale: false }).localizeHref(
      "/account",
      "en",
    ),
    "/en/account",
  );
  assert.equal(
    createWeb({ localePrefix: "except-default", prefixDefaultLocale: true }).localizeHref(
      "/account",
      "en",
    ),
    "/account",
  );
  assert.equal(
    createWeb({ localePrefix: "never", prefixDefaultLocale: true }).localizeHref("/account", "fr"),
    "/account",
  );
});

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

test("anchor skip attributes use the same link-localization policy", () => {
  const web = createWeb();

  assert.equal(web.shouldLocalizeLink("/account"), true);
  assert.equal(web.shouldLocalizeLink("/account", { download: true }), false);
  assert.equal(web.shouldLocalizeLink("/account", { ignored: true }), false);
  assert.equal(web.shouldLocalizeLink("/account", { rel: "help EXTERNAL" }), false);
  assert.equal(web.shouldLocalizeLink("/account", { rel: "help" }), true);
  assert.equal(web.shouldLocalizeLink("https://outside.example/account"), false);
});

test("web locale matching delegates fallback instead of truncating tags", () => {
  const web = createWeb({ sources: ["cookie"] });

  assert.equal(web.matchLocale("fr"), "fr");
  assert.equal(web.matchLocale("fr-CA"), undefined);
  assert.equal(
    web.resolveLocaleSync({ cookie: "LINGUINI_LOCALE=fr-CA" }),
    "en",
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
      sources: ["path"],
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

test("locale cookies append without replacing existing response headers", () => {
  const web = createWeb();
  const values = ["theme=dark; Path=/"];
  let overwritten = false;
  web.setLocaleCookie(
    {
      headers: {
        append(name: string, value: string) {
          assert.equal(name, "set-cookie");
          values.push(value);
        },
      },
      setHeaders() {
        overwritten = true;
      },
    },
    "fr",
  );

  assert.equal(overwritten, false);
  assert.equal(values[0], "theme=dark; Path=/");
  assert.match(values[1], /^LINGUINI_LOCALE=fr;/);
});

test("locale cookies use the SvelteKit cookies interface when available", () => {
  const web = createWeb();
  const calls: unknown[][] = [];
  web.setLocaleCookie(
    {
      cookies: {
        set(...args: unknown[]) {
          calls.push(args);
        },
      },
      headers: {
        append() {
          assert.fail("headers must not be used when cookies.set is available");
        },
      },
    },
    "fr",
  );

  assert.equal(calls.length, 1);
  assert.equal(calls[0][0], "LINGUINI_LOCALE");
  assert.equal(calls[0][1], "fr");
  assert.deepEqual(calls[0][2], {
    path: "/",
    domain: undefined,
    maxAge: 31_536_000,
    sameSite: "lax",
    secure: false,
    httpOnly: false,
  });
});

test("default locale sources match the validated web configuration", () => {
  const web = createWeb();

  assert.deepEqual(web.options.sources, [
    "path",
    "cookie",
    "accept-language",
  ]);
  assert.deepEqual(web.options.localeSwitch, {
    writesPath: true,
    writesCookie: true,
    writesLocalStorage: false,
  });
  assert.equal(
    web.resolveLocaleSync({
      url: "https://app.example/account",
      cookie: "LINGUINI_LOCALE=fr",
      headers: new Headers({ "accept-language": "en" }),
    }),
    "fr",
  );
});

test("locale switch fallback derives writable transports from custom sources", () => {
  const cases: Array<{
    sources: LinguiniWebOptions["sources"];
    expected: { writesPath: boolean; writesCookie: boolean; writesLocalStorage: boolean };
  }> = [
    {
      sources: ["path"],
      expected: { writesPath: true, writesCookie: false, writesLocalStorage: false },
    },
    {
      sources: ["cookie"],
      expected: { writesPath: false, writesCookie: true, writesLocalStorage: false },
    },
    {
      sources: ["local-storage"],
      expected: { writesPath: false, writesCookie: false, writesLocalStorage: true },
    },
    {
      sources: ["accept-language"],
      expected: { writesPath: false, writesCookie: false, writesLocalStorage: false },
    },
  ];

  for (const { sources, expected } of cases) {
    const web = createWeb({ sources });
    assert.deepEqual(web.options.localeSwitch, expected);
  }

  const explicit = createWeb({
    sources: ["cookie"],
    localeSwitch: { writesPath: true, writesCookie: false, writesLocalStorage: true },
  });
  assert.deepEqual(explicit.options.localeSwitch, {
    writesPath: true,
    writesCookie: false,
    writesLocalStorage: true,
  });
});

test("Accept-Language uses standard Headers, quality weights, and wildcards", async () => {
  const web = createWeb({ sources: ["accept-language"] });

  const requestContext = await web.resolveRequest(
    new Request("https://app.example/account", {
      headers: { "accept-language": "fr;q=0.9, en;q=0.1" },
    }),
  );
  assert.equal(requestContext.locale, "fr");
  assert.equal(
    web.resolveLocaleSync({
      headers: new Headers({
        "accept-language": "de;q=1, en;q=0.2, fr;q=0.9",
      }),
    }),
    "fr",
  );
  assert.equal(
    web.resolveLocaleSync({
      headers: new Headers({
        "accept-language": "de;q=1, fr;q=0.7",
      }),
    }),
    "fr",
  );
  assert.equal(
    web.resolveLocaleSync({
      headers: new Headers({
        "accept-language": "*;q=0.8, en;q=0",
      }),
    }),
    "fr",
  );
  assert.equal(
    web.resolveLocaleSync({
      headers: new Headers({
        "accept-language": "fr;q=0, *;q=0.5",
      }),
    }),
    "en",
  );
  assert.equal(
    web.resolveLocaleSync({
      headers: new Headers({
        "accept-language": "fr;q=invalid, en;q=0.5",
      }),
    }),
    "en",
  );
  assert.equal(
    web.resolveLocaleSync({
      headers: { "accept-language": "fr" },
    }),
    "en",
  );
});

test("Accept-Language uses browser navigator preferences when headers are absent", () => {
  const web = createWeb({ sources: ["accept-language"] });

  assert.equal(
    web.resolveLocaleSync({
      headers: new Headers({ "accept-language": "fr" }),
      navigator: { languages: ["en"], language: "en" },
    }),
    "fr",
  );
  assert.equal(
    web.resolveLocaleSync({
      navigator: { languages: ["fr-CA", "en-US"], language: "en-US" },
    }),
    "fr",
  );
  assert.equal(
    web.resolveLocaleSync({
      navigator: { languages: ["de", "en-US"], language: "fr" },
    }),
    "en",
  );
  assert.equal(
    web.resolveLocaleSync({ navigator: { languages: [], language: "fr" } }),
    "fr",
  );
});

test("browser Accept-Language keeps configured source precedence and tolerates malformed navigator", () => {
  const cookieFirst = createWeb({ sources: ["cookie", "accept-language"] });
  assert.equal(
    cookieFirst.resolveLocaleSync({
      cookie: "LINGUINI_LOCALE=fr",
      navigator: { languages: ["en"], language: "en" },
    }),
    "fr",
  );

  const acceptLanguageFirst = createWeb({ sources: ["accept-language", "cookie"] });
  assert.equal(
    acceptLanguageFirst.resolveLocaleSync({
      cookie: "LINGUINI_LOCALE=fr",
      navigator: { languages: ["en"], language: "en" },
    }),
    "en",
  );
  assert.equal(acceptLanguageFirst.resolveLocaleSync({ navigator: undefined }), "en");
  assert.equal(acceptLanguageFirst.resolveLocaleSync({ navigator: null }), "en");
  assert.equal(
    acceptLanguageFirst.resolveLocaleSync({
      navigator: {
        languages: "fr",
        get language() {
          throw new Error("navigator denied");
        },
      },
    }),
    "en",
  );
});

test("malformed locale cookie encoding is ignored", () => {
  const web = createWeb({ sources: ["cookie"] });

  assert.doesNotThrow(() =>
    web.resolveLocaleSync({ cookie: "LINGUINI_LOCALE=%E0%A4%A" }),
  );
  assert.equal(
    web.resolveLocaleSync({ cookie: "LINGUINI_LOCALE=%E0%A4%A" }),
    "en",
  );
  assert.equal(
    web.resolveLocaleSync({ cookie: "LINGUINI_LOCALE=fr" }),
    "fr",
  );
});

test("local-storage source uses only the configured storage capability", () => {
  const web = createWeb({
    sources: ["local-storage"],
    localStorageKey: "SHOP_LOCALE",
  });
  let requestedKey: string | undefined;

  assert.equal(
    web.resolveLocaleSync({
      localStorage: {
        getItem(key: string) {
          requestedKey = key;
          return "fr";
        },
      },
    }),
    "fr",
  );
  assert.equal(requestedKey, "SHOP_LOCALE");
  assert.equal(
    web.resolveLocaleSync({
      localStorage: {
        getItem() {
          throw new Error("storage is unavailable");
        },
      },
    }),
    "en",
  );
});
