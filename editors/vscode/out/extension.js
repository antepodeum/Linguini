"use strict";
var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    var desc = Object.getOwnPropertyDescriptor(m, k);
    if (!desc || ("get" in desc ? !m.__esModule : desc.writable || desc.configurable)) {
      desc = { enumerable: true, get: function() { return m[k]; } };
    }
    Object.defineProperty(o, k2, desc);
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __setModuleDefault = (this && this.__setModuleDefault) || (Object.create ? (function(o, v) {
    Object.defineProperty(o, "default", { enumerable: true, value: v });
}) : function(o, v) {
    o["default"] = v;
});
var __importStar = (this && this.__importStar) || (function () {
    var ownKeys = function(o) {
        ownKeys = Object.getOwnPropertyNames || function (o) {
            var ar = [];
            for (var k in o) if (Object.prototype.hasOwnProperty.call(o, k)) ar[ar.length] = k;
            return ar;
        };
        return ownKeys(o);
    };
    return function (mod) {
        if (mod && mod.__esModule) return mod;
        var result = {};
        if (mod != null) for (var k = ownKeys(mod), i = 0; i < k.length; i++) if (k[i] !== "default") __createBinding(result, mod, k[i]);
        __setModuleDefault(result, mod);
        return result;
    };
})();
Object.defineProperty(exports, "__esModule", { value: true });
exports.activate = activate;
exports.deactivate = deactivate;
const cp = __importStar(require("child_process"));
const vscode = __importStar(require("vscode"));
const node_1 = require("vscode-languageclient/node");
let client;
let traceOutputChannel;
let formatterOutputChannel;
const documentSelector = [
    { language: 'linguini-schema', scheme: 'file' },
    { language: 'linguini-locale', scheme: 'file' },
    { language: 'linguini-schema', scheme: 'untitled' },
    { language: 'linguini-locale', scheme: 'untitled' }
];
async function activate(context) {
    traceOutputChannel = vscode.window.createOutputChannel('Linguini Language Server Trace');
    formatterOutputChannel = vscode.window.createOutputChannel('Linguini Formatter');
    client = createClient();
    context.subscriptions.push(traceOutputChannel, formatterOutputChannel, vscode.languages.registerDocumentFormattingEditProvider(documentSelector, new LinguiniDocumentFormattingProvider()), vscode.commands.registerCommand('linguini.restartServer', restartClient), vscode.workspace.onDidChangeConfiguration((event) => {
        if (event.affectsConfiguration('linguini.server') ||
            event.affectsConfiguration('linguini.semanticHighlighting')) {
            void restartClient();
        }
    }), client);
    await startClient(client);
}
async function deactivate() {
    await client?.stop();
    client = undefined;
}
class LinguiniDocumentFormattingProvider {
    async provideDocumentFormattingEdits(document, _options, token) {
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
                vscode.TextEdit.replace(new vscode.Range(document.positionAt(0), document.positionAt(input.length)), formatted)
            ];
        }
        catch (error) {
            const message = error instanceof Error ? error.message : String(error);
            formatterOutputChannel?.appendLine(message);
            void vscode.window.showErrorMessage(`Linguini formatting failed: ${message}`);
            return [];
        }
    }
}
function createClient() {
    const config = vscode.workspace.getConfiguration('linguini.server');
    const command = expandVariables(config.get('path', 'linguini'));
    const args = config.get('args', ['lsp']).map((arg) => expandVariables(arg));
    const serverOptions = {
        command,
        args,
        options: {
            cwd: getWorkspaceRoot()
        }
    };
    const clientOptions = {
        documentSelector,
        synchronize: {
            configurationSection: 'linguini'
        },
        outputChannelName: 'Linguini Language Server',
        traceOutputChannel,
        middleware: createClientMiddleware()
    };
    return new node_1.LanguageClient('linguiniLanguageServer', 'Linguini Language Server', serverOptions, clientOptions);
}
async function restartClient() {
    const previous = client;
    client = createClient();
    await previous?.stop();
    await startClient(client);
}
async function startClient(nextClient) {
    try {
        await nextClient.start();
    }
    catch (error) {
        const message = error instanceof Error ? error.message : String(error);
        void vscode.window.showErrorMessage(`Linguini language server failed to start: ${message}`);
    }
}
function runFormatter(document, input, token) {
    const config = vscode.workspace.getConfiguration('linguini.formatter');
    const command = expandVariables(config.get('path', 'linguini'), document);
    const args = expandFormatterArgs(config.get('args', ['formatting']), document);
    const timeoutMs = config.get('timeoutMs', 10000);
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
        const finish = (callback) => {
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
        child.stdout.on('data', (chunk) => {
            stdout += chunk;
        });
        child.stderr.on('data', (chunk) => {
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
                }
                else {
                    reject(new Error(`formatter exited with code ${code}${stderr ? `: ${stderr.trim()}` : ''}`));
                }
            });
        });
        child.stdin.on('error', (error) => {
            if (error.code !== 'EPIPE') {
                formatterOutputChannel?.appendLine(error.message);
            }
        });
        child.stdin.end(input, 'utf8');
    });
}
function expandFormatterArgs(args, document) {
    return args.map((arg) => expandVariables(arg, document));
}
function expandVariables(value, document) {
    const workspaceFolder = document
        ? vscode.workspace.getWorkspaceFolder(document.uri)?.uri.fsPath ?? getWorkspaceRoot() ?? ''
        : getWorkspaceRoot() ?? '';
    return value
        .replaceAll('${file}', document?.uri.scheme === 'file' ? document.uri.fsPath : '')
        .replaceAll('${workspaceFolder}', workspaceFolder)
        .replaceAll('${languageId}', document?.languageId ?? '');
}
function createClientMiddleware() {
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
function isSemanticHighlightingEnabled() {
    return vscode.workspace
        .getConfiguration('linguini.semanticHighlighting')
        .get('enabled', false);
}
function getWorkspaceRoot() {
    return vscode.workspace.workspaceFolders?.[0]?.uri.fsPath;
}
//# sourceMappingURL=extension.js.map