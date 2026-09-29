import { describe, expect, test } from "bun:test";
import { readFile } from "node:fs/promises";
import { createRequire } from "node:module";
import { pathToFileURL } from "node:url";

const requireSite = createRequire(new URL("../../../../../site/package.json", import.meta.url));
const { compileModule } = requireSite("svelte/compiler");
const transpiler = new Bun.Transpiler({ loader: "ts", target: "browser" });
const localeUrl = new URL("../js-project/locale.js", import.meta.url).href;
const pageKey = Symbol.for("linguini.svelte.locale.parity.page");
let nonce = 0;

type Locale = "en" | "fr";
type State = {
  getCurrentLocale(): Locale;
  setCurrentLocale(locale: unknown): Locale;
  prepareLocale(locale: unknown): Promise<Locale>;
  registerLocaleLoader(loader: (locale: Locale) => void | Promise<void>): () => void;
  initializeCurrentLocale?(locale: unknown): Locale;
  clearCurrentLocaleOverride?(): void;
};

async function compileState(mode: string, target: "javascript" | "typescript", generate: "client" | "server") {
  const filename = target === "javascript" ? `${mode}/svelte-locale.svelte.js` : `${mode}/typescript/svelte-locale.svelte.ts`;
  const source = await readFile(new URL(filename, import.meta.url), "utf8");
  const javascript = target === "typescript" ? transpiler.transformSync(source) : source;
  const result = compileModule(javascript, { filename, generate, dev: false });
  expect(result.warnings).toEqual([]);
  const compiled = result.js.code
    .replace(/from (["'])svelte\/internal\/(client|server)\1/g, (_match: string, _quote: string, name: string) => `from ${JSON.stringify(pathToFileURL(requireSite.resolve(`svelte/internal/${name}`)).href)}`)
    .replace(/import \{ page \} from ["']\$app\/state["'];/, `const page = globalThis[Symbol.for(${JSON.stringify(Symbol.keyFor(pageKey))})];`)
    .replace(/from ["']\.\/locale(?:\.js)?["']/g, `from ${JSON.stringify(localeUrl)}`);
  return import(`data:text/javascript;base64,${Buffer.from(`${compiled}\n// instance ${nonce++}`).toString("base64")}`) as Promise<State>;
}

describe("Svelte locale target parity through real rune compilation", () => {
  for (const mode of ["context", "standalone", "sveltekit"]) {
    for (const generate of ["client", "server"] as const) {
      test(`${mode}: ${generate} state, snapshots, disposal and failure semantics`, async () => {
        const run = async (target: "javascript" | "typescript") => {
          const page = { data: { linguini: { locale: "en" } } };
          Reflect.set(globalThis, pageKey, page);
          try {
            const state = await compileState(mode, target, generate);
            expect(state.getCurrentLocale()).toBe("en");
            expect(await state.prepareLocale("unknown")).toBe("en");
            const seen: string[] = [];
            let disposeSecond = () => {};
            let disposeLate = () => {};
            const disposeFirst = state.registerLocaleLoader((locale) => {
              seen.push(`first:${locale}`);
              disposeSecond();
              disposeLate = state.registerLocaleLoader((next) => { seen.push(`late:${next}`); });
            });
            disposeSecond = state.registerLocaleLoader(async (locale) => { seen.push(`second:${locale}`); });
            expect(await state.prepareLocale("FR")).toBe("fr");
            expect(seen).toEqual(["first:fr", "second:fr"]);
            disposeFirst();
            disposeFirst();
            expect(await state.prepareLocale("en")).toBe("en");
            expect(seen).toEqual(["first:fr", "second:fr", "late:en"]);
            disposeLate();
            const disposeFailure = state.registerLocaleLoader(() => Promise.reject(new Error("loader failed")));
            await expect(state.prepareLocale("fr")).rejects.toThrow("loader failed");
            expect(state.getCurrentLocale()).toBe("en");
            disposeFailure();
            expect(state.setCurrentLocale("FR")).toBe("fr");
            expect(state.getCurrentLocale()).toBe("fr");
            if (mode === "sveltekit") {
              state.clearCurrentLocaleOverride!();
              expect(state.getCurrentLocale()).toBe("en");
              page.data.linguini.locale = "fr";
              expect(state.getCurrentLocale()).toBe("fr");
              // A subsequent SSR request reads its own serialized page data.
              page.data.linguini.locale = "en";
              expect(state.getCurrentLocale()).toBe("en");
              state.initializeCurrentLocale!("fr");
              expect(state.getCurrentLocale()).toBe("en");
            } else if (mode === "standalone") {
              expect(state.initializeCurrentLocale!(null)).toBe("en");
            } else {
              expect(state.initializeCurrentLocale).toBeUndefined();
            }
            return { seen, locale: state.getCurrentLocale(), exports: Object.keys(state).sort() };
          } finally {
            Reflect.deleteProperty(globalThis, pageKey);
          }
        };
        expect(await run("javascript")).toEqual(await run("typescript"));
      });
    }
  }
});
