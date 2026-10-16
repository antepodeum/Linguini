import { test, expect } from "bun:test";

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
