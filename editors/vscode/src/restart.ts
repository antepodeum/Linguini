export class DebouncedSerialTask {
  private timer: ReturnType<typeof setTimeout> | undefined;
  private chain: Promise<void> = Promise.resolve();
  private waiters: Array<{
    resolve: () => void;
    reject: (error: unknown) => void;
  }> = [];
  private disposed = false;

  public constructor(private readonly task: () => Promise<void>) {}

  public schedule(delayMs: number): Promise<void> {
    if (this.disposed) {
      return Promise.reject(new Error('restart queue is disposed'));
    }

    const promise = new Promise<void>((resolve, reject) => {
      this.waiters.push({ resolve, reject });
    });
    if (this.timer !== undefined) {
      clearTimeout(this.timer);
    }
    this.timer = setTimeout(() => {
      this.timer = undefined;
      this.launch();
    }, Math.max(0, delayMs));
    return promise;
  }

  public runNow(): Promise<void> {
    if (this.disposed) {
      return Promise.reject(new Error('restart queue is disposed'));
    }
    if (this.timer !== undefined) {
      clearTimeout(this.timer);
      this.timer = undefined;
    }
    const promise = new Promise<void>((resolve, reject) => {
      this.waiters.push({ resolve, reject });
    });
    this.launch();
    return promise;
  }

  public async flush(): Promise<void> {
    if (this.timer !== undefined) {
      clearTimeout(this.timer);
      this.timer = undefined;
      this.launch();
    }
    await this.chain;
  }

  public dispose(): void {
    this.disposed = true;
    if (this.timer !== undefined) {
      clearTimeout(this.timer);
      this.timer = undefined;
    }
    const error = new Error('restart queue is disposed');
    for (const waiter of this.waiters.splice(0)) {
      waiter.reject(error);
    }
  }

  private launch(): void {
    const waiters = this.waiters.splice(0);
    if (waiters.length === 0) {
      return;
    }
    const execution = this.chain.catch(() => undefined).then(this.task);
    this.chain = execution.catch(() => undefined);
    void execution.then(
      () => {
        for (const waiter of waiters) {
          waiter.resolve();
        }
      },
      (error) => {
        for (const waiter of waiters) {
          waiter.reject(error);
        }
      }
    );
  }
}
