import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { PACKAGED_RUNTIME_CRATES } from "./docs-packaged.mjs";
import test from "node:test";

import {
  REPOSITORY_ROOT,
  PROJECTS,
  TASKS,
  main,
  parseArgs,
  preflight,
  runTasks,
  selectTasks,
} from "./test-all.mjs";
import { extractLinguiniFences } from "./docs-syntax.mjs";
import { extractCodeblocks } from "./docs-codeblocks.mjs";
import {
  executableDirectUseExample,
  findUniqueBlock,
  standaloneJavaScriptTypeContract,
} from "./docs-examples.mjs";
import { contractEvidence, validateContractEvidence } from "./check-public-contract.mjs";

test("public contract policy requires docs, migrations, and tests together", () => {
  const complete = [
    "crates/linguini-config/src/model.rs",
    "docs/reference.md",
    "docs/migrations.md",
    "crates/linguini-config/src/tests.rs",
  ];
  assert.equal(validateContractEvidence(complete).contracts.length, 1);
  assert.throws(
    () => validateContractEvidence(["packages/vite/src/index.js", "docs/web-sveltekit.md"]),
    /docs\/migrations\.md, tests or fixtures/,
  );
  assert.throws(() => validateContractEvidence(["packages/project/src/ecmascript.js"]), /public documentation, docs\/migrations\.md, tests or fixtures/);
  assert.deepEqual(contractEvidence(["crates/linguini-ir/src/lower.rs"]).contracts, []);
  assert.throws(() => validateContractEvidence(["crates/linguini-schema/src/highlight.rs"]), /public documentation, docs\/migrations\.md, tests or fixtures/);
  assert.throws(() => validateContractEvidence(["packages/tsc/src/index.js"]), /public documentation, docs\/migrations\.md, tests or fixtures/);
  for (const name of ["linguini-codegen", "linguini-codegen-ecmascript", "linguini-codegen-ecmascript-web", "linguini-codegen-svelte", "linguini-codegen-sveltekit", "linguini-codegen-vite"]) {
    assert.throws(
      () => validateContractEvidence([`crates/${name}/src/lib.rs`]),
      /public documentation, docs\/migrations\.md, tests or fixtures/,
    );
  }
});

test("documentation runtime fixture selection and transformation stay exact", () => {
  const blocks = [
    { sourcePath: "docs/example.md", language: "ts", source: "const unrelated = true;\n" },
    {
      sourcePath: "docs/example.md",
      language: "ts",
      source: 'const l = configureLinguini({ language: () => getRequestLocale() });\nl.main.hello("Artemy"); // output\nl.main.field_required({ field: "Email" }); // output\n',
    },
  ];
  const selected = findUniqueBlock(blocks, {
    sourcePath: "docs/example.md",
    language: "ts",
    includes: ["configureLinguini", "l.main.hello"],
  });
  const executable = executableDirectUseExample(selected.source);
  assert.match(executable, /language: \(\) => "en"/);
  assert.match(executable, /export const positional/);
  assert.match(executable, /export const named/);
});

test("standalone JavaScript contract covers JSDoc and negative overloads", () => {
  const contract = standaloneJavaScriptTypeContract();
  assert.match(contract, /@param \{string\} name/);
  assert.match(contract, /l\.main\.hello\(\{ name \}\)/);
  assert.equal(contract.match(/@ts-expect-error/g)?.length, 4);
  assert.match(contract, /checkout_total\(\{ amount: 3 \}\)/);
  assert.match(contract, /extra: true/);
});

test("documentation code-block registry covers typed, plain, and fragmented fences", () => {
  assert.equal(extractCodeblocks('```json\n{"compilerOptions":{"strict":true}}\n```\n', "docs/tsc.md")[0].language, "json");
  assert.throws(() => extractCodeblocks('```json\n{"strict":true,}\n```\n', "docs/tsc.md"), /docs\/tsc.md:1: invalid JSON/);
  const blocks = extractCodeblocks(
    "```ts\nconst value: string = \"ok\";\n```\n```\nplain output\n```\n```toml fragment=policy\n[key]\n```\n",
    "docs/example.md",
  );

  assert.deepEqual(
    blocks.map(({ language, line, fragment }) => ({ language, line, fragment })),
    [
      { language: "ts", line: 1, fragment: undefined },
      { language: "text", line: 4, fragment: undefined },
      { language: "toml", line: 7, fragment: "policy" },
    ],
  );
  assert.throws(
    () => extractCodeblocks("```typescript\nconst value = 1;\n```\n", "docs/bad.md"),
    /unsupported documentation fence language/,
  );
  assert.throws(
    () => extractCodeblocks("```ts executable\nconst value = 1;\n```\n", "docs/bad.md"),
    /unsupported documentation fence metadata/,
  );
});

test("documentation syntax fences distinguish standalone examples and registered fragments", () => {
  const fences = extractLinguiniFences(
    "```lgs\nmessage\n```\n```lgl fragment=expression\nfruit.form(count)\n```\n",
    "docs/example.md",
  );

  assert.deepEqual(
    fences.map(({ language, line, fragment }) => ({ language, line, fragment })),
    [
      { language: "lgs", line: 1, fragment: undefined },
      { language: "lgl", line: 4, fragment: "expression" },
    ],
  );
  assert.throws(
    () => extractLinguiniFences("```lgs ignore\nmessage\n```\n", "docs/bad.md"),
    /unsupported Linguini fence metadata/,
  );
});

test("registry is explicit and ordered across repository projects", () => {
  assert.deepEqual(
    TASKS.map((task) => task.id),
    [
      "repo:versions",
      "repo:contracts",
      "repo:js-boundaries",
      "project:test",
    "web:test",
    "svelte:test",
      "sveltekit:test",
      "rust:fmt",
      "codegen:schema-js",
      "codegen:runtime-js",
      "codegen:date-js",
      "codegen:date-runtime",
      "codegen:globals-js",
      "codegen:globals-runtime",
      "codegen:locale-js",
      "codegen:locale-runtime",
      "codegen:project-js",
      "codegen:project-runtime",
      "codegen:web-accept-language-js",
      "codegen:web-accept-language-runtime",
      "codegen:web-cookie-js",
      "codegen:web-cookie-runtime",
      "codegen:web-link-transform-js",
      "codegen:web-link-transform-runtime",
      "codegen:web-local-storage-js",
      "codegen:web-local-storage-runtime",
      "codegen:web-path-js",
      "codegen:web-path-runtime",
      "codegen:web-routes-js",
      "codegen:web-routes-runtime",
      "codegen:web-runtime-links-js",
      "codegen:web-runtime-links-runtime",
      "codegen:svelte-locale-js",
      "codegen:svelte-locale-runtime",
      "codegen:svelte-effects-js",
      "codegen:svelte-effects-runtime",
      "codegen:svelte-control-js",
      "codegen:svelte-control-runtime",
      "codegen:svelte-facade-js",
      "codegen:svelte-facade-runtime",
      "codegen:web-server-js",
      "codegen:web-server-runtime",
    "codegen:web-js",
    "codegen:web-runtime",
    "codegen:sveltekit-js",
    "codegen:sveltekit-runtime",
    "codegen:complete-project-js",
    "codegen:complete-project-ts",
    "codegen:complete-project-runtime",
      "codegen:highlight",
      "rust:test",
      "rust:clippy",
      "docs:syntax",
      "docs:config",
      "docs:codeblocks",
      "docs:examples",
      "docs:packaged",
      "vite:test",
      "tsc:test",
      "cli:test",
      "vscode:test",
      "site:generate",
      "site:test",
      "site:check",
      "site:build",
      "site:graph",
    ],
  );
  assert.deepEqual(PROJECTS.map((project) => project.id), ["rust", "project", "web", "svelte", "sveltekit", "tsc", "vite", "cli", "vscode", "site"]);
});

test("profiles select the expected ordered task groups", () => {
  assert.deepEqual(selectTasks({ profile: "quick" }).map((task) => task.id), [
    "repo:versions",
    "repo:contracts",
    "repo:js-boundaries",
    "project:test",
    "web:test",
    "svelte:test",
      "sveltekit:test",
    "rust:fmt",
    "codegen:schema-js",
    "codegen:runtime-js",
    "codegen:date-js",
    "codegen:date-runtime",
    "codegen:globals-js",
    "codegen:globals-runtime",
    "codegen:locale-js",
    "codegen:locale-runtime",
    "codegen:project-js",
    "codegen:project-runtime",
    "codegen:web-accept-language-js",
    "codegen:web-accept-language-runtime",
    "codegen:web-cookie-js",
    "codegen:web-cookie-runtime",
    "codegen:web-link-transform-js",
    "codegen:web-link-transform-runtime",
    "codegen:web-local-storage-js",
    "codegen:web-local-storage-runtime",
    "codegen:web-path-js",
    "codegen:web-path-runtime",
    "codegen:web-routes-js",
    "codegen:web-routes-runtime",
    "codegen:web-runtime-links-js",
    "codegen:web-runtime-links-runtime",
    "codegen:svelte-locale-js",
    "codegen:svelte-locale-runtime",
    "codegen:svelte-effects-js",
    "codegen:svelte-effects-runtime",
    "codegen:svelte-control-js",
    "codegen:svelte-control-runtime",
    "codegen:svelte-facade-js",
    "codegen:svelte-facade-runtime",
    "codegen:web-server-js",
    "codegen:web-server-runtime",
    "codegen:web-js",
    "codegen:web-runtime",
    "codegen:sveltekit-js",
    "codegen:sveltekit-runtime",
    "codegen:complete-project-js",
    "codegen:complete-project-ts",
    "codegen:complete-project-runtime",
    "vite:test",
    "tsc:test",
    "cli:test",
  ]);
  assert.deepEqual(selectTasks({ profile: "site" }).map((task) => task.id), [
    "repo:versions",
    "repo:js-boundaries",
    "site:generate",
    "site:test",
    "site:check",
    "site:build",
    "site:graph",
  ]);
  assert.equal(selectTasks({ profile: "full" }).length, TASKS.length);
});

test("repeated project and task filters are accepted and intersect", () => {
  const options = parseArgs([
    "--profile",
    "full",
    "--project=vite",
    "--project",
    "site",
    "--task",
    "vite:test",
    "--task=site:check",
  ]);
  assert.deepEqual(options.projects, ["vite", "site"]);
  assert.deepEqual(selectTasks({ projects: ["tsc"] }).map((task) => task.id), ["tsc:test"]);
  assert.deepEqual(options.tasks, ["vite:test", "site:check"]);
  assert.deepEqual(selectTasks(options).map((task) => task.id), ["vite:test", "site:check"]);
});

test("unknown profile, project, task, and options fail during parsing", () => {
  assert.throws(() => parseArgs(["--profile", "nightly"]), /Unknown profile/);
  assert.throws(() => parseArgs(["--project", "unknown"]), /Unknown project/);
  assert.throws(() => parseArgs(["--task", "site:nope"]), /Unknown task/);
  assert.throws(() => parseArgs(["--wat"]), /Unknown option/);
});

test("preflight reports missing tools and dependencies with remediation", () => {
  const task = TASKS.find((candidate) => candidate.id === "site:check");
  const check = preflight([task], {
    root: "/repo",
    which: () => false,
    exists: (path) => path === "/repo/site",
  });
  assert.equal(check.ok, false);
  assert.match(check.errors.map((error) => error.message).join("\n"), /pnpm/);
  assert.match(check.errors.map((error) => error.remediation).join("\n"), /pnpm install --offline/);
});

test("preflight deduplicates repeated tool and dependency failures", () => {
  const tasks = TASKS.filter((task) => task.project === "site");
  const check = preflight(tasks, {
    root: "/repo",
    which: () => false,
    exists: () => false,
  });
  assert.equal(check.errors.filter((error) => error.message.includes("pnpm")).length, 1);
  assert.equal(check.errors.filter((error) => error.message.includes("node_modules")).length, 1);
});

test("task execution continues after a failure and aggregates exit status", () => {
  const tasks = selectTasks({ profile: "quick" });
  const calls = [];
  let tick = 0;
  const summary = runTasks(tasks, {
    execute: (task) => {
      calls.push(task.id);
      return { status: task.id === "vite:test" ? 7 : 0 };
    },
    now: () => (tick += 12),
  });
  assert.deepEqual(calls, [
    "repo:versions",
    "repo:contracts",
    "repo:js-boundaries",
    "project:test",
    "web:test",
    "svelte:test",
      "sveltekit:test",
    "rust:fmt",
    "codegen:schema-js",
    "codegen:runtime-js",
    "codegen:date-js",
    "codegen:date-runtime",
    "codegen:globals-js",
    "codegen:globals-runtime",
    "codegen:locale-js",
    "codegen:locale-runtime",
    "codegen:project-js",
    "codegen:project-runtime",
    "codegen:web-accept-language-js",
    "codegen:web-accept-language-runtime",
    "codegen:web-cookie-js",
    "codegen:web-cookie-runtime",
    "codegen:web-link-transform-js",
    "codegen:web-link-transform-runtime",
    "codegen:web-local-storage-js",
    "codegen:web-local-storage-runtime",
    "codegen:web-path-js",
    "codegen:web-path-runtime",
    "codegen:web-routes-js",
    "codegen:web-routes-runtime",
    "codegen:web-runtime-links-js",
    "codegen:web-runtime-links-runtime",
    "codegen:svelte-locale-js",
    "codegen:svelte-locale-runtime",
    "codegen:svelte-effects-js",
    "codegen:svelte-effects-runtime",
    "codegen:svelte-control-js",
    "codegen:svelte-control-runtime",
    "codegen:svelte-facade-js",
    "codegen:svelte-facade-runtime",
    "codegen:web-server-js",
    "codegen:web-server-runtime",
    "codegen:web-js",
    "codegen:web-runtime",
    "codegen:sveltekit-js",
    "codegen:sveltekit-runtime",
    "codegen:complete-project-js",
    "codegen:complete-project-ts",
    "codegen:complete-project-runtime",
    "vite:test",
    "tsc:test",
    "cli:test",
  ]);
  assert.deepEqual(summary.failed.map((result) => result.task.id), ["vite:test"]);
  assert.deepEqual(
    summary.results.map((result) => result.durationMs),
    Array(tasks.length).fill(12),
  );
});

test("main returns nonzero for failures without calling real suites", () => {
  const output = [];
  const code = main(["--profile", "quick"], {
    io: { log: (line) => output.push(line), error: (line) => output.push(line) },
    which: () => true,
    exists: () => true,
    execute: (task) => ({ status: task.id === "cli:test" ? 1 : 0 }),
    now: () => 0,
  });
  assert.equal(code, 1);
  assert.match(output.join("\n"), /CLI:test|cli:test/);
  assert.match(output.join("\n"), /1 failed/);
});

test("message codegen and typed runtime adapters retain independent ownership", () => {
  const crate = name => readFileSync(resolve(REPOSITORY_ROOT, "crates", name, "Cargo.toml"), "utf8");
  const ecma=crate("linguini-codegen-ecmascript"), vite=crate("linguini-codegen-vite"), svelte=crate("linguini-codegen-svelte");
  for (const owner of [ecma,vite,svelte]) assert.doesNotMatch(owner,/linguini-codegen-(?:ecmascript-web|sveltekit)\s*=/);
  for (const owner of [ecma,vite]) assert.doesNotMatch(owner,/linguini-codegen-svelte\s*=/);
  for (const retired of ["linguini-codegen-ecmascript-web","linguini-codegen-sveltekit"]) assert(!PACKAGED_RUNTIME_CRATES.includes(retired));
  for (const name of ["web","svelte","sveltekit"]) {
    const manifest=JSON.parse(readFileSync(resolve(REPOSITORY_ROOT,"packages",name,"package.json"),"utf8"));
    assert.equal(manifest.name,"@antepodeum/linguini-"+name);
    assert.match(manifest.scripts.build,/tsc/);
    assert.ok(manifest.files.includes("dist"));
  }
});
