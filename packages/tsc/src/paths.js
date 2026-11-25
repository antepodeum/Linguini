import path from 'node:path';

/** Compare path components using the host's separator and case rules. */
export function isWithin(root,file,paths = path) {
  const relative = paths.relative(root,file);
  return relative !== '..' && !relative.startsWith('..' + paths.sep) && !paths.isAbsolute(relative);
}
