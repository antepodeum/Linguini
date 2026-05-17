const cp = require('child_process');
const fs = require('fs');
const path = require('path');

const extensionRoot = path.resolve(__dirname, '..');
const outDir = path.join(extensionRoot, 'dist');
const serverDir = path.join(extensionRoot, 'server');

const targets = [
  'darwin-arm64',
  'darwin-x64',
  'linux-arm64',
  'linux-armhf',
  'linux-x64',
  'alpine-arm64',
  'alpine-x64',
  'win32-arm64',
  'win32-x64'
];

const args = parseArgs(process.argv.slice(2));
const selectedTargets = args.all ? targets : args.target ? [args.target] : [];

fs.rmSync(outDir, { recursive: true, force: true });
fs.mkdirSync(outDir, { recursive: true });
run('npm', ['run', 'compile']);

if (selectedTargets.length === 0) {
  packageUniversalVsix();
} else {
  packageBundledVsix(selectedTargets);
}

function parseArgs(argv) {
  const parsed = { all: false, target: undefined };
  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index];
    if (arg === '--all') {
      parsed.all = true;
    } else if (arg === '--universal') {
      continue;
    } else if (arg === '--target') {
      parsed.target = argv[index + 1];
      index += 1;
    } else if (arg.startsWith('--target=')) {
      parsed.target = arg.slice('--target='.length);
    } else if (!parsed.target) {
      parsed.target = arg;
    } else {
      throw new Error(`unknown argument: ${arg}`);
    }
  }

  if (parsed.all && parsed.target) {
    throw new Error('choose either --all or one --target, not both');
  }
  if (parsed.target && !targets.includes(parsed.target)) {
    throw new Error(`unsupported VS Code target: ${parsed.target}`);
  }

  return parsed;
}

function packageUniversalVsix() {
  fs.rmSync(serverDir, { recursive: true, force: true });
  run(npxCommand(), [
    '--yes',
    '@vscode/vsce',
    'package',
    '--out',
    path.join('dist', 'linguini-vscode.vsix')
  ]);
}

function packageBundledVsix(selectedTargets) {
  for (const target of selectedTargets) {
    run('node', ['./scripts/build-server.js', '--target', target]);
    run(npxCommand(), [
      '--yes',
      '@vscode/vsce',
      'package',
      '--target',
      target,
      '--out',
      path.join('dist', `linguini-vscode-${target}.vsix`)
    ]);
  }
}

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
