const uriFields = new Set(['uri', 'oldUri', 'newUri', 'targetUri', 'rootUri', 'baseUri']);

/** Stable synthetic roots let one WASM engine handle file, remote, and virtual workspaces. */
export class WorkspaceUris {
  private roots: Array<{ external: URL; internal: URL }>;
  public constructor(roots: string[]) {
    this.roots = roots.map((uri, index) => ({ external: new URL(uri), internal: new URL(`file:///linguini-workspace/${index}`) }));
    for (const root of this.roots) {
      if (root.external.search || root.external.hash) throw new Error('workspace root URI cannot have query or fragment');
      root.external.pathname = root.external.pathname.replace(/\/$/, '');
    }
    this.roots.sort((left, right) => right.external.pathname.length - left.external.pathname.length);
  }
  public toServer(uri: string): string { return this.map(uri, true); }
  public fromServer(uri: string): string { return this.map(uri, false); }
  public message<T>(value: T, toServer: boolean): T {
    const mapUri = (uri: string) => toServer ? this.toServer(uri) : this.fromServer(uri);
    const walk = (value: unknown, field?: string): unknown => {
      if (typeof value === 'string') return field && uriFields.has(field) ? mapUri(value) : value;
      if (Array.isArray(value)) return value.map((item) => walk(item));
      if (value && typeof value === 'object') return Object.fromEntries(Object.entries(value).map(([key, item]) => [
        field === 'changes' || field === 'relatedDocuments' ? mapUri(key) : key, walk(item, key)
      ]));
      return value;
    };
    return walk(value) as T;
  }
  private map(uri: string, toServer: boolean): string {
    let parsed: URL;
    try { parsed = new URL(uri); } catch { return uri; }
    for (const root of this.roots) {
      const source = toServer ? root.external : root.internal;
      const target = toServer ? root.internal : root.external;
      const sourcePath = source.pathname.replace(/\/$/, '');
      const targetPath = target.pathname.replace(/\/$/, '');
      if (parsed.protocol !== source.protocol || parsed.host !== source.host ||
        !(parsed.pathname === sourcePath || parsed.pathname.startsWith(sourcePath + '/'))) continue;
      const result = new URL(target.href);
      result.pathname = targetPath + parsed.pathname.slice(sourcePath.length);
      result.search = parsed.search;
      result.hash = parsed.hash;
      return result.href;
    }
    return uri;
  }
}
