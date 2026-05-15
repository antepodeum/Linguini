const cp = require('child_process');
const fs = require('fs');
const path = require('path');

const repoRoot = path.resolve(__dirname, '..', '..', '..');
const extensionRoot = path.resolve(__dirname, '..');

const targets = {
  'darwin-arm64': { rust: 'aarch64-apple-darwin', exe: 'linguini' },
  'darwin-x64': { rust: 'x86_64-apple-darwin', exe: 'linguini' },
  'linux-arm64': { rust: 'aarch64-unknown-linux-gnu', exe: 'linguini' },
  'linux-armhf': { rust: 'armv7-unknown-linux-gnueabihf', exe: 'linguini' },
  'linux-x64': { rust: 'x86_64-unknown-linux-gnu', exe: 'linguini' },
  'alpine-arm64': { rust: 'aarch64-unknown-linux-musl', exe: 'linguini' },
  'alpine-x64': { rust: 'x86_64-unknown-linux-musl', exe: 'linguini' },
  'win32-arm64': { rust: 'aarch64-pc-windows-msvc', exe: 'linguini.exe' },
  'win32-ia32': { rust: 'i686-pc-windows-msvc', exe: 'linguini.exe' },
  'win32-x64': { rust: 'x86_64-pc-windows-msvc', exe: 'linguini.exe' }
};

const args = parseArgs(process.argv.slice(2));
const selectedTargets = args.all ? Object.keys(targets) : [args.target ?? hostTarget()];

for (const target of selectedTargets) {
  buildTarget(target);
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
    } else {
      throw new Error(`unknown argument: ${arg}`);
    }
  }
  return parsed;
}

function hostTarget() {
  if (process.platform === 'linux' && process.arch === 'arm') {
    return 'linux-armhf';
  }
  return `${process.platform}-${process.arch}`;
}

function buildTarget(target) {
  const config = targets[target];
  if (!config) {
    throw new Error(`unsupported VS Code target: ${target}`);
  }

  run('rustup', ['target', 'add', config.rust], repoRoot);
  run('cargo', ['build', '--release', '-p', 'linguini-cli', '--target', config.rust], repoRoot);

  const sourceBinary = path.join(repoRoot, 'target', config.rust, 'release', config.exe);
  const outDir = path.join(extensionRoot, 'server', target);
  const destBinary = path.join(outDir, config.exe);
  fs.mkdirSync(outDir, { recursive: true });
  fs.copyFileSync(sourceBinary, destBinary);
  if (!target.startsWith('win32-')) {
    fs.chmodSync(destBinary, 0o755);
  }
  console.log(`wrote ${path.relative(extensionRoot, destBinary)}`);
}

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
