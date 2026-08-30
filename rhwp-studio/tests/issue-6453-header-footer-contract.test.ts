import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { createServer } from 'vite';
import { functionBodyFrom } from './support/source-guard.ts';

const rootDir = dirname(dirname(fileURLToPath(import.meta.url)));
const src = (rel: string): string => readFileSync(join(rootDir, rel), 'utf8');

test('#6453 HF 커서 위치 API는 대표 편집 페이지를 바꾸지 않는다', async () => {
  const vite = await createServer({
    root: rootDir, appType: 'custom', logLevel: 'silent', server: { middlewareMode: true },
  });
  try {
    const { CursorState } = await vite.ssrLoadModule('/src/engine/cursor.ts');
    const wasm = {
      getHeaderFooterPreviewPage: () => 2,
      getCursorRectInHeaderFooter: (
        _sec: number, _header: boolean, _apply: number,
        paraIdx: number, charOffset: number, previewPage: number,
      ) => ({ pageIndex: previewPage, x: charOffset * 8, y: paraIdx * 20, height: 12 }),
      getHeaderFooterParaInfo: () => JSON.stringify({ paraCount: 1, charCount: 8 }),
    };
    const cursor: any = new CursorState(wasm);

    cursor.enterHeaderFooterMode(true, 0, 1, 9);
    assert.equal(cursor.hfPreviewPage, 2, '진입 쪽과 무관하게 구역 대표 페이지를 사용한다');

    cursor.setHfCursorPosition(0, 4);
    assert.equal(cursor.hfPreviewPage, 2, '텍스트 위치 변경은 대표 페이지를 바꾸지 않는다');

    assert.equal(cursor.selectHeaderFooterRange(
      { sectionIdx: 0, isHeader: true, applyTo: 1, paraIdx: 0, charOffset: 1 },
      { sectionIdx: 0, isHeader: true, applyTo: 1, paraIdx: 0, charOffset: 4 },
      9,
    ), true);
    assert.equal(cursor.getHeaderFooterSelectionOrdered()?.previewPage, 2,
      'history의 이전 페이지 힌트도 현재 대표 페이지로 정규화한다');
  } finally {
    await vite.close();
  }
});

test('#6453 HF 공개 커서 위치 API는 문단·문자 좌표만 받는다', () => {
  const cursor = src('src/engine/cursor.ts');
  const setPosition = functionBodyFrom(cursor, 'setHfCursorPosition(');

  assert.match(cursor, /setHfCursorPosition\(paraIdx: number, charOffset: number\): void/);
  assert.doesNotMatch(setPosition, /page|resolveHeaderFooterPreviewPage/);
});

test('#6453 IME 조합 캐럿도 HF 대표 편집 페이지를 직접 사용한다', () => {
  const handler = src('src/engine/input-handler.ts');
  const updateCaret = functionBodyFrom(handler, 'private updateCaret(');

  assert.match(
    updateCaret,
    /getCursorRectInHeaderFooter\([\s\S]*?this\.cursor\.hfPreviewPage/,
  );
  assert.doesNotMatch(
    updateCaret,
    /getCursorRectInHeaderFooter\([\s\S]*?this\.cursor\.getRect\(\)\?\.pageIndex/,
  );
});
