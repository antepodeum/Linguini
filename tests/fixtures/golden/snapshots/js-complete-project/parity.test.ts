import { test, expect } from "bun:test";
import { AsyncLocalStorage } from "node:async_hooks";
import { readFile } from "node:fs/promises";
import { withRuntimeHost } from "../../runtime-host.ts";

test("Kit host reads request locale only when called and preserves concurrent contexts", async () => {
  const requests = new AsyncLocalStorage<{ locale: string }>();
  let reads = 0;
  const page = {
    get data() {
      reads++;
      return { linguini: requests.getStore() };
    },
  };
  const source = await readFile(new URL("kit/javascript/sveltekit-locale.js", import.meta.url), "utf8");
  await withRuntimeHost(source, "javascript", { modules: { "$app/state": { page } } }, async (host) => {
    expect(reads).toBe(0);
    const result = await Promise.all(["en", "fr"].map((locale) => requests.run({ locale }, async () => {
      expect(host.getRequestLocale()).toBe(locale);
      await Promise.resolve();
      expect(host.getRequestLocale()).toBe(locale);
      return host.getRequestLocale();
    })));
    expect(result).toEqual(["en", "fr"]);
    expect(reads).toBe(6);
    expect(host.getRequestLocale()).toBeUndefined();
  });
});

test("Kit navigation bridge forwards options and preserves completion and rejection", async () => {
  let complete!: () => void;
  let reject!: (error: Error) => void;
  let pending: Promise<void>;
  const calls: unknown[] = [];
  const source = await readFile(new URL("kit/javascript/sveltekit-navigation.js", import.meta.url), "utf8");
  const goto = (href: string, options: unknown) => {
    calls.push([href, options]);
    return pending = new Promise<void>((resolve, fail) => { complete = resolve; reject = fail; });
  };
  await withRuntimeHost(source, "javascript", { modules: { "$app/navigation": { goto } } }, async (host) => {
    const state = { tab: "account" };
    const operation = host.navigate("/fr/account", {
      replaceState: true, invalidateAll: false, keepFocus: true, noScroll: false, state,
      cookie: true, navigate: true,
    });
    expect(operation).toBe(pending!);
    expect(calls).toEqual([["/fr/account", {
      replaceState: true, invalidateAll: false, keepFocus: true, noScroll: false, state,
    }]]);
    complete();
    await operation;
    const failed = host.navigate("/en/account", {});
    expect(failed).toBe(pending!);
    const error = new Error("navigation failed");
    reject(error);
    await expect(failed).rejects.toBe(error);
  });
});

test("shared helpers preserve dispatch precedence and named-argument validation across targets", async () => {
  for (const target of ["javascript/shared.js", "typescript/shared.ts"]) {
    const { selectBranch, normalizeMessageArgs } = await import(`./plain/${target}`);
    expect(selectBranch("key", { key: undefined, _: "fallback" })).toBeUndefined();
    expect(selectBranch("missing", { _: "default", other: "other" })).toBe("default");
    expect(selectBranch("missing", { other: "other" })).toBe("other");
    const branches = Object.assign(Object.create({ key: "inherited" }), { _: "own" });
    expect(selectBranch("key", branches)).toBe("own");
    expect(() => selectBranch("missing", Object.create({ _: "inherited" }))).toThrow(
      'Linguini dispatch has no branch for key "missing"'
    );

    const named = Object.assign(Object.create(null), { name: "Ada" });
    expect(normalizeMessageArgs([named], ["name"])).toEqual(["Ada"]);
    expect(normalizeMessageArgs([{ value: "positional" }], ["name"])).toEqual([
      { value: "positional" }
    ]);
    expect(normalizeMessageArgs([null], ["name"])).toEqual([null]);
    expect(normalizeMessageArgs([named, "tail"], ["name"])).toEqual([named, "tail"]);
    expect(() => normalizeMessageArgs([{ name: "Ada", z: 1, a: 2 }], ["name"])).toThrow(
      "Linguini message arguments contain unknown keys: a, z"
    );
    expect(() => normalizeMessageArgs([{ name: "Ada", [Symbol("extra")]: 1 }], ["name"])).toThrow(
      "Symbol(extra)"
    );
  }
});

for (const variant of ["plain", "source-only"]) {
  test(`${variant}: native project imports preserve named/positional calls, fallback and preparation`, async () => {
    for (const target of ["javascript/index.js", "typescript/index.ts"]) {
      const runtime = await import(`./${variant}/${target}`);
      expect(runtime.locales).toEqual(["en", "fr"]);
      expect(runtime.createLinguini("en").hello("Ada")).toBe("Hello Ada");
      expect(() => runtime.createLinguini("fr")).toThrow("not prepared");
      const [first, second] = await Promise.all([
        runtime.prepareLinguini("fr"),
        runtime.prepareLinguini("fr")
      ]);
      expect(first).toBe(second);
      expect(runtime.createLinguini("fr").hello({ name: "Ada" })).toBe("Bonjour Ada");
      expect(runtime.createLinguini("fr").section.title).toBe("English");
      expect(runtime.createLinguini("en").section.title).toBe("English");
      expect(() => runtime.createLinguini("fr").hello({ name: "Ada", extra: true })).toThrow(
        "extra"
      );
      expect((await runtime.prepareLinguini("fr")).hello("Ada")).toBe("Bonjour Ada");
      const provider = runtime.configureLinguini({ language: () => "fr" });
      expect(provider.hello({ name: "Ada" })).toBe("Bonjour Ada");
      expect(provider.section.title).toBe("English");
    }
  });
}
