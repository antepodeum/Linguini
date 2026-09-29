import { describe, expect, test } from "bun:test";

import { localizeTransformedHref as javaScriptLocalize } from "./web/link-transform.js";
import { localizeTransformedHref as typeScriptLocalize } from "./typescript/web/link-transform.ts";

describe("web transformed-link target parity", () => {
  test("selected and ignored links agree", () => {
    const cases = [
      { href: "/shop", attributes: {}, expected: "/de/shop" },
      { href: "/shop", attributes: { ignored: true }, expected: "/shop" },
      { href: "/download", attributes: { download: true }, expected: "/download" },
      { href: "/external", attributes: { rel: "external" }, expected: "/external" },
      { href: "https://example.test/shop", attributes: {}, expected: "https://example.test/shop" },
    ];
    for (const { href, attributes, expected } of cases) {
      expect(javaScriptLocalize(href, attributes)).toBe(expected);
      expect(typeScriptLocalize(href, attributes)).toBe(expected);
    }
  });
});
