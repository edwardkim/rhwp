import test from 'node:test';
import assert from 'node:assert/strict';

import { resolvePrintSurfaceUrl } from '../src/command/print-surface.ts';

test('print surface URL은 Studio와 같은 origin의 전용 문서로 해석된다', () => {
  assert.equal(
    resolvePrintSurfaceUrl('https://studio.example.test/app/index.html'),
    'https://studio.example.test/app/print.html',
  );
  assert.equal(
    resolvePrintSurfaceUrl('chrome-extension://abcdefghijklmnop/index.html'),
    'chrome-extension://abcdefghijklmnop/print.html',
  );
});

test('print surface URL은 about:blank를 사용하지 않는다', () => {
  const url = resolvePrintSurfaceUrl('https://studio.example.test/');
  assert.equal(url.startsWith('about:'), false);
  assert.equal(new URL(url).origin, 'https://studio.example.test');
});
