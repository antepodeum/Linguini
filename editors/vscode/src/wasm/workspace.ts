import * as vscode from 'vscode';
import type { WorkerChannel } from './channel';
import type { DiscoveryOptions } from './protocol';
import { discoverWorkspace, type FileKind, type WorkspaceFilesystem } from './snapshot';
import { WorkspaceUris } from './uris';

function kind(type: vscode.FileType): FileKind {
  if (type & vscode.FileType.SymbolicLink) return 'symlink';
  if (type & vscode.FileType.Directory) return 'directory';
  if (type & vscode.FileType.File) return 'file';
  return 'unknown';
}
const filesystem: WorkspaceFilesystem = {
  async stat(uri) { const info = await vscode.workspace.fs.stat(vscode.Uri.parse(uri)); return { kind: kind(info.type), size: info.size }; },
  async readFile(uri) { return vscode.workspace.fs.readFile(vscode.Uri.parse(uri)); },
  async readDirectory(uri) { return (await vscode.workspace.fs.readDirectory(vscode.Uri.parse(uri))).map(([name, type]) => [name, kind(type)]); },
  join(directory, name) { return vscode.Uri.joinPath(vscode.Uri.parse(directory), name).toString(); }
};

export async function loadWorkspace(channel: WorkerChannel, uris: WorkspaceUris, roots: readonly vscode.Uri[], log: (message: string) => void) {
  return discoverWorkspace(filesystem, channel, uris, roots.map((root) => root.toString()), log);
}

export async function updateSavedFile(channel: WorkerChannel, uris: WorkspaceUris, uri: vscode.Uri, deleted: boolean, limits: DiscoveryOptions): Promise<void> {
  const mapped = uris.toServer(uri.toString());
  if (deleted) { await channel.update(mapped); return; }
  const folder = vscode.workspace.getWorkspaceFolder(uri);
  if (!folder) throw new Error('saved file is outside the workspace');
  const relative = uri.path.slice(folder.uri.path.replace(/\/$/, '').length).split('/').filter(Boolean);
  let current = folder.uri;
  for (const name of relative) {
    current = vscode.Uri.joinPath(current, name);
    const metadata = await vscode.workspace.fs.stat(current);
    if (metadata.type & vscode.FileType.SymbolicLink) throw new Error(`workspace source path contains a symlink: ${current.toString()}`);
  }
  const metadata = await vscode.workspace.fs.stat(uri);
  if (metadata.type & vscode.FileType.SymbolicLink) throw new Error(`workspace source is a symlink: ${uri.toString()}`);
  if (metadata.size > limits.maxFileBytes) throw new Error('workspace source exceeds the language server file budget');
  const bytes = await vscode.workspace.fs.readFile(uri);
  if (bytes.byteLength > limits.maxFileBytes) throw new Error('workspace source exceeds the language server file budget');
  await channel.update(mapped, new TextDecoder('utf-8', { fatal: true }).decode(bytes));
}
