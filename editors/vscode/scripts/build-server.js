const cp = require('child_process');
const fs = require('fs');
const path = require('path');

const repoRoot = path.resolve(__dirname, '..', '..', '..');
const extensionRoot = path.resolve(__dirname, '..');
const executable = process.platform === 'win32' ? 'linguini.exe' : 'linguini';
const outDir = path.join(extensionRoot, 'server', `${process.platform}-${process.arch}`);
const sourceBinary = path.join(repoRoot, 'target', 'release', executable);
const destBinary = path.join(outDir, executable);

run('cargo', ['build', '--release', '-p', 'linguini-cli'], repoRoot);
fs.mkdirSync(outDir, { recursive: true });
fs.copyFileSync(sourceBinary, destBinary);
if (process.platform !== 'win32') {
  fs.chmodSync(destBinary, 0o755);
}
console.log(`wrote ${path.relative(extensionRoot, destBinary)}`);

function run(command, args, cwd) {
  const result = cp.spawnSync(command, args, {
    cwd,
    stdio: 'inherit',
    shell: process.platform === 'win32'
  });

  if (result.error) {
    throw result.error;
  }
  if (result.status !== 0) {
    process.exit(result.status ?? 1);
  }
}
