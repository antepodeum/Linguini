import type { WorkerChannel } from './channel';
import type { SavedFile, SourceRoots } from './protocol';
import type { WorkspaceUris } from './uris';

export type FileKind = 'file' | 'directory' | 'symlink' | 'unknown';
export interface WorkspaceFilesystem {
  stat(uri: string): Promise<{ kind: FileKind; size: number }>;
  readFile(uri: string): Promise<Uint8Array>;
  readDirectory(uri: string): Promise<Array<[string, FileKind]>>;
  join(directory: string, name: string): string;
}
type SnapshotChannel = Pick<WorkerChannel, 'ready' | 'replace' | 'sourcePlan'>;

/** Only filesystem work lives here. Rust selects and parses projects and source namespaces. */
export async function discoverWorkspace(fs: WorkspaceFilesystem, channel: SnapshotChannel, uris: WorkspaceUris, roots: readonly string[], log: (message: string) => void): Promise<SourceRoots[]> {
  const options = await channel.ready;
  const files = new Map<string, SavedFile>();
  const visited = new Set<string>();
  let entries = 0;
  let bytes = 0;
  async function read(uri: string): Promise<void> {
    const key = uri;
    if (files.has(key)) return;
    const metadata = await fs.stat(uri);
    if (metadata.kind === 'symlink') throw new Error(`workspace source is a symlink: ${uri}`);
    if (metadata.size > options.maxFileBytes) throw new Error(`workspace file exceeds ${options.maxFileBytes} bytes: ${key}`);
    const content = await fs.readFile(uri);
    if (content.byteLength > options.maxFileBytes || bytes + content.byteLength > options.maxBytes) {
      throw new Error('workspace files exceed the language server byte budget');
    }
    bytes += content.byteLength;
    files.set(key, { uri: uris.toServer(key), text: new TextDecoder('utf-8', { fatal: true }).decode(content) });
  }
  async function walk(directory: string, mode: 'config' | 'schema' | 'locale', depth = 0): Promise<void> {
    if (depth > options.maxDepth) throw new Error(`workspace discovery depth exceeds ${options.maxDepth}`);
    const key = `${mode}:${directory}`;
    if (visited.has(key)) return;
    visited.add(key);
    const metadata = await fs.stat(directory);
    if (metadata.kind === 'symlink') throw new Error(`workspace directory is a symlink: ${directory}`);
    for (const [name, type] of await fs.readDirectory(directory)) {
      if (++entries > options.maxEntries) throw new Error(`workspace discovery exceeds ${options.maxEntries} entries`);
      if (mode === 'config' && options.ignoredDirectories.includes(name)) continue;
      const uri = fs.join(directory, name);
      if (type === 'symlink') {
        if (mode === 'config') { log(`workspace discovery skipped symlink ${uri}`); continue; }
        throw new Error(`workspace source is a symlink: ${uri}`);
      }
      if (type === 'directory') {
        if (mode === 'config') {
          try { await walk(uri, mode, depth + 1); }
          catch (error) { if (entries > options.maxEntries) throw error; log(String(error)); }
        } else await walk(uri, mode, depth + 1);
      } else if (type === 'file' && (mode === 'config' ? name === 'linguini.toml' : name.endsWith(mode === 'schema' ? '.lgs' : '.lgl'))) {
        if (mode === 'config') {
          try { await read(uri); } catch (error) { log(String(error)); }
        } else await read(uri);
      }
    }
  }
  async function checkConfiguredPath(configUri: string, sourceUri: string): Promise<void> {
    const project = new URL('.', configUri);
    const source = new URL(sourceUri);
    const prefix = project.pathname.replace(/\/$/, '');
    if (source.protocol !== project.protocol || source.host !== project.host ||
      !(source.pathname === prefix || source.pathname.startsWith(prefix + '/'))) throw new Error('configured source root is outside its project');
    const parts = source.pathname.slice(prefix.length).split('/').filter(Boolean);
    if (parts.length > options.maxDepth) throw new Error('configured source path exceeds discovery depth');
    for (const part of parts) {
      project.pathname = project.pathname.replace(/\/$/, '') + '/' + part;
      if ((await fs.stat(uris.fromServer(project.href))).kind === 'symlink') throw new Error(`configured source path contains a symlink: ${uris.fromServer(project.href)}`);
    }
  }
  if (roots.length > options.maxRoots) throw new Error(`workspace exposes more than ${options.maxRoots} roots`);
  for (const root of roots) await walk(root, 'config');
  await channel.replace([...files.values()]);
  const plan = await channel.sourcePlan(roots.map((root) => uris.toServer(root)));
  for (const error of plan.errors) log(error);
  // Sources under explicitly configured roots are not filtered by project-discovery exclusions.
  entries = 0;
  for (const project of plan.roots) {
    for (const [root, mode] of [[project.schemaUri, 'schema'], [project.localeUri, 'locale']] as const) {
      const previousFiles = new Set(files.keys());
      const previousVisits = new Set(visited);
      const previousBytes = bytes;
      try { await checkConfiguredPath(project.configUri, root); await walk(uris.fromServer(root), mode); }
      catch (error) {
        for (const key of files.keys()) if (!previousFiles.has(key)) files.delete(key);
        for (const key of visited) if (!previousVisits.has(key)) visited.delete(key);
        bytes = previousBytes;
        log(String(error));
      }
    }
  }
  await channel.replace([...files.values()]);
  return plan.roots;
}
