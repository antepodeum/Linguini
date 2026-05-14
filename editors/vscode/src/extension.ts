import * as cp from 'child_process';
import * as vscode from 'vscode';
import {
  DocumentSelector,
  LanguageClient,
  LanguageClientOptions,
  ServerOptions
} from 'vscode-languageclient/node';

let client: LanguageClient | undefined;
let traceOutputChannel: vscode.OutputChannel | undefined;
let formatterOutputChannel: vscode.OutputChannel | undefined;

const documentSelector: DocumentSelector = [
  { language: 'linguini-schema', scheme: 'file' },
  { language: 'linguini-locale', scheme: 'file' },
  { language: 'linguini-schema', scheme: 'untitled' },
  { language: 'linguini-locale', scheme: 'untitled' }
];

export async function activate(context: vscode.ExtensionContext): Promise<void> {
  traceOutputChannel = vscode.window.createOutputChannel('Linguini Language Server Trace');
  formatterOutputChannel = vscode.window.createOutputChannel('Linguini Formatter');
  client = createClient();

  context.subscriptions.push(
    traceOutputChannel,
    formatterOutputChannel,
    vscode.languages.registerDocumentFormattingEditProvider(
      documentSelector,
      new LinguiniDocumentFormattingProvider()
    ),
    vscode.commands.registerCommand('linguini.restartServer', restartClient),
    vscode.workspace.onDidChangeConfiguration((event) => {
      if (
        event.affectsConfiguration('linguini.server') ||
        event.affectsConfiguration('linguini.semanticHighlighting')
      ) {
        void restartClient();
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

class LinguiniDocumentFormattingProvider implements vscode.DocumentFormattingEditProvider {
  async provideDocumentFormattingEdits(
    document: vscode.TextDocument,
    _options: vscode.FormattingOptions,
    token: vscode.CancellationToken
  ): Promise<vscode.TextEdit[]> {
    try {
      const input = document.getText();
      const formatted = await runFormatter(document, input, token);

      if (formatted.length === 0 && input.length > 0) {
        throw new Error('formatter returned empty stdout; refusing to replace the document with empty content');
      }

      if (formatted === input) {
        return [];
      }

      return [
        vscode.TextEdit.replace(
          new vscode.Range(document.positionAt(0), document.positionAt(input.length)),
          formatted
        )
      ];
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      formatterOutputChannel?.appendLine(message);
      void vscode.window.showErrorMessage(`Linguini formatting failed: ${message}`);
      return [];
    }
  }
}

function createClient(): LanguageClient {
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

async function restartClient(): Promise<void> {
  const previous = client;
  client = createClient();

  await previous?.stop();
  await startClient(client);
}

async function startClient(nextClient: LanguageClient): Promise<void> {
  try {
    await nextClient.start();
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    void vscode.window.showErrorMessage(`Linguini language server failed to start: ${message}`);
  }
}

function runFormatter(
  document: vscode.TextDocument,
  input: string,
  token: vscode.CancellationToken
): Promise<string> {
  const config = vscode.workspace.getConfiguration('linguini.formatter');
  const command = expandVariables(config.get<string>('path', 'linguini'), document);
  const args = expandFormatterArgs(config.get<string[]>('args', ['formatting']), document);
  const timeoutMs = config.get<number>('timeoutMs', 10000);
  const cwd = getWorkspaceRoot() ?? process.cwd();

  formatterOutputChannel?.appendLine(`Running: ${command} ${args.join(' ')}`);

  return new Promise((resolve, reject) => {
    const child = cp.spawn(command, args, {
      cwd,
      windowsHide: true
    });

    let stdout = '';
    let stderr = '';
    let settled = false;

    const finish = (callback: () => void): void => {
      if (settled) {
        return;
      }

      settled = true;
      clearTimeout(timeout);
      cancellation.dispose();
      callback();
    };

    const timeout = setTimeout(() => {
      child.kill();
      finish(() => reject(new Error(`formatter timed out after ${timeoutMs}ms`)));
    }, timeoutMs);

    const cancellation = token.onCancellationRequested(() => {
      child.kill();
      finish(() => reject(new Error('formatting was cancelled')));
    });

    child.stdout.setEncoding('utf8');
    child.stderr.setEncoding('utf8');

    child.stdout.on('data', (chunk: string) => {
      stdout += chunk;
    });

    child.stderr.on('data', (chunk: string) => {
      stderr += chunk;
    });

    child.on('error', (error) => {
      finish(() => reject(error));
    });

    child.on('close', (code) => {
      finish(() => {
        if (stderr.trim().length > 0) {
          formatterOutputChannel?.appendLine(stderr.trimEnd());
        }

        if (code === 0) {
          resolve(stdout);
        } else {
          reject(new Error(`formatter exited with code ${code}${stderr ? `: ${stderr.trim()}` : ''}`));
        }
      });
    });

    child.stdin.on('error', (error: NodeJS.ErrnoException) => {
      if (error.code !== 'EPIPE') {
        formatterOutputChannel?.appendLine(error.message);
      }
    });

    child.stdin.end(input, 'utf8');
  });
}

function expandFormatterArgs(args: string[], document: vscode.TextDocument): string[] {
  return args.map((arg) => expandVariables(arg, document));
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
