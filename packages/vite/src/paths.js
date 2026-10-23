import path from "node:path";
import { realpathSync } from "node:fs";

export function toVitePath(file) {
  const normalized = file.replaceAll(path.sep, "/");
  return /^[A-Za-z]:\//.test(normalized) ? `/@fs/${normalized}` : normalized;
}

// Each payload owns its transitive generated modules. Sharing them between lazy
// locale chunks would add extra requests or pull inactive messages into a payload.
export function scopedPhysicalModuleId(file, scope, locale) {
  const owner = `${scope ?? "global"}-${Buffer.from(locale, "utf8").toString("hex")}`;
  return `${toVitePath(file)}?linguini-scope=${owner}`;
}

export async function resolvePayloadDependency(source, importer, generatedRoot, resolve) {
  if (!importer || !generatedRoot || !source.startsWith(".")) return undefined;
  const queryStart = importer.indexOf("?");
  if (queryStart < 0) return undefined;
  const owner = new URLSearchParams(importer.slice(queryStart + 1)).get("linguini-scope");
  if (!owner || !/^(?:global|(?:[0-9a-f]{2})+)-(?:[0-9a-f]{2})+$/.test(owner)) return undefined;
  const resolved = await resolve(source, importer);
  if (!resolved || resolved.external) return undefined;
  const [file] = resolved.id.split("?");
  const physical = file.startsWith("/@fs/") ? file.slice(5) : file;
  if (!path.isAbsolute(physical) || !physical.endsWith(".js")) return undefined;
  if (!isWithin(generatedRoot, physical) && !isWithin(realpathSync(generatedRoot), physical)) return undefined;
  const id = /[?&]linguini-scope=/.test(resolved.id)
    ? resolved.id.replace(/([?&]linguini-scope=)[^&]*/, (_match, prefix) => prefix + owner)
    : `${resolved.id}${resolved.id.includes("?") ? "&" : "?"}linguini-scope=${owner}`;
  return { ...resolved, id };
}

function isWithin(root, file) {
  const relative = path.relative(root, file);
  return relative !== ".." && !relative.startsWith(`..${path.sep}`) && !path.isAbsolute(relative);
}
