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
        paraIdx: number, charOffset: number, previewPage: number,
      ) => ({ pageIndex: previewPage, x: charOffset * 8, y: paraIdx * 20, height: 12 }),
      getHeaderFooterParaInfo: (_sec: number, _header: boolean, _apply: number, paraIdx: number) =>
        JSON.stringify({ paraCount: 2, charCount: paraIdx === 0 ? 5 : 4 }),
    };
    const cursor: any = new CursorState(wasm);
    cursor.enterHeaderFooterMode(true, 2, 1, 4);
    cursor.setHfCursorPosition(0, 4);
    cursor.setHfAnchor();
    cursor.setHfCursorPosition(1, 2);

    assert.equal(cursor.hasSelection(), true);
    assert.deepEqual(cursor.getHeaderFooterSelectionOrdered(), {
      start: { sectionIdx: 2, isHeader: true, applyTo: 1, paraIdx: 0, charOffset: 4 },
      end: { sectionIdx: 2, isHeader: true, applyTo: 1, paraIdx: 1, charOffset: 2 },
      previewPage: 4,
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
    cursor.setHfCursorPosition(1, 6);
    cursor.setHfAnchor();
    cursor.setHfCursorPosition(0, 2);

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
        paraIdx: number, charOffset: number, previewPage: number,
      ) => ({ pageIndex: previewPage, x: charOffset * 8, y: paraIdx * 20, height: 12 }),
      getHeaderFooterParaInfo: () => JSON.stringify({ paraCount: 2, charCount: 8 }),
      hitTestInHeaderFooter: () => ({
        hit: true, sectionIndex: 0, applyTo: 2, paraIndex: 1, charOffset: 3,
      }),
    };
    const cursor: any = new CursorState(wasm);
    cursor.enterHeaderFooterMode(true, 0, 2, 5);
    cursor.setHfCursorPosition(0, 2);
    cursor.setHfAnchor();
    cursor.moveVerticalInHf(1);

    assert.equal(cursor.hfParaIdx, 1);
    assert.equal(cursor.hfCharOffset, 3);
    assert.equal(cursor.getHeaderFooterSelectionOrdered()?.previewPage, 5);
  } finally {
    await vite.close();
  }
});

test('#4121 HF 단어·문단·target 경계 이동은 HF 좌표계를 유지한다', async () => {
  const vite = await createServer({
    root: rootDir, appType: 'custom', logLevel: 'silent', server: { middlewareMode: true },
  });
  try {
    const { CursorState } = await vite.ssrLoadModule('/src/engine/cursor.ts');
    const texts = ['alpha beta', '둘째 문단'];
    const wasm = {
      getCursorRectInHeaderFooter: (
        _sec: number, _header: boolean, _apply: number,
        paraIdx: number, charOffset: number, previewPage: number,
      ) => ({ pageIndex: previewPage, x: charOffset * 8, y: paraIdx * 20, height: 12 }),
      getHeaderFooterParaInfo: (_sec: number, _header: boolean, _apply: number, paraIdx: number) =>
        JSON.stringify({ paraCount: texts.length, charCount: Array.from(texts[paraIdx]).length, text: texts[paraIdx] }),
    };
    const cursor: any = new CursorState(wasm);
    cursor.enterHeaderFooterMode(true, 0, 0, 2);
    cursor.setHfCursorPosition(0, 10);

    cursor.moveToWordBoundaryInHf(-1);
    assert.equal(cursor.hfCharOffset, 6, 'Option+Left는 이전 단어 시작으로 이동');
    cursor.moveToWordBoundaryInHf(-1);
    assert.equal(cursor.hfCharOffset, 0);
    cursor.moveToWordBoundaryInHf(1);
    assert.equal(cursor.hfCharOffset, 6, 'Option+Right는 다음 단어 시작으로 이동');

    cursor.moveToParagraphBoundaryInHf(1);
    assert.deepEqual([cursor.hfParaIdx, cursor.hfCharOffset], [1, 0]);
    cursor.moveToHeaderFooterBoundary(1);
    assert.deepEqual([cursor.hfParaIdx, cursor.hfCharOffset], [1, 5]);
    cursor.moveToHeaderFooterBoundary(-1);
    assert.deepEqual([cursor.hfParaIdx, cursor.hfCharOffset], [0, 0]);
  } finally {
    await vite.close();
  }
});

test('#4121 macOS HF Option+Shift·Command+Shift 탐색은 실제 선택 범위를 만든다', async () => {
  const vite = await createServer({
    root: rootDir, appType: 'custom', logLevel: 'silent', server: { middlewareMode: true },
  });
  try {
    const { CursorState } = await vite.ssrLoadModule('/src/engine/cursor.ts');
    const { onKeyDown } = await vite.ssrLoadModule('/src/engine/input-handler-keyboard.ts');
    const texts = ['alpha beta', '둘째 문단'];
    const wasm = {
      getCursorRectInHeaderFooter: (
        _sec: number, _header: boolean, _apply: number,
        paraIdx: number, charOffset: number, previewPage: number,
      ) => ({ pageIndex: previewPage, x: charOffset * 8, y: paraIdx * 20, height: 12 }),
      getHeaderFooterParaInfo: (_sec: number, _header: boolean, _apply: number, paraIdx: number) =>
        JSON.stringify({ paraCount: texts.length, charCount: Array.from(texts[paraIdx]).length, text: texts[paraIdx] }),
    };
    const cursor: any = new CursorState(wasm);
    cursor.enterHeaderFooterMode(true, 0, 0, 2);
    cursor.setHfCursorPosition(0, 10);
    let caretUpdates = 0;
    const handler: any = {
      active: true,
      cursor,
      wasm,
      flushDeferredPaginationIfNeeded: () => {},
      updateCaret: () => { caretUpdates++; },
    };
    const key = (keyName: string, modifiers: Record<string, boolean>) => ({
      key: keyName,
      code: keyName,
      shiftKey: false,
      ctrlKey: false,
      metaKey: false,
      altKey: false,
      isComposing: false,
      keyCode: 0,
      preventDefault: () => {},
      ...modifiers,
    });

    (globalThis as any).__rhwpTestPlatformKind = 'mac';
    onKeyDown.call(handler, key('ArrowLeft', { altKey: true, shiftKey: true }));
    assert.deepEqual(cursor.getHeaderFooterSelectionOrdered(), {
      start: { sectionIdx: 0, isHeader: true, applyTo: 0, paraIdx: 0, charOffset: 6 },
      end: { sectionIdx: 0, isHeader: true, applyTo: 0, paraIdx: 0, charOffset: 10 },
      previewPage: 2,
    });

    cursor.clearSelection();
    cursor.setHfCursorPosition(1, 3);
    onKeyDown.call(handler, key('ArrowUp', { metaKey: true, shiftKey: true }));
    assert.deepEqual(cursor.getHeaderFooterSelectionOrdered()?.start, {
      sectionIdx: 0, isHeader: true, applyTo: 0, paraIdx: 0, charOffset: 0,
    });
    assert.equal(caretUpdates, 2);
  } finally {
    delete (globalThis as any).__rhwpTestPlatformKind;
    await vite.close();
  }
});

test('#4121 HF 모두 선택은 메뉴와 Ctrl/Cmd+A 모두 현재 정의만 대상으로 한다', async () => {
  const vite = await createServer({
    root: rootDir, appType: 'custom', logLevel: 'silent', server: { middlewareMode: true },
  });
  try {
    const { CursorState } = await vite.ssrLoadModule('/src/engine/cursor.ts');
    const { handleSelectAll, onKeyDown } = await vite.ssrLoadModule('/src/engine/input-handler-keyboard.ts');
    const texts = ['첫 문단', 'second paragraph'];
    const wasm = {
      getCursorRectInHeaderFooter: (
        _sec: number, _header: boolean, _apply: number,
        paraIdx: number, charOffset: number, previewPage: number,
      ) => ({ pageIndex: previewPage, x: charOffset * 8, y: paraIdx * 20, height: 12 }),
      getHeaderFooterParaInfo: (_sec: number, _header: boolean, _apply: number, paraIdx: number) =>
        JSON.stringify({
          paraCount: texts.length,
          charCount: Array.from(texts[paraIdx]).length,
          text: texts[paraIdx],
        }),
    };
    const cursor: any = new CursorState(wasm);
    cursor.enterHeaderFooterMode(true, 1, 2, 3);
    cursor.setHfCursorPosition(0, 2);
    let caretUpdates = 0;
    let dispatched = '';
    const handler: any = {
      active: true,
      cursor,
      wasm,
      flushDeferredPaginationIfNeeded: () => {},
      updateCaret: () => { caretUpdates++; },
    };
    handler.dispatcher = {
      isEnabled: () => false,
      dispatch: (commandId: string) => {
        dispatched = commandId;
        handleSelectAll.call(handler);
      },
    };

    handleSelectAll.call(handler);
    assert.deepEqual(cursor.getHeaderFooterSelectionOrdered(), {
      start: { sectionIdx: 1, isHeader: true, applyTo: 2, paraIdx: 0, charOffset: 0 },
      end: { sectionIdx: 1, isHeader: true, applyTo: 2, paraIdx: 1, charOffset: 16 },
      previewPage: 3,
    });
    assert.equal(cursor.getSelectionOrdered(), null, '본문 anchor는 만들지 않는다');

    cursor.clearSelection();
    onKeyDown.call(handler, {
      key: 'a', code: 'KeyA', shiftKey: false, ctrlKey: false, metaKey: true, altKey: false,
      isComposing: false, keyCode: 0, preventDefault: () => {},
    });
    assert.equal(dispatched, 'edit:select-all');
    assert.equal(cursor.getHeaderFooterSelectionOrdered()?.end.paraIdx, 1);
    assert.equal(caretUpdates, 2);
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
  assert.match(hf, /handleHeaderFooterNavigationShortcut/);
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
