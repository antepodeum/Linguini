import assert from 'node:assert/strict';
import test from 'node:test';
import { WorkspaceUris } from './uris';
import { discoverWorkspace, type WorkspaceFilesystem, type FileKind } from './snapshot';
import type { DiscoveryOptions, SavedFile } from './protocol';

test('URI mapping preserves documents, workspace edits, virtual roots and source text', () => {
  const uris = new WorkspaceUris(['vscode-vfs://github/org/repo', 'file:///C:/my%20project', 'vscode-vfs://github/org/repo/nested']);
  const locale = 'vscode-vfs://github/org/repo/locale/caf%C3%A9/en.lgl';
  assert.equal(uris.toServer(locale), 'file:///linguini-workspace/0/locale/caf%C3%A9/en.lgl');
  assert.equal(uris.toServer('vscode-vfs://github/org/repo/nested/a.lgs'), 'file:///linguini-workspace/2/a.lgs');
  for (const uri of [locale, 'file:///C:/my%20project/a%23b%25.lgs', 'untitled:Untitled-1', 'file:///other/a.lgl']) assert.equal(uris.fromServer(uris.toServer(uri)), uri);
  const message = { params: { textDocument: { uri: locale }, text: locale, changes: { [locale]: [{ newText: locale }] }, documentChanges: [{ oldUri: locale, newUri: locale + '.bak' }] } };
  const mapped = uris.message(message, true);
  assert.equal(mapped.params.textDocument.uri, uris.toServer(locale));
  assert.equal(mapped.params.text, locale);
  assert.equal(Object.values(mapped.params.changes)[0][0].newText, locale);
  assert.deepEqual(uris.message(mapped, false), message);
  const filesystemRoot = new WorkspaceUris(['file:///']);
  assert.equal(filesystemRoot.toServer('file:///a/b.lgs'), 'file:///linguini-workspace/0/a/b.lgs');
  assert.equal(filesystemRoot.fromServer('file:///linguini-workspace/0/a/b.lgs'), 'file:///a/b.lgs');
});

test('host discovery uses Rust roots, rejects partial sources and applies shared budgets', async () => {
  const root = 'vscode-vfs://github/org/repo';
  const uris = new WorkspaceUris([root]);
  const entries = new Map<string, { kind: FileKind; text?: string }>([
    [root, { kind: 'directory' }], [`${root}/linguini.toml`, { kind: 'file', text: 'parsed by Rust' }],
    [`${root}/build`, { kind: 'directory' }], [`${root}/build/schema`, { kind: 'directory' }],
    [`${root}/build/schema/shop.lgs`, { kind: 'file', text: 'delivery(count: Number)\n' }],
    [`${root}/locale`, { kind: 'directory' }], [`${root}/locale/en.lgl`, { kind: 'file', text: 'delivery = {count}\n' }],
    [`${root}/node_modules`, { kind: 'directory' }], [`${root}/node_modules/linguini.toml`, { kind: 'file', text: 'must never be read' }]
  ]);
  const reads: string[] = [];
  const filesystem: WorkspaceFilesystem = {
    async stat(uri) { const item = entries.get(uri); if (!item) throw new Error(`missing ${uri}`); return { kind: item.kind, size: new TextEncoder().encode(item.text ?? '').length }; },
    async readFile(uri) { reads.push(uri); return new TextEncoder().encode(entries.get(uri)!.text); },
    async readDirectory(uri) { return [...entries].filter(([key]) => key.startsWith(uri + '/') && !key.slice(uri.length + 1).includes('/')).map(([key, item]) => [key.slice(uri.length + 1), item.kind]); },
    join(uri, name) { return `${uri}/${encodeURIComponent(name)}`; }
  };
  const options: DiscoveryOptions = { ignoredDirectories: ['build', 'node_modules'], maxBytes: 1024, maxFileBytes: 512, maxEntries: 100, maxDepth: 5, maxRoots: 2 };
  let snapshots: SavedFile[][] = [];
  const errors: string[] = [];
  const plan = { roots: [{ configUri: uris.toServer(`${root}/linguini.toml`), schemaUri: uris.toServer(`${root}/build/schema`), localeUri: uris.toServer(`${root}/locale`) }], errors: ['invalid unrelated project'] };
  const channel = { ready: Promise.resolve(options), async replace(files: SavedFile[]) { snapshots.push(files); }, async sourcePlan() { return plan; } };
  await discoverWorkspace(filesystem, channel, uris, [root], (error) => errors.push(error));
  assert.deepEqual(snapshots[0].map((file) => file.uri), [uris.toServer(`${root}/linguini.toml`)]);
  assert.equal(snapshots[1].length, 3);
  assert.ok(reads.includes(`${root}/build/schema/shop.lgs`));
  assert.ok(!reads.some((uri) => uri.includes('node_modules')));
  assert.deepEqual(errors, ['invalid unrelated project']);
  entries.set(`${root}/build/schema/z-link`, { kind: 'symlink' });
  snapshots = []; errors.length = 0;
  await discoverWorkspace(filesystem, channel, uris, [root], (error) => errors.push(error));
  assert.ok(errors.some((error) => error.includes('symlink')));
  assert.ok(!snapshots[1].some((file) => file.uri.endsWith('.lgs')), 'a failed source tree cannot leave partially indexed files');
  assert.ok(snapshots[1].some((file) => file.uri.endsWith('.lgl')));
  entries.delete(`${root}/build/schema/z-link`);
  entries.set(`${root}/build`, { kind: 'symlink' });
  snapshots = []; errors.length = 0;
  await discoverWorkspace(filesystem, channel, uris, [root], (error) => errors.push(error));
  assert.ok(errors.some((error) => error.includes('symlink')));
  assert.ok(!snapshots[1].some((file) => file.uri.endsWith('.lgs')), 'configured roots must inspect parents skipped by config discovery');
  await assert.rejects(discoverWorkspace(filesystem, { ...channel, ready: Promise.resolve({ ...options, maxEntries: 1 }) }, uris, [root], () => {}), /entries/);
});
