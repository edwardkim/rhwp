import assert from 'node:assert/strict';
import { test } from 'node:test';
import { readSse, type SseEvent } from '../src/ai/provider/http.ts';

async function readChunks(chunks: Uint8Array[]): Promise<SseEvent[]> {
  const events: SseEvent[] = [];
  const body = new ReadableStream<Uint8Array>({
    start(controller) {
      for (const chunk of chunks) controller.enqueue(chunk);
      controller.close();
    },
  });
  await readSse(body, (event) => events.push(event));
  return events;
}

test('SSE preserves CRLF event boundaries and UTF-8 across every network split', async () => {
  const bytes = new TextEncoder().encode(
    ': keepalive\r\nevent: response.delta\r\ndata: {"message":\r\ndata: "한글"}\r\n\r\ndata: done\r\n\r\n',
  );
  const expected = [
    { event: 'response.delta', data: '{"message":\n"한글"}' },
    { event: 'message', data: 'done' },
  ];
  for (let split = 0; split <= bytes.length; split++) {
    assert.deepEqual(
      await readChunks([bytes.slice(0, split), bytes.slice(split)]),
      expected,
      `network split at byte ${split}`,
    );
  }
  assert.deepEqual(await readChunks(Array.from(bytes, (byte) => Uint8Array.of(byte))), expected);
});

test('SSE also accepts LF and standalone CR line endings', async () => {
  const encoder = new TextEncoder();
  for (const ending of ['\n', '\r', '\r\n']) {
    const bytes = encoder.encode(`event: delta${ending}data: one${ending}data: two${ending}${ending}`);
    assert.deepEqual(
      await readChunks(Array.from(bytes, (byte) => Uint8Array.of(byte))),
      [{ event: 'delta', data: 'one\ntwo' }],
      `line ending ${JSON.stringify(ending)}`,
    );
  }
});
