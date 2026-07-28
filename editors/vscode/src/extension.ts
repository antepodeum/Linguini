import * as fs from 'node:fs';
import * as vscode from 'vscode';
import {
  DocumentSelector,
  LanguageClient,
  LanguageClientOptions,
  ServerOptions
} from 'vscode-languageclient/node';
import { DebouncedSerialTask } from './restart';
import {
  expandServerArgument,
  resolveServer,
  ResolvedServer
} from './server-resolution';

const RESTART_DEBOUNCE_MS = 200;
const PROTOCOL_VERSION = '1';
const EXTENSION_VERSION = '0.1.0-alpha.4';

interface ProtocolVersion {
  protocolVersion: string;
  compilerVersion: string;
}

let client: LanguageClient | undefined;
let restarter: DebouncedSerialTask | undefined;
let traceOutputChannel: vscode.LogOutputChannel | undefined;

const documentSelector: DocumentSelector = [
  { language: 'linguini-schema', scheme: 'file' },
  { language: 'linguini-locale', scheme: 'file' },
  { language: 'linguini-schema', scheme: 'untitled' },
  { language: 'linguini-locale', scheme: 'untitled' }
];

export async function activate(context: vscode.ExtensionContext): Promise<void> {
  traceOutputChannel = vscode.window.createOutputChannel('Linguini Language Server Trace', {
    log: true
  });
  restarter = new DebouncedSerialTask(() => replaceClient(context));
  const configWatcher = vscode.workspace.createFileSystemWatcher('**/linguini.toml');
  const scheduleRestart = () => {
    void restarter?.schedule(RESTART_DEBOUNCE_MS).catch(showStartError);
  };

  context.subscriptions.push(
    traceOutputChannel,
    configWatcher,
    configWatcher.onDidCreate(scheduleRestart),
    configWatcher.onDidChange(scheduleRestart),
    configWatcher.onDidDelete(scheduleRestart),
    vscode.commands.registerCommand('linguini.restartServer', async () => {
      await restarter?.runNow();
    }),
    vscode.workspace.onDidChangeConfiguration((event) => {
      if (
        event.affectsConfiguration('linguini.server') ||
        event.affectsConfiguration('linguini.semanticHighlighting')
      ) {
        scheduleRestart();
      }
    }),
    vscode.workspace.onDidChangeWorkspaceFolders(scheduleRestart),
    {
      dispose() {
        restarter?.dispose();
      }
    }
  );

  try {
    await restarter.runNow();
  } catch (error) {
    showStartError(error);
  }
}

export async function deactivate(): Promise<void> {
  await restarter?.flush();
  restarter?.dispose();
  restarter = undefined;
  const active = client;
  client = undefined;
  await active?.stop();
}

function createClient(context: vscode.ExtensionContext): {
  client: LanguageClient;
  server: ResolvedServer;
} {
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
  const clientOptions: LanguageClientOptions = {
    documentSelector,
    synchronize: {
      configurationSection: 'linguini'
    },
    initializationOptions: {
      extensionVersion: EXTENSION_VERSION,
      protocolVersion: PROTOCOL_VERSION,
      serverSource: server.source
    },
    outputChannelName: 'Linguini Language Server',
    traceOutputChannel,
    middleware: createClientMiddleware()
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

async function replaceClient(context: vscode.ExtensionContext): Promise<void> {
  const previous = client;
  client = undefined;
  await previous?.stop();

  const next = createClient(context);
  try {
    await next.client.start();
    const version = await next.client.sendRequest<ProtocolVersion>('linguini/protocolVersion');
    validateProtocol(version, next.server);
    client = next.client;
  } catch (error) {
    await next.client.stop().catch(() => undefined);
    throw error;
  }
}

function validateProtocol(version: ProtocolVersion, server: ResolvedServer): void {
  if (version.protocolVersion !== PROTOCOL_VERSION) {
    throw new Error(
      `language-server protocol ${version.protocolVersion} is incompatible with extension protocol ${PROTOCOL_VERSION}`
    );
  }
  if (version.compilerVersion !== EXTENSION_VERSION) {
    throw new Error(
      `Linguini server ${version.compilerVersion} is incompatible with extension ${EXTENSION_VERSION} (${server.source} server)`
    );
  }
}

function showStartError(error: unknown): void {
  const message = error instanceof Error ? error.message : String(error);
  void vscode.window.showErrorMessage(
    `Linguini language server failed to start: ${message}. Set linguini.server.path to a compatible Linguini ${EXTENSION_VERSION} binary.`
  );
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
    .get<boolean>('enabled', true);
}
