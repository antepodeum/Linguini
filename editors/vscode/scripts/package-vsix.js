const cp = require('node:child_process');
const crypto = require('node:crypto');
const fs = require('node:fs');
const path = require('node:path');
const esbuild = require('esbuild');

const extensionRoot = path.resolve(__dirname, '..');
const repositoryRoot = path.resolve(extensionRoot, '..', '..');
const outDir = path.join(extensionRoot, 'dist');
const serverDir = path.join(outDir, 'server');
const packageJson = require(path.join(extensionRoot, 'package.json'));
const target = process.env.VSCE_TARGET?.trim();
const executableName = target?.startsWith('win32') ? 'linguini.exe' : 'linguini';
const configuredBinary = process.env.LINGUINI_SERVER_BINARY?.trim();
const defaultBinary = path.join(
  repositoryRoot,
  'target',
  'release',
  process.platform === 'win32' ? 'linguini.exe' : 'linguini'
);
const sourceBinary = path.resolve(configuredBinary || defaultBinary);

fs.rmSync(outDir, { recursive: true, force: true });
fs.mkdirSync(serverDir, { recursive: true });
run('npm', ['run', 'compile']);
bundleExtension();
vendorServer();
packageVsix();

function bundleExtension() {
  esbuild.buildSync({
    entryPoints: [path.join(extensionRoot, 'src', 'extension.ts')],
    bundle: true,
    platform: 'node',
    format: 'cjs',
    target: 'node20',
    external: ['vscode'],
    outfile: path.join(outDir, 'extension.js'),
    sourcemap: true,
    sourcesContent: false
  });
}

function vendorServer() {
  if (!fs.statSync(sourceBinary, { throwIfNoEntry: false })?.isFile()) {
    throw new Error(
      `Missing platform server binary at ${sourceBinary}. Set LINGUINI_SERVER_BINARY explicitly.`
    );
  }

  const version = cp.spawnSync(sourceBinary, ['--version'], {
    encoding: 'utf8',
    shell: false
  });
  if (version.error || version.status !== 0) {
    throw version.error || new Error(`Cannot execute ${sourceBinary}: ${version.stderr}`);
  }
  if (version.stdout.trim() !== `linguini ${packageJson.version}`) {
    throw new Error(
      `Server version mismatch: expected ${packageJson.version}, received ${version.stdout.trim()}`
    );
  }

  const destination = path.join(serverDir, executableName);
  fs.copyFileSync(sourceBinary, destination);
  if (process.platform !== 'win32') {
    fs.chmodSync(destination, 0o755);
  }
  const bytes = fs.readFileSync(destination);
  fs.writeFileSync(
    path.join(serverDir, 'manifest.json'),
    `${JSON.stringify(
      {
        version: packageJson.version,
        protocolVersion: '1',
        target: target || 'development',
        executable: executableName,
        sha256: crypto.createHash('sha256').update(bytes).digest('hex')
      },
      null,
      2
    )}\n`
  );
}

function packageVsix() {
  const suffix = target ? `-${target}` : '';
  const args = [
    'package',
    '--no-dependencies',
    '--out',
    path.join('dist', `linguini-vscode${suffix}.vsix`)
  ];
  if (target) {
    args.push('--target', target);
  }
  run(localExecutable('vsce'), args);
}

function localExecutable(name) {
  return path.join(
    extensionRoot,
    'node_modules',
    '.bin',
    process.platform === 'win32' ? `${name}.cmd` : name
  );
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
    throw new Error(`${command} exited with status ${result.status ?? 'unknown'}`);
  }
}
