import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';

const fontLoaderSource = await readFile(
  new URL('../src/core/font-loader.ts', import.meta.url),
  'utf8',
);

test('한양중고딕 문서 요청명은 Canvas2D 대체 웹폰트 face로 등록한다', () => {
  assert.match(
    fontLoaderSource,
    /\{ name: '한양중고딕', file: 'fonts\/NotoSansKR-Regular\.woff2' \}/,
  );
});
