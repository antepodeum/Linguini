const cp = require('node:child_process');
const crypto = require('node:crypto');
const fs = require('node:fs');
const path = require('node:path');
const esbuild = require('esbuild');
const extension = path.resolve(__dirname, '..');
const wasm = path.join(extension, 'dist', 'wasm');
const prepared = process.env.LINGUINI_WASM_DIR;
if (prepared) {
  if (path.resolve(prepared) !== wasm) {
    fs.rmSync(wasm, { recursive: true, force: true });
    fs.cpSync(path.resolve(prepared), wasm, { recursive: true });
  }
} else {
  const result = cp.spawnSync(process.execPath, [path.join(__dirname, 'build-wasm.js')], { stdio: 'inherit' });
  if (result.error) throw result.error;
  if (result.status !== 0) throw new Error('WASM language server build failed');
}
const manifest = JSON.parse(fs.readFileSync(path.join(wasm, 'manifest.json'), 'utf8'));
if (manifest.version !== require('../package.json').version || manifest.protocolVersion !== '1') throw new Error('WASM server version mismatch');
for (const [file, digest] of Object.entries(manifest.sha256)) {
  if (crypto.createHash('sha256').update(fs.readFileSync(path.join(wasm, file))).digest('hex') !== digest) throw new Error(`WASM artifact checksum mismatch: ${file}`);
}
for (const [entry, platform, target, name] of [
  ['extension', 'node', 'node20', 'extension.js'],
  ['extension-browser', 'browser', 'es2022', 'extension-browser.js']
]) {
  esbuild.buildSync({ entryPoints: [path.join(extension, `src/${entry}.ts`)],
    bundle: true, platform, format: 'cjs', target, external: ['vscode'],
    outfile: path.join(extension, 'dist', name), sourcemap: true, sourcesContent: false });
}
