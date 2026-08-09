import { readFileSync, writeFileSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const here = dirname(fileURLToPath(import.meta.url));
const defaultGeneratedFiles = [
  resolve(here, '../src/lib/generated/linguini/svelte-effects.svelte.ts'),
  resolve(here, '../src/lib/generated/linguini/sveltekit-control.ts')
];

if (resolve(process.argv[1] ?? '') === fileURLToPath(import.meta.url)) {
  const basePath = configureGeneratedRuntime(
    process.env.BASE_PATH ?? '',
    defaultGeneratedFiles
  );
  console.log(`Configured Linguini for GitHub Pages with base_path=${basePath || '(root)'}`);
}

export function configureGeneratedRuntime(rawBasePath, generatedFiles) {
  const basePath = normalizeBasePath(rawBasePath);
  const cookiePath = basePath || '/';

  for (const generatedFile of generatedFiles) {
    let source = readFileSync(generatedFile, 'utf8');
    source = replaceGeneratedOption(source, 'basePath', basePath, generatedFile);
    source = replaceGeneratedOption(source, 'cookiePath', cookiePath, generatedFile);
    writeFileSync(generatedFile, source);
  }

  return basePath;
}

export function normalizeBasePath(value) {
  const trimmed = String(value).trim();
  if (!trimmed || trimmed === '/') return '';
  return `/${trimmed.replace(/^\/+|\/+$/g, '')}`;
}

function replaceGeneratedOption(source, key, value, file) {
  const pattern = new RegExp(`\\b${key}:\\s*"(?:[^"\\\\]|\\\\.)*"`);
  if (!pattern.test(source)) {
    throw new Error(`Could not find generated ${key} option in ${file}`);
  }
  return source.replace(pattern, `${key}: "${escapeTypeScriptString(value)}"`);
}

function escapeTypeScriptString(value) {
  return String(value)
    .replace(/\\/g, '\\\\')
    .replace(/"/g, '\\"')
    .replace(/\r/g, '\\r')
    .replace(/\n/g, '\\n');
}
