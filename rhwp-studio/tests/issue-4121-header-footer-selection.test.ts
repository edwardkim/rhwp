import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { createServer } from 'vite';
import { functionBodyFrom } from './support/source-guard.ts';

const rootDir = dirname(dirname(fileURLToPath(import.meta.url)));
const src = (rel: string): string => readFileSync(join(rootDir, rel), 'utf8');

test('#4121 HF anchor는 본문·각주와 독립된 target 소유 범위를 만든다', async () => {
  const vite = await createServer({
    root: rootDir, appType: 'custom', logLevel: 'silent', server: { middlewareMode: true },
  });
  try {
    const { CursorState } = await vite.ssrLoadModule('/src/engine/cursor.ts');
    const wasm = {
      getCursorRectInHeaderFooter: (
        _sec: number, _header: boolean, _apply: number,
        paraIdx: number, charOffset: number, preferredPage: number,
      ) => ({ pageIndex: preferredPage, x: charOffset * 8, y: paraIdx * 20, height: 12 }),
      getHeaderFooterParaInfo: (_sec: number, _header: boolean, _apply: number, paraIdx: number) =>
        JSON.stringify({ paraCount: 2, charCount: paraIdx === 0 ? 5 : 4 }),
    };
    const cursor: any = new CursorState(wasm);
    cursor.enterHeaderFooterMode(true, 2, 1, 4);
    cursor.setHfCursorPosition(0, 4, 4);
    cursor.setHfAnchor();
    cursor.setHfCursorPosition(1, 2, 6);

    assert.equal(cursor.hasSelection(), true);
    assert.deepEqual(cursor.getHeaderFooterSelectionOrdered(), {
      start: { sectionIdx: 2, isHeader: true, applyTo: 1, paraIdx: 0, charOffset: 4 },
      end: { sectionIdx: 2, isHeader: true, applyTo: 1, paraIdx: 1, charOffset: 2 },
      preferredPage: 6,
    });

    cursor.switchHeaderFooterTarget(true, 2, 2, 7);
    assert.equal(cursor.getHeaderFooterSelectionOrdered(), null, 'Odd/Even target 전환은 선택을 지운다');
  } finally {
    await vite.close();
  }
});

test('#4121 HF 역방향 범위는 문단·문자 사전식으로 정렬된다', async () => {
  const vite = await createServer({
    root: rootDir, appType: 'custom', logLevel: 'silent', server: { middlewareMode: true },
  });
  try {
    const { CursorState } = await vite.ssrLoadModule('/src/engine/cursor.ts');
    const wasm = {
      getCursorRectInHeaderFooter: () => ({ pageIndex: 3, x: 0, y: 0, height: 12 }),
      getHeaderFooterParaInfo: () => JSON.stringify({ paraCount: 2, charCount: 8 }),
    };
    const cursor: any = new CursorState(wasm);
    cursor.enterHeaderFooterMode(false, 0, 0, 3);
    cursor.setHfCursorPosition(1, 6, 3);
    cursor.setHfAnchor();
    cursor.setHfCursorPosition(0, 2, 3);

    const selection = cursor.getHeaderFooterSelectionOrdered();
    assert.deepEqual(selection?.start, {
      sectionIdx: 0, isHeader: false, applyTo: 0, paraIdx: 0, charOffset: 2,
    });
    assert.deepEqual(selection?.end, {
      sectionIdx: 0, isHeader: false, applyTo: 0, paraIdx: 1, charOffset: 6,
    });
  } finally {
    await vite.close();
  }
});

test('#4121 HF 위아래 이동은 같은 resolved target의 시각 줄만 따른다', async () => {
  const vite = await createServer({
    root: rootDir, appType: 'custom', logLevel: 'silent', server: { middlewareMode: true },
  });
  try {
    const { CursorState } = await vite.ssrLoadModule('/src/engine/cursor.ts');
    const wasm = {
      getCursorRectInHeaderFooter: (
        _sec: number, _header: boolean, _apply: number,
        paraIdx: number, charOffset: number, preferredPage: number,
      ) => ({ pageIndex: preferredPage, x: charOffset * 8, y: paraIdx * 20, height: 12 }),
      getHeaderFooterParaInfo: () => JSON.stringify({ paraCount: 2, charCount: 8 }),
      hitTestInHeaderFooter: () => ({
        hit: true, sectionIndex: 0, applyTo: 2, paraIndex: 1, charOffset: 3,
      }),
    };
    const cursor: any = new CursorState(wasm);
    cursor.enterHeaderFooterMode(true, 0, 2, 5);
    cursor.setHfCursorPosition(0, 2, 5);
    cursor.setHfAnchor();
    cursor.moveVerticalInHf(1);

    assert.equal(cursor.hfParaIdx, 1);
    assert.equal(cursor.hfCharOffset, 3);
    assert.equal(cursor.getHeaderFooterSelectionOrdered()?.preferredPage, 5);
  } finally {
    await vite.close();
  }
});

test('#4121 마우스 HF 선택은 클릭 페이지 target을 확인하고 drag lifecycle을 시작한다', () => {
  const mouse = src('src/engine/input-handler-mouse.ts');
  const click = functionBodyFrom(mouse, 'export function onClick(');
  assert.match(click, /inHfHit\.sectionIndex/);
  assert.match(click, /inHfHit\.applyTo/);
  assert.match(click, /setHfAnchor\(\)/);
  assert.match(click, /startTextSelectionDrag\(e\)/);
  assert.match(click, /switchHeaderFooterTarget/);
});

test('#4121 HF 키보드는 Shift 선택과 Esc 2단계를 제공한다', () => {
  const keyboard = src('src/engine/input-handler-keyboard.ts');
  const keydown = functionBodyFrom(keyboard, 'export function onKeyDown(');
  const hfStart = keydown.indexOf('if (this.cursor.isInHeaderFooter())');
  const fnStart = keydown.indexOf('if (this.cursor.isInFootnote())');
  const hf = keydown.slice(hfStart, fnStart);
  assert.match(hf, /e\.shiftKey[\s\S]*setHfAnchor\(\)/);
  assert.match(hf, /moveVerticalInHf/);
  assert.match(hf, /hasHeaderFooterSelection\(\)/);
  assert.match(hf, /clearSelection\(\)/);
});

test('#4121 HF overlay는 visible page마다 코어 기하를 조회한다', () => {
  const handler = src('src/engine/input-handler.ts');
  const update = functionBodyFrom(handler, 'private updateSelection()');
  assert.match(update, /getHeaderFooterSelectionOrdered\(\)/);
  assert.match(update, /getVisiblePages\(/);
  assert.match(update, /getSelectionRectsInHeaderFooter\(/);
  assert.match(handler, /eventBus\.on\('viewport-scroll',[\s\S]*updateSelection\(\)/);
});

test('#4121 선택 renderer는 모든 쪽 배치의 resolved page-left를 사용한다', () => {
  const renderer = src('src/engine/selection-renderer.ts');
  const render = functionBodyFrom(renderer, 'render(');
  assert.match(render, /getPageLeftResolved\(/);
  assert.doesNotMatch(render, /\(contentWidth - pageDisplayWidth\) \/ 2/);
});
