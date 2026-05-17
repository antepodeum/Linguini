import * as fs from 'fs';
import * as path from 'path';
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

  context.subscriptions.push(
    traceOutputChannel,
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

function createClient(context: vscode.ExtensionContext): LanguageClient {
  const config = vscode.workspace.getConfiguration('linguini.server');
  const command = expandVariables(resolveServerCommand(context, config));
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

function resolveServerCommand(
  context: vscode.ExtensionContext,
  config: vscode.WorkspaceConfiguration
): string {
  const configuredPath = config.get<string>('path', '').trim();
  return configuredPath || findBundledServer(context.extensionPath) || 'linguini';
}

function findBundledServer(extensionPath: string): string | undefined {
  for (const target of platformTargetCandidates()) {
    const executable = target.startsWith('win32-') ? 'linguini.exe' : 'linguini';
    const candidate = path.join(extensionPath, 'server', target, executable);
    if (fs.existsSync(candidate)) {
      return candidate;
    }
  }
  return undefined;
}

function platformTargetCandidates(): string[] {
  const candidates = [hostPlatformTarget()].filter((target): target is string => Boolean(target));

  if (process.platform === 'linux') {
    if (process.arch === 'x64') {
      candidates.push(isMuslLinux() ? 'alpine-x64' : 'linux-x64');
      candidates.push(isMuslLinux() ? 'linux-x64' : 'alpine-x64');
    } else if (process.arch === 'arm64') {
      candidates.push(isMuslLinux() ? 'alpine-arm64' : 'linux-arm64');
      candidates.push(isMuslLinux() ? 'linux-arm64' : 'alpine-arm64');
    }
  }

  return [...new Set(candidates)];
}

function hostPlatformTarget(): string | undefined {
  if (process.platform === 'darwin' && process.arch === 'arm64') {
    return 'darwin-arm64';
  }
  if (process.platform === 'darwin' && process.arch === 'x64') {
    return 'darwin-x64';
  }
  if (process.platform === 'linux' && process.arch === 'arm') {
    return 'linux-armhf';
  }
  if (process.platform === 'linux' && process.arch === 'arm64') {
    return isMuslLinux() ? 'alpine-arm64' : 'linux-arm64';
  }
  if (process.platform === 'linux' && process.arch === 'x64') {
    return isMuslLinux() ? 'alpine-x64' : 'linux-x64';
  }
  if (process.platform === 'win32' && process.arch === 'arm64') {
    return 'win32-arm64';
  }
  if (process.platform === 'win32' && process.arch === 'x64') {
    return 'win32-x64';
  }
  return undefined;
}

function isMuslLinux(): boolean {
  if (process.platform !== 'linux') {
    return false;
  }
  const report = process.report?.getReport?.() as { header?: { glibcVersionRuntime?: string } } | undefined;
  return Boolean(report?.header && !report.header.glibcVersionRuntime);
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
