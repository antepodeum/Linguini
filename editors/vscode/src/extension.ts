import * as vscode from 'vscode';
import {
  DocumentSelector,
  LanguageClient,
  LanguageClientOptions,
  ServerOptions
} from 'vscode-languageclient/node';

let client: LanguageClient | undefined;

const documentSelector: DocumentSelector = [
  { language: 'linguini-schema', scheme: 'file' },
  { language: 'linguini-locale', scheme: 'file' },
  { language: 'linguini-schema', scheme: 'untitled' },
  { language: 'linguini-locale', scheme: 'untitled' }
];

export async function activate(context: vscode.ExtensionContext): Promise<void> {
  client = createClient();

  context.subscriptions.push(
    vscode.workspace.onDidChangeConfiguration((event) => {
      if (event.affectsConfiguration('linguini.server')) {
        void restartClient();
      }
    })
  );

  context.subscriptions.push(client);
  await client.start();
}

export async function deactivate(): Promise<void> {
  await client?.stop();
  client = undefined;
}

function createClient(): LanguageClient {
  const config = vscode.workspace.getConfiguration('linguini.server');
  const command = config.get<string>('path', 'linguini');
  const args = config.get<string[]>('args', ['lsp']);

  const serverOptions: ServerOptions = {
    command,
    args,
    options: {
      cwd: getWorkspaceRoot()
    }
  };

  const clientOptions: LanguageClientOptions = {
    documentSelector,
    synchronize: {
      configurationSection: 'linguini'
    },
    outputChannelName: 'Linguini Language Server',
    traceOutputChannel: vscode.window.createOutputChannel('Linguini Language Server Trace')
  };

  return new LanguageClient(
    'linguiniLanguageServer',
    'Linguini Language Server',
    serverOptions,
    clientOptions
  );
}

async function restartClient(): Promise<void> {
  const previous = client;
  client = createClient();

  await previous?.stop();
  await client.start();
}

function getWorkspaceRoot(): string | undefined {
  return vscode.workspace.workspaceFolders?.[0]?.uri.fsPath;
}
