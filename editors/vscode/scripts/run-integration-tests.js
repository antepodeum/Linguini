const path = require('node:path');
const { runTests } = require('@vscode/test-electron');

async function main() {
  const extensionDevelopmentPath = path.resolve(__dirname, '..');
  const extensionTestsPath = path.resolve(extensionDevelopmentPath, 'test', 'integration');
  const workspace = path.resolve(extensionDevelopmentPath, 'sample-workspace');
  await runTests({
    version: '1.120.0',
    extensionDevelopmentPath,
    extensionTestsPath,
    launchArgs: [workspace, '--disable-extensions']
  });
}

main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
