import { describe, expect, test } from "bun:test";

import { resolvePathLocale as javaScriptResolve } from "./web/path.js";
import { resolvePathLocale as typeScriptResolve } from "./typescript/web/path.ts";

const locales = ["en", "de", "fr"] as const;

describe("web path target parity", () => {
  test("base paths, boundaries, invalid URLs, and locale casing agree", () => {
    const cases: { url?: URL | string; base: string; expected: string | undefined }[] = [
      { url: "https://example.test/shop/de/orders", base: "/shop", expected: "de" },
      { url: "https://example.test/shopkeeper/de", base: "/shop", expected: undefined },
      { url: "https://example.test/shop", base: "/shop", expected: undefined },
      { url: "https://example.test/FR", base: "", expected: "fr" },
      { url: new URL("https://example.test/de"), base: "/", expected: "de" },
      { url: "not a URL", base: "", expected: undefined },
      { base: "", expected: undefined },
    ];
    for (const { url, base, expected } of cases) {
      const input = { url };
      const options = { environment: { base } };
      expect(javaScriptResolve(input, options, locales)).toBe(expected);
      expect(typeScriptResolve(input, options, locales)).toBe(expected);
    }
  });
});
