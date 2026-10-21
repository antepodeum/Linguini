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

  test("specific exclusions, input order, base preference and locale order stay distinct", () => {
    const supported = ["en", "en-US", "en-GB", "fr", "fr-CA", "de"] as const;
    const match = (value: unknown) => supported.find((locale) => locale === value);
    const cases = [
      ["EN-us;q=0.7, fr;q=0.6", "en"],
      ["en-US;q=0, *;q=0.5", "en-GB"],
      ["en-US;q=0, en;q=1, fr;q=0.5", "en-GB"],
      ["fr;q=0.1, fr-CA;q=0.8, en;q=0.7", "fr"],
      ["fr;q=0.1, fr;q=1, en;q=0.5", "en"],
      ["fr;q=1, fr;q=0, en;q=0.5", "fr"],
      ["*;q=0.5", "en"],
      ["fr;q=0.5, en;q=0.5", "fr"],
      ["*;q=0, fr-CA;q=0.4", "fr"],
      ["*;q=0", undefined],
      ["fr;q=1.001, de;q=0.001", "de"],
      ["fr;q=.8, de;q=1.000", "de"],
      ["fr;q=0.1234, en", "en"],
      ["fr; Q = 0.8; q=0, en;q=0.7", "fr"],
    ] as const;
    for (const resolve of [javaScriptResolve, typeScriptResolve]) {
      for (const [header, expected] of cases) {
        expect(resolve({ headers: new Headers({ "accept-language": header }) }, supported, "en", match)).toBe(expected);
      }
      expect(resolve({ headers: new Headers({ "accept-language": "*" }) }, ["fr", "en"], "en", match)).toBe("en");
      expect(resolve({ headers: new Headers({ "accept-language": "*" }) }, ["fr-CA", "fr"], "en", match)).toBe("fr-CA");
    }
  });

  test("linear selection agrees with the frozen implementation across mixed ranges", () => {
    const supported = ["fr-CA", "en-GB", "de", "EN-us", "fr", "en"] as const;
    const match = (value: unknown) => supported.find((locale) => locale === value);
    const ranges = ["*", "en", "EN-us", "en-GB", "fr", "fr-CA", "de", "invalid_", "zz"];
    const qualities = ["0", "0.1", "0.5", "1", "invalid"];
    for (let index = 0; index < 512; index++) {
      const header = Array.from({ length: 5 }, (_, position) => {
        const range = ranges[(index + position * 7 + Math.floor(index / 11)) % ranges.length];
        return `${range};q=${qualities[(index * (position + 1) + position) % qualities.length]}`;
      }).join(",");
      const input = { headers: new Headers({ "accept-language": header }) };
      expect(javaScriptResolve(input, supported, "en", match)).toBe(typeScriptResolve(input, supported, "en", match));
    }
  });
});
