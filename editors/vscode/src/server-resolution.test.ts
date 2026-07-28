import assert from 'node:assert/strict';
import * as path from 'node:path';
import test from 'node:test';
import {
  expandServerArgument,
  resolveServer,
  workspaceCwd
} from './server-resolution';

const base = {
  explicitPath: '',
  useWorkspaceCli: false,
  workspaceFolders: ['/workspace'],
  extensionPath: '/extension',
  platform: 'linux' as const,
  exists: () => false
};

test('uses explicit, workspace, bundled, then PATH resolution order', () => {
  assert.equal(resolveServer({ ...base, explicitPath: '/custom/linguini' }).source, 'explicit');
  assert.equal(
    resolveServer({
      ...base,
      useWorkspaceCli: true,
      exists: (candidate) => candidate.includes('node_modules')
    }).source,
    'workspace'
  );
  assert.equal(
    resolveServer({
      ...base,
      exists: (candidate) => candidate.includes(path.join('dist', 'server'))
    }).source,
    'bundled'
  );
  assert.equal(resolveServer(base).source, 'path');
});

test('does not silently select the first multi-root folder', () => {
  assert.equal(workspaceCwd(['/one', '/two']), undefined);
  assert.throws(
    () => expandServerArgument('${workspaceFolder}/linguini', ['/one', '/two']),
    /ambiguous/
  );
});

test('rejects document-only launch variables', () => {
  assert.throws(() => expandServerArgument('${file}', ['/workspace']), /cannot be used/);
  assert.throws(() => expandServerArgument('${languageId}', ['/workspace']), /cannot be used/);
});
