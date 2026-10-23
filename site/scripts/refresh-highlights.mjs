import { spawnSync } from 'node:child_process';
import { mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { examples } from '../src/lib/server/examples.mjs';
import { examplesHash } from '../src/lib/server/highlight-render.mjs';

const siteRoot = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const output = join(siteRoot, '.svelte-kit/linguini');
mkdirSync(output, { recursive: true });
const highlights = {};
for (const [name, example] of Object.entries(examples)) {
  if (example.language === 'svelte') continue;
  const path = join(output, `${name}.${example.language === 'linguini-schema' ? 'lgs' : 'lgl'}`);
  writeFileSync(path, example.source);
  const result = spawnSync('cargo', ['run', '--locked', '--quiet', '-p', 'linguini-cli', '--', 'highlight', path], {
    cwd: resolve(siteRoot, '..'), encoding: 'utf8', maxBuffer: 8 * 1024 * 1024
  });
  if (result.error || result.status !== 0) throw result.error ?? new Error(result.stderr);
  highlights[name] = JSON.parse(result.stdout);
}
const artifact = join(siteRoot, 'generated/highlights.json');
const content = JSON.stringify({ version: 1, sourceHash: examplesHash(examples), highlights }) + '\n';
if (process.argv.includes('--check')) {
  if (readFileSync(artifact, 'utf8') !== content) throw new Error('Native highlights are stale. Run pnpm --dir site highlight:refresh.');
} else {
  writeFileSync(artifact, content);
}
console.log('Verified site highlight artifacts against the shared Rust language pipeline.');
