import * as path from 'node:path';

export interface ServerResolutionOptions {
  explicitPath?: string;
  useWorkspaceCli: boolean;
  workspaceFolders: string[];
  workspaceFile?: string;
  extensionPath: string;
  platform: NodeJS.Platform;
  exists(path: string): boolean;
}

export interface ResolvedServer {
  command: string;
  source: 'explicit' | 'workspace' | 'bundled' | 'path';
  cwd?: string;
}

export function resolveServer(options: ServerResolutionOptions): ResolvedServer {
  const cwd = workspaceCwd(options.workspaceFolders, options.workspaceFile);
  const explicit = options.explicitPath?.trim();
  if (explicit) {
    return {
      command: expandWorkspaceFolder(explicit, options.workspaceFolders),
      source: 'explicit',
      cwd
    };
  }

  if (options.useWorkspaceCli && options.workspaceFolders.length === 1) {
    const executable = options.platform === 'win32' ? 'linguini.cmd' : 'linguini';
    const local = path.join(options.workspaceFolders[0], 'node_modules', '.bin', executable);
    if (options.exists(local)) {
      return { command: local, source: 'workspace', cwd };
    }
  }

  const executable = options.platform === 'win32' ? 'linguini.exe' : 'linguini';
  const bundled = path.join(options.extensionPath, 'dist', 'server', executable);
  if (options.exists(bundled)) {
    return { command: bundled, source: 'bundled', cwd };
  }

  return { command: 'linguini', source: 'path', cwd };
}

export function expandServerArgument(value: string, workspaceFolders: string[]): string {
  if (value.includes('${file}') || value.includes('${languageId}')) {
    throw new Error(
      '`${file}` and `${languageId}` cannot be used to launch a workspace language server'
    );
  }
  return expandWorkspaceFolder(value, workspaceFolders);
}

export function workspaceCwd(
  workspaceFolders: string[],
  workspaceFile?: string
): string | undefined {
  if (workspaceFolders.length === 1) {
    return workspaceFolders[0];
  }
  return workspaceFile ? path.dirname(workspaceFile) : undefined;
}

function expandWorkspaceFolder(value: string, workspaceFolders: string[]): string {
  if (!value.includes('${workspaceFolder}')) {
    return value;
  }
  if (workspaceFolders.length !== 1) {
    throw new Error(
      '`${workspaceFolder}` is ambiguous in a multi-root workspace; use an absolute server path'
    );
  }
  return value.replaceAll('${workspaceFolder}', workspaceFolders[0]);
}
