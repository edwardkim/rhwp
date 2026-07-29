import test from 'node:test';
import assert from 'node:assert/strict';

import { readBoundedResponseArrayBuffer } from '../src/view/canvaskit/bounded-response.ts';

function responseWithStream(
  chunks: readonly Uint8Array[],
  contentLength?: string,
  onCancel: (reason: unknown) => void = () => {},
): Response {
  const stream = new ReadableStream<Uint8Array>({
    start(controller) {
      for (const chunk of chunks) controller.enqueue(chunk);
      controller.close();
    },
    cancel: onCancel,
  });
  return {
    body: stream,
    headers: new Headers(contentLength === undefined ? {} : { 'content-length': contentLength }),
  } as Response;
}

test('bounded response reads missing and underreported Content-Length by actual stream bytes', async () => {
  for (const contentLength of [undefined, '1', 'not-a-number']) {
    const bytes = await readBoundedResponseArrayBuffer(
      responseWithStream([new Uint8Array([1, 2]), new Uint8Array([3, 4, 5])], contentLength),
      { maxBytes: 5 },
    );
    assert.deepEqual([...new Uint8Array(bytes)], [1, 2, 3, 4, 5]);
  }
});

test('bounded response cancels the stream as soon as actual bytes exceed the cap', async () => {
  let cancelReason: unknown = null;
  const chunks = [new Uint8Array([1, 2, 3]), new Uint8Array([4, 5, 6])];
  const reader = {
    read: async () => ({ done: false as const, value: chunks.shift()! }),
    cancel: async (reason: unknown) => { cancelReason = reason; },
    releaseLock: () => {},
  };
  const response = {
    body: { getReader: () => reader },
    headers: new Headers({ 'content-length': '1' }),
  } as unknown as Response;

  await assert.rejects(
    readBoundedResponseArrayBuffer(response, { maxBytes: 5 }),
    /exceeds 5 bytes/,
  );
  assert.match(String(cancelReason), /exceeds 5 bytes/);
});

test('bounded response aborts a pending stream read promptly', async () => {
  const controller = new AbortController();
  let finishRead: ((result: ReadableStreamReadResult<Uint8Array>) => void) | null = null;
  let cancelCalls = 0;
  const reader = {
    read: () => new Promise<ReadableStreamReadResult<Uint8Array>>((resolve) => {
      finishRead = resolve;
    }),
    cancel: async () => {
      cancelCalls += 1;
      finishRead?.({ done: true, value: undefined });
    },
    releaseLock: () => {},
  };
  const response = {
    body: { getReader: () => reader },
    headers: new Headers(),
  } as unknown as Response;

  const read = readBoundedResponseArrayBuffer(response, {
    maxBytes: 5,
    signal: controller.signal,
    cancelledMessage: 'stale document request',
  });
  while (!finishRead) await Promise.resolve();
  controller.abort();

  await assert.rejects(read, /stale document request/);
  assert.equal(cancelCalls, 1);
});

test('bounded response rejects a non-streaming mock without calling arrayBuffer', async () => {
  let arrayBufferCalls = 0;
  const response = {
    body: null,
    headers: new Headers({ 'content-length': '4' }),
    arrayBuffer: async () => {
      arrayBufferCalls += 1;
      return new Uint8Array([1, 2, 3, 4]).buffer;
    },
  } as unknown as Response;

  await assert.rejects(
    readBoundedResponseArrayBuffer(response, { maxBytes: 5 }),
    /stream is unavailable/,
  );
  assert.equal(arrayBufferCalls, 0);
});
