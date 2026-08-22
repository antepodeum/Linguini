import assert from 'node:assert/strict';
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import test from 'node:test';
import { brotliCompressSync } from 'node:zlib';
import { createHash } from 'node:crypto';
import { packageOutput, restoreOutput } from './linguini-output.mjs';

const siteRoot = join(import.meta.dirname, '..');

test('packaged output round-trips every generated file deterministically', () => {
	const temporary = mkdtempSync(join(tmpdir(), 'linguini-site-output-'));
	try {
		const archive = join(temporary, 'output.br');
		const manifest = join(temporary, 'output.json');
		const first = packageOutput(join(siteRoot, 'src/lib/generated/linguini'), archive, manifest);
		const firstBytes = readFileSync(archive);
		const second = packageOutput(join(siteRoot, 'src/lib/generated/linguini'), archive, manifest);
		assert.deepEqual(second, first);
		assert.deepEqual(readFileSync(archive), firstBytes);

		const destination = join(temporary, 'restored/linguini');
		restoreOutput(archive, manifest, destination, temporary);
		const restoredArchive = join(temporary, 'restored.br');
		const restoredManifest = join(temporary, 'restored.json');
		assert.deepEqual(packageOutput(destination, restoredArchive, restoredManifest), first);
		assert.deepEqual(readFileSync(restoredArchive), firstBytes);
	} finally {
		rmSync(temporary, { recursive: true, force: true });
	}
});

test('restore rejects traversal before replacing destination', () => {
	const temporary = mkdtempSync(join(tmpdir(), 'linguini-site-traversal-'));
	try {
		const archivePath = join(temporary, 'output.br');
		const manifestPath = join(temporary, 'output.json');
		const payload = Buffer.from(JSON.stringify({
			format: 1,
			files: [{ path: '../escape.ts', content: 'bad' }]
		}));
		const archive = brotliCompressSync(payload);
		writeFileSync(archivePath, archive);
		writeFileSync(manifestPath, JSON.stringify({
			format: 1,
			sha256: createHash('sha256').update(archive).digest('hex'),
			files: 1,
			uncompressedBytes: payload.length
		}));
		assert.throws(
			() => restoreOutput(archivePath, manifestPath, join(temporary, 'output'), temporary),
			/Unsafe packaged Linguini output path/
		);
	} finally {
		rmSync(temporary, { recursive: true, force: true });
	}
});
