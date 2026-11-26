export interface SavedFile { uri: string; text: string }
export interface SourceRoots { configUri: string; schemaUri: string; localeUri: string }
export interface SourcePlan { roots: SourceRoots[]; errors: string[] }
export interface DiscoveryOptions { ignoredDirectories: string[]; maxEntries: number; maxDepth: number; maxRoots: number; maxFileBytes: number; maxBytes: number }
export interface WorkerServer {
  receive(message: string): void;
  replace_files(files: string): void;
  update_file(uri: string, text?: string): void;
  source_plan(roots: string): string;
  free(): void;
}
export interface WasmModule {
  default(options: { module_or_path: Uint8Array }): Promise<unknown>;
  WorkerServer: new (send: (message: string) => void) => WorkerServer;
  discovery_options(): string;
}
export type WorkerInput =
  | { kind: 'start'; moduleUri: string; bytes: Uint8Array }
  | { kind: 'lsp'; message: unknown }
  | { kind: 'replace'; id: number; files: SavedFile[] }
  | { kind: 'roots'; id: number; roots: string[] }
  | { kind: 'update'; id: number; uri: string; text?: string };
export type WorkerOutput =
  | { kind: 'ready'; options: DiscoveryOptions }
  | { kind: 'lsp'; message: unknown }
  | { kind: 'result'; id: number; result?: unknown; error?: string }
  | { kind: 'error'; error: string };

export interface WorkerEndpoint {
  post(message: WorkerInput): void;
  onMessage(listener: (message: WorkerOutput) => void): { dispose(): void };
  onError(listener: (error: Error) => void): { dispose(): void };
  terminate(): void;
}
