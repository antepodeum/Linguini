import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { mkdirSync, mkdtempSync, rmSync, symlinkSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

import { documentedCodeblocks } from "./docs-codeblocks.mjs";

const SCRIPT_ROOT = dirname(fileURLToPath(import.meta.url));
const REPOSITORY_ROOT = resolve(SCRIPT_ROOT, "..");

export function findUniqueBlock(blocks, { sourcePath, language, includes }) {
  const matches = blocks.filter(
    (block) =>
      block.sourcePath === sourcePath &&
      block.language === language &&
      includes.every((value) => block.source.includes(value)),
  );
  if (matches.length !== 1) {
    throw new Error(
      `${sourcePath}: expected one ${language} block containing ${includes.join(", ")}; found ${matches.length}`,
    );
  }
  return matches[0];
}

export function executableDirectUseExample(source) {
  const transformed = source
    .replace("getRequestLocale()", '"en"')
    .replace(
      /l\.main\.hello\("Artemy"\);[^\n]*/,
      'export const positional = l.main.hello("Artemy");',
    )
    .replace(
      /l\.main\.field_required\(\{ field: "Email" \}\);[^\n]*/,
      'export const named = l.main.field_required({ field: "Email" });',
    );
  if (!transformed.includes("export const positional") || !transformed.includes("export const named")) {
    throw new Error("documented direct-use example no longer has executable calls");
  }
  return transformed;
}

function writeDocumentedSvelteKitExample(blocks, root, projectRoot) {
  const sourcePath = "docs/examples/sveltekit-locale-provider.md";
  const fixtures = [
    ["linguini.toml", "toml", ["[web.switch_route]", 'name = "app"']],
    ["linguini/schema/account.lgs", "lgs", ["// linguini/schema/account.lgs", "greeting(name: String)"]],
    ["linguini/locale/account/en.lgl", "lgl", ["// linguini/locale/account/en.lgl", "title = Account"]],
    ["linguini/locale/account/ru.lgl", "lgl", ["// linguini/locale/account/ru.lgl", "title = Аккаунт"]],
    ["src/hooks.server.ts", "ts", ["// src/hooks.server.ts", "export { handle }"]],
    ["src/hooks.ts", "ts", ["// src/hooks.ts", "export { reroute }"]],
    ["src/routes/+layout.server.ts", "ts", ["// src/routes/+layout.server.ts", "export { load }"]],
    ["src/routes/account/+page.svelte", "svelte", ["<!-- src/routes/account/+page.svelte -->", "l.account.greeting"]],
  ];
  for (const [relativePath, language, includes] of fixtures) {
    const block = findUniqueBlock(blocks, { sourcePath, language, includes });
    const destination = join(projectRoot, relativePath);
    mkdirSync(dirname(destination), { recursive: true });
    writeFileSync(destination, block.source);
  }
  writeFileSync(join(projectRoot, "package.json"), '{"private":true,"type":"module"}\n');
  writeFileSync(
    join(projectRoot, "svelte.config.js"),
    'import { vitePreprocess } from "@sveltejs/vite-plugin-svelte";\n\nexport default { preprocess: vitePreprocess() };\n',
  );
  writeFileSync(
    join(projectRoot, "tsconfig.json"),
    '{"extends":"./.svelte-kit/tsconfig.json","compilerOptions":{"allowJs":true,"checkJs":true,"strict":true}}\n',
  );
  symlinkSync(resolve(root, "site/node_modules"), join(projectRoot, "node_modules"), "dir");
}

function run(command, args, options) {
  const result = spawnSync(command, args, { encoding: "utf8", ...options });
  if (result.error) throw result.error;
  if (result.status !== 0) {
    process.stdout.write(result.stdout ?? "");
    process.stderr.write(result.stderr ?? "");
    throw new Error(`${command} exited with status ${result.status}`);
  }
}

export async function main({
  root = REPOSITORY_ROOT,
  cliManifestPath = resolve(root, "Cargo.toml"),
  cargoTargetDir,
} = {}) {
  const blocks = documentedCodeblocks(root);
  const sourcePath = "docs/getting-started.md";
  const config = findUniqueBlock(blocks, {
    sourcePath,
    language: "toml",
    includes: ['name           = "my-app"', '[targets.ts]', '[web.routing]'],
  });
  const schema = findUniqueBlock(blocks, {
    sourcePath,
    language: "lgs",
    includes: ["// linguini/schema/main.lgs", "hello(name: String)", "field_required(field: String)"],
  });
  const english = findUniqueBlock(blocks, {
    sourcePath,
    language: "lgl",
    includes: ["// linguini/locale/main/en.lgl", "hello = Hello", "field_required ="],
  });
  const russian = findUniqueBlock(blocks, {
    sourcePath,
    language: "lgl",
    includes: ["// linguini/locale/main/ru.lgl", "hello = Привет", "field_required ="],
  });
  const directUse = findUniqueBlock(blocks, {
    sourcePath,
    language: "ts",
    includes: ["configureLinguini", "l.main.hello", "l.main.field_required"],
  });

  const targetRoot = resolve(root, "target");
  mkdirSync(targetRoot, { recursive: true });
  const projectRoot = mkdtempSync(join(targetRoot, "docs-examples-"));
  const schemaRoot = join(projectRoot, "linguini/schema");
  const localeRoot = join(projectRoot, "linguini/locale/main");
  const sourceRoot = join(projectRoot, "src");
  mkdirSync(schemaRoot, { recursive: true });
  mkdirSync(localeRoot, { recursive: true });
  mkdirSync(sourceRoot, { recursive: true });

  let server;
  try {
    writeFileSync(join(projectRoot, "linguini.toml"), config.source);
    writeFileSync(join(schemaRoot, "main.lgs"), schema.source);
    writeFileSync(join(localeRoot, "en.lgl"), english.source);
    writeFileSync(join(localeRoot, "ru.lgl"), russian.source);

    const cargoOptions = {
      cwd: projectRoot,
      ...(cargoTargetDir
        ? { env: { ...process.env, CARGO_TARGET_DIR: cargoTargetDir } }
        : {}),
    };
    run(
      "cargo",
      [
        "run",
        "--offline",
        "--locked",
        "--quiet",
        "--manifest-path",
        cliManifestPath,
        "-p",
        "linguini-cli",
        "--",
        "build",
      ],
      cargoOptions,
    );

    const example = executableDirectUseExample(directUse.source);
    writeFileSync(join(sourceRoot, "documented-example.ts"), example);
    writeFileSync(join(sourceRoot, "documented-example.js"), example);
    writeFileSync(
      join(projectRoot, "tsconfig.json"),
      `${JSON.stringify({
        compilerOptions: {
          allowJs: true,
          checkJs: true,
          module: "ESNext",
          moduleResolution: "Bundler",
          noEmit: true,
          strict: true,
          target: "ES2022",
        },
        include: ["src/documented-example.ts", "src/documented-example.js"],
      }, null, 2)}\n`,
    );

    const typeScript = resolve(root, "site/node_modules/.bin/tsc");
    run(typeScript, ["--project", join(projectRoot, "tsconfig.json")], { cwd: projectRoot });

    const vitePath = resolve(root, "site/node_modules/vite/dist/node/index.js");
    const { createServer } = await import(pathToFileURL(vitePath));
    server = await createServer({
      root: projectRoot,
      configFile: false,
      logLevel: "silent",
      appType: "custom",
      server: { middlewareMode: true },
    });
    for (const extension of ["ts", "js"]) {
      const generated = await server.ssrLoadModule(`/src/documented-example.${extension}`);
      assert.equal(generated.positional, "Hello, Artemy!");
      assert.equal(generated.named, "Email is required.");
    }

    const svelteKitRoot = join(projectRoot, "sveltekit");
    mkdirSync(svelteKitRoot, { recursive: true });
    writeDocumentedSvelteKitExample(blocks, root, svelteKitRoot);
    run(
      "cargo",
      [
        "run",
        "--offline",
        "--locked",
        "--quiet",
        "--manifest-path",
        cliManifestPath,
        "-p",
        "linguini-cli",
        "--",
        "build",
      ],
      { ...cargoOptions, cwd: svelteKitRoot },
    );
    run(resolve(root, "site/node_modules/.bin/svelte-kit"), ["sync"], { cwd: svelteKitRoot });
    run(
      resolve(root, "site/node_modules/.bin/svelte-check"),
      ["--workspace", svelteKitRoot, "--tsconfig", "./tsconfig.json", "--threshold", "warning"],
      { cwd: svelteKitRoot },
    );
  } finally {
    if (server) await server.close();
    rmSync(projectRoot, { recursive: true, force: true });
  }

  console.log("Built and typechecked documented SvelteKit; built, typechecked, and executed JavaScript/TypeScript.");
  return 0;
}

if (process.argv[1] && fileURLToPath(import.meta.url) === resolve(process.argv[1])) {
  main().then(
    (code) => {
      process.exitCode = code;
    },
    (error) => {
      console.error(error);
      process.exitCode = 1;
    },
  );
}
