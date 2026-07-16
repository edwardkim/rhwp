import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import test from 'node:test';

import { createEditor } from '../npm/editor/index.js';

test('@rhwp/editor public API uses exact-origin MessageChannel v1 binary transport', async (t) => {
  const packageJson = JSON.parse(readFileSync(
    new URL('../npm/editor/package.json', import.meta.url),
    'utf8',
  ));
  assert.deepEqual(packageJson.dependencies ?? {}, {});

  let server;
  let sessionId;
  const requests = [];
  const harness = installDom(t, ({ message, targetOrigin, transfer }) => {
    assert.equal(targetOrigin, 'https://studio.example');
    assert.equal(message.type, 'rhwp-connect');
    assert.equal(message.version, 1);
    assert.deepEqual(message.capabilities, [
      'transferable-array-buffer',
      'hml-export',
      'renderer-diagnostics-v1',
    ]);
    assert.equal(transfer.length, 1);
    sessionId = message.sessionId;
    server = transfer[0];
    server.onmessage = ({ data }) => {
      requests.push(data);
      const result = responseFor(data);
      const response = {
        type: 'rhwp-response', version: 1, sessionId, id: data.id, result,
      };
      const responseTransfer = result instanceof Uint8Array ? [result.buffer] : [];
      server.postMessage(response, responseTransfer);
    };
    server.start();
    server.postMessage({
      type: 'rhwp-connected', version: 1, sessionId,
      capabilities: [
        'transferable-array-buffer',
        'hml-export',
        'renderer-diagnostics-v1',
      ],
    });
  });
  t.after(() => server?.close());

  const editor = await createEditor('#editor', editorOptions());
  assertEditorFrame(harness);
  assert.ok(requests.every((request) => request.type === 'rhwp-request'
    && request.version === 1 && request.sessionId === sessionId));

  assert.equal(await editor.pageCount(), 3);
  assert.equal(await editor.getPageSvg(2), '<svg data-page="2"/>');
  assert.deepEqual(await editor.getRendererDiagnostics(2), rendererDiagnostics(2));
  await assert.rejects(
    () => editor.getRendererDiagnostics('2'),
    /non-negative safe integer/,
  );

  const input = new Uint8Array([1, 2, 3]);
  assert.deepEqual(await editor.loadFile(input, 'sample.hwp'), { pageCount: 3 });
  const loadRequest = requests.find((request) => request.method === 'loadFile');
  assert.ok(ArrayBuffer.isView(loadRequest.params.data));
  assert.equal(Array.isArray(loadRequest.params.data), false);
  assert.deepEqual([...loadRequest.params.data], [1, 2, 3]);
  assert.deepEqual([...input], [1, 2, 3]);

  assert.deepEqual([...await editor.exportHwp()], [4, 5, 6]);
  assert.deepEqual([...await editor.exportHwpx()], [7, 8, 9]);
  assert.deepEqual([...await editor.exportHml()], [10, 11, 12]);
  assert.deepEqual(await editor.getHmlSaveState(), {
    sourceFormat: 'hwp',
    hmlSavable: false,
    blockers: [{
      code: 'HML_SOURCE_REQUIRED', xmlPath: '/HWPML',
      message: 'HML source metadata is required', preserved: false,
    }],
  });
  assert.deepEqual(await editor.exportHwpVerify(), verifyResult());

  editor.destroy();
  assert.equal(harness.iframe.removed, true);
});

test('@rhwp/editor keeps the bounded legacy request/response fallback', async (t) => {
  const requests = [];
  const harness = installDom(t, ({ message, targetOrigin, transfer, dispatch }) => {
    assert.equal(targetOrigin, 'https://studio.example');
    if (message.type === 'rhwp-connect') {
      transfer[0]?.close();
      return;
    }
    requests.push({ message, transfer });
    queueMicrotask(() => dispatch({
      data: { type: 'rhwp-response', id: message.id, result: responseFor(message, true) },
      origin: 'https://studio.example',
      source: harness.contentWindow,
    }));
  });

  const editor = await createEditor('#editor', {
    ...editorOptions(),
    handshakeTimeoutMs: 0,
  });
  assert.equal(await editor.pageCount(), 3);
  await assert.rejects(
    () => editor.getRendererDiagnostics(0),
    /not supported by this Studio/,
  );

  const input = new Uint8Array([1, 2, 3]);
  assert.deepEqual(await editor.loadFile(input, 'legacy.hwp'), { pageCount: 3 });
  const loadRequest = requests.find(({ message }) => message.method === 'loadFile');
  assert.equal('version' in loadRequest.message, false);
  assert.equal('sessionId' in loadRequest.message, false);
  assert.equal(loadRequest.transfer.length, 1);
  assert.ok(ArrayBuffer.isView(loadRequest.message.params.data));
  assert.deepEqual([...input], [1, 2, 3]);
  assert.deepEqual([...await editor.exportHwp()], [4, 5, 6]);

  editor.destroy();
  assert.equal(harness.iframe.removed, true);
});

function installDom(t, postMessage) {
  const originalDocument = globalThis.document;
  const originalWindow = globalThis.window;
  const messageListeners = new Set();
  const contentWindow = {
    postMessage(message, targetOrigin, transfer = []) {
      postMessage({
        message,
        targetOrigin,
        transfer,
        contentWindow,
        dispatch(event) {
          for (const listener of messageListeners) listener(event);
        },
      });
    },
  };
  const iframe = {
    allow: '', contentWindow, removed: false, src: '', style: {},
    addEventListener(type, listener) {
      if (type === 'load') queueMicrotask(listener);
    },
    remove() { this.removed = true; },
  };
  const container = {
    child: null,
    appendChild(child) { this.child = child; },
  };

  globalThis.window = {
    addEventListener(type, listener) {
      if (type === 'message') messageListeners.add(listener);
    },
    removeEventListener(type, listener) {
      if (type === 'message') messageListeners.delete(listener);
    },
  };
  globalThis.document = {
    createElement(tag) {
      assert.equal(tag, 'iframe');
      return iframe;
    },
    querySelector(selector) {
      assert.equal(selector, '#editor');
      return container;
    },
  };
  t.after(() => {
    globalThis.document = originalDocument;
    globalThis.window = originalWindow;
  });
  return { container, contentWindow, iframe };
}

function editorOptions() {
  return {
    studioUrl: 'https://studio.example/app',
    width: '640px',
    height: '480px',
    requestTimeoutMs: 100,
    handshakeTimeoutMs: 100,
  };
}

function assertEditorFrame({ container, iframe }) {
  assert.equal(container.child, iframe);
  assert.equal(iframe.src, 'https://studio.example/app');
  assert.equal(iframe.style.width, '640px');
  assert.equal(iframe.style.height, '480px');
  assert.equal(iframe.allow, 'clipboard-read; clipboard-write');
}

function responseFor(message, legacy = false) {
  switch (message.method) {
    case 'ready': return true;
    case 'pageCount': return 3;
    case 'getPageSvg': return `<svg data-page="${message.params.page}"/>`;
    case 'getRendererDiagnostics': return rendererDiagnostics(message.params.page);
    case 'loadFile': return { pageCount: 3 };
    case 'exportHwp': return legacy ? [4, 5, 6] : new Uint8Array([4, 5, 6]);
    case 'exportHwpx': return legacy ? [7, 8, 9] : new Uint8Array([7, 8, 9]);
    case 'exportHml': return legacy ? [10, 11, 12] : new Uint8Array([10, 11, 12]);
    case 'getHmlSaveState': return {
      sourceFormat: 'hwp',
      hmlSavable: false,
      blockers: [{
        code: 'HML_SOURCE_REQUIRED', xmlPath: '/HWPML',
        message: 'HML source metadata is required', preserved: false,
      }],
    };
    case 'exportHwpVerify': return verifyResult();
    default: throw new Error(`Unexpected method: ${message.method}`);
  }
}

function rendererDiagnostics(page) {
  return {
    schemaVersion: 1,
    request: null,
    initialized: true,
    initializationError: null,
    effectiveBackend: 'canvaskit',
    backendFallbackReason: null,
    page: { index: page, canvaskit: null },
  };
}

function verifyResult() {
  return {
    bytesLen: 3,
    pageCountBefore: 3,
    pageCountAfter: 3,
    recovered: false,
  };
}
