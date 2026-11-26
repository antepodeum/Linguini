import * as vscode from 'vscode';
import type { BaseLanguageClient, LanguageClientOptions } from 'vscode-languageclient';
import { registerMarkdownHighlighting } from './highlight';
import { DebouncedSerialTask } from './restart';

const RESTART_DEBOUNCE_MS = 200;
export const PROTOCOL_VERSION = '1';
export const EXTENSION_VERSION = '0.1.0-alpha.4';

interface ProtocolVersion {
  protocolVersion: string;
  compilerVersion: string;
}

export interface ClientInstance { client: BaseLanguageClient; source: string; started?(): Promise<void>; dispose?(): void }
export type ClientFactory = (context: vscode.ExtensionContext, options: LanguageClientOptions) => Promise<ClientInstance>;
let active: ClientInstance | undefined;
let factory: ClientFactory;
let markdownHighlighting: vscode.Disposable | undefined;
let restarter: DebouncedSerialTask | undefined;
let traceOutputChannel: vscode.LogOutputChannel | undefined;

const documentSelector: LanguageClientOptions['documentSelector'] = [
  { language: 'linguini-schema' }, { language: 'linguini-locale' }
];

export async function activateWith(context: vscode.ExtensionContext, create: ClientFactory): Promise<void> {
  factory = create;
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
  markdownHighlighting?.dispose();
  markdownHighlighting = undefined;
  const previous = active;
  active = undefined;
  try { await previous?.client.stop(); } finally { previous?.dispose?.(); }
}

async function replaceClient(context: vscode.ExtensionContext): Promise<void> {
  markdownHighlighting?.dispose();
  markdownHighlighting = undefined;
  const previous = active;
  active = undefined;
  try { await previous?.client.stop(); } finally { previous?.dispose?.(); }

  const options: LanguageClientOptions = {
    documentSelector,
    synchronize: { configurationSection: 'linguini' },
    initializationOptions: { extensionVersion: EXTENSION_VERSION, protocolVersion: PROTOCOL_VERSION },
    outputChannelName: 'Linguini Language Server', traceOutputChannel,
    middleware: createClientMiddleware()
  };
  const next = await factory(context, options);
  try {
    await next.client.start();
    await next.started?.();
    const version = await next.client.sendRequest<ProtocolVersion>('linguini/protocolVersion');
    validateProtocol(version, next.source);
    markdownHighlighting = await registerMarkdownHighlighting(next.client, isSemanticHighlightingEnabled);
    active = next;
  } catch (error) {
    await next.client.stop().catch(() => undefined);
    next.dispose?.();
    throw error;
  }
}

function validateProtocol(version: ProtocolVersion, source: string): void {
  if (version.protocolVersion !== PROTOCOL_VERSION) {
    throw new Error(
      `language-server protocol ${version.protocolVersion} is incompatible with extension protocol ${PROTOCOL_VERSION}`
    );
  }
  if (version.compilerVersion !== EXTENSION_VERSION) {
    throw new Error(
      `Linguini server ${version.compilerVersion} is incompatible with extension ${EXTENSION_VERSION} (${source} server)`
    );
  }
}

function showStartError(error: unknown): void {
  const message = error instanceof Error ? error.message : String(error);
  void vscode.window.showErrorMessage(
    `Linguini language server failed to start: ${message}. Use the bundled WASM server or set linguini.server.path to a compatible Linguini ${EXTENSION_VERSION} binary.`
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
