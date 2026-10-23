import { readFileSync } from "node:fs";
import { readFile } from "node:fs/promises";
import path from "node:path";
import { parse } from "smol-toml";

export const DEFAULT_CONFIG_FILE = "linguini.toml";
export const DEFAULT_PATHS = Object.freeze({ schema: "linguini/schema", locale: "linguini/locale" });

/** Resolve a configuration path without allowing it to escape the project. */
export function resolveProjectPath(root, value, field) {
  if (typeof value !== "string" || value.length === 0 || path.isAbsolute(value)) {
    throw new Error(`${field} must be a non-empty relative path`);
  }
  const resolved = path.resolve(root, value);
  const relative = path.relative(root, resolved);
  if (relative === ".." || relative.startsWith(`..${path.sep}`) || path.isAbsolute(relative)) {
    throw new Error(`${field} must stay inside the Linguini project root`);
  }
  return resolved;
}

function location(root, configFile) {
  const projectRoot = path.resolve(root);
  return { projectRoot, configPath: resolveProjectPath(projectRoot, configFile, "config file") };
}

function parseDocument(location, source) {
  let document = {};
  if (source !== undefined) {
    try {
      document = parse(source);
    } catch (error) {
      throw new Error(`cannot parse Linguini config ${location.configPath}: ${error.message}`, { cause: error });
    }
  }
  return Object.freeze({ ...location, configExists: source !== undefined, document });
}

function readFailure(location, error) {
  if (error?.code !== "ENOENT") {
    throw new Error(`cannot read Linguini config ${location.configPath}: ${error.message}`, { cause: error });
  }
}

export async function readProjectConfig(root, configFile = DEFAULT_CONFIG_FILE) {
  const selected = location(root, configFile);
  let source;
  try { source = await readFile(selected.configPath, "utf8"); }
  catch (error) { readFailure(selected, error); }
  return parseDocument(selected, source);
}

export function readProjectConfigSync(root, configFile = DEFAULT_CONFIG_FILE) {
  const selected = location(root, configFile);
  let source;
  try { source = readFileSync(selected.configPath, "utf8"); }
  catch (error) { readFailure(selected, error); }
  return parseDocument(selected, source);
}
