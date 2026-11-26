import type { WasmModule, WorkerInput, WorkerOutput, WorkerServer } from './protocol';

/** Runtime-neutral worker entry: transport only, with no language or configuration parser. */
export function workerDriver(post: (message: WorkerOutput) => void): (message: WorkerInput) => void {
  let server: WorkerServer | undefined;
  let starting = false;
  async function start(message: Extract<WorkerInput, { kind: 'start' }>): Promise<void> {
    if (starting) throw new Error('language worker already started');
    starting = true;
    const wasm: WasmModule = await import(message.moduleUri);
    await wasm.default({ module_or_path: message.bytes });
    server = new wasm.WorkerServer((message) => post({ kind: 'lsp', message: JSON.parse(message) }));
    post({ kind: 'ready', options: JSON.parse(wasm.discovery_options()) });
  }
  return (message) => {
    if (message.kind === 'start') {
      void start(message).catch((error: unknown) => post({ kind: 'error', error: String(error) }));
      return;
    }
    try {
      if (!server) throw new Error('language worker is not ready');
      if (message.kind === 'lsp') { server.receive(JSON.stringify(message.message)); return; }
      let result: unknown;
      switch (message.kind) {
        case 'replace': server.replace_files(JSON.stringify(message.files)); break;
        case 'roots': result = JSON.parse(server.source_plan(JSON.stringify(message.roots))); break;
        case 'update': server.update_file(message.uri, message.text); break;
      }
      post({ kind: 'result', id: message.id, result });
    } catch (error) {
      if ('id' in message) post({ kind: 'result', id: message.id, error: String(error) });
      else post({ kind: 'error', error: String(error) });
    }
  };
}
