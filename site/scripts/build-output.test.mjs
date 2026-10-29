import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {readFileSync} from 'node:fs';
import {join} from 'node:path';
import test from 'node:test';

test('build output matches every current application source', () => {
  const root = join(import.meta.dirname,'..');
  const manifest = JSON.parse(readFileSync(join(root,'src/lib/generated/linguini/bundler/manifest.json'),'utf8'));
  assert.ok(Object.keys(manifest.applications).length > 0);
  for (const [name,application] of Object.entries(manifest.applications)) {
    const source = readFileSync(join(root,name));
    assert.equal(application.byte_length,source.length,`${name} has stale build output`);
    assert.equal(application.sha256,createHash('sha256').update(source).digest('hex'),`${name} has stale build output`);
  }
});
