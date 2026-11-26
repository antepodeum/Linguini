import type { WorkerEndpoint } from './protocol';
export function browserEndpoint(uri: string): WorkerEndpoint {
  const worker = new Worker(uri, { type: 'module', name: 'Linguini language server' });
  return {
    post: (message) => worker.postMessage(message),
    onMessage(listener) {
      const receive = (event: MessageEvent) => listener(event.data);
      worker.addEventListener('message', receive);
      return { dispose: () => worker.removeEventListener('message', receive) };
    },
    onError(listener) {
      const receive = (event: ErrorEvent) => listener(new Error(event.message));
      const decode = () => listener(new Error('language worker message could not be decoded'));
      worker.addEventListener('error', receive);
      worker.addEventListener('messageerror', decode);
      return { dispose: () => { worker.removeEventListener('error', receive); worker.removeEventListener('messageerror', decode); } };
    },
    terminate: () => worker.terminate()
  };
}
