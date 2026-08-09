import assert from "node:assert/strict";
import test from "node:test";

import {
  PROJECTS,
  TASKS,
  main,
  parseArgs,
  preflight,
  runTasks,
  selectTasks,
} from "./test-all.mjs";

test("registry is explicit and ordered across repository projects", () => {
  assert.deepEqual(
    TASKS.map((task) => task.id),
    [
      "rust:fmt",
      "rust:test",
      "rust:clippy",
      "vite:test",
      "cli:test",
      "vscode:test",
      "site:generate",
      "site:test",
      "site:check",
      "site:build",
    ],
  );
  assert.deepEqual(PROJECTS.map((project) => project.id), ["rust", "vite", "cli", "vscode", "site"]);
});

test("profiles select the expected ordered task groups", () => {
  assert.deepEqual(selectTasks({ profile: "quick" }).map((task) => task.id), ["rust:fmt", "vite:test", "cli:test"]);
  assert.deepEqual(selectTasks({ profile: "site" }).map((task) => task.id), ["site:generate", "site:test", "site:check", "site:build"]);
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
  assert.deepEqual(calls, ["rust:fmt", "vite:test", "cli:test"]);
  assert.deepEqual(summary.failed.map((result) => result.task.id), ["vite:test"]);
  assert.deepEqual(summary.results.map((result) => result.durationMs), [12, 12, 12]);
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
