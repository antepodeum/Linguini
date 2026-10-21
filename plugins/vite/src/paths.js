import path from "node:path";

export function toVitePath(file) {
  const normalized = file.replaceAll(path.sep, "/");
  return /^[A-Za-z]:\//.test(normalized) ? `/@fs/${normalized}` : normalized;
}
