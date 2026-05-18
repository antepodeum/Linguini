import assert from "node:assert/strict";
import test from "node:test";
import { createHandle, createLoad, createReroute } from "../src/server.js";

const runtime = {
  locales: ["en", "ru", "ar"],
  baseLocale: "en",
  localeDirections: { en: "ltr", ru: "ltr", ar: "rtl" },
  createLinguini(locale) {
    return { home: { title: () => `title:${locale}` } };
  },
  normalizeLocale(locale) {
    return this.locales.includes(locale) ? locale : undefined;
  },
};

test("handle resolves request context, writes locals, transforms html, and persists cookie", async () => {
  const handle = createHandle(runtime, {
    strategy: ["url", "baseLocale"],
    cookieName: "SHOP_LOCALE",
    redirect: false,
  });
  const event = {
    url: new URL("https://example.com/ar/dashboard"),
    request: new Request("https://example.com/ar/dashboard"),
    cookies: { get: () => undefined },
    locals: {},
  };

  const response = await handle({
    event,
    resolve(receivedEvent, options) {
      assert.equal(receivedEvent.locals.locale, "ar");
      const html = options.transformPageChunk({
        html: '<html lang="%linguini.lang%" dir="%linguini.dir%"><a href="/dashboard">Dashboard</a></html>',
        done: true,
      });
      return new Response(html);
    },
  });

  assert.equal(event.locals.locale, "ar");
  assert.equal(event.locals.direction, "rtl");
  assert.equal(event.locals.l.home.title(), "title:ar");
  assert.equal(await response.text(), '<html lang="ar" dir="rtl"><a href="/ar/dashboard">Dashboard</a></html>');
  assert.match(response.headers.get("set-cookie"), /SHOP_LOCALE=ar/);
});

test("reroute delocalizes localized pathnames", () => {
  const reroute = createReroute(runtime, { strategy: ["url", "baseLocale"] });

  assert.equal(reroute({ url: new URL("https://example.com/ru/products") }), "/products");
  assert.equal(reroute({ url: new URL("https://example.com/products") }), undefined);
});

test("load serializes the request locale context", () => {
  const load = createLoad();
  const locals = {
    linguini: {
      locale: "ru",
      baseLocale: "en",
      locales: ["en", "ru"],
      direction: "ltr",
      lang: "ru",
      htmlAttrs: { lang: "ru", dir: "ltr" },
    },
  };

  assert.deepEqual(load({ locals }).linguini, {
    locale: "ru",
    baseLocale: "en",
    locales: ["en", "ru"],
    direction: "ltr",
    lang: "ru",
    htmlAttrs: { lang: "ru", dir: "ltr" },
  });
});


test("handle can opt out of automatic link localization", async () => {
  const handle = createHandle(runtime, {
    strategy: ["url", "baseLocale"],
    redirect: false,
    localizeLinks: false,
  });
  const event = {
    url: new URL("https://example.com/ru/dashboard"),
    request: new Request("https://example.com/ru/dashboard"),
    cookies: { get: () => undefined },
    locals: {},
  };

  const response = await handle({
    event,
    resolve(_event, options) {
      return new Response(options.transformPageChunk({ html: '<a href="/dashboard">Dashboard</a>', done: true }));
    },
  });

  assert.equal(await response.text(), '<a href="/dashboard">Dashboard</a>');
});
