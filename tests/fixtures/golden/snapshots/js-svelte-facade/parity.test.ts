import { describe, expect, test } from "bun:test";
import { readFile } from "node:fs/promises";
import * as projectRuntime from "../js-project/index.js";
import { withRuntimeHost } from "../../runtime-host.ts";

describe("Svelte facade target and frozen-runtime parity", () => {
  for (const mode of ["context", "web"] as const) {
    for (const hot of [false, true]) {
      for (const failure of [false, true]) {
        test(`${mode}: HMR ${hot}, loader failure ${failure}`, async () => {
          const run = async (target: "javascript" | "typescript" | "legacy") => {
            let locale: "en" | "fr" = "en";
            let release!: () => void;
            const ready = new Promise<void>((resolve) => { release = resolve; });
            const trace: unknown[] = [];
            const loaders = new Set<(locale: "en" | "fr") => Promise<void>>();
            let dispose = () => {};
            const runtime = {
              ...projectRuntime,
              async prepareLinguini(value: "en" | "fr") {
                trace.push(["load", value]);
                if (failure) throw new Error("messages failed");
                return projectRuntime.prepareLinguini(value);
              },
            };
            const state = {
              getCurrentLocale: () => locale,
              setCurrentLocale(value: "en" | "fr") { trace.push(["set", value]); locale = value; return locale; },
              async prepareLocale(value: string) {
                trace.push(["prepare", value]); await ready;
                const resolved = projectRuntime.normalizeLocale(value) ?? "en";
                if (mode === "web") await Promise.all([...loaders].map((loader) => loader(resolved)));
                else await runtime.prepareLinguini(resolved);
                return resolved;
              },
              registerLocaleLoader(loader: (value: "en" | "fr") => Promise<void>) {
                loaders.add(loader); trace.push(["register"]);
                return () => { if (loaders.delete(loader)) trace.push(["dispose"]); };
              },
            };
            const controls: Record<string, any> = {
              get locale() { return locale; }, get lang() { return locale; },
              get direction() { return projectRuntime.getTextDirection(locale); },
              get textDirection() { return projectRuntime.getTextDirection(locale); },
              get htmlAttrs() { return { lang: locale, dir: projectRuntime.getTextDirection(locale) }; },
              async setLocale(next: string) { return state.setCurrentLocale(await state.prepareLocale(next)); },
            };
            for (const name of ["localizeHref", "localizeUrl", "shouldLocalizeHref", "shouldLocalizeLink", "localizeHrefAttribute", "delocalizeUrl", "alternateLinks", "destroy"]) {
              controls[name] = (...args: unknown[]) => { trace.push([name, ...args]); return name; };
            }
            const filename = target === "javascript" ? `${mode}/svelte.js` : target === "typescript" ? `${mode}/typescript/svelte.ts` : `${mode}/legacy/svelte.runtime.ts`;
            const source = await readFile(new URL(filename, import.meta.url), "utf8");
            return withRuntimeHost(source, target === "javascript" ? "javascript" : "typescript", {
              modules: { "./index": runtime, "./index.js": runtime, "./svelte-control.js": { linguini: controls }, "./svelte-locale.svelte.js": state },
              meta: hot ? { hot: { dispose(callback) { dispose = callback; } } } : {},
            }, async (facade) => {
              expect(loaders.size).toBe(mode === "web" ? 1 : 0);
              expect(facade.l).toBe(facade.messages);
              expect(facade.l).toBe(facade.linguini.messages);
              expect(facade.l.hello("Ada")).toBe("Hello Ada");
              if (mode === "web") {
                for (const name of ["setLocale", "localizeHref", "localizeUrl", "shouldLocalizeHref", "shouldLocalizeLink", "localizeHrefAttribute", "delocalizeUrl", "alternateLinks"]) {
                  expect(facade[name]).toBe(controls[name]);
                }
                expect(facade.linguini.destroy).toBe(controls.destroy);
              } else {
                expect(facade.localizeHref).toBeUndefined();
                expect(facade.linguini.destroy).toBeUndefined();
              }
              const operation = facade.setLocale("fr");
              expect(locale).toBe("en");
              expect(trace.at(-1)).toEqual(["prepare", "fr"]);
              release();
              if (failure) await expect(operation).rejects.toThrow("messages failed");
              else expect(await operation).toBe("fr");
              expect(locale).toBe(failure ? "en" : "fr");
              expect(facade.linguini.locale).toBe(locale);
              expect(facade.linguini.lang).toBe(locale);
              expect(facade.linguini.direction).toBe("ltr");
              expect(facade.linguini.textDirection).toBe("ltr");
              expect(facade.linguini.htmlAttrs).toEqual({ lang: locale, dir: "ltr" });
              expect(facade.messages.hello({ name: "Ada" })).toBe(failure ? "Hello Ada" : "Salut Ada");
              dispose(); dispose();
              expect(loaders.size).toBe(mode === "web" && !hot ? 1 : 0);
              return { trace, locale, exports: Object.keys(facade).sort() };
            });
          };
          const legacy = await run("legacy");
          expect(await run("typescript")).toEqual(legacy);
          expect(await run("javascript")).toEqual(legacy);
        });
      }
    }
  }
});
