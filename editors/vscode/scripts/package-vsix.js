const cp = require('child_process');
const fs = require('fs');
const path = require('path');

const extensionRoot = path.resolve(__dirname, '..');
const outDir = path.join(extensionRoot, 'dist');

if (process.argv.length > 2) {
  throw new Error('package-vsix now creates one universal VSIX and does not accept platform targets');
}

fs.rmSync(outDir, { recursive: true, force: true });
fs.mkdirSync(outDir, { recursive: true });

run('npm', ['run', 'compile']);
run(npxCommand(), [
  '--yes',
  '@vscode/vsce',
  'package',
  '--out',
  path.join('dist', 'linguini-vscode.vsix')
]);

function npxCommand() {
  return process.platform === 'win32' ? 'npx.cmd' : 'npx';
}

function run(command, args) {
  const result = cp.spawnSync(command, args, {
    cwd: extensionRoot,
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
