import { mkdirSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { examples } from '../src/lib/server/examples.mjs';
import { examplesHash } from '../src/lib/server/highlight-render.mjs';
import {runCompiler} from './compiler.mjs';

const siteRoot = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const output = join(siteRoot, '.svelte-kit/linguini');
mkdirSync(output, { recursive: true });
const highlights = {};
for (const [name, example] of Object.entries(examples)) {
  if (example.language === 'svelte') continue;
  const path = join(output, `${name}.${example.language === 'linguini-schema' ? 'lgs' : 'lgl'}`);
  writeFileSync(path, example.source);
  const result = runCompiler(['highlight', path], {
    cwd: resolve(siteRoot, '..'), encoding: 'utf8', maxBuffer: 8 * 1024 * 1024
  });
  highlights[name] = JSON.parse(result.stdout);
}
const artifact = join(output, 'native-highlights.json');
const content = JSON.stringify({ version: 1, sourceHash: examplesHash(examples), highlights }) + '\n';
writeFileSync(artifact, content);
console.log('Generated site highlights with the shared Rust language pipeline.');
