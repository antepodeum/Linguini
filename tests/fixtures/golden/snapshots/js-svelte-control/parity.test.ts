import { describe, expect, test } from "bun:test";
import { readFile } from "node:fs/promises";
import { withGlobals, withRuntimeHost } from "../../runtime-host.ts";

const navigation = {
  kit: `        const href = web.localizeHref(window.location.href, resolved);
        await goto(href, {
          replaceState: Boolean(options.replaceState), invalidateAll: Boolean(options.invalidateAll),
          keepFocus: options.keepFocus as boolean | undefined, noScroll: options.noScroll as boolean | undefined,
          state: options.state as App.PageState | undefined,
        });
        clearCurrentLocaleOverride();`,
  plain: `        const href = web.localizeHref(window.location.href, resolved);
        if (options.replaceState) { window.location.replace(href); } else { window.location.assign(href); }`,
};

describe("lightweight control target and frozen-runtime parity", () => {
  for (const framework of ["plain", "kit"] as const) {
    for (const capability of ["none", "path", "cookie", "storage", "all"]) {
      for (const scenario of ["server", "success", "transports-off", "options-off", "denied-persistence", "prepare-failure", "navigation-failure", "replace-navigation"]) {
        test(`${framework}-${capability}: ${scenario}`, async () => {
          const run = async (target: "javascript" | "typescript" | "legacy") => {
            const browser = scenario !== "server";
            const transport = (name: string) => (capability === name || capability === "all") && scenario !== "transports-off";
            const writes = { writesPath: transport("path"), writesCookie: transport("cookie"), writesLocalStorage: transport("storage") };
            let locale = "en";
            const trace: unknown[] = [];
            let release!: () => void;
            const ready = new Promise<void>((resolve) => { release = resolve; });
            const record = (name: string, args: unknown[], result: unknown) => { trace.push([name, ...args]); return result; };
            const web = {
              options: { locale: { switch: writes }, localStorage: { key: "locale" } },
              getTextDirection: (value: string) => value === "fr" ? "rtl" : "ltr",
              htmlAttrs: (value: string) => ({ lang: value, dir: value === "fr" ? "rtl" : "ltr" }),
              localizeHref: (href: string, value: string, input?: unknown) => record("localizeHref", [href, value, input], `/${value}${href}`),
              localizeUrl: (url: string | URL, value: string, input?: unknown) => record("localizeUrl", [String(url), value, input], new URL(`https://app.example/${value}/account`)),
              shouldLocalizeHref: (href: string, input?: unknown) => record("shouldLocalizeHref", [href, input], true),
              shouldLocalizeLink: (href: string, attributes: unknown, input?: unknown) => record("shouldLocalizeLink", [href, attributes, input], false),
              localizeHrefAttribute: (href: string, value: string, input?: unknown) => record("localizeHrefAttribute", [href, value, input], `/${value}${href}`),
              delocalizeUrl: (url: string | URL, input?: unknown) => record("delocalizeUrl", [String(url), input], new URL("https://app.example/account")),
              alternateLinks: (url: string | URL, input?: unknown) => record("alternateLinks", [String(url), input], [{ rel: "alternate", hreflang: "fr", href: "/fr/account" }]),
              serializeLocaleCookie: (value: string, input: unknown) => record("serializeLocaleCookie", [value, input], `locale=${value}`),
            };
            const go = (href: string, options?: unknown) => {
              trace.push(["navigate", href, options]);
              if (scenario === "navigation-failure") throw new Error("navigation failed");
            };
            const effects = { web, refreshLinguiniEffects() { trace.push(["refresh"]); }, destroyLinguiniEffects() { trace.push(["destroy"]); } };
            const state = {
              getCurrentLocale: () => locale,
              async prepareLocale(value: string) {
                trace.push(["prepare", value]); await ready;
                if (scenario === "prepare-failure") throw new Error("prepare failed");
                return value === "fr" ? "fr" : "en";
              },
              setCurrentLocale(value: string) { trace.push(["set", value]); locale = value; return value; },
              clearCurrentLocaleOverride() { trace.push(["clear"]); },
            };
            const window = {
              location: { href: "/account", assign: (href: string) => go(href), replace: (href: string) => go(href) },
              localStorage: { setItem(key: string, value: string) { trace.push(["storage", key, value]); if (scenario === "denied-persistence") throw new Error("storage denied"); } },
            };
            const document = { set cookie(value: string) { trace.push(["cookie", value]); if (scenario === "denied-persistence") throw new Error("cookie denied"); } };
            const root = `${framework}-${capability}`;
            let source = await readFile(new URL(target === "javascript" ? `${root}/svelte-control.js` : target === "typescript" ? `${root}/typescript/svelte-control.ts` : "legacy/svelte-control.runtime.ts", import.meta.url), "utf8");
            if (target === "legacy") {
              source = source.replace("{{NAVIGATION_RUNTIME}}", framework === "kit" ? 'import { browser } from "$app/environment";\nimport { goto } from "$app/navigation";' : 'const browser = typeof window !== "undefined" && typeof document !== "undefined";')
                .replace("{{LOCALE_RUNTIME}}", 'import { clearCurrentLocaleOverride, getCurrentLocale, prepareLocale, setCurrentLocale } from "./svelte-locale.svelte.js";')
                .replace("{{NAVIGATION}}", navigation[framework]);
            }
            return withGlobals({ window: browser ? window : undefined, document: browser ? document : undefined }, () => withRuntimeHost(source, target === "javascript" ? "javascript" : "typescript", {
              modules: { "$app/environment": { browser }, "$app/navigation": { goto: go }, "./svelte-effects.svelte.js": effects, "./svelte-locale.svelte.js": state },
            }, async (controls) => {
              expect(controls.linguini.locale).toBe("en");
              const operation = controls.setLocale("fr", { navigate: scenario !== "options-off", cookie: scenario !== "options-off", replaceState: scenario === "replace-navigation", state: { tab: "account" } });
              expect(trace).toEqual([["prepare", "fr"]]);
              release();
              const failure = scenario === "prepare-failure" || (scenario === "navigation-failure" && browser && writes.writesPath);
              if (failure) await expect(operation).rejects.toThrow(scenario === "prepare-failure" ? "prepare failed" : "navigation failed");
              else expect(await operation).toBe("fr");
              const mutation = browser && scenario !== "prepare-failure";
              expect(locale).toBe(mutation ? "fr" : "en");
              const names = trace.map((event: any) => event[0]);
              expect(names.includes("storage")).toBe(mutation && writes.writesLocalStorage);
              expect(names.includes("cookie")).toBe(mutation && writes.writesCookie && scenario !== "options-off");
              expect(names.includes("navigate")).toBe(mutation && writes.writesPath && scenario !== "options-off");
              expect(names.includes("refresh")).toBe(mutation && !failure);
              expect(names.includes("clear")).toBe(mutation && !failure && writes.writesPath && scenario !== "options-off" && framework === "kit");
              expect(controls.linguini.lang).toBe(locale);
              expect(controls.linguini.direction).toBe(locale === "fr" ? "rtl" : "ltr");
              expect(controls.linguini.textDirection).toBe(controls.linguini.direction);
              expect(controls.linguini.htmlAttrs).toEqual({ lang: locale, dir: controls.linguini.direction });
              const input = { origin: "https://app.example" };
              expect(controls.localizeHref("/account", undefined, input)).toBe(`/${locale}/account`);
              expect(controls.localizeUrl("/account", "fr", input).pathname).toBe("/fr/account");
              expect(controls.shouldLocalizeHref("/account", input)).toBe(true);
              expect(controls.shouldLocalizeLink("/account", { ignored: true }, input)).toBe(false);
              expect(controls.localizeHrefAttribute("/account", "fr", input)).toBe("/fr/account");
              expect(controls.delocalizeUrl("/fr/account", input).pathname).toBe("/account");
              expect(controls.alternateLinks("/account", input)[0].hreflang).toBe("fr");
              controls.destroy();
              expect(trace.at(-1)).toEqual(["destroy"]);
              return trace;
            }));
          };
          const legacy = await run("legacy");
          expect(await run("typescript")).toEqual(legacy);
          expect(await run("javascript")).toEqual(legacy);
        });
      }
    }
  }
});
