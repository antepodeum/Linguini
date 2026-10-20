import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { existsSync, mkdirSync, mkdtempSync, readFileSync, rmSync, statSync, writeFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { after, test } from "node:test";
import { runTypeScript } from "../src/index.js";

const repository = fileURLToPath(new URL("../../../", import.meta.url));
const command = process.env.LINGUINI_COMMAND ?? path.join(repository, "target/debug", process.platform === "win32" ? "linguini.exe" : "linguini");
if (!process.env.LINGUINI_COMMAND) {
  const built = spawnSync("cargo", ["build", "--locked", "-p", "linguini-cli"], { cwd: repository, encoding: "utf8" });
  assert.equal(built.status, 0, built.stderr);
}
const temporary = mkdtempSync(path.join(repository, "target/tsc-integration-"));
after(() => rmSync(temporary, { recursive: true, force: true }));
let sequence = 0;

function write(root, file, value) {
  const destination = path.join(root, file);
  mkdirSync(path.dirname(destination), { recursive: true });
  writeFileSync(destination, value);
}

function fixture(options = {}) {
  const root = path.join(temporary, String(sequence++));
  mkdirSync(root);
  write(root, "package.json", '{"type":"module"}');
  write(root, "linguini.toml", `[project]\nname = "tsc"\ndefault_locale = "en"\nlocales = ["en", "fr"]\n[paths]\nschema = "schema"\nlocale = "locale"\n[targets.ts]\nout = "src/generated"\ndeclaration = true\ngitignore = false\n`);
  write(root, "schema/main.lgs", "greeting(name: String)\n");
  write(root, "locale/main/en.lgl", "greeting = Hello {name}\n");
  write(root, "locale/main/fr.lgl", "greeting = Bonjour {name}\n");
  write(root, "base.json", JSON.stringify({ compilerOptions: {
    strict: true, allowJs: false, module: "NodeNext", moduleResolution: "NodeNext", target: "ES2022",
    rootDir: "src", outDir: "dist", sourceMap: true, incremental: true, ...options,
  } }));
  write(root, "tsconfig.json", JSON.stringify({ extends: "./base.json", include: ["src/**/*.ts"] }));
  write(root, "src/app.ts", `import { createLinguini, prepareLinguini } from "./generated/index.js";
import type { Locale } from "./generated/index.js";
export async function greet(locale: Locale) {
  await prepareLinguini(locale);
  return createLinguini(locale).main.greeting({ name: "Ada" });
}
if (false) {
  // @ts-expect-error Generated declarations retain exact named arguments.
  createLinguini("en").main.greeting({ name: 42 });
  // @ts-expect-error Locale names remain finite without allowJs.
  createLinguini("de");
}
console.log(await greet("fr"));
`);
  return root;
}

function compile(root, args = [], options = {}) {
  const diagnostics = [];
  const result = runTypeScript(args, { cwd: root, command, nativeArgs: ["build", "--format", "json"], reportDiagnostic: (error) => diagnostics.push(error), ...options });
  assert.equal(result.status, 0, diagnostics.map((item) => JSON.stringify(item.messageText)).join("\n"));
  return result;
}

function pnpm(args, cwd) {
  return process.env.npm_execpath
    ? spawnSync(process.execPath, [process.env.npm_execpath, ...args], { cwd, encoding: "utf8" })
    : spawnSync("pnpm", args, { cwd, encoding: "utf8" });
}

function execute(root, file = "dist/app.js") {
  const result = spawnSync(process.execPath, [path.join(root, file)], { cwd: root, encoding: "utf8" });
  assert.equal(result.status, 0, result.stderr);
  return result.stdout.trim();
}

test("strict tsc builds execute native JS with rebased maps and inferred or explicit rootDir", () => {
  for (const inferred of [false, true]) {
    const root = fixture(inferred ? { rootDir: undefined } : {});
    if (inferred) {
      write(root, "root.ts", "export {};\n");
      write(root, "tsconfig.json", JSON.stringify({ extends: "./base.json", include: ["src/**/*.ts", "root.ts"] }));
    }
    compile(root);
    const output = inferred ? "dist/src" : "dist";
    assert.equal(execute(root, `${output}/app.js`), "Bonjour Ada");
    assert(existsSync(path.join(root, output, "generated/index.d.ts")));
    const mapFile = path.join(root, output, "generated/locales/fr.js.map");
    const map = JSON.parse(readFileSync(mapFile, "utf8"));
    assert(map.sources.length > 0);
    for (const source of map.sources) {
      assert(!path.isAbsolute(source));
      assert(!source.includes("\\"));
      assert(existsSync(path.resolve(path.dirname(mapFile), source)), source);
    }
  }
});

test("noEmit and emitDeclarationOnly prepare source types without runtime output", () => {
  for (const flags of [["--noEmit"], ["--declaration", "--emitDeclarationOnly"]]) {
    const root = fixture();
    compile(root, flags);
    assert(existsSync(path.join(root, "src/generated/index.d.ts")));
    assert(!existsSync(path.join(root, "dist/generated/index.js")));
    assert(!existsSync(path.join(root, "dist/app.js")));
    assert.equal(existsSync(path.join(root, "dist/generated/index.d.ts")), !flags.includes("--noEmit"));
  }
});

test("source-file command lines retain TypeScript's inferred output layout", () => {
  const root = fixture();
  compile(root, ["src/app.ts", "--strict", "--module", "NodeNext", "--moduleResolution", "NodeNext", "--target", "ES2022", "--outDir", "dist"]);
  assert.equal(execute(root), "Bonjour Ada");
});

test("type errors prevent runtime delivery and preserve diagnostic failure", () => {
  const root = fixture({ noEmitOnError: true });
  write(root, "src/invalid.ts", "const message: string = 42;\n");
  const diagnostics = [];
  const result = runTypeScript([], { cwd: root, command, nativeArgs: ["build", "--format", "json"], reportDiagnostic: (error) => diagnostics.push(error) });
  assert.equal(result.status, 1);
  assert(diagnostics.some((item) => item.code === 2322));
  assert(!existsSync(path.join(root, "dist/generated/index.js")));
  assert(!existsSync(path.join(root, "dist/app.js")));
});

test("separate declarationDir delivers resolvable generated types without duplicate JS", () => {
  const root = fixture({ declaration: true, declarationDir: "types" });
  compile(root);
  assert.equal(execute(root), "Bonjour Ada");
  assert(existsSync(path.join(root, "types/generated/index.d.ts")));
  assert(!existsSync(path.join(root, "types/generated/index.js")));
  write(root, "src/types-consumer.ts", `import type { greet } from "../types/app.js";
const locale: Parameters<typeof greet>[0] = "fr";
// @ts-expect-error Emitted declaration imports keep the finite locale type.
const invalid: Parameters<typeof greet>[0] = "de";
void [locale, invalid];
`);
  compile(root, ["--noEmit"]);
});

test("solution builds refresh message-only changes and missing runtime output despite up-to-date TS", () => {
  const root = fixture({ composite: true, declaration: true });
  write(root, "solution.json", JSON.stringify({ files: [], references: [{ path: "./tsconfig.json" }] }));
  compile(root, ["--build", "solution.json"]);
  assert.equal(execute(root), "Bonjour Ada");
  const application = path.join(root, "dist/app.js");
  const before = statSync(application).mtimeMs;
  const declarations = statSync(path.join(root, "src/generated/locales/fr.d.ts")).mtimeMs;
  write(root, "locale/main/fr.lgl", "greeting = Salut {name}\n");
  compile(root, ["--build", "solution.json"]);
  assert.equal(execute(root), "Salut Ada");
  assert.equal(statSync(application).mtimeMs, before, "TypeScript app output should remain up to date");
  assert.equal(statSync(path.join(root, "src/generated/locales/fr.d.ts")).mtimeMs, declarations);
  rmSync(path.join(root, "dist/generated"), { recursive: true });
  compile(root, ["--build", "solution.json"]);
  assert.equal(execute(root), "Salut Ada");
  assert.equal(statSync(application).mtimeMs, before);
});

test("a packed installation provides its executable, public types, and runnable output", () => {
  const root = fixture();
  const archive = path.join(root, "linguini-tsc.tgz");
  const packed = pnpm(["pack", "--out", archive], fileURLToPath(new URL("../", import.meta.url)));
  assert.equal(packed.status, 0, packed.stderr);
  write(root, "package.json", JSON.stringify({ type: "module", private: true, dependencies: {
    "@antepod/linguini-tsc": `file:${archive.replaceAll("\\", "/")}`,
    typescript: "6.0.3",
  } }));
  const installed = pnpm(["install", "--offline", "--ignore-scripts"], root);
  assert.equal(installed.status, 0, installed.stderr + installed.stdout);
  write(root, "src/api.ts", `import type { LinguiniTypeScriptOptions } from "@antepod/linguini-tsc";
const options: LinguiniTypeScriptOptions = { cwd: ".", command: "linguini", reportError: (error) => error.message };
// @ts-expect-error Diagnostic reporters receive typed TypeScript diagnostics.
const wrong: LinguiniTypeScriptOptions = { reportDiagnostic: (error: string) => error };
void [options, wrong];
`);
  const executable = path.join(root, "node_modules/@antepod/linguini-tsc/bin/linguini-tsc.js");
  const built = spawnSync(process.execPath, [executable, "--project", "tsconfig.json"], { cwd: root, encoding: "utf8", env: { ...process.env, LINGUINI_COMMAND: command } });
  assert.equal(built.status, 0, built.stderr + built.stdout);
  assert.equal(execute(root), "Bonjour Ada");
});

test("referenced applications and libraries deliver their own native runtimes", async () => {
  const library = fixture({ composite: true, declaration: true, noEmitOnError: true });
  const application = fixture({ composite: true, declaration: true, noEmitOnError: true });
  write(library, "src/app.ts", readFileSync(path.join(library, "src/app.ts"), "utf8").replace('console.log(await greet("fr"));', ""));
  write(application, "tsconfig.json", JSON.stringify({ extends: "./base.json", include: ["src/**/*.ts"], references: [{ path: `../${path.basename(library)}` }] }));
  write(application, "src/app.ts", readFileSync(path.join(application, "src/app.ts"), "utf8") + `\nimport { greet as libraryGreeting } from "../../${path.basename(library)}/dist/app.js";\nconsole.log(await libraryGreeting("fr"));\n`);
  compile(application, ["--build"]);
  assert.equal(execute(application), "Bonjour Ada\nBonjour Ada");
  const before = statSync(path.join(application, "dist/app.js")).mtimeMs;
  write(library, "locale/main/fr.lgl", "greeting = Bibliotheque {name}\n");
  compile(application, ["--build"]);
  assert.equal(execute(application), "Bonjour Ada\nBibliotheque Ada");
  assert.equal(statSync(path.join(application, "dist/app.js")).mtimeMs, before);
  const diagnostics = [];
  const watcher = compile(application, ["--build", "--watch"], { reportDiagnostic: (error) => diagnostics.push(error) });
  try {
    write(application, "src/invalid.ts", "const value: string = 42;\n");
    await eventually(() => diagnostics.some((error) => error.code === 2322));
    write(application, "locale/main/fr.lgl", "greeting = Nouveau {name}\n");
    await eventually(() => readFileSync(path.join(application, "src/generated/locales/fr/main.js"), "utf8").includes("Nouveau"));
    assert.equal(execute(application), "Bonjour Ada\nBibliotheque Ada", "An invalid solution must retain its last successful runtime");
    rmSync(path.join(application, "src/invalid.ts"));
    await eventually(() => readFileSync(path.join(application, "dist/generated/locales/fr/main.js"), "utf8").includes("Nouveau"));
    assert.equal(execute(application), "Nouveau Ada\nBibliotheque Ada");
  } finally { watcher.close(); }
});

async function eventually(predicate) {
  const deadline = Date.now() + 15_000;
  while (!predicate()) {
    if (Date.now() >= deadline) throw new Error("Watch output did not update before the deadline");
    await new Promise((resolve) => setTimeout(resolve, 40));
  }
}

test("watch and solution-watch deliver text-only edits, recover from native errors, and close", async () => {
  for (const solution of [false, true]) {
    const root = fixture(solution ? { composite: true, declaration: true } : {});
    const nativeErrors = [];
    const result = compile(root, solution ? ["--build", "--watch"] : ["--watch"], { reportError: (error) => nativeErrors.push(error) });
    try {
      assert.equal(execute(root), "Bonjour Ada");
      const application = statSync(path.join(root, "dist/app.js")).mtimeMs;
      write(root, "locale/main/fr.lgl", "greeting = Salut {name}\n");
      await eventually(() => readFileSync(path.join(root, "dist/generated/locales/fr/main.js"), "utf8").includes("Salut"));
      assert.equal(execute(root), "Salut Ada");
      assert.equal(statSync(path.join(root, "dist/app.js")).mtimeMs, application);
      write(root, "locale/main/fr.lgl", "greeting = {\n");
      await eventually(() => nativeErrors.length > 0);
      assert.equal(execute(root), "Salut Ada", "A failed native rebuild must retain the last valid runtime");
      write(root, "locale/main/fr.lgl", "greeting = Encore {name}\n");
      await eventually(() => readFileSync(path.join(root, "dist/generated/locales/fr/main.js"), "utf8").includes("Encore"));
      assert.equal(execute(root), "Encore Ada");
      write(root, "schema/secondary.lgs", "farewell\n");
      write(root, "locale/secondary/en.lgl", "farewell = Bye\n");
      await eventually(() => readFileSync(path.join(root, "dist/generated/messages.d.ts"), "utf8").includes("secondary"));
      assert.equal(execute(root), "Encore Ada");
      rmSync(path.join(root, "schema/secondary.lgs"));
      rmSync(path.join(root, "locale/secondary"), { recursive: true });
      await eventually(() => !readFileSync(path.join(root, "dist/generated/messages.d.ts"), "utf8").includes("secondary"));
      const base = JSON.parse(readFileSync(path.join(root, "base.json"), "utf8"));
      base.compilerOptions.outDir = "updated-dist";
      write(root, "base.json", JSON.stringify(base));
      await eventually(() => existsSync(path.join(root, "updated-dist/app.js")) && existsSync(path.join(root, "updated-dist/generated/index.js")));
      assert.equal(execute(root, "updated-dist/app.js"), "Encore Ada");
    } finally { result.close(); }
  }
});
