import test from 'node:test';
import assert from 'node:assert/strict';

import { EditorTransport } from '../transport.js';

test('a correlated malformed v1 response fails explicitly instead of timing out', async () => {
  const contentWindow = {
    postMessage(connect, _origin, [server]) {
      server.onmessage = ({ data }) => server.postMessage({
        type: 'rhwp-response', version: 1, sessionId: data.sessionId, id: data.id,
        result: 1, error: { code: 'RPC_ERROR', message: 'both result and error' },
      });
      server.start();
      server.postMessage({
        type: 'rhwp-connected', version: 1, sessionId: connect.sessionId,
        capabilities: ['transferable-array-buffer'],
      });
    },
  };
  const transport = new EditorTransport(
    { contentWindow },
    'https://studio.example/app',
    { window: { addEventListener() {}, removeEventListener() {} }, requestTimeoutMs: 1_000 },
  );

  try {
    await transport.connect();
    await assert.rejects(
      Promise.race([
        transport.request('pageCount'),
        new Promise((_, reject) => setTimeout(
          () => reject(new Error('malformed response was not explicitly rejected')),
          50,
        )),
      ]),
      (error) => typeof error?.code === 'string' && /invalid|malformed/i.test(error.message),
    );
  } finally {
    transport.destroy();
  }
});
