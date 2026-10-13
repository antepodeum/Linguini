import { test, expect } from "bun:test";
import { readFile, writeFile, unlink } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { pathToFileURL } from "node:url";

const transpiler = new Bun.Transpiler({ loader: "ts", target: "bun" });
let nonce = 0;
async function adapter(name: string, messages: boolean, target: "js" | "ts" | "legacy", probe: object, overrides?: object) {
  const moduleName = messages ? "sveltekit" : "sveltekit-control";
  const actual = await readFile(new URL(`./${name}/${target === "ts" ? "typescript/" : ""}${moduleName}.${target === "ts" ? "ts" : "js"}`, import.meta.url), "utf8");
  const policy = actual.match(/^const options = (.*);$/m)![1];
  let source = target === "legacy" ? await readFile(new URL(`./legacy/${moduleName}.runtime.ts`, import.meta.url), "utf8") : actual;
  const cookie = name === "all" || name === "default", switching = name === "all";
  if (target === "legacy") {
    for (const [key, value] of Object.entries({
      OPTIONS: policy, SERVER_COOKIE_IMPORT: "", SWITCH_ROUTE_IMPORT: "",
      PERSIST_COOKIE_DECLARATION: cookie ? "  const persistCookie = options.persistCookie !== false && web.options.locale.switch.writesCookie;" : "",
      COOKIE_INPUT: cookie ? '      cookie: event.request.headers.get("cookie") ?? undefined,' : "",
      PERSIST_REDIRECT_COOKIE: cookie ? "      if (persistCookie) persistLocaleCookie(web, response, context.locale, { origin: event.url.origin });" : "",
      PERSIST_RESPONSE_COOKIE: cookie ? "    if (persistCookie) persistLocaleCookie(web, response, context.locale, { origin: event.url.origin });" : "",
      SWITCH_ROUTE_BRANCH: switching ? "    const switchResponse = handleLocaleSwitchRoute(web, event);\n    if (switchResponse) return switchResponse;" : "",
    })) source = source.replaceAll(`{{${key}}}`, value);
  }
  if (overrides) source = source.replace(/^const options = .*;$/m, `const options = ${JSON.stringify(overrides)};`);
  source = source.replace(/^import .*;\n/gm, "");
  const web = await import(`./${name}/web.js`);
  const cookieLeaf = cookie ? await import(`./${name}/web/server-cookie.js`) : {};
  const switchLeaf = switching ? await import(`./${name}/web/switch-route.js`) : {};
  const key = `__linguiniKitParity${nonce++}`;
  Object.assign(globalThis, { [key]: { ...probe, ...web, ...cookieLeaf, ...switchLeaf } });
  const header = `const { runtime, base, createWebI18n, createWebLocaleI18n, persistLocaleCookie, handleLocaleSwitchRoute } = globalThis[${JSON.stringify(key)}]; const locale = runtime;\n`;
  const path = join(tmpdir(), `linguini-kit-parity-${process.pid}-${nonce}.mjs`);
  await writeFile(path, transpiler.transformSync(header + source));
  try { return await import(pathToFileURL(path).href); }
  finally { await unlink(path); delete (globalThis as any)[key]; }
}

async function run(name: string, messages: boolean, target: "js" | "ts" | "legacy", persistence = true) {
  const factoryCalls: string[] = [], prepared: string[] = [], transformed: string[] = [];
  const runtime = {
    locales: ["en", "fr"], baseLocale: "en",
    normalizeLocale: (value: unknown) => value === "en" || value === "fr" ? value : undefined,
    getTextDirection: () => "ltr",
    async prepareLinguini(locale: string) { await Bun.sleep(locale === "fr" ? 5 : 1); prepared.push(locale); },
    createLinguini(locale: string) { expect(prepared).toContain(locale); factoryCalls.push(locale); return { locale }; },
  };
  const mod = await adapter(name, messages, target, { runtime, base: "/app" }, persistence ? undefined : { persistCookie: false });
  expect(mod.handle).toBe(mod.linguiniHandle); expect(mod.load).toBe(mod.linguiniLoad); expect(mod.reroute).toBe(mod.linguiniReroute);
  const requests = [
    new Request("https://fr.test/app/fr/shop?q=1", { headers: { cookie: "LINGUINI_LOCALE=fr", "accept-language": "fr" } }),
    new Request("https://en.test/app/en/shop"),
    new Request("https://cookie.test/app/shop", { headers: { cookie: "LINGUINI_LOCALE=fr", "accept-language": "fr" } }),
    new Request("https://excluded.test/api/users", { headers: { cookie: "LINGUINI_LOCALE=fr" } }),
    new Request("https://switch.test/app/locale/fr?return=%2Fapp%2Fshop"),
    new Request("https://switch.test/app/locale/invalid?return=https%3A%2F%2Fevil.test"),
  ];
  let resolveCalls = 0;
  const observations = await Promise.all(requests.map(async request => {
    const event = { request, url: new URL(request.url), locals: {} as any };
    const response = await mod.handle({ event, resolve: async (_event: unknown, options?: any) => {
      resolveCalls++;
      // Apply chunks immediately, before the next chunk is produced.
      for (const html of ['<html lang="%linguini.lang%" dir="%linguini.dir%">', "%linguini.locale%", "tail"]) {
        const chunk = options?.transformPageChunk({ html, done: html === "tail" }) ?? html;
        transformed.push(`${event.url.origin}:${chunk}`);
      }
      return new Response("ok", { headers: { "set-cookie": "theme=dark" } });
    } });
    const ctx = event.locals.linguini;
    expect(Object.keys(event.locals)).toEqual(ctx ? ["linguini"] : []);
    const data = mod.load({ locals: event.locals });
    if (ctx) {
      expect(ctx.localizeUrl("/next").origin).toBe(event.url.origin);
      expect(ctx.l).toBe(ctx.messages);
      expect(data.linguini.locale).toBe(ctx.locale);
      expect(data.linguini).not.toHaveProperty("messages");
      expect(data.linguini).not.toHaveProperty("localizeHref");
      expect(Boolean(ctx.messages)).toBe(messages);
    }
    return { status: response.status, location: response.headers.get("location"), cookie: response.headers.get("set-cookie"), locale: ctx?.locale, localized: ctx?.localizeHref("/next"), data };
  }));
  const reroutes = ["/app/fr/shop", "/app/en/shop", "/app/shop", "/api/fr/shop"].map(path => mod.reroute({ url: new URL(path, "https://example.test") }));
  if (!persistence || name === "path" || name === "empty") {
    expect(observations.every(o => !o.cookie?.includes("LINGUINI_LOCALE"))).toBe(true);
  }
  if (name === "all") {
    expect(observations[3].locale).toBeUndefined();
    expect(observations[4].status).toBe(303);
    expect(observations[4].location).toBe("/app/fr/shop");
    expect(observations[4].locale).toBeUndefined();
  }
  expect(factoryCalls.length > 0).toBe(messages);
  return { observations, reroutes, resolveCalls, transformed: transformed.sort(), prepared: prepared.sort(), factoryCalls: factoryCalls.sort() };
}

for (const name of ["default", "path", "empty", "all"]) {
  for (const messages of [false, true]) {
    test(`${name}: ${messages ? "messages" : "metadata"} adapter request isolation, streaming, routing and persistence parity`, async () => {
      const expected = await run(name, messages, "legacy");
      for (const target of ["js", "ts"] as const) expect(await run(name, messages, target)).toEqual(expected);
    });
  }
}

test("explicit persistence opt-out remains authoritative in both server adapters", async () => {
  for (const messages of [false, true]) {
    const expected = await run("default", messages, "legacy", false);
    for (const target of ["js", "ts"] as const) expect(await run("default", messages, target, false)).toEqual(expected);
  }
});

test("failed message preparation rejects without mutating request locals or resolving", async () => {
  for (const target of ["js", "ts", "legacy"] as const) {
    const runtime = { locales: ["en", "fr"], baseLocale: "en", prepareLinguini: async () => { throw Error("prepare failed"); }, createLinguini: () => { throw Error("factory ran"); } };
    const mod = await adapter("default", true, target, { runtime, base: "/app" });
    const request = new Request("https://example.test/app/fr");
    const event = { request, url: new URL(request.url), locals: {} };
    await expect(mod.handle({ event, resolve: () => { throw Error("resolve ran"); } })).rejects.toThrow("prepare failed");
    expect(event.locals).toEqual({});
  }
});
