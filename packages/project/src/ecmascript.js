// ECMAScript build layout shared by CLI frontends, bundlers, and compiler adapters.
import { DEFAULT_PATHS, readProjectConfig, readProjectConfigSync, resolveProjectPath } from "./index.js";
import { readModuleConfiguration, readModuleConfigurationSync, moduleConfigPaths } from "./modules.js";
const resolveWithin = resolveProjectPath;
const DEFAULT_SCHEMA_DIR = DEFAULT_PATHS.schema;
const DEFAULT_LOCALE_DIR = DEFAULT_PATHS.locale;
const DEFAULT_GENERATED_DIR = "generated/linguini";

export async function readEcmascriptProject(root, configFile) {
  return ecmascriptLayout(await readProjectConfig(root, configFile), await readModuleConfiguration(root));
}

export function readEcmascriptProjectSync(root, configFile) {
  return ecmascriptLayout(readProjectConfigSync(root, configFile), readModuleConfigurationSync(root));
}

function ecmascriptLayout({ projectRoot, configPath, configExists, document }, moduleConfiguration) {
  const modulePlan = moduleConfiguration.plan;
  if (document?.web !== undefined || document?.targets?.ts?.framework !== undefined) {
    throw new Error("Configure web and framework modules in linguini.config.mjs; TOML web/framework policy was removed");
  }
  const web = modulePlan.modules.find(module => module.id === "ecmascript.web");
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
  const linkMode = validateLinkMode(
    web?.options?.links?.mode,
    web === undefined ? "manual" : "transform",
    "ecmascript.web.links.mode"
  );
  validateBundlerDynamic(rawBundler?.dynamic);

  return Object.freeze({
    projectRoot,
    configPath,
    configExists,
    modulePlan,
    moduleDependencies: moduleConfiguration.dependencies,
    moduleConfigExists: moduleConfiguration.configPath !== undefined,
    moduleConfigPaths: Object.freeze(moduleConfigPaths(projectRoot)),
    targetConfigured: document?.targets?.ts !== undefined,
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
    localeLoading,
    linkMode,
    bundlerEnabled: rawBundler !== undefined
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

export function validateLocaleLoading(value, field) {
  if (value === undefined) {
    return "eager";
  }
  if (value !== "eager" && value !== "dynamic") {
    throw new TypeError(`Linguini config field ${field} must be "eager" or "dynamic"`);
  }
  return value;
}

function validateLinkMode(value, fallback, field) {
  const mode = value ?? fallback;
  if (mode !== "transform" && mode !== "runtime" && mode !== "manual") {
    throw new TypeError(
      `Linguini config field ${field} must be "transform", "runtime", or "manual"`
    );
  }
  return mode;
}

export function isDynamicMessagePath(message) {
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


function isRecord(value) {
  return value !== null && typeof value === "object" && !Array.isArray(value);
}
