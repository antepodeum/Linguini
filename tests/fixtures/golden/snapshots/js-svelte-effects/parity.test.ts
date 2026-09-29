import { describe, expect, test } from "bun:test";
import { readFile } from "node:fs/promises";

const transpiler = new Bun.Transpiler({ loader: "ts", target: "browser" });
const probeKey = Symbol.for("linguini.effects.parity.probe");
let nonce = 0;
const keys: Record<string, string[]> = {
  none: [], path: ["url"], cookie: ["cookie"], storage: ["localStorage"],
  language: ["navigator"], all: ["url", "cookie", "localStorage", "navigator"],
};

describe("browser effect target parity and capability ownership", () => {
  for (const framework of ["plain", "kit"]) {
    for (const capability of Object.keys(keys)) {
      for (const environment of ["server", "denied", "browser"]) {
        test(`${framework}-${capability}: ${environment}`, async () => {
          const run = async (target: "javascript" | "typescript") => {
            const root = `${framework}-${capability}`;
            const filename = target === "javascript" ? `${root}/svelte-effects.svelte.js` : `${root}/typescript/svelte-effects.svelte.ts`;
            // Inject the host before stripping TS: Bun otherwise folds import.meta.hot away.
            const source = (await readFile(new URL(filename, import.meta.url), "utf8"))
              .replaceAll("import.meta", "({ hot: { dispose(callback) { probe.dispose = callback; } } })");
            let javascript = target === "typescript" ? transpiler.transformSync(source) : source;
            const probe = {
              browser: environment !== "server", reads: [] as string[], input: undefined as Record<string, unknown> | undefined,
              locale: "", starts: 0, refreshes: 0, destroys: 0, dispose: () => {}, options: {} as any, environment: {} as any,
            };
            const descriptors = ["window", "document"].map((key) => [key, Object.getOwnPropertyDescriptor(globalThis, key)] as const);
            const getter = (key: string, value: unknown) => () => {
              probe.reads.push(key);
              if (environment === "denied") throw new Error(`${key} denied`);
              return value;
            };
            if (environment === "server") {
              for (const [key] of descriptors) Reflect.deleteProperty(globalThis, key);
            } else {
              Object.defineProperty(globalThis, "window", { configurable: true, value: Object.defineProperties({}, {
                location: { get: getter("url", { href: "https://app.example/fr" }) },
                localStorage: { get: getter("localStorage", { getItem: () => "fr" }) },
                navigator: { get: getter("navigator", { languages: ["fr", "en"] }) },
              }) });
              Object.defineProperty(globalThis, "document", { configurable: true, value: Object.defineProperty({}, "cookie", { get: getter("cookie", "LINGUINI_LOCALE=fr") }) });
            }
            Reflect.set(globalThis, probeKey, probe);
            javascript = `const probe = globalThis[Symbol.for(${JSON.stringify(Symbol.keyFor(probeKey))})];\n` + javascript
              .replace(/import \{ browser \} from ["']\$app\/environment["'];/, "const browser = probe.browser;")
              .replace(/import \{ base \} from ["']\$app\/paths["'];/, 'const base = "/app";')
              .replace(/import \* as locale from ["']\.\/locale(?:\.js)?["'];/, 'const locale = { baseLocale: "en", locales: ["en", "fr"] };')
              .replace(/import \{ createWebLocaleI18n \} from ["']\.\/web(?:\.js)?["'];/, `const createWebLocaleI18n = (runtime, options, environment) => {
                probe.options = options; probe.environment = environment;
                return { ...runtime, resolveLocaleSync(input) { probe.input = input; return Object.values(input).some(Boolean) ? "fr" : "en"; } };
              };`)
              .replace(/import \{ getCurrentLocale, initializeCurrentLocale \} from ["']\.\/svelte-locale\.svelte\.js["'];/, 'const getCurrentLocale = () => probe.locale; const initializeCurrentLocale = (locale) => { probe.locale = locale; };')
              .replace(/import \{ startRuntimeLinkLocalization \} from ["']\.\/web\/runtime-links\.js["'];/, `const startRuntimeLinkLocalization = (_web, current) => {
                probe.starts++; if (current() !== probe.locale) throw new Error("wrong locale reader");
                return { refresh() { probe.refreshes++; }, destroy() { probe.destroys++; } };
              };`);
            try {
              const effects = await import(`data:text/javascript;base64,${Buffer.from(`${javascript}\n// ${nonce++}`).toString("base64")}`);
              const expectedKeys = environment === "server" ? [] : keys[capability];
              expect(probe.reads).toEqual(expectedKeys);
              expect(Object.keys(probe.input ?? {})).toEqual(expectedKeys);
              if (environment === "denied") expect(Object.values(probe.input ?? {}).every((value) => value === undefined)).toBe(true);
              expect(probe.locale).toBe(environment === "browser" && capability !== "none" ? "fr" : "en");
              const activeLinks = capability === "all" && environment !== "server";
              expect(probe.starts).toBe(activeLinks ? 1 : 0);
              effects.refreshLinguiniEffects();
              effects.destroyLinguiniEffects();
              probe.dispose();
              expect(probe.refreshes).toBe(activeLinks ? 1 : 0);
              expect(probe.destroys).toBe(activeLinks ? 2 : 0);
              expect(probe.environment).toEqual(framework === "kit" ? { base: "/app" } : {});
              expect("cookie" in probe.options).toBe(keys[capability].includes("cookie"));
              expect("localStorage" in probe.options).toBe(keys[capability].includes("localStorage"));
              return { reads: probe.reads, inputKeys: Object.keys(probe.input ?? {}), locale: probe.locale, starts: probe.starts, refreshes: probe.refreshes, destroys: probe.destroys, options: probe.options, environment: probe.environment };
            } finally {
              for (const [key, descriptor] of descriptors) {
                if (descriptor) Object.defineProperty(globalThis, key, descriptor);
                else Reflect.deleteProperty(globalThis, key);
              }
              Reflect.deleteProperty(globalThis, probeKey);
            }
          };
          expect(await run("javascript")).toEqual(await run("typescript"));
        });
      }
    }
  }
});
