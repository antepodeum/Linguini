import * as vscode from 'vscode';
import type { BaseLanguageClient, LanguageClientOptions, MessageTransports } from 'vscode-languageclient';
import { WorkerChannel } from './channel';
import type { WorkerEndpoint } from './protocol';
import { workerTransports } from './transports';
import { WorkspaceUris } from './uris';
import { loadWorkspace, updateSavedFile } from './workspace';

export async function wasmClient(
  context: vscode.ExtensionContext,
  endpoint: () => WorkerEndpoint,
  moduleUri: string,
  options: LanguageClientOptions,
  createClient: (server: () => Promise<MessageTransports>, options: LanguageClientOptions) => BaseLanguageClient
): Promise<{ client: BaseLanguageClient; source: string; started(): Promise<void>; dispose(): void }> {
  const roots = vscode.workspace.workspaceFolders?.map((folder) => folder.uri) ?? [];
  const uris = new WorkspaceUris(roots.map((root) => root.toString()));
  const bytes = await vscode.workspace.fs.readFile(vscode.Uri.joinPath(context.extensionUri, 'dist/wasm/linguini_lsp_bg.wasm'));
  const channel = new WorkerChannel(endpoint(), moduleUri, bytes);
  const disposables: vscode.Disposable[] = [];
  const log = (message: string) => options.traceOutputChannel?.warn(message);
  try {
    const limits = await channel.ready;
    const projects = await loadWorkspace(channel, uris, roots, log);
    const client = createClient(async () => workerTransports(channel, uris), options);
    let changes = Promise.resolve();
    const startupChanges = new Map<string, number>();
    const notify = async (uri: string, type: number) => {
      if (client.isRunning()) await client.sendNotification('workspace/didChangeWatchedFiles', { changes: [{ uri, type }] });
      else startupChanges.set(uri, type);
    };
    const saved = (uri: vscode.Uri, type: number) => {
      changes = changes.catch(() => undefined).then(async () => {
        try {
          await updateSavedFile(channel, uris, uri, type === 3, limits);
          await notify(uri.toString(), type);
        } catch (error) {
          // Never keep a stale saved file when the new source cannot be loaded safely.
          await channel.update(uris.toServer(uri.toString()));
          await notify(uri.toString(), 3);
          log(String(error));
        }
      }).catch((error: unknown) => log(String(error)));
    };
    const watched = new Set<string>();
    for (const project of projects) for (const [root, pattern] of [[project.schemaUri, '**/*.lgs'], [project.localeUri, '**/*.lgl']]) {
      const key = `${root}:${pattern}`;
      if (watched.has(key)) continue;
      watched.add(key);
      const watcher = vscode.workspace.createFileSystemWatcher(new vscode.RelativePattern(vscode.Uri.parse(uris.fromServer(root)), pattern));
      disposables.push(watcher, watcher.onDidCreate((uri) => saved(uri, 1)), watcher.onDidChange((uri) => saved(uri, 2)), watcher.onDidDelete((uri) => saved(uri, 3)));
    }
    return { client, source: 'wasm', started: async () => {
      await changes;
      if (startupChanges.size) {
        await client.sendNotification('workspace/didChangeWatchedFiles', { changes: [...startupChanges].map(([uri, type]) => ({ uri, type })) });
        startupChanges.clear();
      }
    }, dispose: () => { for (const disposable of disposables) disposable.dispose(); channel.dispose(); } };
  } catch (error) {
    for (const disposable of disposables) disposable.dispose();
    channel.dispose();
    throw error;
  }
}
