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
    bundlerManifest = await readBundlerManifest(layout);
    return bundlerManifest;
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

  function invalidateGeneratedModules(server, timestamp = Date.now()) {
    const generatedRoot = layout?.generatedRoot;
    const invalidated = new Set();
    for (const module of server.moduleGraph.idToModuleMap.values()) {
      if (
        !module.id ||
        (!isGeneratedModule(module.id, generatedRoot, options) &&
          !bundlerManifest?.applicationsByFile.has(
            path.resolve(stripQueryAndHash(module.id))
          ))
      ) {
        continue;
      }
      server.moduleGraph.invalidateModule(module, invalidated, timestamp, true);
    }
  }

  async function rebuildForFile(file, server, reason, timestamp) {
    const absolute = path.resolve(file);
    const previousLayout = layout ?? (await refreshLayout());
    const manifestPath = path.join(previousLayout.generatedRoot, MANIFEST_RELATIVE_PATH);
    if (absolute === manifestPath) {
      await reloadManifest();
      await reconcileWatches(server);
      invalidateGeneratedModules(server, timestamp);
      server.ws.send({
        type: "custom",
        event: "linguini:update",
        data: { file: absolute, reason: "manifest-change" }
      });
      return "manifest";
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
    return wasApplication ? "application" : "source";
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
        return rebuilt === "source" ? [] : undefined;
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
    load(id) {
      if (!id.startsWith(RESOLVED_VIRTUAL_MESSAGE_PREFIX)) {
        return undefined;
      }
      if (!bundlerManifest) {
        throw new Error("Linguini bundler manifest v2 is required for virtual messages");
      }
      const message = decodeMessageId(id.slice(RESOLVED_VIRTUAL_MESSAGE_PREFIX.length));
      return renderVirtualMessageModule(bundlerManifest, message);
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
    ])
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

function readStringArray(value, fallback, field) {
  if (value === undefined) {
    return fallback;
  }
  if (!Array.isArray(value) || !value.every((item) => typeof item === "string" && item.length > 0)) {
    throw new TypeError(`Linguini config field ${field} must be an array of non-empty strings`);
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
  if (raw.version !== 2) {
    throw new Error(`unsupported Linguini bundler manifest version ${raw.version}`);
  }
  return validateManifestV2(raw, layout, manifestPath);
}

function validateManifestV2(raw, layout, manifestPath) {
  const context = `Linguini bundler manifest ${manifestPath}`;
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
    const application = validateApplication(rawApplication, relative, context, messages);
    if (applicationSourceIds.has(rawApplication.source_id)) {
      throw new Error(`${context}.applications has duplicate source_id ${rawApplication.source_id}`);
    }
    applicationSourceIds.add(rawApplication.source_id);
    applicationsByFile.set(file, application);
  }
  return Object.freeze({
    version: 2,
    manifestPath,
    baseLocale,
    configuredLocales,
    effectiveLocales,
    localeHelper,
    effectsHelper,
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

function validateApplication(raw, relative, context, messages) {
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
      transformable: item.transformable
    };
    validateNestedSpans(binding, raw.byte_length, field);
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
    imports,
    references: Object.freeze(references)
  });
}

function validateNestedSpans(binding, byteLength, field) {
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
}

async function transformApplication(code, id, application, manifest, generatedRoot) {
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
  for (const reference of references) {
    const aliasKey = `${reference.bindingId}\0${reference.message}`;
    let alias = aliases.get(aliasKey);
    if (!alias) {
      alias = `__linguini_message_${aliases.size}`;
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

function renderVirtualMessageModule(manifest, canonical) {
  const message = manifest.messages.get(canonical);
  if (!message) {
    throw new Error(`unknown Linguini message ${canonical}`);
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
  const extension = path.extname(file).toLowerCase();
  if (extension === ".svelte") {
    return id === file;
  }
  return JS_TS_EXTENSIONS.has(extension);
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
  let file = stripQueryAndHash(id);
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
