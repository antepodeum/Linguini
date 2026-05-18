import assert from "node:assert/strict";
import test from "node:test";
import { createWebI18n, parseAcceptLanguage } from "../src/index.js";

const runtime = {
  locales: ["en", "ru"],
  baseLocale: "en",
  createLinguini(locale) {
    return { hello: () => locale };
  },
  localeDirections: { en: "ltr", ru: "ltr" }
};

test("resolves locales through url, cookie, header, navigator, preferred language, and base locale", async () => {
  const i18n = createWebI18n(runtime, { strategy: ["url", "cookie", "preferredLanguage", "baseLocale"] });

  assert.equal(await i18n.resolveLocale({ url: "https://example.com/ru/cart" }), "ru");
  assert.equal(await i18n.resolveLocale({ cookie: "LINGUINI_LOCALE=ru" }), "ru");
  assert.equal(await i18n.resolveLocale({ headers: new Headers({ "accept-language": "de,ru;q=0.8" }) }), "ru");
  assert.equal(await i18n.resolveLocale({ navigator: { languages: ["ru-RU"] } }), "ru");
  assert.equal(await i18n.resolveLocale({ headers: new Headers({ "accept-language": "de" }) }), "en");
});

test("supports explicit header and navigator strategy names", async () => {
  const header = createWebI18n(runtime, { strategy: ["header", "baseLocale"] });
  const navigator = createWebI18n(runtime, { strategy: ["navigator", "baseLocale"] });

  assert.equal(await header.resolveLocale({ headers: new Headers({ "accept-language": "ru;q=0.9,en;q=0.1" }) }), "ru");
  assert.equal(await navigator.resolveLocale({ navigator: { language: "ru" } }), "ru");
});

test("localizes and delocalizes urls without prefixing the base locale by default", () => {
  const i18n = createWebI18n(runtime);

  assert.equal(i18n.localizeHref("/cart", "ru"), "/ru/cart");
  assert.equal(i18n.localizeHref("/ru/cart", "en"), "/cart");
  assert.equal(i18n.delocalizePathname("/ru/cart"), "/cart");
});

test("supports prefixed base locale, origin, and alternate links", () => {
  const i18n = createWebI18n(runtime, { prefixDefaultLocale: true, origin: "https://example.com" });

  assert.equal(i18n.localizeHref("/cart", "en"), "/en/cart");
  assert.equal(i18n.localizeUrl("/cart", "ru").toString(), "https://example.com/ru/cart");
  assert.deepEqual(i18n.htmlAttrs("ru"), { lang: "ru", dir: "ltr" });
  assert.equal(i18n.alternateLinks("https://example.com/cart").length, 3);
});

test("parses accept-language by q value", () => {
  assert.deepEqual(parseAcceptLanguage("de;q=0.5, ru-RU;q=0.9, en"), ["en", "ru-RU", "de"]);
});

test("serializes configured locale cookies", () => {
  const i18n = createWebI18n(runtime, {
    cookieName: "SHOP_LOCALE",
    cookiePath: "/shop",
    cookieDomain: "example.com",
    cookieMaxAge: 86400,
    cookieSameSite: "strict",
    cookieSecure: true,
    cookieHttpOnly: true,
  });

  assert.equal(
    i18n.serializeLocaleCookie("ru"),
    "SHOP_LOCALE=ru; Max-Age=86400; Path=/shop; Domain=example.com; SameSite=strict; Secure; HttpOnly",
  );
});

test("excludes routes from canonical redirects", () => {
  const i18n = createWebI18n(runtime, {
    exclude: ["/api/**"],
    prefixDefaultLocale: true,
    origin: "https://example.com",
  });

  assert.equal(i18n.shouldExclude("https://example.com/api/products"), true);
  assert.equal(i18n.getCanonicalRedirect("https://example.com/api/products", "ru"), undefined);
  assert.equal(i18n.getCanonicalRedirect("https://example.com/products", "ru"), "/ru/products");
});

test("uses route-level strategy overrides", async () => {
  const i18n = createWebI18n(runtime, {
    strategy: ["baseLocale"],
    routeStrategies: [{ match: "/admin/**", strategy: ["cookie", "baseLocale"] }],
  });

  assert.equal(await i18n.resolveLocale({ url: "https://example.com/admin/reports", cookie: "LINGUINI_LOCALE=ru" }), "ru");
  assert.equal(await i18n.resolveLocale({ url: "https://example.com/shop", cookie: "LINGUINI_LOCALE=ru" }), "en");
});

test("localizes markup links while skipping unsafe and opted-out anchors", () => {
  const i18n = createWebI18n(runtime, {
    origin: "https://example.com",
    exclude: ["/api/**"],
  });

  const html = i18n.localizeMarkupLinks(
    '<a href="/cart">Cart</a>' +
      '<a href="https://example.com/pricing?plan=pro#buy">Pricing</a>' +
      '<a href="https://other.example/cart">External</a>' +
      '<a href="mailto:support@example.com">Mail</a>' +
      '<a href="#section">Hash</a>' +
      '<a download href="/file.pdf">File</a>' +
      '<a data-linguini-ignore href="/raw">Raw</a>' +
      '<a href="/api/products">API</a>',
    "ru",
    { currentUrl: "https://example.com/shop" },
  );

  assert.equal(
    html,
    '<a href="/ru/cart">Cart</a>' +
      '<a href="https://example.com/ru/pricing?plan=pro#buy">Pricing</a>' +
      '<a href="https://other.example/cart">External</a>' +
      '<a href="mailto:support@example.com">Mail</a>' +
      '<a href="#section">Hash</a>' +
      '<a download href="/file.pdf">File</a>' +
      '<a data-linguini-ignore href="/raw">Raw</a>' +
      '<a href="/api/products">API</a>',
  );
});

test("can disable automatic link localization", () => {
  const i18n = createWebI18n(runtime, { localizeLinks: false });

  assert.equal(i18n.shouldLocalizeHref("/cart"), false);
  assert.equal(i18n.localizeHrefAttribute("/cart", "ru"), "/cart");
  assert.equal(i18n.localizeMarkupLinks('<a href="/cart">Cart</a>', "ru"), '<a href="/cart">Cart</a>');
});
