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

export function standaloneJavaScriptTypeContract() {
  return `import { configureLinguini } from "./generated/linguini/index.js";

const l = configureLinguini({ language: () => "en" });

/**
 * @param {string} name
 * @returns {string}
 */
export function documentedGreeting(name) {
  return l.main.hello({ name });
}

/** @type {string} */
export const positional = l.main.hello("Artemy");
/** @type {string} */
export const named = l.main.field_required({ field: "Email" });

// @ts-expect-error required positional parameter is missing
l.main.hello();
// @ts-expect-error named call requires every declared property
l.main.checkout_total({ amount: 3 });
// @ts-expect-error named call rejects unknown object-literal properties
l.main.field_required({ field: "Email", extra: true });
// @ts-expect-error JSDoc parameter contract rejects numbers
documentedGreeting(42);
`;
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

function writeWebGuideExample(blocks, root, projectRoot) {
  const sourcePath = "docs/web-sveltekit.md";
  const expected = blocks.filter(
    (block) => block.sourcePath === sourcePath && ["html", "svelte", "ts"].includes(block.language),
  );
  const consumed = new Set();
  const select = (language, includes) => {
    const block = findUniqueBlock(blocks, { sourcePath, language, includes });
    consumed.add(`${block.sourcePath}:${block.line}`);
    return block;
  };
  const writeBlock = (relativePath, language, includes) => {
    const block = select(language, includes);
    const destination = join(projectRoot, relativePath);
    mkdirSync(dirname(destination), { recursive: true });
    writeFileSync(destination, block.source);
  };

  const baseConfig = findUniqueBlock(blocks, {
    sourcePath,
    language: "toml",
    includes: ['name = "app"', '[targets.ts]', 'framework = "sveltekit"'],
  });
  const webConfig = findUniqueBlock(blocks, {
    sourcePath,
    language: "toml",
    includes: ["[web.routing]", "[web.locale]", "[web.switch_route]"],
  });
  writeFileSync(join(projectRoot, "linguini.toml"), `${baseConfig.source}\n${webConfig.source}`);
  const schemas = {
    home: "title\nsubtitle\ngreeting(name: String)\n",
    nav: "pricing\nsettings\n",
  };
  const messages = {
    en: {
      home: "title = Home\nsubtitle = Welcome\ngreeting = Hello, {name}!\n",
      nav: "pricing = Pricing\nsettings = Settings\n",
    },
    ru: {
      home: "title = Главная\nsubtitle = Добро пожаловать\ngreeting = Привет, {name}!\n",
      nav: "pricing = Цены\nsettings = Настройки\n",
    },
    ar: {
      home: "title = الرئيسية\nsubtitle = أهلا وسهلا\ngreeting = مرحبا، {name}!\n",
      nav: "pricing = الأسعار\nsettings = الإعدادات\n",
    },
  };
  for (const [namespace, source] of Object.entries(schemas)) {
    const destination = join(projectRoot, "schema", `${namespace}.lgs`);
    mkdirSync(dirname(destination), { recursive: true });
    writeFileSync(destination, source);
  }
  for (const [locale, namespaces] of Object.entries(messages)) {
    for (const [namespace, source] of Object.entries(namespaces)) {
      const destination = join(projectRoot, "locales", namespace, `${locale}.lgl`);
      mkdirSync(dirname(destination), { recursive: true });
      writeFileSync(destination, source);
    }
  }

  writeFileSync(
    join(projectRoot, "package.json"),
    '{"private":true,"type":"module","dependencies":{"@antepod/linguini-vite":"file:placeholder"}}\n',
  );
  writeFileSync(
    join(projectRoot, "svelte.config.js"),
    'import { vitePreprocess } from "@sveltejs/vite-plugin-svelte";\n\nexport default { preprocess: vitePreprocess() };\n',
  );
  writeFileSync(
    join(projectRoot, "tsconfig.json"),
    '{"extends":"./.svelte-kit/tsconfig.json","compilerOptions":{"allowJs":true,"checkJs":true,"strict":true}}\n',
  );
  symlinkSync(resolve(root, "site/node_modules"), join(projectRoot, "node_modules"), "dir");

  writeBlock("src/routes/+layout.ts", "ts", ["trailingSlash", '"never"']);
  writeBlock("vite.config.ts", "ts", ["defineConfig", "buildOnStart", "sveltekit()"]);
  writeBlock("src/hooks.server.ts", "ts", ["sveltekit-control", "export { handle }"]);
  writeBlock("src/hooks.ts", "ts", ["sveltekit-control", "export { reroute }"]);
  writeBlock("src/routes/+layout.server.ts", "ts", ["sveltekit-control", "export { load }"]);
  writeBlock("src/app.html", "html", ["%linguini.lang%", "%sveltekit.body%"]);
  writeBlock("src/app.d.ts", "ts", ["interface Error", "export {}"]);
  writeBlock("src/routes/examples/basic/+page.svelte", "svelte", ["l.home.title"]);
  writeBlock("src/routes/examples/links/+page.svelte", "svelte", ["l.nav.pricing", "localizeHref"]);
  writeBlock("src/routes/examples/no-script/+page.svelte", "svelte", ["/_linguini/locale/en"]);
  writeBlock("src/routes/examples/switch/+page.svelte", "svelte", ["setLocale", "l.home.subtitle"]);
  writeBlock("src/lib/documented-client-helpers.ts", "ts", ["alternateLinks", "shouldLocalizeLink"]);
  writeBlock("src/routes/server-example/+page.server.ts", "ts", ["linguini.l.home.title"]);

  mkdirSync(join(projectRoot, "src/lib/server"), { recursive: true });
  writeFileSync(
    join(projectRoot, "src/lib/server/app-handle.ts"),
    'import type { Handle } from "@sveltejs/kit";\nexport const appHandle: Handle = ({ event, resolve }) => resolve(event);\n',
  );
  writeFileSync(
    join(projectRoot, "src/lib/app-reroute.ts"),
    'import type { Reroute } from "@sveltejs/kit";\nexport const appReroute: Reroute = () => undefined;\n',
  );
  writeFileSync(
    join(projectRoot, "src/lib/server/app-load.ts"),
    'import type { LayoutServerLoad } from "../../routes/$types";\nexport const appLoad: LayoutServerLoad = () => ({ app: true });\n',
  );

  return {
    assertComplete() {
      const missing = expected.filter((block) => !consumed.has(`${block.sourcePath}:${block.line}`));
      if (missing.length > 0 || consumed.size !== expected.length) {
        throw new Error(
          `${sourcePath}: executable fixture coverage mismatch; missing ${missing.map((block) => block.line).join(", ") || "none"}`,
        );
      }
    },
    writeCompositionVariants() {
      writeBlock("src/hooks.server.ts", "ts", ["sequence", "linguiniHandle", "appHandle"]);
      writeBlock("src/hooks.ts", "ts", ["Reroute", "linguiniReroute", "appReroute"]);
      writeBlock("src/routes/+layout.server.ts", "ts", ["LayoutServerLoad", "linguiniLoad", "appLoad"]);
    },
  };
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
      join(sourceRoot, "documented-jsdoc-contract.js"),
      standaloneJavaScriptTypeContract(),
    );
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
        include: [
          "src/documented-example.ts",
          "src/documented-example.js",
          "src/documented-jsdoc-contract.js",
        ],
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

    const webGuideRoot = join(projectRoot, "web-guide");
    mkdirSync(webGuideRoot, { recursive: true });
    const webGuide = writeWebGuideExample(blocks, root, webGuideRoot);
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
      { ...cargoOptions, cwd: webGuideRoot },
    );
    const checkWebGuide = () => {
      run(resolve(root, "site/node_modules/.bin/svelte-kit"), ["sync"], { cwd: webGuideRoot });
      run(
        resolve(root, "site/node_modules/.bin/svelte-check"),
        ["--workspace", webGuideRoot, "--tsconfig", "./tsconfig.json", "--threshold", "warning"],
        { cwd: webGuideRoot },
      );
    };
    checkWebGuide();
    webGuide.writeCompositionVariants();
    checkWebGuide();
    webGuide.assertComplete();
  } finally {
    if (server) await server.close();
    rmSync(projectRoot, { recursive: true, force: true });
  }

  console.log("Built and typechecked every documented SvelteKit variant; built, typechecked, and executed JavaScript/TypeScript.");
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
