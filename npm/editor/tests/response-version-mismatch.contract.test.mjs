import test from 'node:test';
import assert from 'node:assert/strict';

import { EditorTransport } from '../transport.js';

test('a correlated response protocol mismatch fails explicitly instead of timing out', async () => {
  const contentWindow = {
    postMessage(connect, _origin, [server]) {
      server.onmessage = ({ data }) => server.postMessage({
        type: 'rhwp-response', version: 2, sessionId: data.sessionId, id: data.id,
        error: {
          code: 'UNSUPPORTED_VERSION', message: 'Unsupported embed protocol version: 2',
          supportedVersions: [1],
        },
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
          () => reject(new Error('protocol mismatch was not explicitly rejected')),
          50,
        )),
      ]),
      (error) => error?.code === 'UNSUPPORTED_VERSION',
    );
  } finally {
    transport.destroy();
  }
});
