const path = require('node:path');
const fs = require('node:fs');
const { runTests, runVSCodeCommand } = require('@vscode/test-electron');

async function main() {
  const extensionRoot = path.resolve(__dirname, '..');
  let extensionDevelopmentPath = extensionRoot;
  const extensionTestsPath = path.resolve(extensionRoot, 'test', 'integration');
  const runtime = process.env.LINGUINI_TEST_RUNTIME || 'native';
  const profile = path.resolve(extensionRoot, '../../target/vscode-test-profile', runtime);
  const workspace = path.join(profile, 'workspace');
  fs.rmSync(workspace, { recursive: true, force: true });
  fs.cpSync(path.resolve(extensionRoot, 'sample-workspace'), workspace, {
    recursive: true, filter: (source) => path.basename(source) !== '.vscode'
  });
  const launchArgs = [workspace, '--user-data-dir', profile];
  if (process.env.LINGUINI_TEST_VSIX) {
    const extensions = path.join(profile, 'extensions');
    const result = await runVSCodeCommand(['--user-data-dir', profile, '--extensions-dir', extensions,
      '--install-extension', path.resolve(process.env.LINGUINI_TEST_VSIX), '--force'], { version: '1.120.0' });
    console.log(result.stdout);
    extensionDevelopmentPath = path.join(profile, 'probe');
    fs.mkdirSync(extensionDevelopmentPath, { recursive: true });
    fs.writeFileSync(path.join(extensionDevelopmentPath, 'package.json'), JSON.stringify({
      name: 'integration-probe', publisher: 'linguini-tests', version: '0.0.0', engines: { vscode: '^1.120.0' }
    }));
    launchArgs.push('--extensions-dir', extensions);
  } else { launchArgs.push('--disable-extensions'); }
  await runTests({
    version: '1.120.0',
    extensionDevelopmentPath,
    extensionTestsPath,
    extensionTestsEnv: { LINGUINI_TEST_RUNTIME: runtime, LINGUINI_TEST_NODE: process.execPath,
      LINGUINI_TEST_LAUNCHER: path.resolve(extensionRoot, 'test/launcher.cjs') },
    launchArgs
  });
}

main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
