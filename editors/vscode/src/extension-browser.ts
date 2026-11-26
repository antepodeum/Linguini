import * as vscode from 'vscode';
import { LanguageClient } from 'vscode-languageclient/browser';
import { activateWith } from './activation';
import { wasmClient } from './wasm/client';
import { browserEndpoint } from './wasm/browser-endpoint';
export { deactivate } from './activation';

export async function activate(context: vscode.ExtensionContext): Promise<void> {
  await activateWith(context, async (context, options) => {
    if (vscode.workspace.getConfiguration('linguini.server').get('runtime') === 'native') {
      throw new Error('Native servers cannot run in a browser extension. Select auto or wasm.');
    }
    return wasmClient(context, () => browserEndpoint(vscode.Uri.joinPath(context.extensionUri, 'dist/wasm/worker-browser.js').toString(true)),
      vscode.Uri.joinPath(context.extensionUri, 'dist/wasm/linguini_lsp.js').toString(true), options,
      (server, options) => new LanguageClient('linguiniLanguageServer', 'Linguini Language Server', server, options));
  });
}
