import { AbstractMessageReader, AbstractMessageWriter } from 'vscode-languageclient';
import type { DataCallback, Message, MessageTransports } from 'vscode-languageclient';
import type { WorkerChannel } from './channel';
import type { WorkspaceUris } from './uris';

export function workerTransports(channel: WorkerChannel, uris: WorkspaceUris): MessageTransports {
  class Reader extends AbstractMessageReader {
    private listener: { dispose(): void } | undefined;
    private errors = channel.onError((error) => { this.fireError(error); this.fireClose(); });
    public listen(callback: DataCallback): { dispose(): void } {
      this.listener = channel.onMessage((message) => callback(uris.message(message, false) as Message));
      return this.listener;
    }
    public override dispose(): void { this.listener?.dispose(); this.errors.dispose(); super.dispose(); }
  }
  class Writer extends AbstractMessageWriter {
    private errors = channel.onError((error) => { this.fireError(error); this.fireClose(); });
    public async write(message: Message): Promise<void> { channel.send(uris.message(message, true)); }
    public end(): void { channel.dispose(); }
    public override dispose(): void { this.errors.dispose(); super.dispose(); }
  }
  return { reader: new Reader(), writer: new Writer() };
}
