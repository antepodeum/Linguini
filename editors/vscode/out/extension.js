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
const vscode = __importStar(require("vscode"));
const node_1 = require("vscode-languageclient/node");
let client;
const documentSelector = [
    { language: 'linguini-schema', scheme: 'file' },
    { language: 'linguini-locale', scheme: 'file' },
    { language: 'linguini-schema', scheme: 'untitled' },
    { language: 'linguini-locale', scheme: 'untitled' }
];
async function activate(context) {
    client = createClient();
    context.subscriptions.push(vscode.workspace.onDidChangeConfiguration((event) => {
        if (event.affectsConfiguration('linguini.server')) {
            void restartClient();
        }
    }));
    context.subscriptions.push(client);
    await client.start();
}
async function deactivate() {
    await client?.stop();
    client = undefined;
}
function createClient() {
    const config = vscode.workspace.getConfiguration('linguini.server');
    const command = config.get('path', 'linguini');
    const args = config.get('args', ['lsp']);
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
        traceOutputChannel: vscode.window.createOutputChannel('Linguini Language Server Trace')
    };
    return new node_1.LanguageClient('linguiniLanguageServer', 'Linguini Language Server', serverOptions, clientOptions);
}
async function restartClient() {
    const previous = client;
    client = createClient();
    await previous?.stop();
    await client.start();
}
function getWorkspaceRoot() {
    return vscode.workspace.workspaceFolders?.[0]?.uri.fsPath;
}
//# sourceMappingURL=extension.js.map