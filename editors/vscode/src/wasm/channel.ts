import type { DiscoveryOptions, SavedFile, SourcePlan, WorkerEndpoint, WorkerInput } from './protocol';

/** Separates host-file commands from JSON-RPC while preserving message order. */
export class WorkerChannel {
  private nextId = 0;
  private pending = new Map<number, { resolve(value: unknown): void; reject(error: Error): void }>();
  private lspListeners = new Set<(message: unknown) => void>();
  private errorListeners = new Set<(error: Error) => void>();
  private subscriptions: Array<{ dispose(): void }>;
  private disposed = false;
  private rejectReady!: (error: Error) => void;
  public readonly ready: Promise<DiscoveryOptions>;

  public constructor(private readonly endpoint: WorkerEndpoint, moduleUri: string, bytes: Uint8Array) {
    let resolveReady!: (options: DiscoveryOptions) => void;
    const timer = setTimeout(() => this.fail(new Error('language worker startup timed out')), 30_000);
    this.ready = new Promise((resolve, reject) => { resolveReady = resolve; this.rejectReady = reject; });
    this.subscriptions = [
      endpoint.onMessage((message) => {
        switch (message.kind) {
          case 'ready': clearTimeout(timer); resolveReady(message.options); break;
          case 'result': {
            const pending = this.pending.get(message.id);
            this.pending.delete(message.id);
            if (message.error !== undefined) pending?.reject(new Error(message.error));
            else pending?.resolve(message.result);
            break;
          }
          case 'lsp': for (const listener of this.lspListeners) listener(message.message); break;
          case 'error': this.fail(new Error(message.error)); break;
        }
      }),
      endpoint.onError((error) => this.fail(error)),
      { dispose: () => { clearTimeout(timer); this.rejectReady(new Error('language worker stopped')); } }
    ];
    try { endpoint.post({ kind: 'start', moduleUri, bytes }); } catch (error) { this.fail(error instanceof Error ? error : new Error(String(error))); }
  }

  public onMessage(listener: (message: unknown) => void): { dispose(): void } {
    this.lspListeners.add(listener);
    return { dispose: () => { this.lspListeners.delete(listener); } };
  }
  public onError(listener: (error: Error) => void): { dispose(): void } {
    this.errorListeners.add(listener);
    return { dispose: () => { this.errorListeners.delete(listener); } };
  }
  public send(message: unknown): void {
    if (this.disposed) throw new Error('language worker stopped');
    this.endpoint.post({ kind: 'lsp', message });
  }
  public async replace(files: SavedFile[]): Promise<void> { await this.command({ kind: 'replace', id: ++this.nextId, files }); }
  public async update(uri: string, text?: string): Promise<void> { await this.command({ kind: 'update', id: ++this.nextId, uri, text }); }
  public async sourcePlan(roots: string[]): Promise<SourcePlan> {
    return await this.command({ kind: 'roots', id: ++this.nextId, roots }) as SourcePlan;
  }
  public dispose(): void {
    if (this.disposed) return;
    this.disposed = true;
    for (const subscription of this.subscriptions) subscription.dispose();
    const error = new Error('language worker stopped');
    for (const pending of this.pending.values()) pending.reject(error);
    this.pending.clear();
    this.lspListeners.clear();
    this.errorListeners.clear();
    this.endpoint.terminate();
  }
  private command(message: Exclude<WorkerInput, { kind: 'start' | 'lsp' }>): Promise<unknown> {
    if (this.disposed) return Promise.reject(new Error('language worker stopped'));
    return new Promise((resolve, reject) => {
      this.pending.set(message.id, { resolve, reject });
      try { this.endpoint.post(message); }
      catch (error) { this.pending.delete(message.id); reject(error); }
    });
  }
  private fail(error: Error): void {
    if (this.disposed) return;
    this.rejectReady(error);
    for (const listener of this.errorListeners) listener(error);
    for (const pending of this.pending.values()) pending.reject(error);
    this.dispose();
  }
}
