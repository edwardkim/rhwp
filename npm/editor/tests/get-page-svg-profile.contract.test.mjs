import test from 'node:test';
import assert from 'node:assert/strict';

import { RhwpEditor } from '../index.js';

test('getPageSvg keeps the legacy envelope and adds profile only when given', async () => {
  const requests = [];
  const transport = {
    request(method, params) {
      requests.push({ method, params });
      return Promise.resolve('<svg/>');
    },
  };
  const editor = new RhwpEditor({}, transport);

  await editor.getPageSvg();
  await editor.getPageSvg(3);
  await editor.getPageSvg(3, {});
  await editor.getPageSvg(3, { profile: 'print' });

  assert.deepEqual(requests, [
    { method: 'getPageSvg', params: { page: 0 } },
    { method: 'getPageSvg', params: { page: 3 } },
    { method: 'getPageSvg', params: { page: 3 } },
    { method: 'getPageSvg', params: { page: 3, profile: 'print' } },
  ]);
});
