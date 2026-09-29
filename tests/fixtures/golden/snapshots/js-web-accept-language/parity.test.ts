import { describe, expect, test } from "bun:test";

import { resolveAcceptLanguageLocale as javaScriptResolve } from "./web/accept-language.js";
import { resolveAcceptLanguageLocale as typeScriptResolve } from "./typescript/web/accept-language.ts";

const locales = ["en", "de", "fr"] as const;
const matchLocale = (value: unknown) => locales.find((locale) => locale === value);

describe("web Accept-Language target parity", () => {
  test("quality, specificity, wildcards, navigator, and hostile getters agree", () => {
    const cases: { input: Record<string, unknown>; expected: string | undefined }[] = [
      { input: { headers: new Headers({ "accept-language": "fr-CA, de;q=0.8" }) }, expected: "fr" },
      { input: { headers: new Headers({ "accept-language": "de;q=0.9, fr;q=0.7" }) }, expected: "de" },
      { input: { headers: new Headers({ "accept-language": "fr;q=0, *;q=0.5" }) }, expected: "en" },
      { input: { headers: new Headers({ "accept-language": "fr;q=invalid, en;q=0.5" }) }, expected: "en" },
      { input: { headers: new Headers({ "accept-language": "*" }) }, expected: "en" },
      { input: { headers: { get: () => { throw new Error("denied"); } } }, expected: undefined },
      { input: { navigator: { languages: ["fr-CA", "en-US"], language: "de" } }, expected: "fr" },
      { input: { navigator: { languages: "fr", language: "de" } }, expected: "de" },
      { input: {}, expected: undefined },
    ];
    for (const { input, expected } of cases) {
      expect(javaScriptResolve(input, locales, "en", matchLocale)).toBe(expected);
      expect(typeScriptResolve(input, locales, "en", matchLocale)).toBe(expected);
    }
  });
});
