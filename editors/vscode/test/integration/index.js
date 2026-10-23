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
  const deliveryOffset = document.getText().indexOf('delivery(');
  assert.ok(deliveryOffset >= 0);
  const hover = await vscode.commands.executeCommand(
    'vscode.executeHoverProvider',
    document.uri,
    document.positionAt(deliveryOffset)
  );
  assert.ok(Array.isArray(hover), 'real LSP hover request must return a response');
  assert.ok(hover.length > 0, 'bundled server must resolve the delivery declaration');
  const hoverText = hover
    .flatMap((item) => item.contents)
    .map((content) => (typeof content === 'string' ? content : content.value))
    .join('\n');
  assert.match(hoverText, /Product delivery sentence shown after checkout\./);
  const locale = await vscode.workspace.openTextDocument({ language: 'linguini-locale', content:
    'plain = 🍎 He said "hello" {name}\r\nblock = """\r\n  text {name}\r\n"""\r\n' });
  await assertHighlight(locale, '"hello"', 'string');
  await assertHighlight(locale, 'text', 'string');
  const markdown = await vscode.workspace.openTextDocument({ language: 'markdown', content:
    '# Examples\r\n  ~~~lgl\r\n  text = 🍎 "hello" {name}\r\n  ~~~\r\n' });
  await assertHighlight(markdown, '"hello"', 'string');
  const replacement = new vscode.WorkspaceEdit();
  replacement.insert(locale.uri, new vscode.Position(0, 0), '// edited\n');
  assert.equal(await vscode.workspace.applyEdit(replacement), true);
  await assertHighlight(locale, '"hello"', 'string');
  await vscode.commands.executeCommand('linguini.restartServer');
  await assertHighlight(markdown, '"hello"', 'string');

}

async function assertHighlight(document, text, expectedType) {
  const legend = await vscode.commands.executeCommand('vscode.provideDocumentSemanticTokensLegend', document.uri);
  const tokens = await vscode.commands.executeCommand('vscode.provideDocumentSemanticTokens', document.uri);
  assert.ok(legend && tokens, `semantic provider must support ${document.languageId}`);
  const expected = document.positionAt(document.getText().indexOf(text));
  let line = 0;
  let column = 0;
  for (let index = 0; index < tokens.data.length; index += 5) {
    line += tokens.data[index];
    column = tokens.data[index] === 0 ? column + tokens.data[index + 1] : tokens.data[index + 1];
    if (line === expected.line && column <= expected.character && column + tokens.data[index + 2] > expected.character) {
      assert.equal(legend.tokenTypes[tokens.data[index + 3]], expectedType);
      return;
    }
  }
  assert.fail(`${document.languageId}: no semantic highlight for ${text}`);
}

module.exports = { run };
