import assert from 'node:assert/strict';
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import test from 'node:test';

import {
  configureGeneratedRuntime,
  normalizeBasePath
} from './configure-github-pages.mjs';

test('normalizes root and nested GitHub Pages base paths', () => {
  assert.equal(normalizeBasePath(''), '');
  assert.equal(normalizeBasePath('/'), '');
  assert.equal(normalizeBasePath(' /Linguini/ '), '/Linguini');
});

test('updates only generated base and cookie path options', (context) => {
  const directory = mkdtempSync(join(tmpdir(), 'linguini-pages-'));
  context.after(() => rmSync(directory, { recursive: true, force: true }));
  const client = join(directory, 'svelte.ts');
  const server = join(directory, 'sveltekit.ts');
  const source =
    'const options = { basePath: "", cookiePath: "/", label: "basePath: untouched" } as const;\n';
  writeFileSync(client, source);
  writeFileSync(server, source);

  assert.equal(
    configureGeneratedRuntime('/Linguini/', [client, server]),
    '/Linguini'
  );

  for (const file of [client, server]) {
    const configured = readFileSync(file, 'utf8');
    assert.match(configured, /basePath: "\/Linguini"/);
    assert.match(configured, /cookiePath: "\/Linguini"/);
    assert.match(configured, /label: "basePath: untouched"/);
  }
});
