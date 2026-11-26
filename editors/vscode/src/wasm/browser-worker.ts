import { workerDriver } from './driver';
import type { WorkerInput, WorkerOutput } from './protocol';
declare const self: {
  postMessage(message: WorkerOutput): void;
  onmessage: (event: { data: WorkerInput }) => void;
};
const receive = workerDriver((message) => self.postMessage(message));
self.onmessage = (event) => receive(event.data);
