import { describe, expect, test } from "bun:test";

import {
  Render as renderJavaScript,
  __lgl_form_4672756974 as fruitJavaScript,
  prefix as prefixJavaScript,
} from "./locales/en-us/_globals.js";
import {
  Render as renderTypeScript,
  __lgl_form_4672756974 as fruitTypeScript,
  prefix as prefixTypeScript,
} from "./typescript/locales/en-us/_globals.ts";

const cases = [
  ["male", 1],
  ["male", 2],
  ["other", 1],
  ["other", 2],
] as const;

describe("locale globals target parity", () => {
  test("functions execute identically", () => {
    for (const [gender, count] of cases) {
      expect(renderJavaScript(gender, count)).toBe(renderTypeScript(gender, count));
    }
  });

  test("forms and variables execute identically", () => {
    expect(prefixJavaScript).toBe(prefixTypeScript);
    for (const count of [1, 2]) {
      expect(fruitJavaScript.apple.label(count)).toBe(fruitTypeScript.apple.label(count));
    }
  });
});
