import { spawn, spawnSync } from "node:child_process";
import { existsSync, mkdtempSync, readFileSync, realpathSync, rmSync, statSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

export const MODULE_CONFIG_FILE = "linguini.config.mjs";
export const MODULE_PLAN_FILE = "linguini.modules.json";
const MAX_PLAN_BYTES = 4 * 1024 * 1024;
const worker = fileURLToPath(new URL("./module-worker.js", import.meta.url));

function record(value) {
  return value !== null && typeof value === "object" && !Array.isArray(value);
}

function cloneData(value, ancestors = new Set()) {
  if (value === null || typeof value === "string" || typeof value === "boolean") return value;
  if (typeof value === "number" && Number.isFinite(value)) return value;
  if (!record(value) && !Array.isArray(value)) throw new TypeError("Module options must contain JSON data only");
  if (ancestors.has(value)) throw new TypeError("Module options must not contain cycles");
  const prototype = Object.getPrototypeOf(value);
  if (!Array.isArray(value) && prototype !== Object.prototype && prototype !== null) {
    throw new TypeError("Module options must use plain objects");
  }
  if (Reflect.ownKeys(value).some(key => typeof key !== "string")) throw new TypeError("Module options must not contain symbol keys");
  ancestors.add(value);
  let clone;
  if (Array.isArray(value)) {
    clone = Array.from(value, item => cloneData(item, ancestors));
  } else {
    clone = Object.fromEntries(Object.entries(value).map(([key, item]) => [key, cloneData(item, ancestors)]));
  }
  ancestors.delete(value);
  return Object.freeze(clone);
}

export function defineModule(id, options = {}) {
  if (typeof id !== "string" || id.length === 0 || !record(options)) {
    throw new TypeError("A module requires a nonempty id and an options object");
  }
  return Object.freeze({ id, options: cloneData(options) });
}

export function defineModules(modules) {
  return normalizeModulePlan({ version: 1, modules });
}

export function normalizeModulePlan(value) {
  if (!record(value) || Object.keys(value).some(key => key !== "version" && key !== "modules") || value.version !== 1 || !Array.isArray(value.modules)) {
    throw new TypeError("Expected a version 1 module plan with a modules array");
  }
  const ids = new Set();
  const modules = value.modules.map(module => {
    if (!record(module) || Object.keys(module).some(key => key !== "id" && key !== "options")) throw new TypeError("Invalid module definition");
    const selected = defineModule(module.id, module.options === undefined ? {} : module.options);
    if (ids.has(selected.id)) throw new TypeError(`Duplicate module ${selected.id}`);
    ids.add(selected.id);
    return selected;
  });
  return Object.freeze({ version: 1, modules: Object.freeze(modules) });
}

export function moduleConfigPaths(root) {
  return [MODULE_CONFIG_FILE, MODULE_PLAN_FILE].map(name => path.resolve(root, name));
}

function selectedSource(root) {
  const [code, json] = moduleConfigPaths(root);
  if (existsSync(code) && existsSync(json)) throw new Error("Choose one module configuration source: linguini.config.mjs or linguini.modules.json");
  return existsSync(code) ? { path: code, code: true } : existsSync(json) ? { path: json, code: false } : undefined;
}

function readPlanFile(file) {
  if (statSync(file).size > MAX_PLAN_BYTES) throw new Error("Module plan exceeds the 4 MiB limit");
  return JSON.parse(readFileSync(file, "utf8"));
}

function workerResult(source, output, status, error) {
  if (error) throw new Error(`Cannot load module configuration ${source}: ${error.message}`, { cause: error });
  if (!existsSync(output)) throw new Error(`Module configuration worker for ${source} exited ${status} without a result`);
  const result = readPlanFile(output);
  const root = path.dirname(source), canonical = realpathSync(root);
  const dependencies = [...new Set([source, ...(result.dependencies ?? [])].map(file => {
    const relative = path.relative(canonical, file);
    return relative !== ".." && !relative.startsWith(`..${path.sep}`) && !path.isAbsolute(relative)
      ? path.resolve(root, relative) : file;
  }))].sort();
  if (status !== 0 || result.error) {
    const failure = new Error(`Cannot load module configuration ${source}: ${result.error ?? `worker exited ${status}`}`);
    failure.moduleDependencies = Object.freeze(dependencies);
    throw failure;
  }
  return Object.freeze({ plan: normalizeModulePlan(result.plan), configPath: source, dependencies: Object.freeze(dependencies) });
}

export function readModuleConfigurationSync(root) {
  const source = selectedSource(root);
  if (!source) return Object.freeze({ plan: defineModules([]), configPath: undefined, dependencies: Object.freeze([]) });
  if (!source.code) return Object.freeze({ plan: normalizeModulePlan(readPlanFile(source.path)), configPath: source.path, dependencies: Object.freeze([source.path]) });
  const directory = mkdtempSync(path.join(tmpdir(), "linguini-module-plan-"));
  const output = path.join(directory, "plan.json");
  try {
    const result = spawnSync(process.execPath, [worker, source.path, output], { cwd: path.dirname(source.path), stdio: ["ignore", 2, 2] });
    return workerResult(source.path, output, result.status, result.error);
  } finally {
    rmSync(directory, { recursive: true, force: true });
  }
}

export async function readModuleConfiguration(root) {
  const source = selectedSource(root);
  if (!source) return Object.freeze({ plan: defineModules([]), configPath: undefined, dependencies: Object.freeze([]) });
  if (!source.code) return Object.freeze({ plan: normalizeModulePlan(readPlanFile(source.path)), configPath: source.path, dependencies: Object.freeze([source.path]) });
  const directory = mkdtempSync(path.join(tmpdir(), "linguini-module-plan-"));
  const output = path.join(directory, "plan.json");
  try {
    return await new Promise((resolve, reject) => {
      const child = spawn(process.execPath, [worker, source.path, output], { cwd: path.dirname(source.path), stdio: ["ignore", 2, 2] });
      child.once("error", reject);
      child.once("close", status => {
        try { resolve(workerResult(source.path, output, status)); }
        catch (error) { reject(error); }
      });
    });
  } finally {
    rmSync(directory, { recursive: true, force: true });
  }
}
