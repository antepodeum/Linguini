import { describe, expect, test } from "bun:test";

import { matchesRoute as javaScriptMatches } from "./web/routes.js";
import { matchesRoute as typeScriptMatches } from "./typescript/web/routes.ts";

describe("web route target parity", () => {
  test("exact, recursive, predicate, and reusable regex patterns match", () => {
    const urls = [
      new URL("https://example.test/api"),
      new URL("https://example.test/api/orders"),
      new URL("https://example.test/apiculture"),
    ];
    for (const url of urls) {
      for (const pattern of ["/api", "/api/**", "/**", (value: URL) => value.pathname === "/api"]) {
        expect(javaScriptMatches(pattern, url)).toBe(typeScriptMatches(pattern, url));
      }
    }
    const pattern = /^\/api/g;
    expect(javaScriptMatches(pattern, urls[1])).toBe(true);
    expect(javaScriptMatches(pattern, urls[1])).toBe(true);
    expect(typeScriptMatches(pattern, urls[1])).toBe(true);
    expect(typeScriptMatches(pattern, urls[1])).toBe(true);
    expect(pattern.lastIndex).toBe(0);
  });
});
