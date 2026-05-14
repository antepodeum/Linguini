const { spawnSync } = require('child_process');
const fs = require('fs');
const path = require('path');

const workspace = path.resolve(__dirname, '..');
const sampleWorkspace = path.join(workspace, 'sample-workspace');
const openTargets = [
  sampleWorkspace,
  path.join(sampleWorkspace, 'example.lgs'),
  path.join(sampleWorkspace, 'en.lgl')
].filter((candidate) => fs.existsSync(candidate));

const args = [
  '--new-window',
  `--extensionDevelopmentPath=${workspace}`,
  ...openTargets
];

const configured = process.env.VSCODE_BIN?.trim();
const candidates = configured
  ? [configured]
  : ['codium', 'vscodium', 'code', 'code-insiders', 'code-oss'];

let lastError;
for (const command of candidates) {
  const result = spawnSync(command, args, {
    stdio: 'inherit',
    shell: process.platform === 'win32' || command.includes(' ')
  });

  if (!result.error) {
    process.exit(result.status ?? 0);
  }

  lastError = result.error;
  if (result.error.code !== 'ENOENT' || configured) {
    break;
  }
}

console.error(
  `Failed to start VS Code/VSCodium with the test extension: ${lastError?.message ?? 'unknown error'}`
);
console.error('Install the codium/code CLI in PATH, run from VSCodium with F5, or set VSCODE_BIN.');
console.error('Example for Flatpak: VSCODE_BIN="flatpak run com.vscodium.codium" npm run dev:host');
process.exit(1);
