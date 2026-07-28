import assert from 'node:assert/strict';
import test from 'node:test';
import { DebouncedSerialTask } from './restart';

test('debounces pending restarts', async () => {
  let calls = 0;
  const queue = new DebouncedSerialTask(async () => {
    calls += 1;
  });

  await Promise.all([queue.schedule(0), queue.schedule(0), queue.schedule(0)]);

  assert.equal(calls, 1);
});

test('serializes a change arriving during restart', async () => {
  let calls = 0;
  let release: () => void = () => undefined;
  let started: () => void = () => undefined;
  const firstPending = new Promise<void>((resolve) => {
    release = resolve;
  });
  const firstStarted = new Promise<void>((resolve) => {
    started = resolve;
  });
  const queue = new DebouncedSerialTask(async () => {
    calls += 1;
    if (calls === 1) {
      started();
      await firstPending;
    }
  });

  const first = queue.runNow();
  await firstStarted;
  const second = queue.runNow();
  release();
  await Promise.all([first, second]);

  assert.equal(calls, 2);
});

test('a failed restart does not poison the queue', async () => {
  let calls = 0;
  const queue = new DebouncedSerialTask(async () => {
    calls += 1;
    if (calls === 1) {
      throw new Error('first failed');
    }
  });

  await assert.rejects(queue.runNow(), /first failed/);
  await queue.runNow();
  assert.equal(calls, 2);
});
