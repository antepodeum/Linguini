import { execFile } from "node:child_process";
import { createHash } from "node:crypto";
import { lstat, readdir, readFile } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";
import MagicString from "magic-string";
import { parse as parseToml } from "smol-toml";

const DEFAULT_CONFIG_FILE = "linguini.toml";
const DEFAULT_SCHEMA_DIR = "linguini/schema";
const DEFAULT_LOCALE_DIR = "linguini/locale";
const DEFAULT_GENERATED_DIR = "generated/linguini";
const DEFAULT_DEBOUNCE_MS = 40;
const MAX_DISCOVERY_DEPTH = 64;
const MAX_DISCOVERED_FILES = 100_000;
const MANIFEST_RELATIVE_PATH = "bundler/manifest.json";
const VIRTUAL_MESSAGE_PREFIX = "virtual:linguini/message/";
const RESOLVED_VIRTUAL_MESSAGE_PREFIX = `\0${VIRTUAL_MESSAGE_PREFIX}`;

export function linguini(options = {}) {
  let viteConfig;
  let projectRoot = process.cwd();
  let layout;
  let pendingBuild;
  let buildDirty = false;
  let buildReasons = new Set();
  let watchedFiles = new Set();
  let bundlerManifest;

  async function refreshLayout() {
    layout = await readProjectLayout(projectRoot, options.configFile);
    return layout;
  }

  async function reloadManifest() {
    const previous = bundlerManifest;
    const next = await readBundlerManifest(layout);
    bundlerManifest = next;
    return { previous, next };
  }

  async function drainBuilds() {
    const previous = bundlerManifest;
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
          await reloadManifest();
        } catch (error) {
          if (!buildDirty) {
            throw error;
          }
        }
      }
    } finally {
      pendingBuild = undefined;
    }
    return { previous, next: bundlerManifest };
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
    nextFiles.add(path.join(layout.generatedRoot, MANIFEST_RELATIVE_PATH));
    for (const application of bundlerManifest?.applicationFiles ?? []) {
      nextFiles.add(application);
    }
    for (const sourceRoot of layout.bundlerSourceRoots) {
      nextFiles.add(sourceRoot);
    }
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

  function invalidateBroad(server, previous, next, timestamp = Date.now()) {
    const generatedRoots = new Set([
      layout?.generatedRoot,
      previous?.generatedRoot,
      next?.generatedRoot
    ]);
    const applicationFiles = new Set([
      ...(previous?.applicationFiles ?? []),
      ...(next?.applicationFiles ?? [])
    ]);
    return invalidateMatchingModules(server, timestamp, (id) => {
      if (
        [...generatedRoots].some(
          (generatedRoot) =>
            generatedRoot && isGeneratedModule(id, generatedRoot, options)
        )
      ) {
        return true;
      }
      const file = normalizeGraphFileId(id);
      return file !== undefined && applicationFiles.has(file);
    });
  }

  function invalidateDelta(server, transition, changedFile, timestamp, forceBroad = false) {
    const { previous, next } = transition;
    if (
      forceBroad ||
      !previous ||
      !next ||
      manifestContractFingerprint(previous) !== manifestContractFingerprint(next)
    ) {
      return invalidateBroad(server, previous, next, timestamp);
    }
    const delta = manifestDelta(previous, next, changedFile);
    const virtualIds = new Set(
      [...delta.messages].map((message) => `\0${virtualMessageId(message)}`)
    );
    return invalidateMatchingModules(server, timestamp, (id) => {
      if (virtualIds.has(id)) {
        return true;
      }
      const file = normalizeGraphFileId(id);
      return (
        file !== undefined &&
        (delta.physicalFiles.has(file) || delta.applicationFiles.has(file))
      );
    });
  }

  function invalidateMatchingModules(server, timestamp, matches) {
    const invalidated = new Set();
    const moduleMap = server.moduleGraph.idToModuleMap;
    const seenModules = new Set();
    const modules = [...(moduleMap?.values?.() ?? [])]
      .filter((module) => module.id && matches(module.id))
      .sort((left, right) => comparePaths(left.id, right.id))
      .filter((module) => {
        if (seenModules.has(module)) {
          return false;
        }
        seenModules.add(module);
        return true;
      });
    for (const module of modules) {
      server.moduleGraph.invalidateModule(module, invalidated, timestamp, true);
      invalidated.add(module);
    }
    return modules;
  }

  async function rebuildForFile(file, server, reason, timestamp) {
    const absolute = path.resolve(file);
    const previousLayout = layout ?? (await refreshLayout());
    const manifestPath = path.join(previousLayout.generatedRoot, MANIFEST_RELATIVE_PATH);
    if (absolute === manifestPath) {
      const transition = await reloadManifest();
      await reconcileWatches(server);
      const modules = invalidateDelta(server, transition, undefined, timestamp);
      server.ws.send({
        type: "custom",
        event: "linguini:update",
        data: { file: absolute, reason: "manifest-change" }
      });
      return {
        kind: "manifest",
        modules,
        propagate: Boolean(transition.previous || transition.next)
      };
    }
    const wasWatched = watchedFiles.has(absolute);
    const wasApplication =
      bundlerManifest?.applicationsByFile.has(absolute) ||
      isBundlerApplicationSource(absolute, previousLayout);
    const relevant =
      wasWatched ||
      wasApplication ||
      isLinguiniSource(absolute, projectRoot, options.configFile, previousLayout);
    if (!relevant) {
      return undefined;
    }

    const configChanged = absolute === previousLayout.configPath;
    if (configChanged || reason !== "hot-update") {
      await refreshLayout();
    }
    const transition = await runBuild(reason);
    await reconcileWatches(server);
    const modules = invalidateDelta(
      server,
      transition,
      absolute,
      timestamp,
      configChanged
    );
    server.ws.send({
      type: "custom",
      event: "linguini:update",
      data: { file: absolute, reason }
    });
    return {
      kind: wasApplication ? "application" : "source",
      modules,
      propagate: Boolean(transition.previous || transition.next)
    };
  }

  function registerWatcher(server, event, reason) {
    server.watcher.on(event, async (file) => {
      try {
        const rebuilt = await rebuildForFile(file, server, reason, Date.now());
        if (rebuilt?.modules.length > 0) {
          await propagateWatcherModules(server, rebuilt.modules);
        }
        return rebuilt;
      } catch (error) {
        sendViteError(server, error, file);
        return false;
      }
    });
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
      } else {
        await reloadManifest();
      }
      const files = await discoverLinguiniFiles(projectRoot, options.configFile);
      files.push(path.join(layout.generatedRoot, MANIFEST_RELATIVE_PATH));
      files.push(...(bundlerManifest?.applicationFiles ?? []));
      files.push(...layout.bundlerSourceRoots);
      for (const file of files) {
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
        if (!rebuilt || rebuilt.kind === "application") {
          return undefined;
        }
        return rebuilt.propagate ? rebuilt.modules : [];
      } catch (error) {
        sendViteError(ctx.server, error, ctx.file);
        return [];
      }
    },
    async resolveId(id) {
      if (!id.startsWith(VIRTUAL_MESSAGE_PREFIX)) {
        return undefined;
      }
      decodeMessageId(id.slice(VIRTUAL_MESSAGE_PREFIX.length));
      return `\0${id}`;
    },
    load(id, options) {
      if (!id.startsWith(RESOLVED_VIRTUAL_MESSAGE_PREFIX)) {
        return undefined;
      }
      if (!bundlerManifest) {
        throw new Error("Linguini bundler manifest v2, v3, or v4 is required for virtual messages");
      }
      const message = decodeMessageId(id.slice(RESOLVED_VIRTUAL_MESSAGE_PREFIX.length));
      return renderVirtualMessageModule(
        bundlerManifest,
        message,
        shouldUseDynamicLocaleLoading(this, options)
      );
    },
    async transform(code, id) {
      if (!bundlerManifest || !isRawApplicationModule(id)) {
        return undefined;
      }
      const file = path.resolve(stripQueryAndHash(id));
      const application = bundlerManifest.applicationsByFile.get(file);
      if (!application) {
        return undefined;
      }
      return transformApplication.call(
        this,
        code,
        id,
        application,
        bundlerManifest,
        layout.generatedRoot
      );
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
  const rawBundler = document?.targets?.ts?.bundler;
  if (rawBundler !== undefined && !isRecord(rawBundler)) {
    throw new TypeError("Linguini config field targets.ts.bundler must be a table");
  }
  const bundlerSources = readStringArray(
    rawBundler?.sources,
    [],
    "targets.ts.bundler.sources"
  );
  if (rawBundler !== undefined && bundlerSources.length === 0) {
    throw new TypeError(
      "Linguini config field targets.ts.bundler.sources must be a non-empty string array"
    );
  }
  const bundlerExclude = readStringArray(
    rawBundler?.exclude,
    [],
    "targets.ts.bundler.exclude"
  );
  const localeLoading = validateLocaleLoading(
    rawBundler?.locale_loading,
    "targets.ts.bundler.locale_loading"
  );
  validateBundlerDynamic(rawBundler?.dynamic);

  return Object.freeze({
    projectRoot,
    configPath,
    configExists: source !== undefined,
    schemaRoot: resolveWithin(projectRoot, schema, "paths.schema"),
    localeRoot: resolveWithin(projectRoot, locale, "paths.locale"),
    generatedRoot: resolveWithin(projectRoot, generated, "targets.ts.out"),
    bundlerSourceRoots: Object.freeze(
      bundlerSources.map((value) =>
        resolveWithin(projectRoot, value, "targets.ts.bundler.sources")
      )
    ),
    bundlerExcludeRoots: Object.freeze([
      resolveWithin(projectRoot, generated, "targets.ts.out"),
      ...bundlerExclude.map((value) =>
        resolveWithin(projectRoot, value, "targets.ts.bundler.exclude")
      )
    ]),
    localeLoading
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

async function propagateWatcherModules(server, modules) {
  const reloadModule =
    (typeof server.reloadModule === "function" && server.reloadModule.bind(server)) ||
    (typeof server.environments?.client?.reloadModule === "function" &&
      server.environments.client.reloadModule.bind(server.environments.client));
  if (!reloadModule) {
    server.ws.send({ type: "full-reload", path: "*" });
    return;
  }
  const seen = new Set();
  const ordered = [...modules]
    .filter((module) => {
      if (seen.has(module)) {
        return false;
      }
      seen.add(module);
      return true;
    })
    .sort((left, right) => comparePaths(left.id ?? "", right.id ?? ""));
  for (const module of ordered) {
    await reloadModule(module);
  }
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

function readStringArray(value, fallback, field) {
  if (value === undefined) {
    return fallback;
  }
  if (!Array.isArray(value) || !value.every((item) => typeof item === "string" && item.length > 0)) {
    throw new TypeError(`Linguini config field ${field} must be an array of non-empty strings`);
  }
  return value;
}

function validateBundlerDynamic(value) {
  const field = "targets.ts.bundler.dynamic";
  if (value === undefined) {
    return;
  }
  if (!isRecord(value)) {
    throw new TypeError(`Linguini config field ${field} must be a table`);
  }
  const unknown = Object.keys(value).filter((key) => key !== "mode" && key !== "allow");
  if (unknown.length > 0) {
    throw new TypeError(`Linguini config field ${field}.${unknown[0]} is unknown`);
  }
  const mode = value.mode ?? "error";
  if (mode !== "error" && mode !== "bundle") {
    throw new TypeError(`Linguini config field ${field}.mode must be "error" or "bundle"`);
  }
  const allow = readStringArray(value.allow, [], `${field}.allow`);
  if (mode === "error" && allow.length > 0) {
    throw new TypeError(`Linguini config field ${field}.allow requires mode = "bundle"`);
  }
  if (mode === "bundle" && allow.length === 0) {
    throw new TypeError(`Linguini config field ${field}.allow must be non-empty in bundle mode`);
  }
  const seen = new Set();
  for (const message of allow) {
    if (!isDynamicMessagePath(message)) {
      throw new TypeError(`Linguini config field ${field}.allow has invalid path ${message}`);
    }
    const folded = message.toLowerCase();
    if (seen.has(folded)) {
      throw new TypeError(`Linguini config field ${field}.allow has duplicate path ${message}`);
    }
    seen.add(folded);
  }
}

function validateLocaleLoading(value, field) {
  if (value === undefined) {
    return "eager";
  }
  if (value !== "eager" && value !== "dynamic") {
    throw new TypeError(`Linguini config field ${field} must be "eager" or "dynamic"`);
  }
  return value;
}

function isDynamicMessagePath(message) {
  if (
    message.length === 0 ||
    message.trim() !== message ||
    /[\\/*?\[\]]/.test(message)
  ) {
    return false;
  }
  return message.split(".").every((segment) => {
    if (segment.length === 0) {
      return false;
    }
    return ![...segment].some((character) => {
      const code = character.codePointAt(0);
      return /\s/u.test(character) || code < 0x20 || (code >= 0x7f && code <= 0x9f);
    });
  });
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

async function readBundlerManifest(layout) {
  const manifestPath = path.join(layout.generatedRoot, MANIFEST_RELATIVE_PATH);
  let source;
  try {
    source = await readFile(manifestPath, "utf8");
  } catch (error) {
    if (error?.code === "ENOENT") {
      return undefined;
    }
    throw withContext(error, `cannot read Linguini bundler manifest ${manifestPath}`);
  }
  let raw;
  try {
    raw = JSON.parse(source);
  } catch (error) {
    throw withContext(error, `cannot parse Linguini bundler manifest ${manifestPath}`);
  }
  if (!isRecord(raw) || !Number.isInteger(raw.version)) {
    throw new Error(`Linguini bundler manifest ${manifestPath} has no integer version`);
  }
  if (raw.version === 1) {
    return undefined;
  }
  if (raw.version !== 2 && raw.version !== 3 && raw.version !== 4) {
    throw new Error(`unsupported Linguini bundler manifest version ${raw.version}`);
  }
  return validateManifest(raw, layout, manifestPath);
}

function validateManifest(raw, layout, manifestPath) {
  const context = `Linguini bundler manifest ${manifestPath}`;
  if (raw.version === 4 && raw.locale_loading === undefined) {
    throw new Error(
      `unsupported Linguini bundler manifest version 4: ${context}.locale_loading must be "eager" or "dynamic"`
    );
  }
  const localeLoading =
    raw.version === 4
      ? validateLocaleLoading(raw.locale_loading, `${context}.locale_loading`)
      : "eager";
  if (raw.version !== 4 && layout.localeLoading === "dynamic") {
    throw new Error(
      `${context}.locale_loading dynamic requires manifest version 4 (config requests dynamic loading)`
    );
  }
  if (layout.localeLoading !== localeLoading) {
    throw new Error(
      `${context}.locale_loading ${localeLoading} does not match config targets.ts.bundler.locale_loading ${layout.localeLoading}`
    );
  }
  const baseLocale = requireString(raw.base_locale, `${context}.base_locale`);
  const configuredLocales = requireStringArray(
    raw.configured_locales,
    `${context}.configured_locales`
  );
  const effectiveLocales = requireStringArray(
    raw.effective_locales,
    `${context}.effective_locales`
  );
  if (!effectiveLocales.includes(baseLocale)) {
    throw new Error(`${context}.effective_locales must contain base locale ${baseLocale}`);
  }
  if (new Set(configuredLocales).size !== configuredLocales.length || new Set(effectiveLocales).size !== effectiveLocales.length) {
    throw new Error(`${context} locale arrays must not contain duplicates`);
  }
  if (!Array.isArray(raw.sources)) {
    throw new Error(`${context}.sources must be an array`);
  }
  const sourceIds = new Set();
  const sourcePaths = new Set();
  const sourceIdsByFile = new Map();
  for (const [index, source] of raw.sources.entries()) {
    if (!isRecord(source) || !isNonnegativeInteger(source.id)) {
      throw new Error(`${context}.sources.${index} must contain an integer id`);
    }
    const sourcePath = validatePortableRelativePath(
      source.path,
      `${context}.sources.${index}.path`
    );
    if (sourceIds.has(source.id) || sourcePaths.has(sourcePath)) {
      throw new Error(`${context}.sources must have unique ids and paths`);
    }
    sourceIds.add(source.id);
    sourcePaths.add(sourcePath);
    sourceIdsByFile.set(
      resolveProjectPath(layout.projectRoot, sourcePath, `${context}.sources.${index}.path`),
      source.id
    );
  }
  if (!isRecord(raw.runtime_helpers)) {
    throw new Error(`${context}.runtime_helpers must be an object`);
  }
  const localeHelper = validateRuntimeHelper(
    raw.runtime_helpers.svelte_locale,
    "runtime_helpers.svelte_locale",
    layout.generatedRoot,
    context
  );
  const effectsHelper =
    raw.runtime_helpers.svelte_effects === undefined
      ? undefined
      : validateRuntimeHelper(
          raw.runtime_helpers.svelte_effects,
          "runtime_helpers.svelte_effects",
          layout.generatedRoot,
          context
        );
  if (!isRecord(raw.messages)) {
    throw new Error(`${context}.messages must be an object`);
  }
  const messages = new Map();
  for (const [canonical, rawMessage] of Object.entries(raw.messages)) {
    if (!canonical || !isRecord(rawMessage) || !isNonnegativeInteger(rawMessage.arity)) {
      throw new Error(`${context}.messages.${canonical} has invalid arity or shape`);
    }
    if (!isRecord(rawMessage.locales) || Object.keys(rawMessage.locales).length === 0) {
      throw new Error(`${context}.messages.${canonical}.locales must be non-empty`);
    }
    const locales = new Map();
    for (const [locale, rawLocale] of Object.entries(rawMessage.locales)) {
      if (!effectiveLocales.includes(locale) || !isRecord(rawLocale)) {
        throw new Error(`${context}.messages.${canonical}.locales.${locale} is invalid`);
      }
      const module = resolveGeneratedPath(
        layout.generatedRoot,
        rawLocale.module,
        `${context}.messages.${canonical}.locales.${locale}.module`
      );
      if (
        !Array.isArray(rawLocale.source_ids) ||
        !rawLocale.source_ids.every(
          (sourceId) => isNonnegativeInteger(sourceId) && sourceIds.has(sourceId)
        )
      ) {
        throw new Error(
          `${context}.messages.${canonical}.locales.${locale}.source_ids must be integers`
        );
      }
      locales.set(locale, { module, sourceIds: [...rawLocale.source_ids] });
    }
    if (!locales.has(baseLocale)) {
      throw new Error(`${context}.messages.${canonical} has no base-locale module`);
    }
    messages.set(canonical, { arity: rawMessage.arity, locales });
  }
  if (!isRecord(raw.applications)) {
    throw new Error(`${context}.applications must be an object`);
  }
  const applicationsByFile = new Map();
  const applicationSourceIds = new Set();
  for (const [relative, rawApplication] of Object.entries(raw.applications)) {
    const file = resolveProjectPath(layout.projectRoot, relative, `${context}.applications`);
    const application = validateApplication(
      rawApplication,
      relative,
      context,
      messages,
      raw.version
    );
    if (applicationSourceIds.has(rawApplication.source_id)) {
      throw new Error(`${context}.applications has duplicate source_id ${rawApplication.source_id}`);
    }
    applicationSourceIds.add(rawApplication.source_id);
    applicationsByFile.set(file, application);
  }
  return Object.freeze({
    version: raw.version,
    localeLoading,
    manifestPath,
    generatedRoot: layout.generatedRoot,
    baseLocale,
    configuredLocales,
    effectiveLocales,
    localeHelper,
    effectsHelper,
    sourceIdsByFile,
    messages,
    applicationsByFile,
    applicationFiles: Object.freeze([...applicationsByFile.keys()].sort(comparePaths))
  });
}

function validateRuntimeHelper(raw, field, generatedRoot, context) {
  if (!isRecord(raw)) {
    throw new Error(`${context}.${field} must be an object`);
  }
  const logicalImport = requireString(raw.import, `${context}.${field}.import`);
  const file = resolveGeneratedPath(generatedRoot, raw.file, `${context}.${field}.file`);
  return Object.freeze({ logicalImport, file });
}

function validateApplication(raw, relative, context, messages, manifestVersion) {
  const field = `${context}.applications.${relative}`;
  if (
    !isRecord(raw) ||
    !isNonnegativeInteger(raw.source_id) ||
    !isNonnegativeInteger(raw.byte_length) ||
    typeof raw.sha256 !== "string" ||
    !/^[0-9a-f]{64}$/.test(raw.sha256) ||
    !Array.isArray(raw.references) ||
    !Array.isArray(raw.imports) ||
    !Array.isArray(raw.unresolved) ||
    !Array.isArray(raw.analysis_dynamic_prefixes) ||
    !raw.analysis_dynamic_prefixes.every((value) => typeof value === "string")
  ) {
    throw new Error(`${field} has invalid structural fields`);
  }
  const imports = new Map();
  for (const item of raw.imports) {
    if (!isRecord(item)) {
      throw new Error(`${field}.imports entries must be objects`);
    }
    const bindingId = requireString(item.binding_id, `${field}.imports.binding_id`);
    if (imports.has(bindingId)) {
      throw new Error(`${field}.imports has duplicate binding ${bindingId}`);
    }
    if (typeof item.transformable !== "boolean") {
      throw new Error(`${field}.imports.${bindingId}.transformable must be boolean`);
    }
    const binding = {
      bindingId,
      moduleSpecifier: requireString(
        item.module_specifier,
        `${field}.imports.${bindingId}.module_specifier`
      ),
      imported: requireString(item.imported, `${field}.imports.${bindingId}.imported`),
      local: requireString(item.local, `${field}.imports.${bindingId}.local`),
      declarationStart: requireOffset(item.declaration_start, `${field}.imports.declaration_start`),
      declarationEnd: requireOffset(item.declaration_end, `${field}.imports.declaration_end`),
      itemStart: requireOffset(item.item_start, `${field}.imports.item_start`),
      itemEnd: requireOffset(item.item_end, `${field}.imports.item_end`),
      removalStart: requireOffset(item.removal_start, `${field}.imports.removal_start`),
      removalEnd: requireOffset(item.removal_end, `${field}.imports.removal_end`),
      moduleSpecifierStart: requireOffset(
        item.module_specifier_start,
        `${field}.imports.module_specifier_start`
      ),
      moduleSpecifierEnd: requireOffset(
        item.module_specifier_end,
        `${field}.imports.module_specifier_end`
      ),
      importedStart: requireOffset(item.imported_start, `${field}.imports.imported_start`),
      importedEnd: requireOffset(item.imported_end, `${field}.imports.imported_end`),
      localStart: requireOffset(item.local_start, `${field}.imports.local_start`),
      localEnd: requireOffset(item.local_end, `${field}.imports.local_end`),
      analyzerTrackedUsesOnly:
        manifestVersion >= 3
          ? requireBoolean(
              item.analyzer_tracked_uses_only,
              `${field}.imports.${bindingId}.analyzer_tracked_uses_only`
            )
          : undefined,
      transformable: item.transformable
    };
    if (
      manifestVersion >= 3 &&
      binding.transformable &&
      !binding.analyzerTrackedUsesOnly
    ) {
      throw new Error(
        `${field}.imports.${bindingId}.transformable requires analyzer_tracked_uses_only`
      );
    }
    validateNestedSpans(binding, raw.byte_length, field, manifestVersion >= 3);
    imports.set(bindingId, binding);
  }
  const references = raw.references.map((item, index) => {
    const itemField = `${field}.references.${index}`;
    if (!isRecord(item)) {
      throw new Error(`${itemField} must be an object`);
    }
    const message = requireString(item.message, `${itemField}.message`);
    const declared = messages.get(message);
    const bindingId =
      item.binding_id === null
        ? null
        : requireString(item.binding_id, `${itemField}.binding_id`);
    const provenance = item.provenance;
    if (
      !declared ||
      item.arity !== declared.arity ||
      (item.kind === "value" ? item.arity !== 0 : item.arity === 0) ||
      (item.kind !== "call" && item.kind !== "value") ||
      typeof item.local !== "string" ||
      !isRecord(provenance)
    ) {
      throw new Error(`${itemField} has invalid message, binding, arity, kind, or provenance`);
    }
    if (bindingId !== null) {
      const binding = imports.get(bindingId);
      if (
        !binding ||
        provenance.kind !== "imported" ||
        provenance.module_specifier !== binding.moduleSpecifier ||
        provenance.symbol !== binding.imported ||
        item.local !== binding.local
      ) {
        throw new Error(`${itemField} does not match imported binding ${bindingId}`);
      }
    } else {
      const validImplicit = provenance.kind === "implicit";
      const validFactory =
        provenance.kind === "factory" &&
        typeof provenance.factory === "string" &&
        provenance.factory.length > 0;
      if (!validImplicit && !validFactory) {
        throw new Error(`${itemField} has invalid unbound provenance`);
      }
    }
    const start = requireOffset(item.start, `${itemField}.start`);
    const end = requireOffset(item.end, `${itemField}.end`);
    if (start >= end || end > raw.byte_length) {
      throw new Error(`${itemField} span is outside application bytes`);
    }
    return { message, bindingId, kind: item.kind, start, end };
  });
  const ordered = [...references].sort((left, right) => left.start - right.start || left.end - right.end);
  let previousEnd = 0;
  for (const reference of ordered) {
    if (reference.start < previousEnd) {
      throw new Error(`${field}.references contains overlapping spans`);
    }
    previousEnd = reference.end;
  }
  const dynamicReferences =
    manifestVersion >= 3
      ? validateDynamicReferences(raw.dynamic_references, field, raw.byte_length, messages, imports)
      : Object.freeze([]);
  validateReferenceNonoverlap(references, dynamicReferences, field);
  if (manifestVersion >= 3) {
    validateImportRemovalNonoverlap(imports, field);
    validateReferencesOutsideImports(references, dynamicReferences, imports, field);
  }
  for (const [index, unresolved] of raw.unresolved.entries()) {
    if (
      !isRecord(unresolved) ||
      typeof unresolved.reason !== "string" ||
      unresolved.reason.length === 0 ||
      (unresolved.start !== undefined && !isNonnegativeInteger(unresolved.start)) ||
      (unresolved.end !== undefined && !isNonnegativeInteger(unresolved.end))
    ) {
      throw new Error(`${field}.unresolved.${index} has invalid shape`);
    }
  }
  return Object.freeze({
    relative,
    sha256: raw.sha256,
    byteLength: raw.byte_length,
    fingerprint: JSON.stringify({
      sha256: raw.sha256,
      byte_length: raw.byte_length,
      references: raw.references,
      dynamic_references: manifestVersion >= 3 ? raw.dynamic_references : undefined,
      imports: raw.imports
    }),
    imports,
    references: Object.freeze(references),
    dynamicReferences
  });
}

function validateDynamicReferences(raw, field, byteLength, messages, imports) {
  if (!Array.isArray(raw)) {
    throw new Error(`${field}.dynamic_references must be an array`);
  }
  const references = raw.map((item, index) => {
    const itemField = `${field}.dynamic_references.${index}`;
    if (!isRecord(item) || item.kind !== "computed") {
      throw new Error(`${itemField} must be a computed dynamic reference`);
    }
    const prefix = requireDynamicPrefix(item.prefix, `${itemField}.prefix`);
    const span = validateByteSpan(item.span, `${itemField}.span`, byteLength);
    const receiverSpan = validateByteSpan(
      item.receiver_span,
      `${itemField}.receiver_span`,
      byteLength
    );
    const keySpan = validateByteSpan(item.key_span, `${itemField}.key_span`, byteLength);
    if (
      receiverSpan.start !== span.start ||
      receiverSpan.end >= keySpan.start ||
      keySpan.end >= span.end ||
      receiverSpan.end > span.end ||
      keySpan.start < span.start
    ) {
      throw new Error(`${itemField} has invalid nested dynamic spans`);
    }
    if (item.reference_kind !== "value" && item.reference_kind !== "call") {
      throw new Error(`${itemField}.reference_kind must be value or call`);
    }
    const bindingId = requireString(item.binding_id, `${itemField}.binding_id`);
    const binding = imports.get(bindingId);
    const provenance = item.provenance;
    if (
      !binding ||
      !binding.transformable ||
      !binding.analyzerTrackedUsesOnly ||
      (binding.imported !== "l" && binding.imported !== "messages") ||
      typeof item.local !== "string" ||
      item.local !== binding.local ||
      !isRecord(provenance) ||
      provenance.kind !== "imported" ||
      provenance.module_specifier !== binding.moduleSpecifier ||
      provenance.symbol !== binding.imported
    ) {
      throw new Error(`${itemField} does not match a tracked transformable import binding`);
    }
    if (!Array.isArray(item.messages) || item.messages.length === 0) {
      throw new Error(`${itemField}.messages must be a non-empty array`);
    }
    const dynamicMessages = [];
    const keys = new Set();
    let previousMessage;
    for (const [messageIndex, rawMessage] of item.messages.entries()) {
      const messageField = `${itemField}.messages.${messageIndex}`;
      if (!isRecord(rawMessage)) {
        throw new Error(`${messageField} must be an object`);
      }
      const message = requireString(rawMessage.message, `${messageField}.message`);
      const key = requireString(rawMessage.key, `${messageField}.key`);
      const declared = messages.get(message);
      if (
        !declared ||
        rawMessage.arity !== declared.arity ||
        immediateMessageKey(prefix, message) !== key
      ) {
        throw new Error(`${messageField} has invalid message, key, or arity`);
      }
      if (
        previousMessage !== undefined &&
        Buffer.compare(Buffer.from(previousMessage, "utf8"), Buffer.from(message, "utf8")) >= 0
      ) {
        throw new Error(`${itemField}.messages must be sorted and unique`);
      }
      if (keys.has(key)) {
        throw new Error(`${itemField}.messages must have unique immediate keys`);
      }
      previousMessage = message;
      keys.add(key);
      dynamicMessages.push(Object.freeze({ message, key, arity: declared.arity }));
    }
    return Object.freeze({
      kind: "computed",
      prefix,
      span,
      receiverSpan,
      keySpan,
      referenceKind: item.reference_kind,
      bindingId,
      messages: Object.freeze(dynamicMessages)
    });
  });
  return Object.freeze(references);
}

function requireDynamicPrefix(value, field) {
  if (typeof value !== "string") {
    throw new Error(`${field} must be a string`);
  }
  if (value !== "" && !isDynamicMessagePath(value)) {
    throw new Error(`${field} must be a canonical message prefix`);
  }
  return value;
}

function validateByteSpan(raw, field, byteLength) {
  if (!isRecord(raw)) {
    throw new Error(`${field} must be an object`);
  }
  const start = requireOffset(raw.start, `${field}.start`);
  const end = requireOffset(raw.end, `${field}.end`);
  if (start >= end || end > byteLength) {
    throw new Error(`${field} is outside application bytes`);
  }
  return Object.freeze({ start, end });
}

function immediateMessageKey(prefix, message) {
  if (prefix === "") {
    return message.includes(".") ? undefined : message;
  }
  const marker = `${prefix}.`;
  if (!message.startsWith(marker)) {
    return undefined;
  }
  const key = message.slice(marker.length);
  return key.length > 0 && !key.includes(".") ? key : undefined;
}

function validateReferenceNonoverlap(staticReferences, dynamicReferences, field) {
  const spans = [
    ...staticReferences.map((reference) => ({
      start: reference.start,
      end: reference.end
    })),
    ...dynamicReferences.map((reference) => reference.span)
  ].sort((left, right) => left.start - right.start || left.end - right.end);
  for (let index = 1; index < spans.length; index += 1) {
    if (spans[index].start < spans[index - 1].end) {
      throw new Error(`${field} contains overlapping static or dynamic spans`);
    }
  }
}

function validateImportRemovalNonoverlap(imports, field) {
  const spans = [...imports.values()]
    .map((binding) => ({
      start: binding.removalStart,
      end: binding.removalEnd
    }))
    .sort((left, right) => left.start - right.start || left.end - right.end);
  for (let index = 1; index < spans.length; index += 1) {
    if (spans[index].start < spans[index - 1].end) {
      throw new Error(`${field}.imports contains overlapping removal spans`);
    }
  }
}

function validateReferencesOutsideImports(staticReferences, dynamicReferences, imports, field) {
  const references = [
    ...staticReferences.map((reference) => ({
      start: reference.start,
      end: reference.end
    })),
    ...dynamicReferences.map((reference) => reference.span)
  ];
  for (const reference of references) {
    for (const binding of imports.values()) {
      if (
        reference.start < binding.declarationEnd &&
        reference.end > binding.declarationStart
      ) {
        throw new Error(`${field} has a reference span inside an import declaration`);
      }
    }
  }
}

function manifestContractFingerprint(manifest) {
  return JSON.stringify({
    version: manifest.version,
    localeLoading: manifest.localeLoading,
    baseLocale: manifest.baseLocale,
    configuredLocales: manifest.configuredLocales,
    effectiveLocales: manifest.effectiveLocales,
    localeHelper: manifest.localeHelper,
    effectsHelper: manifest.effectsHelper
  });
}

function messageDescriptorFingerprint(message) {
  if (!message) {
    return undefined;
  }
  return JSON.stringify({
    arity: message.arity,
    locales: [...message.locales].map(([locale, entry]) => [
      locale,
      entry.module,
      entry.sourceIds
    ])
  });
}

function manifestDelta(previous, next, changedFile) {
  const changedSourceIds = new Set();
  if (changedFile) {
    const normalized = path.resolve(changedFile);
    const previousId = previous.sourceIdsByFile.get(normalized);
    const nextId = next.sourceIdsByFile.get(normalized);
    if (previousId !== undefined) {
      changedSourceIds.add(previousId);
    }
    if (nextId !== undefined) {
      changedSourceIds.add(nextId);
    }
  }
  const messages = new Set();
  const physicalFiles = new Set();
  const canonicals = new Set([...previous.messages.keys(), ...next.messages.keys()]);
  for (const canonical of [...canonicals].sort()) {
    const previousMessage = previous.messages.get(canonical);
    const nextMessage = next.messages.get(canonical);
    const messageAddedOrRemoved = !previousMessage || !nextMessage;
    const arityChanged =
      previousMessage &&
      nextMessage &&
      previousMessage.arity !== nextMessage.arity;
    const descriptorChanged =
      messageDescriptorFingerprint(previousMessage) !==
      messageDescriptorFingerprint(nextMessage);
    let sourceChanged = false;
    const locales = new Set([
      ...(previousMessage?.locales.keys() ?? []),
      ...(nextMessage?.locales.keys() ?? [])
    ]);
    for (const locale of [...locales].sort()) {
      const previousLocale = previousMessage?.locales.get(locale);
      const nextLocale = nextMessage?.locales.get(locale);
      const localeDescriptorChanged =
        !previousLocale ||
        !nextLocale ||
        previousLocale.module !== nextLocale.module ||
        !sameNumberArray(previousLocale.sourceIds, nextLocale.sourceIds);
      const localeSourceChanged = [previousLocale, nextLocale].some((entry) =>
        entry?.sourceIds.some((sourceId) => changedSourceIds.has(sourceId))
      );
      sourceChanged ||= localeSourceChanged;
      if (
        messageAddedOrRemoved ||
        arityChanged ||
        localeDescriptorChanged ||
        localeSourceChanged
      ) {
        if (previousLocale) {
          physicalFiles.add(previousLocale.module);
        }
        if (nextLocale) {
          physicalFiles.add(nextLocale.module);
        }
      }
    }
    if (!descriptorChanged && !sourceChanged) {
      continue;
    }
    messages.add(canonical);
  }
  const applicationFiles = new Set();
  const files = new Set([
    ...previous.applicationsByFile.keys(),
    ...next.applicationsByFile.keys()
  ]);
  for (const file of [...files].sort(comparePaths)) {
    if (
      previous.applicationsByFile.get(file)?.fingerprint !==
      next.applicationsByFile.get(file)?.fingerprint
    ) {
      applicationFiles.add(file);
    }
  }
  if (changedFile) {
    applicationFiles.delete(path.resolve(changedFile));
  }
  return { messages, physicalFiles, applicationFiles };
}

function sameNumberArray(left, right) {
  return left.length === right.length && left.every((value, index) => value === right[index]);
}

function validateNestedSpans(binding, byteLength, field, strict = false) {
  const spans = [
    [binding.declarationStart, binding.declarationEnd, "declaration"],
    [binding.itemStart, binding.itemEnd, "item"],
    [binding.removalStart, binding.removalEnd, "removal"],
    [binding.moduleSpecifierStart, binding.moduleSpecifierEnd, "module specifier"],
    [binding.importedStart, binding.importedEnd, "imported"],
    [binding.localStart, binding.localEnd, "local"]
  ];
  for (const [start, end, name] of spans) {
    if (start >= end || end > byteLength) {
      throw new Error(`${field} has invalid ${name} span`);
    }
  }
  if (
    binding.itemStart < binding.declarationStart ||
    binding.itemEnd > binding.declarationEnd ||
    binding.removalStart < binding.declarationStart ||
    binding.removalEnd > binding.declarationEnd ||
    binding.moduleSpecifierStart < binding.declarationStart ||
    binding.moduleSpecifierEnd > binding.declarationEnd ||
    binding.importedStart < binding.declarationStart ||
    binding.importedEnd > binding.declarationEnd ||
    binding.localStart < binding.declarationStart ||
    binding.localEnd > binding.declarationEnd
  ) {
    throw new Error(`${field} has import spans outside declaration`);
  }
  if (
    strict &&
    (binding.importedStart < binding.itemStart ||
      binding.importedEnd > binding.itemEnd ||
      binding.localStart < binding.itemStart ||
      binding.localEnd > binding.itemEnd ||
      binding.itemStart < binding.removalStart ||
      binding.itemEnd > binding.removalEnd)
  ) {
    throw new Error(`${field} has invalid nested import item spans`);
  }
}

async function transformApplication(code, id, application, manifest, generatedRoot) {
  if (manifest.version === 2) {
    return transformApplicationV2.call(
      this,
      code,
      id,
      application,
      manifest,
      generatedRoot
    );
  }
  return transformApplicationV3.call(
    this,
    code,
    id,
    application,
    manifest,
    generatedRoot
  );
}

async function transformApplicationV2(code, id, application, manifest, generatedRoot) {
  const bytes = Buffer.from(code, "utf8");
  const digest = createHash("sha256").update(bytes).digest("hex");
  if (bytes.length !== application.byteLength || digest !== application.sha256) {
    throw new Error(
      `Linguini application manifest is stale for ${stripQueryAndHash(id)}; rebuild generated output`
    );
  }
  const byteToUtf16 = createByteToUtf16Map(code, application.byteLength);
  for (const reference of application.references) {
    byteOffset(byteToUtf16, reference.start, application.relative);
    byteOffset(byteToUtf16, reference.end, application.relative);
  }
  for (const binding of application.imports.values()) {
    for (const offset of [
      binding.declarationStart,
      binding.declarationEnd,
      binding.itemStart,
      binding.itemEnd,
      binding.removalStart,
      binding.removalEnd,
      binding.moduleSpecifierStart,
      binding.moduleSpecifierEnd,
      binding.importedStart,
      binding.importedEnd,
      binding.localStart,
      binding.localEnd
    ]) {
      byteOffset(byteToUtf16, offset, application.relative);
    }
  }
  const generatedEntry = normalizeResolvedFileId(path.resolve(generatedRoot, "svelte.ts"));
  const acceptedBindings = new Map();
  for (const binding of application.imports.values()) {
    if (!binding.transformable || (binding.imported !== "l" && binding.imported !== "messages")) {
      continue;
    }
    let resolved;
    try {
      resolved = await this.resolve(binding.moduleSpecifier, stripQueryAndHash(id), {
        skipSelf: true
      });
    } catch {
      continue;
    }
    const resolvedId = typeof resolved === "string" ? resolved : resolved?.id;
    if (normalizeResolvedFileId(resolvedId) !== generatedEntry) {
      continue;
    }
    acceptedBindings.set(binding.bindingId, binding);
  }
  const references = application.references.filter(
    (reference) => reference.bindingId !== null && acceptedBindings.has(reference.bindingId)
  );
  if (references.length === 0) {
    return undefined;
  }
  const magic = new MagicString(code);
  const aliases = new Map();
  const bindingImports = new Map();
  const allocatedAliases = new Set();
  let nextAliasIndex = 0;
  for (const reference of references) {
    const aliasKey = `${reference.bindingId}\0${reference.message}`;
    let alias = aliases.get(aliasKey);
    if (!alias) {
      do {
        alias = `__linguini_message_${nextAliasIndex}`;
        nextAliasIndex += 1;
      } while (code.includes(alias) || allocatedAliases.has(alias));
      allocatedAliases.add(alias);
      aliases.set(aliasKey, alias);
      const imports = bindingImports.get(reference.bindingId) ?? [];
      imports.push({ message: reference.message, alias });
      bindingImports.set(reference.bindingId, imports);
    }
    const start = byteOffset(byteToUtf16, reference.start, application.relative);
    const end = byteOffset(byteToUtf16, reference.end, application.relative);
    magic.overwrite(start, end, reference.kind === "value" ? `${alias}()` : alias);
  }
  const activeBindings = [...bindingImports.keys()].map((bindingId) => acceptedBindings.get(bindingId));
  const deletions = importDeletions(code, activeBindings, byteToUtf16, application.relative);
  for (const [start, end] of deletions) {
    magic.remove(start, end);
  }
  const importsByDeclaration = new Map();
  for (const [bindingId, entries] of bindingImports) {
    const binding = acceptedBindings.get(bindingId);
    const group = importsByDeclaration.get(binding.declarationStart) ?? [];
    group.push(...entries);
    importsByDeclaration.set(binding.declarationStart, group);
  }
  for (const [insertionByte, entries] of importsByDeclaration) {
    const insertion = byteOffset(byteToUtf16, insertionByte, application.relative);
    const imports = entries
      .map(
        ({ message, alias }) =>
          `import { message as ${alias} } from ${JSON.stringify(virtualMessageId(message))};\n`
      )
      .join("");
    magic.prependLeft(insertion, imports);
  }
  return {
    code: magic.toString(),
    map: magic.generateMap({
      hires: true,
      source: stripQueryAndHash(id),
      includeContent: true
    })
  };
}

async function transformApplicationV3(code, id, application, manifest, generatedRoot) {
  const bytes = Buffer.from(code, "utf8");
  const digest = createHash("sha256").update(bytes).digest("hex");
  if (bytes.length !== application.byteLength || digest !== application.sha256) {
    throw new Error(
      `Linguini application manifest is stale for ${stripQueryAndHash(id)}; rebuild generated output`
    );
  }
  const byteToUtf16 = createByteToUtf16Map(code, application.byteLength);
  for (const reference of application.references) {
    byteOffset(byteToUtf16, reference.start, application.relative);
    byteOffset(byteToUtf16, reference.end, application.relative);
  }
  for (const reference of application.dynamicReferences) {
    const start = byteOffset(byteToUtf16, reference.span.start, application.relative);
    const end = byteOffset(byteToUtf16, reference.span.end, application.relative);
    const receiverEnd = byteOffset(
      byteToUtf16,
      reference.receiverSpan.end,
      application.relative
    );
    const keyStart = byteOffset(byteToUtf16, reference.keySpan.start, application.relative);
    const keyEnd = byteOffset(byteToUtf16, reference.keySpan.end, application.relative);
    if (code.slice(receiverEnd, keyStart) !== "[" || code.slice(keyEnd, end) !== "]") {
      throw new Error(
        `Linguini dynamic reference has invalid computed-key boundaries in ${application.relative}`
      );
    }
    if (start >= receiverEnd || keyStart >= keyEnd) {
      throw new Error(`Linguini dynamic reference has invalid spans in ${application.relative}`);
    }
  }
  for (const binding of application.imports.values()) {
    for (const offset of [
      binding.declarationStart,
      binding.declarationEnd,
      binding.itemStart,
      binding.itemEnd,
      binding.removalStart,
      binding.removalEnd,
      binding.moduleSpecifierStart,
      binding.moduleSpecifierEnd,
      binding.importedStart,
      binding.importedEnd,
      binding.localStart,
      binding.localEnd
    ]) {
      byteOffset(byteToUtf16, offset, application.relative);
    }
  }

  const dynamicBindingIds = new Set(
    application.dynamicReferences.map((reference) => reference.bindingId)
  );
  const referencedBindingIds = new Set([
    ...application.references.flatMap((reference) =>
      reference.bindingId === null ? [] : [reference.bindingId]
    ),
    ...dynamicBindingIds
  ]);
  const generatedEntry = normalizeResolvedFileId(path.resolve(generatedRoot, "svelte.ts"));
  const acceptedBindings = new Map();
  for (const bindingId of referencedBindingIds) {
    const binding = application.imports.get(bindingId);
    if (
      !binding?.transformable ||
      !binding.analyzerTrackedUsesOnly ||
      (binding.imported !== "l" && binding.imported !== "messages")
    ) {
      if (dynamicBindingIds.has(bindingId)) {
        throw new Error(
          `Linguini dynamic binding ${bindingId} is not tracked and transformable in ${application.relative}`
        );
      }
      continue;
    }
    let resolved;
    try {
      resolved = await this.resolve(binding.moduleSpecifier, stripQueryAndHash(id), {
        skipSelf: true
      });
    } catch (error) {
      if (dynamicBindingIds.has(bindingId)) {
        throw new Error(
          `Linguini dynamic binding ${bindingId} could not resolve ${binding.moduleSpecifier}`,
          { cause: error }
        );
      }
      continue;
    }
    const resolvedId = typeof resolved === "string" ? resolved : resolved?.id;
    if (normalizeResolvedFileId(resolvedId) !== generatedEntry) {
      if (dynamicBindingIds.has(bindingId)) {
        throw new Error(
          `Linguini dynamic binding ${bindingId} must resolve to generated svelte.ts`
        );
      }
      continue;
    }
    acceptedBindings.set(bindingId, binding);
  }

  const operations = [
    ...application.references
      .filter(
        (reference) =>
          reference.bindingId !== null && acceptedBindings.has(reference.bindingId)
      )
      .map((reference) => ({
        type: "static",
        start: reference.start,
        end: reference.end,
        bindingId: reference.bindingId,
        reference
      })),
    ...application.dynamicReferences
      .filter((reference) => acceptedBindings.has(reference.bindingId))
      .map((reference) => ({
        type: "dynamic",
        start: reference.span.start,
        end: reference.span.end,
        bindingId: reference.bindingId,
        reference
      }))
  ].sort((left, right) => left.start - right.start || left.end - right.end);
  if (operations.length === 0) {
    return undefined;
  }

  const magic = new MagicString(code);
  const scopes = new Map();
  const bindingScopes = new Map();
  for (const operation of operations) {
    let scopeKey = bindingScopes.get(operation.bindingId);
    const binding = acceptedBindings.get(operation.bindingId);
    if (!scopeKey) {
      const declarationStart = byteOffset(
        byteToUtf16,
        binding.declarationStart,
        application.relative
      );
      const scopeInfo = applicationScope(code, declarationStart, application.relative);
      scopeKey = scopeInfo.key;
      bindingScopes.set(operation.bindingId, scopeKey);
      if (!scopes.has(scopeKey)) {
        scopes.set(scopeKey, {
          insertionByte: scopeInfo.insertionByte,
          imports: new Map(),
          dispatches: new Map()
        });
      }
    }
  }

  const allocatedAliases = new Set();
  let nextMessageAlias = 0;
  let nextDispatchAlias = 0;
  const allocateAlias = (prefix, nextIndex) => {
    let alias;
    do {
      alias = `${prefix}${nextIndex()}`;
    } while (code.includes(alias) || allocatedAliases.has(alias));
    allocatedAliases.add(alias);
    return alias;
  };
  const allocateMessageAlias = () =>
    allocateAlias("__linguini_message_", () => nextMessageAlias++);
  const allocateDispatchAlias = () =>
    allocateAlias("__linguini_dispatch_", () => nextDispatchAlias++);
  const messageAlias = (scope, message) => {
    let alias = scope.imports.get(message);
    if (!alias) {
      alias = allocateMessageAlias();
      scope.imports.set(message, alias);
    }
    return alias;
  };

  const transformedCounts = new Map();
  for (const operation of operations) {
    const scope = scopes.get(bindingScopes.get(operation.bindingId));
    const start = byteOffset(byteToUtf16, operation.start, application.relative);
    const end = byteOffset(byteToUtf16, operation.end, application.relative);
    if (operation.type === "static") {
      const alias = messageAlias(scope, operation.reference.message);
      magic.overwrite(
        start,
        end,
        operation.reference.kind === "value" ? `${alias}()` : alias
      );
    } else {
      const signature = JSON.stringify(
        operation.reference.messages.map(({ message, key, arity }) => [message, key, arity])
      );
      const dispatchKey = JSON.stringify([
        operation.bindingId,
        operation.reference.prefix,
        signature
      ]);
      let dispatch = scope.dispatches.get(dispatchKey);
      if (!dispatch) {
        dispatch = {
          alias: allocateDispatchAlias(),
          entries: operation.reference.messages.map(({ message, key, arity }) => ({
            message,
            key,
            arity,
            alias: messageAlias(scope, message)
          }))
        };
        scope.dispatches.set(dispatchKey, dispatch);
      }
      const keyStart = byteOffset(
        byteToUtf16,
        operation.reference.keySpan.start,
        application.relative
      );
      const keyEnd = byteOffset(
        byteToUtf16,
        operation.reference.keySpan.end,
        application.relative
      );
      magic.overwrite(start, end, `${dispatch.alias}[${code.slice(keyStart, keyEnd)}]`);
    }
    transformedCounts.set(
      operation.bindingId,
      (transformedCounts.get(operation.bindingId) ?? 0) + 1
    );
  }

  const manifestUseCounts = new Map();
  for (const reference of [...application.references, ...application.dynamicReferences]) {
    if (reference.bindingId !== null) {
      manifestUseCounts.set(
        reference.bindingId,
        (manifestUseCounts.get(reference.bindingId) ?? 0) + 1
      );
    }
  }
  const activeBindings = [...acceptedBindings.values()].filter(
    (binding) =>
      (transformedCounts.get(binding.bindingId) ?? 0) > 0 &&
      transformedCounts.get(binding.bindingId) === manifestUseCounts.get(binding.bindingId)
  );
  const deletions = importDeletions(code, activeBindings, byteToUtf16, application.relative);
  for (const [start, end] of deletions) {
    magic.remove(start, end);
  }
  for (const scope of scopes.values()) {
    const insertion = byteOffset(byteToUtf16, scope.insertionByte, application.relative);
    const imports = [...scope.imports]
      .map(
        ([message, alias]) =>
          `import { message as ${alias} } from ${JSON.stringify(virtualMessageId(message))};\n`
      )
      .join("");
    const dispatches = [...scope.dispatches.values()]
      .map((dispatch) => renderDynamicDispatch(dispatch.alias, dispatch.entries))
      .join("");
    magic.prependLeft(insertion, `${imports}${dispatches}`);
  }
  return {
    code: magic.toString(),
    map: magic.generateMap({
      hires: true,
      source: stripQueryAndHash(id),
      includeContent: true
    })
  };
}

function renderDynamicDispatch(alias, entries) {
  const descriptors = entries
    .map(({ key, arity, alias: messageAlias }) => {
      const value =
        arity === 0
          ? `{ enumerable: true, get: () => ${messageAlias}() }`
          : `{ enumerable: true, value: ${messageAlias} }`;
      return `[${JSON.stringify(key)}]: ${value}`;
    })
    .join(", ");
  return `const ${alias} = Object.freeze(Object.defineProperties(Object.create(null), { ${descriptors} }));\n`;
}

function applicationScope(code, insertion, sourceName) {
  if (path.extname(sourceName).toLowerCase() !== ".svelte") {
    return { key: "module", insertionByte: 0 };
  }
  const openPattern = /<script\b[^>]*>/gi;
  let match;
  while ((match = openPattern.exec(code))) {
    const close = code.indexOf("</script>", openPattern.lastIndex);
    if (close === -1) {
      break;
    }
    if (insertion >= openPattern.lastIndex && insertion <= close) {
      return {
        key: `${match.index}:${close}`,
        insertionByte: Buffer.byteLength(code.slice(0, openPattern.lastIndex), "utf8")
      };
    }
    openPattern.lastIndex = close + "</script>".length;
  }
  throw new Error(`Linguini import binding is outside a Svelte script in ${sourceName}`);
}

function importDeletions(code, bindings, byteToUtf16, sourceName) {
  const byDeclaration = new Map();
  for (const binding of bindings) {
    const key = `${binding.declarationStart}:${binding.declarationEnd}`;
    const group = byDeclaration.get(key) ?? [];
    group.push(binding);
    byDeclaration.set(key, group);
  }
  const deletions = [];
  for (const group of byDeclaration.values()) {
    const declarationStart = byteOffset(byteToUtf16, group[0].declarationStart, sourceName);
    const declarationEnd = byteOffset(byteToUtf16, group[0].declarationEnd, sourceName);
    let remaining = code.slice(declarationStart, declarationEnd);
    const relativeRemovals = group
      .map((binding) => [
        byteOffset(byteToUtf16, binding.removalStart, sourceName) - declarationStart,
        byteOffset(byteToUtf16, binding.removalEnd, sourceName) - declarationStart
      ])
      .sort((left, right) => right[0] - left[0]);
    for (const [start, end] of relativeRemovals) {
      remaining = remaining.slice(0, start) + remaining.slice(end);
    }
    if (/^\s*import\s*\{\s*\}\s*from\s*["'][^"']+["']\s*;?\s*$/s.test(remaining)) {
      deletions.push([declarationStart, declarationEnd]);
    } else {
      for (const binding of group) {
        deletions.push([
          byteOffset(byteToUtf16, binding.removalStart, sourceName),
          byteOffset(byteToUtf16, binding.removalEnd, sourceName)
        ]);
      }
    }
  }
  deletions.sort((left, right) => left[0] - right[0]);
  for (let index = 1; index < deletions.length; index += 1) {
    if (deletions[index][0] < deletions[index - 1][1]) {
      throw new Error(`Linguini import removal spans overlap in ${sourceName}`);
    }
  }
  return deletions;
}

function shouldUseDynamicLocaleLoading(context, options) {
  const consumer = context?.environment?.config?.consumer;
  if (consumer !== undefined) {
    return consumer !== "server";
  }
  return options?.ssr !== true;
}

function renderVirtualMessageModule(manifest, canonical, dynamicClient = false) {
  const message = manifest.messages.get(canonical);
  if (!message) {
    throw new Error(`unknown Linguini message ${canonical}`);
  }
  if (dynamicClient && manifest.version === 4 && manifest.localeLoading === "dynamic") {
    return renderDynamicVirtualMessageModule(manifest, canonical, message);
  }
  const imports = [
    `import { getCurrentLocale } from ${JSON.stringify(toVitePath(manifest.localeHelper.file))};`
  ];
  if (manifest.effectsHelper) {
    imports.push(`import ${JSON.stringify(toVitePath(manifest.effectsHelper.file))};`);
  }
  const entries = [];
  let index = 0;
  for (const [locale, localeEntry] of message.locales) {
    const local = `__linguini_locale_${index}`;
    imports.push(`import { message as ${local} } from ${JSON.stringify(toVitePath(localeEntry.module))};`);
    entries.push(`${JSON.stringify(locale)}: ${local}`);
    index += 1;
  }
  return [
    ...imports,
    `const __linguini_messages = { ${entries.join(", ")} };`,
    `const __linguini_base = __linguini_messages[${JSON.stringify(manifest.baseLocale)}];`,
    "export function message(...args) {",
    "  const selected = __linguini_messages[getCurrentLocale()] ?? __linguini_base;",
    "  return selected(...args);",
    "}",
    ""
  ].join("\n");
}

function renderDynamicVirtualMessageModule(manifest, canonical, message) {
  const imports = [
    `import { getCurrentLocale, registerLocaleLoader } from ${JSON.stringify(
      toVitePath(manifest.localeHelper.file)
    )};`
  ];
  if (manifest.effectsHelper) {
    imports.push(`import ${JSON.stringify(toVitePath(manifest.effectsHelper.file))};`);
  }
  const loaders = [];
  for (const [locale, localeEntry] of message.locales) {
    loaders.push(
      `[${JSON.stringify(locale)}]: () => import(${JSON.stringify(
        toVitePath(localeEntry.module)
      )})`
    );
  }
  return [
    ...imports,
    `const __linguini_loaders = Object.freeze({ ${loaders.join(", ")} });`,
    `const __linguini_base_locale = ${JSON.stringify(manifest.baseLocale)};`,
    "const __linguini_functions = new Map();",
    "const __linguini_pending = new Map();",
    "function __linguini_load(locale) {",
    "  if (__linguini_functions.has(locale)) return Promise.resolve(__linguini_functions.get(locale));",
    "  const pending = __linguini_pending.get(locale);",
    "  if (pending) return pending;",
    "  const loader = Object.prototype.hasOwnProperty.call(__linguini_loaders, locale)",
    "    ? __linguini_loaders[locale]",
    "    : undefined;",
    "  if (!loader) return Promise.resolve(undefined);",
    "  const task = Promise.resolve()",
    "    .then(() => loader())",
    "    .then((module) => {",
    "      if (typeof module?.message !== \"function\") {",
    "        throw new Error(`Linguini locale module for ${locale} does not export message()`);",
    "      }",
    "      __linguini_functions.set(locale, module.message);",
    "      return module.message;",
    "    })",
    "    .finally(() => {",
    "      __linguini_pending.delete(locale);",
    "    });",
    "  __linguini_pending.set(locale, task);",
    "  return task;",
    "}",
    "const __linguini_dispose_loader = registerLocaleLoader(__linguini_load);",
    "const __linguini_initial_locale = getCurrentLocale();",
    "if (!(await __linguini_load(__linguini_initial_locale))) {",
    "  await __linguini_load(__linguini_base_locale);",
    "}",
    "if (import.meta.hot) {",
    "  import.meta.hot.dispose(() => __linguini_dispose_loader());",
    "}",
    "export function message(...args) {",
    "  const selectedLocale = getCurrentLocale();",
    "  const selected = __linguini_functions.get(selectedLocale) ?? __linguini_functions.get(__linguini_base_locale);",
    "  if (!selected) {",
    "    throw new Error(`Linguini locale ${selectedLocale} is not prepared`);",
    "  }",
    "  return selected(...args);",
    "}",
    ""
  ].join("\n");
}

function createByteToUtf16Map(source, byteLength) {
  const map = new Map([[0, 0]]);
  let byte = 0;
  let utf16 = 0;
  for (const scalar of source) {
    byte += Buffer.byteLength(scalar, "utf8");
    utf16 += scalar.length;
    map.set(byte, utf16);
  }
  if (byte !== byteLength) {
    throw new Error("Linguini application byte length changed during transform");
  }
  return map;
}

function byteOffset(map, offset, sourceName) {
  const converted = map.get(offset);
  if (converted === undefined) {
    throw new Error(`Linguini byte span does not end on a UTF-8 boundary in ${sourceName}`);
  }
  return converted;
}

function virtualMessageId(canonical) {
  return `${VIRTUAL_MESSAGE_PREFIX}${Buffer.from(canonical, "utf8").toString("hex")}`;
}

function decodeMessageId(encoded) {
  if (!/^(?:[0-9a-f]{2})+$/.test(encoded)) {
    throw new Error(`invalid Linguini virtual message id ${encoded}`);
  }
  const decoded = Buffer.from(encoded, "hex").toString("utf8");
  if (Buffer.from(decoded, "utf8").toString("hex") !== encoded) {
    throw new Error(`invalid UTF-8 Linguini virtual message id ${encoded}`);
  }
  return decoded;
}

function isRawApplicationModule(id) {
  if (id.includes("\0")) {
    return false;
  }
  const file = stripQueryAndHash(id);
  if (id !== file) {
    return false;
  }
  const extension = path.extname(file).toLowerCase();
  return extension === ".svelte" || JS_TS_EXTENSIONS.has(extension);
}

function isBundlerApplicationSource(file, projectLayout) {
  const extension = path.extname(file).toLowerCase();
  if (extension !== ".svelte" && !JS_TS_EXTENSIONS.has(extension)) {
    return false;
  }
  const generatedRoot = projectLayout.generatedRoot;
  if (isWithin(generatedRoot, file)) {
    return false;
  }
  if (
    projectLayout.bundlerExcludeRoots.some(
      (excluded) => pathRuleMatches(excluded, file)
    )
  ) {
    return false;
  }
  return projectLayout.bundlerSourceRoots.some((source) => {
    return pathRuleMatches(source, file);
  });
}

function pathRuleMatches(rule, file) {
  const extension = path.extname(rule).toLowerCase();
  if (extension === ".svelte" || JS_TS_EXTENSIONS.has(extension)) {
    return file === rule;
  }
  return file === rule || isWithin(rule, file);
}

const JS_TS_EXTENSIONS = new Set([
  ".js",
  ".mjs",
  ".cjs",
  ".jsx",
  ".ts",
  ".mts",
  ".cts",
  ".tsx"
]);

function stripQueryAndHash(id) {
  return id.split(/[?#]/, 1)[0];
}

function normalizeResolvedFileId(id) {
  if (typeof id !== "string" || id.length === 0 || id.includes("\0")) {
    return undefined;
  }
  if (id !== stripQueryAndHash(id)) {
    return undefined;
  }
  let file = id;
  if (file.startsWith("file:")) {
    try {
      file = fileURLToPath(file);
    } catch {
      return undefined;
    }
  }
  if (!path.isAbsolute(file)) {
    return undefined;
  }
  const normalized = path.normalize(file).replaceAll("\\", "/");
  return process.platform === "win32" ? normalized.toLowerCase() : normalized;
}

function normalizeGraphFileId(id) {
  if (typeof id !== "string" || id.length === 0 || id.includes("\0")) {
    return undefined;
  }
  const exact = stripQueryAndHash(id);
  return normalizeResolvedFileId(exact);
}

function resolveGeneratedPath(root, value, field) {
  const relative = validatePortableRelativePath(value, field);
  const resolved = path.resolve(root, ...relative.split("/"));
  if (!isWithin(root, resolved)) {
    throw new Error(`${field} must stay inside generated root`);
  }
  return resolved;
}

function resolveProjectPath(root, value, field) {
  const relative = validatePortableRelativePath(value, field);
  const resolved = path.resolve(root, ...relative.split("/"));
  if (!isWithin(root, resolved)) {
    throw new Error(`${field} path must stay inside project root`);
  }
  return resolved;
}

function validatePortableRelativePath(value, field) {
  if (
    typeof value !== "string" ||
    value.length === 0 ||
    value.includes("\\") ||
    value.startsWith("/") ||
    value.split("/").some((component) => !component || component === "." || component === "..")
  ) {
    throw new Error(`${field} must be a clean project-relative POSIX path`);
  }
  return value;
}

function requireString(value, field) {
  if (typeof value !== "string" || value.length === 0) {
    throw new Error(`${field} must be a non-empty string`);
  }
  return value;
}

function requireBoolean(value, field) {
  if (typeof value !== "boolean") {
    throw new Error(`${field} must be boolean`);
  }
  return value;
}

function requireStringArray(value, field) {
  if (!Array.isArray(value) || value.length === 0 || !value.every((item) => typeof item === "string" && item.length > 0)) {
    throw new Error(`${field} must be a non-empty string array`);
  }
  return [...value];
}

function requireOffset(value, field) {
  if (!isNonnegativeInteger(value)) {
    throw new Error(`${field} must be a nonnegative integer`);
  }
  return value;
}

function isRecord(value) {
  return value !== null && typeof value === "object" && !Array.isArray(value);
}

function isNonnegativeInteger(value) {
  return Number.isSafeInteger(value) && value >= 0;
}

function toVitePath(file) {
  const normalized = file.replaceAll(path.sep, "/");
  return /^[A-Za-z]:\//.test(normalized) ? `/@fs/${normalized}` : normalized;
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
