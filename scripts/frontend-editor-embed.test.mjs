import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import test from 'node:test';

import { createEditor } from '../npm/editor/index.js';

test('@rhwp/editor iframe message contract', async (t) => {
  const packageJson = JSON.parse(readFileSync(new URL('../npm/editor/package.json', import.meta.url), 'utf8'));
  assert.deepEqual(packageJson.dependencies ?? {}, {});

  const originalDocument = globalThis.document;
  const originalWindow = globalThis.window;
  const originalSetTimeout = globalThis.setTimeout;
  const requests = [];
  const messageListeners = new Set();

  const contentWindow = {
    postMessage(message, targetOrigin) {
      requests.push({ message, targetOrigin });
      const result = responseFor(message);
      queueMicrotask(() => {
        for (const listener of messageListeners) {
          listener({
            data: { type: 'rhwp-response', id: message.id, result },
            origin: 'https://studio.example',
            source: contentWindow,
          });
        }
      });
    },
  };

  const iframe = {
    allow: '',
    contentWindow,
    removed: false,
    src: '',
    style: {},
    addEventListener(type, listener) {
      if (type === 'load') queueMicrotask(listener);
    },
    remove() {
      this.removed = true;
    },
  };
  const container = {
    child: null,
    appendChild(child) {
      this.child = child;
    },
  };

  globalThis.window = {
    addEventListener(type, listener) {
      if (type === 'message') messageListeners.add(listener);
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
  globalThis.setTimeout = () => 0;

  t.after(() => {
    globalThis.document = originalDocument;
    globalThis.window = originalWindow;
    globalThis.setTimeout = originalSetTimeout;
  });

  const editor = await createEditor('#editor', {
    studioUrl: 'https://studio.example/',
    width: '640px',
    height: '480px',
  });

  assert.equal(container.child, iframe);
  assert.equal(iframe.src, 'https://studio.example/');
  assert.equal(iframe.style.width, '640px');
  assert.equal(iframe.style.height, '480px');
  assert.equal(iframe.allow, 'clipboard-read; clipboard-write');
  assert.equal(requests[0].message.method, 'ready');
  assert.equal(requests[0].targetOrigin, '*');

  assert.equal(await editor.pageCount(), 3);
  assert.equal(await editor.getPageSvg(2), '<svg data-page="2"/>');

  const input = new Uint8Array([1, 2, 3]);
  assert.deepEqual(await editor.loadFile(input, 'sample.hwp'), { pageCount: 3 });
  const loadRequest = requests.find(({ message }) => message.method === 'loadFile').message;
  assert.deepEqual(loadRequest.params, { data: [1, 2, 3], fileName: 'sample.hwp' });

  assert.deepEqual([...await editor.exportHwp()], [4, 5, 6]);
  assert.deepEqual([...await editor.exportHwpx()], [7, 8, 9]);
  assert.deepEqual(await editor.exportHwpVerify(), {
    bytesLen: 3,
    pageCountBefore: 3,
    pageCountAfter: 3,
    recovered: false,
  });

  editor.destroy();
  assert.equal(iframe.removed, true);
});

function responseFor(message) {
  switch (message.method) {
    case 'ready':
      return true;
    case 'pageCount':
      return 3;
    case 'getPageSvg':
      return `<svg data-page="${message.params.page}"/>`;
    case 'loadFile':
      return { pageCount: 3 };
    case 'exportHwp':
      return [4, 5, 6];
    case 'exportHwpx':
      return [7, 8, 9];
    case 'exportHwpVerify':
      return {
        bytesLen: 3,
        pageCountBefore: 3,
        pageCountAfter: 3,
        recovered: false,
      };
    default:
      throw new Error(`Unexpected method: ${message.method}`);
  }
}
