import { readFileSync, writeFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { dirname, resolve } from 'node:path';

const here = dirname(fileURLToPath(import.meta.url));
const configPath = resolve(here, '../linguini.toml');
const basePath = normalizeBasePath(process.env.BASE_PATH ?? '');
const cookiePath = basePath || '/';

let source = readFileSync(configPath, 'utf8');
source = replaceStringSetting(source, 'base_path', basePath);
source = replaceStringSetting(source, 'cookie_path', cookiePath);
writeFileSync(configPath, source);

console.log(`Configured Linguini for GitHub Pages with base_path=${basePath || '(root)'}`);

function normalizeBasePath(value) {
  const trimmed = String(value).trim();
  if (!trimmed || trimmed === '/') return '';
  return `/${trimmed.replace(/^\/+|\/+$/g, '')}`;
}

function replaceStringSetting(source, key, value) {
  const pattern = new RegExp(`^${key}\\s*=\\s*".*"$`, 'm');
  if (!pattern.test(source)) {
    throw new Error(`Could not find ${key} in linguini.toml`);
  }
  return source.replace(pattern, `${key} = "${escapeTomlString(value)}"`);
}

function escapeTomlString(value) {
  return String(value).replace(/\\/g, '\\\\').replace(/"/g, '\\"');
}
