const cp = require('node:child_process');
const crypto = require('node:crypto');
const fs = require('node:fs');
const path = require('node:path');
const esbuild = require('esbuild');
const extension = path.resolve(__dirname, '..');
const root = path.resolve(extension, '../..');
const out = path.join(extension, 'dist', 'wasm');
const crate = fs.readFileSync(path.join(root, 'crates/linguini-lsp-wasm/Cargo.toml'), 'utf8');
const version = crate.match(/^wasm-bindgen = "=([^"]+)"/m)[1];
const suffix = process.platform === 'win32' ? '.exe' : '';
const tools = path.join(root, 'target', 'wasm-bindgen-cli');
const bindingTool = process.env.WASM_BINDGEN || path.join(tools, 'bin', `wasm-bindgen${suffix}`);

function run(command, args) {
  const result = cp.spawnSync(command, args, { cwd: root, stdio: 'inherit', shell: false });
  if (result.error) throw result.error;
  if (result.status !== 0) throw new Error(`${command} exited with status ${result.status}`);
}

if (!process.env.WASM_BINDGEN && !fs.existsSync(bindingTool)) {
  run('cargo', ['install', 'wasm-bindgen-cli', '--version', version, '--locked', '--root', tools,
    '--target-dir', path.join(root, 'target', 'wasm-bindgen-tools')]);
}
const installed = cp.spawnSync(bindingTool, ['--version'], { encoding: 'utf8' });
if (installed.error) throw installed.error;
if (installed.status !== 0 || installed.stdout.trim() !== `wasm-bindgen ${version}`) {
  throw new Error(`WASM_BINDGEN must point to wasm-bindgen ${version}; received ${installed.stdout.trim()}`);
}
run('cargo', ['build', '--locked', '--release', '-p', 'linguini-lsp-wasm', '--target', 'wasm32-unknown-unknown']);
fs.rmSync(out, { recursive: true, force: true });
fs.mkdirSync(out, { recursive: true });
run(bindingTool, [path.join(root, 'target/wasm32-unknown-unknown/release/linguini_lsp_wasm.wasm'),
  '--target', 'web', '--remove-name-section', '--remove-producers-section', '--out-name', 'linguini_lsp', '--out-dir', out]);
fs.writeFileSync(path.join(out, 'package.json'), '{"type":"module"}\n');
for (const [runtime, platform, format, target, file] of [
  ['node', 'node', 'cjs', 'node20', 'worker-node.cjs'],
  ['browser', 'browser', 'esm', 'es2022', 'worker-browser.js']
]) {
  esbuild.buildSync({ entryPoints: [path.join(extension, `src/wasm/${runtime}-worker.ts`)],
    bundle: true, platform, format, target, outfile: path.join(out, file) });
}
const hashes = {};
for (const name of ['linguini_lsp.js', 'linguini_lsp_bg.wasm', 'worker-node.cjs', 'worker-browser.js']) {
  hashes[name] = crypto.createHash('sha256').update(fs.readFileSync(path.join(out, name))).digest('hex');
}
fs.writeFileSync(path.join(out, 'manifest.json'), `${JSON.stringify({
  version: require('../package.json').version, protocolVersion: '1', wasmBindgenVersion: version, sha256: hashes
}, null, 2)}\n`);
run(process.execPath, [path.join(extension, 'node_modules/typescript/bin/tsc'), '--noEmit', '--strict',
  '--skipLibCheck', '--target', 'ES2022', '--module', 'Node16', '--moduleResolution', 'Node16',
  path.join(extension, 'test/wasm-contract.mts')]);
console.log('Built shared WASM LSP and Node/browser workers.');
