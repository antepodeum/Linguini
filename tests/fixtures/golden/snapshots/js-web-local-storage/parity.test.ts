import { describe, expect, test } from "bun:test";

import { resolveLocalStorageLocale as javaScriptResolve } from "./web/local-storage.js";
import { resolveLocalStorageLocale as typeScriptResolve } from "./typescript/web/local-storage.ts";

const locales = ["en", "de", "fr"] as const;
const matchLocale = (value: unknown) => locales.find((locale) => locale === value);
const options = { localStorage: { key: "locale" } };

describe("web local-storage target parity", () => {
  test("stored, absent, and denied values agree", () => {
    const cases: { localStorage?: { getItem(key: string): string | null }; expected: string | undefined }[] = [
      { localStorage: { getItem: (key) => key === "locale" ? "de" : null }, expected: "de" },
      { localStorage: { getItem: () => null }, expected: undefined },
      { localStorage: { getItem: () => "unknown" }, expected: undefined },
      { localStorage: { getItem: () => { throw new Error("denied"); } }, expected: undefined },
      { expected: undefined },
    ];
    for (const { localStorage, expected } of cases) {
      expect(javaScriptResolve({ localStorage }, options, matchLocale)).toBe(expected);
      expect(typeScriptResolve({ localStorage }, options, matchLocale)).toBe(expected);
    }
  });
});
