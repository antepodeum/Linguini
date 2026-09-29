import { describe, expect, test } from "bun:test";

import javaScriptLocale from "./locales/en-US.js";
import typeScriptLocale from "./typescript/locales/en-US.ts";

describe("locale module target parity", () => {
  test("regional overrides and fallback messages match", () => {
    expect(javaScriptLocale.root("Ada")).toBe(typeScriptLocale.root("Ada"));
    expect(javaScriptLocale.root({ name: "Ada" })).toBe(
      typeScriptLocale.root({ name: "Ada" }),
    );
    expect(javaScriptLocale.account.label).toBe(typeScriptLocale.account.label);
    expect(javaScriptLocale.account.personalized("Ada")).toBe(
      typeScriptLocale.account.personalized("Ada"),
    );
    expect(javaScriptLocale.account.personalized({ name: "Ada" })).toBe(
      typeScriptLocale.account.personalized({ name: "Ada" }),
    );
    expect(javaScriptLocale.account.nested.status).toBe(
      typeScriptLocale.account.nested.status,
    );
  });
});
