import { test, expect } from "bun:test";

for (const variant of ["plain", "source-only"]) {
  test(`${variant}: native project imports preserve named/positional calls, fallback and preparation`, async () => {
    for (const target of ["javascript/index.js", "typescript/index.ts"]) {
      const runtime = await import(`./${variant}/${target}`);
      expect(runtime.locales).toEqual(["en", "fr"]);
      expect(runtime.createLinguini("en").hello("Ada")).toBe("Hello Ada");
      expect(() => runtime.createLinguini("fr")).toThrow("not prepared");
      const [first, second] = await Promise.all([runtime.prepareLinguini("fr"), runtime.prepareLinguini("fr")]);
      expect(first).toBe(second);
      expect(runtime.createLinguini("fr").hello({ name: "Ada" })).toBe("Bonjour Ada");
      expect(runtime.createLinguini("fr").section.title).toBe("English");
      expect(runtime.createLinguini("en").section.title).toBe("English");
      expect(() => runtime.createLinguini("fr").hello({ name: "Ada", extra: true })).toThrow("extra");
      expect((await runtime.prepareLinguini("fr")).hello("Ada")).toBe("Bonjour Ada");
      const provider = runtime.configureLinguini({ language: () => "fr" });
      expect(provider.hello({ name: "Ada" })).toBe("Bonjour Ada");
      expect(provider.section.title).toBe("English");
    }
  });
}
