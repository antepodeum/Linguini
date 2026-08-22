import { createHash } from 'node:crypto';
import {
	brotliCompressSync,
	brotliDecompressSync,
	constants as zlibConstants
} from 'node:zlib';
import {
	existsSync,
	lstatSync,
	mkdirSync,
	readdirSync,
	readFileSync,
	realpathSync,
	rmSync,
	writeFileSync
} from 'node:fs';
import { dirname, isAbsolute, join, relative, resolve, sep } from 'node:path';

export const OUTPUT_FORMAT_VERSION = 1;
const MAX_FILES = 10_000;
const MAX_UNCOMPRESSED_BYTES = 32 * 1024 * 1024;

function sha256(value) {
	return createHash('sha256').update(value).digest('hex');
}

function portablePath(path) {
	return path.split(sep).join('/');
}

function validateEntryPath(path) {
	if (
		typeof path !== 'string' ||
		path.length === 0 ||
		isAbsolute(path) ||
		path.includes('\\') ||
		path.split('/').some((component) => component === '' || component === '.' || component === '..')
	) {
		throw new Error(`Unsafe packaged Linguini output path: ${JSON.stringify(path)}`);
	}
}

function collectFiles(root) {
	const files = [];
	const visit = (directory) => {
		for (const name of readdirSync(directory).sort()) {
			const path = join(directory, name);
			const metadata = lstatSync(path);
			if (metadata.isSymbolicLink()) {
				throw new Error(`Symbolic links are forbidden in packaged Linguini output: ${path}`);
			}
			if (metadata.isDirectory()) {
				visit(path);
			} else if (metadata.isFile()) {
				const entryPath = portablePath(relative(root, path));
				validateEntryPath(entryPath);
				files.push({ path: entryPath, content: readFileSync(path, 'utf8') });
			} else {
				throw new Error(`Unsupported packaged Linguini output entry: ${path}`);
			}
		}
	};
	visit(root);
	return files;
}

function assertSafeDestination(destination, allowedRoot) {
	const resolvedDestination = resolve(destination);
	const resolvedRoot = realpathSync(allowedRoot);
	if (resolvedDestination === resolvedRoot || !resolvedDestination.startsWith(`${resolvedRoot}${sep}`)) {
		throw new Error(`Packaged output destination escapes its allowed root: ${resolvedDestination}`);
	}
	return resolvedDestination;
}

export function packageOutput(sourceRoot, archivePath, manifestPath) {
	const root = realpathSync(sourceRoot);
	const files = collectFiles(root);
	if (files.length === 0) throw new Error('Cannot package empty Linguini output');
	if (files.length > MAX_FILES) throw new Error('Packaged Linguini output has too many files');

	const payload = Buffer.from(JSON.stringify({ format: OUTPUT_FORMAT_VERSION, files }));
	if (payload.length > MAX_UNCOMPRESSED_BYTES) {
		throw new Error('Packaged Linguini output is too large');
	}
	const archive = brotliCompressSync(payload, {
		params: { [zlibConstants.BROTLI_PARAM_QUALITY]: 11 }
	});
	const manifest = {
		format: OUTPUT_FORMAT_VERSION,
		sha256: sha256(archive),
		files: files.length,
		uncompressedBytes: payload.length
	};

	mkdirSync(dirname(archivePath), { recursive: true });
	writeFileSync(archivePath, archive);
	writeFileSync(manifestPath, `${JSON.stringify(manifest, null, 2)}\n`);
	return manifest;
}

export function restoreOutput(archivePath, manifestPath, destination, allowedRoot) {
	const archive = readFileSync(archivePath);
	const manifest = JSON.parse(readFileSync(manifestPath, 'utf8'));
	if (manifest.format !== OUTPUT_FORMAT_VERSION) {
		throw new Error(`Unsupported packaged Linguini output format: ${manifest.format}`);
	}
	if (
		!Number.isSafeInteger(manifest.files) ||
		manifest.files < 1 ||
		manifest.files > MAX_FILES ||
		!Number.isSafeInteger(manifest.uncompressedBytes) ||
		manifest.uncompressedBytes < 1 ||
		manifest.uncompressedBytes > MAX_UNCOMPRESSED_BYTES
	) {
		throw new Error('Packaged Linguini output exceeds safety limits');
	}
	if (sha256(archive) !== manifest.sha256) {
		throw new Error('Packaged Linguini output checksum mismatch');
	}

	const payload = brotliDecompressSync(archive, { maxOutputLength: MAX_UNCOMPRESSED_BYTES });
	if (payload.length !== manifest.uncompressedBytes) {
		throw new Error('Packaged Linguini output size mismatch');
	}
	const parsed = JSON.parse(payload.toString('utf8'));
	if (parsed.format !== OUTPUT_FORMAT_VERSION || !Array.isArray(parsed.files)) {
		throw new Error('Invalid packaged Linguini output payload');
	}
	if (parsed.files.length !== manifest.files || parsed.files.length === 0) {
		throw new Error('Packaged Linguini output file-count mismatch');
	}

	const paths = new Set();
	for (const entry of parsed.files) {
		if (entry === null || typeof entry !== 'object') {
			throw new Error('Invalid packaged output entry');
		}
		validateEntryPath(entry.path);
		if (typeof entry.content !== 'string' || paths.has(entry.path)) {
			throw new Error(`Invalid or duplicate packaged output entry: ${entry.path}`);
		}
		paths.add(entry.path);
	}

	const root = assertSafeDestination(destination, allowedRoot);
	if (existsSync(root)) rmSync(root, { recursive: true, force: true });
	for (const entry of parsed.files) {
		const path = resolve(root, ...entry.path.split('/'));
		if (!path.startsWith(`${root}${sep}`)) throw new Error(`Output path escaped destination: ${entry.path}`);
		mkdirSync(dirname(path), { recursive: true });
		writeFileSync(path, entry.content, 'utf8');
	}
	return manifest;
}
