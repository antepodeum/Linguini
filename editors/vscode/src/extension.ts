import * as fs from 'node:fs';
import { execFile } from 'node:child_process';
import * as vscode from 'vscode';
import { LanguageClient, LanguageClientOptions, ServerOptions } from 'vscode-languageclient/node';
import { activateWith, deactivate, EXTENSION_VERSION, type ClientInstance } from './activation';
import { canFallbackToWasm, expandServerArgument, resolveServer, type ResolvedServer } from './server-resolution';
import { wasmClient } from './wasm/client';
import { nodeEndpoint } from './wasm/node-endpoint';
export { deactivate };

export async function activate(context: vscode.ExtensionContext): Promise<void> {
  await activateWith(context, async (context, options) => {
    const runtime = vscode.workspace.getConfiguration('linguini.server').get<string>('runtime', 'auto');
    const virtual = vscode.workspace.workspaceFolders?.some((folder) => folder.uri.scheme !== 'file');
    if (runtime !== 'wasm' && !(runtime === 'auto' && virtual)) {
      const native = createNativeClient(context, options);
      try {
        if (native.server.source === 'path' || native.server.source === 'bundled') await probeNative(native.server);
        return { client: native.client, source: native.server.source };
      } catch (error) {
        await native.client.dispose();
        if (!canFallbackToWasm(runtime, native.server.source)) throw error;
        options.traceOutputChannel?.info('Native language server unavailable; using bundled WASM.');
      }
    }
    return createWasmClient(context, options);
  });
}

function probeNative(server: ResolvedServer): Promise<void> {
  return new Promise((resolve, reject) => {
    execFile(server.command, ['--version'], { cwd: server.cwd, timeout: 10_000 }, (error, stdout) => {
      if (error) { reject(error); return; }
      if (stdout.trim() !== `linguini ${EXTENSION_VERSION}`) { reject(new Error(`Incompatible native server: ${stdout.trim()}`)); return; }
      resolve();
    });
  });
}

function createWasmClient(context: vscode.ExtensionContext, options: LanguageClientOptions): Promise<ClientInstance> {
  return wasmClient(context, () => nodeEndpoint(vscode.Uri.joinPath(context.extensionUri, 'dist/wasm/worker-node.cjs').fsPath),
    vscode.Uri.joinPath(context.extensionUri, 'dist/wasm/linguini_lsp.js').toString(), options,
    (server, options) => new LanguageClient('linguiniLanguageServer', 'Linguini Language Server', server, options));
}

function createNativeClient(context: vscode.ExtensionContext, clientOptions: LanguageClientOptions): { client: LanguageClient; server: ResolvedServer } {
  const config = vscode.workspace.getConfiguration('linguini.server');
  const workspaceFolders =
    vscode.workspace.workspaceFolders?.map((folder) => folder.uri.fsPath) ?? [];
  const server = resolveServer({
    explicitPath: config.get<string>('path', ''),
    useWorkspaceCli: config.get<boolean>('useWorkspaceCli', false),
    workspaceFolders,
    workspaceFile:
      vscode.workspace.workspaceFile?.scheme === 'file'
        ? vscode.workspace.workspaceFile.fsPath
        : undefined,
    extensionPath: context.extensionPath,
    platform: process.platform,
    exists: fs.existsSync
  });
  const args = config
    .get<string[]>('args', ['lsp'])
    .map((argument) => expandServerArgument(argument, workspaceFolders));

  const serverOptions: ServerOptions = {
    command: server.command,
    args,
    options: {
      cwd: server.cwd
    }
  };

  return {
    client: new LanguageClient(
      'linguiniLanguageServer',
      'Linguini Language Server',
      serverOptions,
      clientOptions
    ),
    server
  };
}
