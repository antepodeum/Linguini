import { spawnSync } from "node:child_process";
import { existsSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const SCRIPT_ROOT = dirname(fileURLToPath(import.meta.url));
export const REPOSITORY_ROOT = resolve(SCRIPT_ROOT, "..");

export const PROJECTS = Object.freeze([
  Object.freeze({ id: "rust", path: ".", manager: "cargo", aliases: ["cargo"] }),
  Object.freeze({ id: "vite", path: "plugins/vite", manager: "pnpm", aliases: ["plugins/vite"] }),
  Object.freeze({ id: "cli", path: "packages/cli", manager: "pnpm", aliases: ["packages/cli"] }),
  Object.freeze({ id: "vscode", path: "editors/vscode", manager: "pnpm", aliases: ["editors/vscode"] }),
  Object.freeze({ id: "site", path: "site", manager: "pnpm", aliases: [] }),
]);

const projectById = new Map(PROJECTS.map((project) => [project.id, project]));
const projectByValue = new Map(
  PROJECTS.flatMap((project) => [project.id, ...project.aliases].map((value) => [value, project.id])),
);

const nodeTask = (task) => ({ ...task, tools: ["node", ...task.tools] });

/**
 * Ordered, intentionally explicit repository test registry.
 *
 * Keep this list static: adding a project should be a reviewed change to the
 * local test contract, rather than an accidental consequence of a directory
 * glob.
 */
export const TASKS = Object.freeze([
  nodeTask({
    id: "repo:versions",
    project: "rust",
    label: "Release version synchronization",
    command: "node",
    args: ["packages/scripts/check-versions.mjs"],
    profiles: ["quick", "full", "site"],
    tools: [],
    requiresNodeModules: false,
  }),
  nodeTask({
    id: "repo:contracts",
    project: "rust",
    label: "Public contract companion evidence",
    command: "node",
    args: ["scripts/check-public-contract.mjs"],
    profiles: ["quick", "full"],
    tools: [],
    requiresNodeModules: false,
  }),
  nodeTask({
    id: "repo:js-boundaries",
    project: "rust",
    label: "JavaScript package-manager boundaries",
    command: "node",
    args: ["scripts/check-js-boundaries.mjs"],
    profiles: ["quick", "full", "site"],
    tools: [],
    requiresNodeModules: false,
  }),
  Object.freeze({
    id: "rust:fmt",
    project: "rust",
    label: "Cargo format check",
    command: "cargo",
    args: ["fmt", "--all", "--check"],
    profiles: ["quick", "full"],
    tools: ["cargo", "rustfmt"],
    requiresNodeModules: false,
  }),
  Object.freeze({
    id: "rust:test",
    project: "rust",
    label: "Cargo workspace tests",
    command: "cargo",
    args: ["test", "--offline", "--locked", "--workspace"],
    profiles: ["full"],
    tools: ["cargo"],
    requiresNodeModules: false,
  }),
  Object.freeze({
    id: "rust:clippy",
    project: "rust",
    label: "Cargo workspace Clippy",
    command: "cargo",
    args: ["clippy", "--offline", "--locked", "--workspace", "--all-targets", "--", "-D", "warnings"],
    profiles: ["full"],
    tools: ["cargo", "clippy-driver"],
    requiresNodeModules: false,
  }),
  nodeTask({
    id: "docs:syntax",
    project: "rust",
    label: "Documentation Linguini syntax conformance",
    command: "node",
    args: ["scripts/docs-syntax.mjs"],
    profiles: ["full"],
    tools: ["cargo"],
    requiresNodeModules: false,
  }),
  Object.freeze({
    id: "docs:config",
    project: "rust",
    label: "Documentation configuration conformance",
    command: "cargo",
    args: ["test", "--offline", "--locked", "-p", "linguini-config", "--test", "documentation"],
    profiles: ["full"],
    tools: ["cargo"],
    requiresNodeModules: false,
  }),
  nodeTask({
    id: "docs:codeblocks",
    project: "rust",
    label: "Documentation code-block registry",
    command: "node",
    args: ["scripts/docs-codeblocks.mjs"],
    profiles: ["full"],
    tools: [],
    requiresNodeModules: false,
  }),
  nodeTask({
    id: "docs:examples",
    project: "site",
    label: "Documentation example compile/type/runtime conformance",
    command: "node",
    args: ["../scripts/docs-examples.mjs"],
    profiles: ["full"],
    tools: ["cargo"],
    requiresNodeModules: true,
  }),
  nodeTask({
    id: "docs:packaged",
    project: "site",
    label: "Documentation packaged-artifact conformance",
    command: "node",
    args: ["../scripts/docs-packaged.mjs"],
    profiles: ["full"],
    tools: ["cargo"],
    requiresNodeModules: true,
  }),
  nodeTask({
    id: "vite:test",
    project: "vite",
    label: "Vite plugin tests",
    command: "node",
    args: ["--test"],
    profiles: ["quick", "full"],
    tools: [],
    requiresNodeModules: true,
  }),
  nodeTask({
    id: "cli:test",
    project: "cli",
    label: "CLI package tests",
    command: "pnpm",
    args: ["test"],
    profiles: ["quick", "full"],
    tools: ["pnpm"],
    requiresNodeModules: false,
  }),
  nodeTask({
    id: "vscode:test",
    project: "vscode",
    label: "VS Code extension tests",
    command: "pnpm",
    args: ["test"],
    profiles: ["full"],
    tools: ["pnpm"],
    requiresNodeModules: true,
  }),
  nodeTask({
    id: "site:generate",
    project: "site",
    label: "Site Linguini generation",
    command: "pnpm",
    args: ["run", "linguini:build"],
    profiles: ["full", "site"],
    tools: ["pnpm"],
    requiresNodeModules: true,
  }),
  nodeTask({
    id: "site:test",
    project: "site",
    label: "Site tests",
    command: "pnpm",
    args: ["test"],
    profiles: ["full", "site"],
    tools: ["pnpm"],
    requiresNodeModules: true,
  }),
  nodeTask({
    id: "site:check",
    project: "site",
    label: "Site type and Svelte checks",
    command: "pnpm",
    args: ["run", "check"],
    profiles: ["full", "site"],
    tools: ["pnpm"],
    requiresNodeModules: true,
  }),
  nodeTask({
    id: "site:build",
    project: "site",
    label: "Site production build",
    command: "pnpm",
    args: ["run", "build:generated"],
    profiles: ["full", "site"],
    tools: ["pnpm"],
    requiresNodeModules: true,
  }),
  nodeTask({
    id: "site:graph",
    project: "site",
    label: "Site production graph assertions",
    command: "node",
    args: ["scripts/production-graph.test.mjs"],
    profiles: ["full", "site"],
    tools: [],
    requiresNodeModules: true,
  }),
].map((task) => Object.freeze(task)));

export const PROFILES = Object.freeze(["quick", "full", "site"]);
const taskById = new Map(TASKS.map((task) => [task.id, task]));

export class UsageError extends Error {
  constructor(message) {
    super(message);
    this.name = "UsageError";
  }
}

function valueForOption(argv, index, option, rawValue) {
  if (rawValue !== undefined) return [rawValue, index];
  const value = argv[index + 1];
  if (!value || value.startsWith("--")) {
    throw new UsageError(`${option} requires a value`);
  }
  return [value, index + 1];
}

export function parseArgs(argv) {
  let profile = "quick";
  const projects = [];
  const tasks = [];
  let help = false;

  for (let index = 0; index < argv.length; index += 1) {
    const argument = argv[index];
    if (argument === "--help" || argument === "-h") {
      help = true;
      continue;
    }
    const separator = argument.indexOf("=");
    const option = separator === -1 ? argument : argument.slice(0, separator);
    const inlineValue = separator === -1 ? undefined : argument.slice(separator + 1);
    if (!["--profile", "--project", "--task"].includes(option)) {
      throw new UsageError(`Unknown option or argument: ${argument}`);
    }

    const [value, nextIndex] = valueForOption(argv, index, option, inlineValue);
    index = nextIndex;
    if (option === "--profile") {
      if (!PROFILES.includes(value)) throw new UsageError(`Unknown profile: ${value}`);
      profile = value;
    } else if (option === "--project") {
      const project = projectByValue.get(value);
      if (!project) throw new UsageError(`Unknown project: ${value}`);
      if (!projects.includes(project)) projects.push(project);
    } else {
      if (!taskById.has(value)) throw new UsageError(`Unknown task: ${value}`);
      if (!tasks.includes(value)) tasks.push(value);
    }
  }

  return Object.freeze({ profile, projects: Object.freeze(projects), tasks: Object.freeze(tasks), help });
}

export function selectTasks({ profile = "quick", projects = [], tasks = [] } = {}) {
  if (!PROFILES.includes(profile)) throw new UsageError(`Unknown profile: ${profile}`);
  const projectIds = projects.map((value) => projectByValue.get(value) ?? value);
  for (const project of projectIds) {
    if (!projectById.has(project)) throw new UsageError(`Unknown project: ${project}`);
  }
  for (const task of tasks) {
    if (!taskById.has(task)) throw new UsageError(`Unknown task: ${task}`);
  }

  return TASKS.filter(
    (task) =>
      task.profiles.includes(profile) &&
      (projectIds.length === 0 || projectIds.includes(task.project)) &&
      (tasks.length === 0 || tasks.includes(task.id)),
  );
}

function defaultWhich(command) {
  const probe = process.platform === "win32" ? "where" : "which";
  return spawnSync(probe, [command], { stdio: "ignore" }).status === 0;
}

const REMEDIATION = Object.freeze({
  node: "Install Node.js 20 or newer, then rerun the selected profile.",
  cargo: "Install Rust with rustup and ensure cargo is on PATH, then rerun the selected profile.",
  rustfmt: "Run `rustup component add rustfmt`, then rerun the selected profile.",
  "clippy-driver": "Run `rustup component add clippy`, then rerun the selected profile.",
  pnpm: "Enable pnpm (for example, `corepack enable pnpm`), then rerun the selected profile.",
});

function installHint(project) {
  if (project.manager === "pnpm") return `cd ${project.path} && pnpm install --offline`;
  return "Install the workspace dependencies with the repository's documented tool.";
}

export function preflight(selectedTasks, { root = REPOSITORY_ROOT, which = defaultWhich, exists = existsSync } = {}) {
  const errors = [];
  const seenTools = new Set();
  const seenProjects = new Set();

  for (const task of selectedTasks) {
    const project = projectById.get(task.project);
    const cwd = resolve(root, project.path);
    if (!exists(cwd)) {
      errors.push({
        taskId: task.id,
        message: `Project directory is missing: ${project.path}`,
        remediation: `Restore ${project.path} or select a profile that does not include ${task.project}.`,
      });
    }
    for (const tool of task.tools) {
      if (seenTools.has(tool)) continue;
      seenTools.add(tool);
      if (!which(tool)) {
        errors.push({
          taskId: task.id,
          message: `Required tool is not available: ${tool}`,
          remediation: REMEDIATION[tool] ?? `Install ${tool} and ensure it is on PATH.`,
        });
      }
    }
    if (task.requiresNodeModules && !seenProjects.has(task.project)) {
      seenProjects.add(task.project);
      const nodeModules = join(cwd, "node_modules");
      if (!exists(nodeModules)) {
        errors.push({
          taskId: task.id,
          message: `Dependencies are not installed: ${nodeModules}`,
          remediation: `Run \`${installHint(project)}\` before running this profile.`,
        });
      }
    }
  }

  return Object.freeze({ ok: errors.length === 0, errors: Object.freeze(errors) });
}

function runCommand(task, { root }) {
  const project = projectById.get(task.project);
  const result = spawnSync(task.command, task.args, {
    cwd: resolve(root, project.path),
    stdio: "inherit",
    env: {
      ...process.env,
      CARGO_NET_OFFLINE: "true",
      npm_config_offline: "true",
      PNPM_CONFIG_OFFLINE: "true",
    },
  });
  return {
    status: result.status ?? 1,
    signal: result.signal,
    error: result.error,
  };
}

export function runTasks(selectedTasks, { root = REPOSITORY_ROOT, execute = runCommand, now = () => performance.now() } = {}) {
  const results = [];
  for (const task of selectedTasks) {
    const started = now();
    let commandResult;
    try {
      commandResult = execute(task, { root });
    } catch (error) {
      commandResult = { status: 1, error };
    }
    const durationMs = Math.max(0, Math.round(now() - started));
    const status = commandResult.status === 0 ? "passed" : "failed";
    results.push(Object.freeze({ task, status, durationMs, commandResult }));
  }
  return Object.freeze({
    results: Object.freeze(results),
    failed: results.filter((result) => result.status === "failed"),
  });
}

export const USAGE = `Usage: node scripts/test-all.mjs [options]

Options:
  --profile <quick|full|site>  Select the ordered task profile (default: quick)
  --project <name>             Repeat to limit to rust, vite, cli, vscode, or site
  --task <id>                  Repeat to run exact task IDs (for example site:check)
  --help                       Show this help

Examples:
  pnpm test:quick
  node scripts/test-all.mjs --profile full --project vite --project cli
  node scripts/test-all.mjs --profile site --task site:check --task site:build`;

function printPreflightFailure(preflightResult, io) {
  io.error("Preflight failed; no tasks were started:");
  for (const error of preflightResult.errors) {
    io.error(`  [${error.taskId}] ${error.message}`);
    io.error(`    Remediation: ${error.remediation}`);
  }
}

function printResults(summary, io) {
  for (const result of summary.results) {
    const suffix = result.commandResult.signal ? ` (${result.commandResult.signal})` : "";
    io.log(`[${result.status.toUpperCase()}] ${result.task.id} — ${result.durationMs}ms${suffix}`);
  }
  const failed = summary.failed.length;
  io.log(`\n${summary.results.length - failed} passed, ${failed} failed`);
}

export function main(argv = process.argv.slice(2), dependencies = {}) {
  const io = dependencies.io ?? console;
  try {
    const options = parseArgs(argv);
    if (options.help) {
      io.log(USAGE);
      return 0;
    }
    const selectedTasks = selectTasks(options);
    if (selectedTasks.length === 0) {
      throw new UsageError("The selected profile and filters do not contain any tasks");
    }
    const check = preflight(selectedTasks, dependencies);
    if (!check.ok) {
      printPreflightFailure(check, io);
      return 2;
    }
    const summary = runTasks(selectedTasks, dependencies);
    printResults(summary, io);
    return summary.failed.length === 0 ? 0 : 1;
  } catch (error) {
    if (!(error instanceof UsageError)) throw error;
    io.error(`${error.message}\n\n${USAGE}`);
    return 2;
  }
}

if (process.argv[1] && fileURLToPath(import.meta.url) === resolve(process.argv[1])) {
  process.exitCode = main();
}
