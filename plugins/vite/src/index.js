import { execFile } from "node:child_process";
import { lstat, readdir, readFile } from "node:fs/promises";
import path from "node:path";
import { parse as parseToml } from "smol-toml";

const DEFAULT_CONFIG_FILE = "linguini.toml";
const DEFAULT_SCHEMA_DIR = "linguini/schema";
const DEFAULT_LOCALE_DIR = "linguini/locale";
const DEFAULT_GENERATED_DIR = "generated/linguini";
const DEFAULT_DEBOUNCE_MS = 40;
const MAX_DISCOVERY_DEPTH = 64;
const MAX_DISCOVERED_FILES = 100_000;

export function linguini(options = {}) {
  let viteConfig;
  let projectRoot = process.cwd();
  let layout;
  let pendingBuild;
  let buildDirty = false;
  let buildReasons = new Set();
  let watchedFiles = new Set();

  async function refreshLayout() {
    layout = await readProjectLayout(projectRoot, options.configFile);
    return layout;
  }

  async function drainBuilds() {
    try {
      while (buildDirty) {
        const debounceMs = options.debounceMs ?? DEFAULT_DEBOUNCE_MS;
        if (debounceMs > 0) {
          await delay(debounceMs);
        }
        buildDirty = false;
        const reason = [...buildReasons].sort().join("+") || "source-change";
        buildReasons = new Set();
        try {
          await buildProject(projectRoot, options, reason);
        } catch (error) {
          if (!buildDirty) {
            throw error;
          }
        }
      }
    } finally {
      pendingBuild = undefined;
    }
  }

  function runBuild(reason) {
    buildDirty = true;
    buildReasons.add(reason);
    if (!pendingBuild) {
      pendingBuild = drainBuilds();
    }
    return pendingBuild;
  }

  async function reconcileWatches(server) {
    const nextFiles = new Set(await discoverLinguiniFiles(projectRoot, options.configFile));
    const removed = [...watchedFiles].filter((file) => !nextFiles.has(file));
    const added = [...nextFiles].filter((file) => !watchedFiles.has(file));
    if (removed.length > 0 && typeof server.watcher.unwatch === "function") {
      await server.watcher.unwatch(removed);
    }
    if (added.length > 0) {
      server.watcher.add(added);
    }
    watchedFiles = nextFiles;
  }

  function invalidateGeneratedModules(server, timestamp = Date.now()) {
    const generatedRoot = layout?.generatedRoot;
    const invalidated = new Set();
    for (const module of server.moduleGraph.idToModuleMap.values()) {
      if (!module.id || !isGeneratedModule(module.id, generatedRoot, options)) {
        continue;
      }
      server.moduleGraph.invalidateModule(module, invalidated, timestamp, true);
    }
  }

  async function rebuildForFile(file, server, reason, timestamp) {
    const absolute = path.resolve(file);
    const previousLayout = layout ?? (await refreshLayout());
    const wasWatched = watchedFiles.has(absolute);
    const relevant =
      wasWatched ||
      isLinguiniSource(absolute, projectRoot, options.configFile, previousLayout);
    if (!relevant) {
      return false;
    }

    if (absolute === previousLayout.configPath || reason !== "hot-update") {
      await refreshLayout();
    }
    await runBuild(reason);
    await reconcileWatches(server);
    invalidateGeneratedModules(server, timestamp);
    server.ws.send({
      type: "custom",
      event: "linguini:update",
      data: { file: absolute, reason }
    });
    return true;
  }

  function registerWatcher(server, event, reason) {
    server.watcher.on(event, (file) =>
      rebuildForFile(file, server, reason, Date.now()).catch((error) => {
        sendViteError(server, error, file);
        return false;
      })
    );
  }

  return {
    name: "vite-plugin-linguini",
    enforce: "pre",
    async configResolved(config) {
      viteConfig = config;
      projectRoot = path.resolve(options.root ?? config.root ?? process.cwd());
      await refreshLayout();
    },
    async buildStart() {
      if (!viteConfig) {
        projectRoot = path.resolve(options.root ?? process.cwd());
        await refreshLayout();
      }
      if (options.buildOnStart ?? true) {
        await runBuild("build-start");
      }
      for (const file of await discoverLinguiniFiles(projectRoot, options.configFile)) {
        this.addWatchFile(file);
      }
    },
    async configureServer(server) {
      await refreshLayout();
      await reconcileWatches(server);
      registerWatcher(server, "add", "source-added");
      registerWatcher(server, "unlink", "source-removed");
      registerWatcher(server, "unlinkDir", "source-directory-removed");
    },
    async handleHotUpdate(ctx) {
      try {
        const rebuilt = await rebuildForFile(
          ctx.file,
          ctx.server,
          "hot-update",
          ctx.timestamp
        );
        return rebuilt ? [] : undefined;
      } catch (error) {
        sendViteError(ctx.server, error, ctx.file);
        return [];
      }
    }
  };
}

export default linguini;

export async function discoverLinguiniFiles(root, configFile = DEFAULT_CONFIG_FILE) {
  const layout = await readProjectLayout(root, configFile);
  const files = [];
  if (layout.configExists) {
    files.push(layout.configPath);
  }
  files.push(...(await collectFiles(layout.schemaRoot, ".lgs")));
  files.push(...(await collectFiles(layout.localeRoot, ".lgl")));
  return files.sort(comparePaths);
}

export async function readProjectLayout(root, configFile = DEFAULT_CONFIG_FILE) {
  const projectRoot = path.resolve(root);
  const configPath = resolveWithin(projectRoot, configFile, "config file");
  let source;
  try {
    source = await readFile(configPath, "utf8");
  } catch (error) {
    if (error?.code !== "ENOENT") {
      throw withContext(error, `cannot read Linguini config ${configPath}`);
    }
  }

  let document = {};
  if (source !== undefined) {
    try {
      document = parseToml(source);
    } catch (error) {
      throw withContext(error, `cannot parse Linguini config ${configPath}`);
    }
  }

  const schema = readString(document?.paths?.schema, DEFAULT_SCHEMA_DIR, "paths.schema");
  const locale = readString(document?.paths?.locale, DEFAULT_LOCALE_DIR, "paths.locale");
  const generated = readString(
    document?.targets?.ts?.out,
    DEFAULT_GENERATED_DIR,
    "targets.ts.out"
  );

  return Object.freeze({
    projectRoot,
    configPath,
    configExists: source !== undefined,
    schemaRoot: resolveWithin(projectRoot, schema, "paths.schema"),
    localeRoot: resolveWithin(projectRoot, locale, "paths.locale"),
    generatedRoot: resolveWithin(projectRoot, generated, "targets.ts.out")
  });
}

export function isLinguiniSource(
  file,
  root = process.cwd(),
  configFile = DEFAULT_CONFIG_FILE,
  projectLayout
) {
  const projectRoot = path.resolve(root);
  const absolute = path.resolve(file);
  const configPath =
    projectLayout?.configPath ?? resolveWithin(projectRoot, configFile, "config file");
  const schemaRoot =
    projectLayout?.schemaRoot ?? path.resolve(projectRoot, DEFAULT_SCHEMA_DIR);
  const localeRoot =
    projectLayout?.localeRoot ?? path.resolve(projectRoot, DEFAULT_LOCALE_DIR);

  return (
    absolute === configPath ||
    (absolute.endsWith(".lgs") && isWithin(schemaRoot, absolute)) ||
    (absolute.endsWith(".lgl") && isWithin(localeRoot, absolute))
  );
}

async function buildProject(root, options, reason) {
  if (options.build) {
    await options.build({ root, reason });
    return;
  }
  const command = options.command ?? "linguini";
  const args = options.args ?? ["build"];
  await execFilePromise(command, args, { cwd: root });
}

function execFilePromise(command, args, options) {
  return new Promise((resolve, reject) => {
    execFile(command, args, options, (error, stdout, stderr) => {
      if (error) {
        error.message = [error.message, stderr.trim(), stdout.trim()].filter(Boolean).join("\n");
        reject(error);
      } else {
        resolve({ stdout, stderr });
      }
    });
  });
}

async function collectFiles(root, extension) {
  const files = [];
  const pending = [{ directory: root, depth: 0 }];
  while (pending.length > 0) {
    const { directory, depth } = pending.pop();
    if (depth > MAX_DISCOVERY_DEPTH) {
      throw new Error(`Linguini source discovery exceeded depth ${MAX_DISCOVERY_DEPTH}`);
    }
    let entries;
    try {
      const metadata = await lstat(directory);
      if (!metadata.isDirectory() || metadata.isSymbolicLink()) {
        continue;
      }
      entries = await readdir(directory, { withFileTypes: true });
    } catch (error) {
      if (error?.code === "ENOENT") {
        continue;
      }
      throw withContext(error, `cannot discover Linguini sources below ${directory}`);
    }

    entries.sort((left, right) => comparePaths(left.name, right.name));
    for (const entry of entries) {
      const entryPath = path.join(directory, entry.name);
      if (entry.isSymbolicLink()) {
        continue;
      }
      if (entry.isDirectory()) {
        pending.push({ directory: entryPath, depth: depth + 1 });
      } else if (entry.isFile() && entry.name.endsWith(extension)) {
        files.push(entryPath);
        if (files.length > MAX_DISCOVERED_FILES) {
          throw new Error(
            `Linguini source discovery exceeded ${MAX_DISCOVERED_FILES} files`
          );
        }
      }
    }
  }
  return files;
}

function isGeneratedModule(id, generatedRoot, options) {
  if (id.startsWith("virtual:linguini/") || id.startsWith("\0virtual:linguini/")) {
    return true;
  }
  const patterns = options.generatedModulePatterns;
  if (patterns?.some((pattern) => id.includes(pattern))) {
    return true;
  }
  if (!generatedRoot || id.includes("\0")) {
    return false;
  }
  const file = id.split("?", 1)[0];
  return path.isAbsolute(file) && isWithin(generatedRoot, path.resolve(file));
}

function sendViteError(server, error, file) {
  const message = error instanceof Error ? error.message : String(error);
  const stack = error instanceof Error ? error.stack : undefined;
  server.ws.send({
    type: "error",
    err: {
      message: file ? `${message}\nLinguini source: ${file}` : message,
      stack,
      plugin: "vite-plugin-linguini",
      id: file
    }
  });
}

function readString(value, fallback, field) {
  if (value === undefined) {
    return fallback;
  }
  if (typeof value !== "string" || value.length === 0) {
    throw new TypeError(`Linguini config field ${field} must be a non-empty string`);
  }
  return value;
}

function resolveWithin(root, value, field) {
  if (typeof value !== "string" || value.length === 0 || path.isAbsolute(value)) {
    throw new Error(`${field} must be a non-empty relative path`);
  }
  const resolved = path.resolve(root, value);
  if (!isWithin(root, resolved)) {
    throw new Error(`${field} must stay inside the Linguini project root`);
  }
  return resolved;
}

function isWithin(root, candidate) {
  const relative = path.relative(root, candidate);
  return relative === "" || (!relative.startsWith(`..${path.sep}`) && relative !== ".." && !path.isAbsolute(relative));
}

function comparePaths(left, right) {
  return left.replaceAll("\\", "/").localeCompare(right.replaceAll("\\", "/"), "en");
}

function withContext(error, context) {
  const wrapped = new Error(`${context}: ${error instanceof Error ? error.message : error}`);
  if (error instanceof Error) {
    wrapped.cause = error;
  }
  return wrapped;
}

function delay(milliseconds) {
  return new Promise((resolve) => setTimeout(resolve, milliseconds));
}
