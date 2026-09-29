import { describe, expect, test } from "bun:test";

import * as javaScript from "./index.js";
import * as typeScript from "./typescript/index.ts";

describe("project entry target parity", () => {
  test("locale metadata, fallback, eager base, and dynamic locale behavior match", async () => {
    expect(javaScript.locales).toEqual(typeScript.locales);
    expect(javaScript.normalizeLocale("FR-latn")).toBe(
      typeScript.normalizeLocale("FR-latn"),
    );
    expect(javaScript.normalizeLocale("unknown")).toBeUndefined();
    expect(javaScript.getTextDirection("en")).toBe(
      typeScript.getTextDirection("en"),
    );
    expect(javaScript.lgl.hello("Ada")).toBe(typeScript.lgl.hello("Ada"));
    expect(javaScript.createLinguini("unknown" as never).hello("Ada")).toBe(
      typeScript.createLinguini("unknown" as never).hello("Ada"),
    );

    const [javaScriptFrench, typeScriptFrench] = await Promise.all([
      javaScript.prepareLinguini("fr"),
      typeScript.prepareLinguini("fr"),
    ]);
    expect(javaScriptFrench.hello("Ada")).toBe(typeScriptFrench.hello("Ada"));

    let selected: "en" | "fr" = "en";
    const javaScriptProvider = javaScript.createLinguiniProvider({
      getLocale: () => selected,
    });
    const typeScriptProvider = typeScript.createLinguiniProvider({
      getLocale: () => selected,
    });
    expect(javaScriptProvider.hello("Ada")).toBe(typeScriptProvider.hello("Ada"));
    selected = "fr";
    expect(javaScriptProvider.hello("Ada")).toBe(typeScriptProvider.hello("Ada"));
  });

  test("concurrent loads deduplicate and failed loads retry", async () => {
    const originalLoader = javaScript.localeLoaders.fr;
    delete javaScript.localeModules.fr;
    let calls = 0;
    let release: ((messages: Awaited<ReturnType<typeof originalLoader>>) => void) | undefined;
    javaScript.localeLoaders.fr = () => {
      calls += 1;
      return new Promise((resolve) => {
        release = resolve;
      });
    };

    const first = javaScript.prepareLinguini("fr");
    const second = javaScript.prepareLinguini("fr");
    expect(calls).toBe(1);
    release?.(await originalLoader());
    const [firstResult, secondResult] = await Promise.all([first, second]);
    expect(firstResult).toBe(secondResult);

    delete javaScript.localeModules.fr;
    javaScript.localeLoaders.fr = async () => {
      calls += 1;
      throw new Error("transient");
    };
    await expect(javaScript.prepareLinguini("fr")).rejects.toThrow("transient");
    javaScript.localeLoaders.fr = originalLoader;
    await expect(javaScript.prepareLinguini("fr")).resolves.toBeDefined();
    expect(calls).toBe(2);
  });
});
