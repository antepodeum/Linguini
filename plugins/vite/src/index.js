import { execFile } from "node:child_process";
import { readdir, readFile, stat } from "node:fs/promises";
import path from "node:path";

const DEFAULT_CONFIG_FILE = "linguini.toml";
const DEFAULT_SCHEMA_DIR = "linguini/schema";
const DEFAULT_LOCALE_DIR = "linguini/locale";
const DEFAULT_GENERATED_PATTERNS = [
  "/generated/linguini/",
  "\\generated\\linguini\\",
  "/linguini/generated/",
  "\\linguini\\generated\\"
];

export function linguini(options = {}) {
  let viteConfig;
  let projectRoot = process.cwd();
  let pendingBuild;

  async function runBuild(reason) {
    if (pendingBuild) {
      return pendingBuild;
    }
    pendingBuild = Promise.resolve()
      .then(() => buildProject(projectRoot, options, reason))
      .finally(() => {
        pendingBuild = undefined;
      });
    return pendingBuild;
  }

  async function watchSources(server) {
    const files = await discoverLinguiniFiles(projectRoot, options.configFile);
    for (const file of files) {
      server.watcher.add(file);
    }
  }

  function invalidateGeneratedModules(server) {
    const patterns = options.generatedModulePatterns ?? DEFAULT_GENERATED_PATTERNS;
    for (const module of server.moduleGraph.idToModuleMap.values()) {
      if (module.id && patterns.some((pattern) => module.id.includes(pattern))) {
        server.moduleGraph.invalidateModule(module);
      }
    }
  }

  return {
    name: "vite-plugin-linguini",
    enforce: "pre",
    configResolved(config) {
      viteConfig = config;
      projectRoot = path.resolve(options.root ?? config.root ?? process.cwd());
    },
    async buildStart() {
      if (!viteConfig) {
        projectRoot = path.resolve(options.root ?? process.cwd());
      }
      if (options.buildOnStart ?? true) {
        await runBuild("build-start");
      }
      for (const file of await discoverLinguiniFiles(projectRoot, options.configFile)) {
        this.addWatchFile(file);
      }
    },
    async configureServer(server) {
      await watchSources(server);
      server.watcher.on("add", async (file) => {
        if (isLinguiniSource(file, projectRoot, options.configFile)) {
          await watchSources(server);
        }
      });
    },
    async handleHotUpdate(ctx) {
      if (!isLinguiniSource(ctx.file, projectRoot, options.configFile)) {
        return;
      }
      await runBuild("hot-update");
      await watchSources(ctx.server);
      invalidateGeneratedModules(ctx.server);
      ctx.server.ws.send({
        type: "custom",
        event: "linguini:update",
        data: { file: ctx.file }
      });
      return [];
    }
  };
}

export default linguini;

export async function discoverLinguiniFiles(root, configFile = DEFAULT_CONFIG_FILE) {
  const configPath = path.resolve(root, configFile);
  const paths = await readProjectPaths(configPath);
  const files = [configPath];
  files.push(...(await collectFiles(path.resolve(root, paths.schema), ".lgs")));
  files.push(...(await collectFiles(path.resolve(root, paths.locale), ".lgl")));
  return files;
}

export function isLinguiniSource(file, root = process.cwd(), configFile = DEFAULT_CONFIG_FILE) {
  const absolute = path.resolve(file);
  const configPath = path.resolve(root, configFile);
  return absolute === configPath || absolute.endsWith(".lgs") || absolute.endsWith(".lgl");
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

async function readProjectPaths(configPath) {
  try {
    const source = await readFile(configPath, "utf8");
    return {
      schema: readTomlString(source, "paths", "schema") ?? DEFAULT_SCHEMA_DIR,
      locale: readTomlString(source, "paths", "locale") ?? DEFAULT_LOCALE_DIR
    };
  } catch (error) {
    if (error && error.code === "ENOENT") {
      return { schema: DEFAULT_SCHEMA_DIR, locale: DEFAULT_LOCALE_DIR };
    }
    throw error;
  }
}

function readTomlString(source, section, key) {
  const lines = source.split(/\r?\n/);
  let currentSection = "";
  for (const line of lines) {
    const trimmed = line.trim();
    const sectionMatch = /^\[([^\]]+)\]$/.exec(trimmed);
    if (sectionMatch) {
      currentSection = sectionMatch[1];
      continue;
    }
    if (currentSection !== section || trimmed.startsWith("#")) {
      continue;
    }
    const valueMatch = new RegExp(`^${key}\\s*=\\s*"([^"]+)"`).exec(trimmed);
    if (valueMatch) {
      return valueMatch[1];
    }
  }
  return undefined;
}

async function collectFiles(root, extension) {
  let metadata;
  try {
    metadata = await stat(root);
  } catch (error) {
    if (error && error.code === "ENOENT") {
      return [];
    }
    throw error;
  }
  if (!metadata.isDirectory()) {
    return [];
  }

  const entries = await readdir(root, { withFileTypes: true });
  const files = [];
  for (const entry of entries) {
    const entryPath = path.join(root, entry.name);
    if (entry.isDirectory()) {
      files.push(...(await collectFiles(entryPath, extension)));
    } else if (entry.isFile() && entry.name.endsWith(extension)) {
      files.push(entryPath);
    }
  }
  return files.sort();
}
