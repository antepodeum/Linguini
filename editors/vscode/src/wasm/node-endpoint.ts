import { Worker } from 'node:worker_threads';
import type { WorkerEndpoint } from './protocol';
export function nodeEndpoint(filename: string): WorkerEndpoint {
  const worker = new Worker(filename);
  return {
    post: (message) => worker.postMessage(message),
    onMessage(listener) {
      worker.on('message', listener);
      return { dispose: () => { worker.off('message', listener); } };
    },
    onError(listener) {
      worker.on('error', listener);
      const onExit = (code: number) => listener(new Error(`language worker exited (${code})`));
      worker.on('exit', onExit);
      return { dispose: () => { worker.off('error', listener); worker.off('exit', onExit); } };
    },
    terminate: () => { void worker.terminate(); }
  };
}
