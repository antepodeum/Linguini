import { parentPort } from 'node:worker_threads';
import { workerDriver } from './driver';
if (!parentPort) throw new Error('language server requires a worker thread');
const port = parentPort;
port.on('message', workerDriver((message) => port.postMessage(message)));
