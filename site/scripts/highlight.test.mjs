import assert from 'node:assert/strict';
import test from 'node:test';
import { examplesHash, renderHighlight } from '../src/lib/server/highlight-render.mjs';

test('native UTF-16 highlights preserve source text and escape all HTML', () => {
  const source = 'hello = 🍎 <script>"&\'</script>\r\nnext = text';
  const html = renderHighlight(source, { version: 1, tokens: [
    { line: 0, start: 0, length: 5, type: 'variable' },
    { line: 0, start: 8, length: 2, type: 'string' },
    { line: 1, start: 7, length: 4, type: 'string' }
  ] });
  assert.ok(html.includes('>🍎</span>'));
  assert.ok(html.includes('&lt;script&gt;&quot;&amp;&#39;&lt;/script&gt;'));
  assert.ok(!html.includes('<script>'));
  assert.ok(html.includes('\nnext = <span'));
  assert.notEqual(examplesHash({ text: source }), examplesHash({ text: source + '!' }));
  for (const token of [
    { line: 5, start: 0, length: 1, type: 'string' },
    { line: 0, start: 0, length: 100, type: 'string' },
    { line: 0, start: 0, length: 1, type: 'unsafe"' }
  ]) assert.throws(() => renderHighlight(source, { version: 1, tokens: [token] }), /Invalid native/);
  assert.throws(() => renderHighlight(source, { version: 1, tokens: [
    { line: 0, start: 0, length: 5, type: 'string' },
    { line: 0, start: 1, length: 1, type: 'string' }
  ] }), /Overlapping/);
});

test('normal site highlighting renders packaged native tokens without external tools', async () => {
  const { spawnSync } = await import('node:child_process');
  const { readFileSync } = await import('node:fs');
  const { fileURLToPath } = await import('node:url');
  const { examples } = await import('../src/lib/server/examples.mjs');
  const siteRoot = fileURLToPath(new URL('../', import.meta.url));
  const result = spawnSync(process.execPath, ['scripts/highlight-examples.mjs'], {
    cwd: siteRoot, encoding: 'utf8', env: { ...process.env, PATH: '' }
  });
  assert.equal(result.status, 0, result.stderr + result.stdout);
  const output = JSON.parse(readFileSync(new URL('../.svelte-kit/linguini/highlights.json', import.meta.url), 'utf8'));
  assert.equal(output.sourceHash, examplesHash(examples));
  assert.deepEqual(Object.keys(output.codeBlocks), Object.keys(examples));
  for (const html of Object.values(output.codeBlocks)) {
    assert.match(html, /<pre\b/);
    assert.match(html, /<span\b/);
  }
});
