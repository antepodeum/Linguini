const assert = require('node:assert/strict');
const vscode = require('vscode');

async function run() {
  const extension = vscode.extensions.getExtension('antepod.linguini-vscode');
  assert.ok(extension, 'development extension must be installed');
  await extension.activate();
  assert.equal(extension.isActive, true);

  await vscode.commands.executeCommand('linguini.restartServer');
  const document = await vscode.workspace.openTextDocument(
    vscode.Uri.joinPath(vscode.workspace.workspaceFolders[0].uri, 'linguini', 'schema', 'shop.lgs')
  );
  const deliveryOffset = document.getText().indexOf('delivery');
  assert.ok(deliveryOffset >= 0);
  const hover = await vscode.commands.executeCommand(
    'vscode.executeHoverProvider',
    document.uri,
    document.positionAt(deliveryOffset)
  );
  assert.ok(Array.isArray(hover), 'real LSP hover request must return a response');
}

module.exports = { run };
