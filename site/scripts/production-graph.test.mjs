import assert from 'node:assert/strict';
import { readdirSync, readFileSync, statSync } from 'node:fs';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';

const siteRoot = join(fileURLToPath(new URL('..', import.meta.url)));
const buildRoot = join(siteRoot, 'build');

function filesUnder(directory) {
  const files = [];
  for (const entry of readdirSync(directory, { withFileTypes: true })) {
    const path = join(directory, entry.name);
    if (entry.isDirectory()) files.push(...filesUnder(path));
    else files.push(path);
  }
  return files;
}

assert.equal(statSync(buildRoot).isDirectory(), true, 'site/build is missing; run pnpm build:generated first');

const outputFiles = filesUnder(buildRoot);
const textFiles = outputFiles
  .filter((file) => /\.(?:html|js|json|css)$/.test(file))
  .map((file) => readFileSync(file, 'utf8'));
const output = textFiles.join('\n');
const english = readFileSync(join(buildRoot, 'en/index.html'), 'utf8');

assert.match(english, /<title>Linguini \| typed localization language<\/title>/);
assert.match(english, /Internationalization for the rest of us/);
assert.doesNotMatch(output, /l\.main\.hero\.title\.toUpperCase/);

// The transformed message graph imports only the exact virtual message modules.
// These symbols are unique to the eager generated index/provider barrel.
assert.doesNotMatch(output, /createLinguiniProvider|localeLoaders/);
