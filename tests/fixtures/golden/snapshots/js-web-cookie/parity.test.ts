import { describe, expect, test } from "bun:test";

import { resolveCookieLocale as javaScriptResolve } from "./web/cookie.js";
import { resolveCookieLocale as typeScriptResolve } from "./typescript/web/cookie.ts";

const locales = ["en", "de", "fr"] as const;
const matchLocale = (value: unknown) => locales.find((locale) => locale === value);
const options = { cookie: { name: "LINGUINI_LOCALE" } };

describe("web cookie target parity", () => {
  test("cookie boundaries, decoding, and malformed values agree", () => {
    const cases: { cookie?: string; expected: string | undefined }[] = [
      { cookie: "other=fr; LINGUINI_LOCALE=de", expected: "de" },
      { cookie: "LINGUINI_LOCALE=fr", expected: "fr" },
      { cookie: "LINGUINI_LOCALE=%66%72", expected: "fr" },
      { cookie: "LINGUINI_LOCALE=de=ignored", expected: undefined },
      { cookie: "LINGUINI_LOCALE=%ZZ", expected: undefined },
      { cookie: "NOT_LINGUINI_LOCALE=de", expected: undefined },
      { expected: undefined },
    ];
    for (const { cookie, expected } of cases) {
      expect(javaScriptResolve({ cookie }, options, matchLocale)).toBe(expected);
      expect(typeScriptResolve({ cookie }, options, matchLocale)).toBe(expected);
    }
  });
});
