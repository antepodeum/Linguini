const cp = require('child_process');
const fs = require('fs');
const path = require('path');

const extensionRoot = path.resolve(__dirname, '..');

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
const selectedTargets = args.all ? targets : [args.target ?? hostTarget()];
const outDir = path.join(extensionRoot, 'dist');

fs.mkdirSync(outDir, { recursive: true });
run('npm', ['run', 'compile']);

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

function parseArgs(argv) {
  const parsed = { all: false, target: undefined };
  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index];
    if (arg === '--all') {
      parsed.all = true;
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

  if (parsed.target && !targets.includes(parsed.target)) {
    throw new Error(`unsupported VS Code target: ${parsed.target}`);
  }

  return parsed;
}

function hostTarget() {
  if (process.platform === 'linux' && process.arch === 'arm') {
    return 'linux-armhf';
  }
  return `${process.platform}-${process.arch}`;
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
