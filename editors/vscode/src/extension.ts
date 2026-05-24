import * as vscode from 'vscode';
import {
  DocumentSelector,
  LanguageClient,
  LanguageClientOptions,
  ServerOptions
} from 'vscode-languageclient/node';

let client: LanguageClient | undefined;
let traceOutputChannel: vscode.OutputChannel | undefined;

const documentSelector: DocumentSelector = [
  { language: 'linguini-schema', scheme: 'file' },
  { language: 'linguini-locale', scheme: 'file' },
  { language: 'linguini-schema', scheme: 'untitled' },
  { language: 'linguini-locale', scheme: 'untitled' }
];

export async function activate(context: vscode.ExtensionContext): Promise<void> {
  traceOutputChannel = vscode.window.createOutputChannel('Linguini Language Server Trace');
  client = createClient(context);

  const configWatcher = vscode.workspace.createFileSystemWatcher('**/linguini.toml');
  const restartForConfigChange = () => restartClient(context);

  context.subscriptions.push(
    traceOutputChannel,
    configWatcher,
    configWatcher.onDidCreate(restartForConfigChange),
    configWatcher.onDidChange(restartForConfigChange),
    configWatcher.onDidDelete(restartForConfigChange),
    vscode.commands.registerCommand('linguini.restartServer', () => restartClient(context)),
    vscode.workspace.onDidChangeConfiguration((event) => {
      if (event.affectsConfiguration('linguini.server') ||
        event.affectsConfiguration('linguini.semanticHighlighting')) {
        void restartClient(context);
      }
    }),
    client
  );

  await startClient(client);
}

export async function deactivate(): Promise<void> {
  await client?.stop();
  client = undefined;
}

function createClient(_context: vscode.ExtensionContext): LanguageClient {
  const config = vscode.workspace.getConfiguration('linguini.server');
  const command = expandVariables(config.get<string>('path', 'linguini'));
  const args = config.get<string[]>('args', ['lsp']).map((arg) => expandVariables(arg));

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
    traceOutputChannel,
    middleware: createClientMiddleware()
  };

  return new LanguageClient(
    'linguiniLanguageServer',
    'Linguini Language Server',
    serverOptions,
    clientOptions
  );
}

async function restartClient(context: vscode.ExtensionContext): Promise<void> {
  const previous = client;
  client = createClient(context);

  await previous?.stop();
  context.subscriptions.push(client);
  await startClient(client);
}

async function startClient(nextClient: LanguageClient): Promise<void> {
  try {
    await nextClient.start();
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    void vscode.window.showErrorMessage(
      `Linguini language server failed to start: ${message}. Install the Linguini CLI and make sure the \`linguini\` command is on PATH, or set linguini.server.path.`
    );
  }
}

function expandVariables(value: string, document?: vscode.TextDocument): string {
  const workspaceFolder = document
    ? vscode.workspace.getWorkspaceFolder(document.uri)?.uri.fsPath ?? getWorkspaceRoot() ?? ''
    : getWorkspaceRoot() ?? '';

  return value
    .replaceAll('${file}', document?.uri.scheme === 'file' ? document.uri.fsPath : '')
    .replaceAll('${workspaceFolder}', workspaceFolder)
    .replaceAll('${languageId}', document?.languageId ?? '');
}

function createClientMiddleware(): NonNullable<LanguageClientOptions['middleware']> {
  return {
    provideDocumentSemanticTokens: (document, token, next) => {
      return isSemanticHighlightingEnabled() ? next(document, token) : null;
    },
    provideDocumentSemanticTokensEdits: (document, previousResultId, token, next) => {
      return isSemanticHighlightingEnabled()
        ? next(document, previousResultId, token)
        : null;
    },
    provideDocumentRangeSemanticTokens: (document, range, token, next) => {
      return isSemanticHighlightingEnabled() ? next(document, range, token) : null;
    }
  };
}

function isSemanticHighlightingEnabled(): boolean {
  return vscode.workspace
    .getConfiguration('linguini.semanticHighlighting')
    .get<boolean>('enabled', false);
}

function getWorkspaceRoot(): string | undefined {
  return vscode.workspace.workspaceFolders?.[0]?.uri.fsPath;
}
