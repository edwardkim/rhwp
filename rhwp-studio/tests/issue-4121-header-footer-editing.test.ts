import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { createServer } from 'vite';
import { functionBodyFrom } from './support/source-guard.ts';

const rootDir = dirname(dirname(fileURLToPath(import.meta.url)));
const src = (rel: string): string => readFileSync(join(rootDir, rel), 'utf8');

test('#4121 history 복원용 HF 선택은 현재 target과 문단 경계를 다시 검증한다', async () => {
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
      getHeaderFooterPreviewPage: () => 4,
      getHeaderFooterParaInfo: (_sec: number, _header: boolean, _apply: number, paraIdx: number) =>
        JSON.stringify({ paraCount: 2, charCount: paraIdx === 0 ? 5 : 4 }),
    };
    const cursor: any = new CursorState(wasm);
    cursor.enterHeaderFooterMode(true, 2, 1, 4);

    assert.equal(cursor.selectHeaderFooterRange(
      { sectionIdx: 2, isHeader: true, applyTo: 1, paraIdx: 0, charOffset: 2 },
      { sectionIdx: 2, isHeader: true, applyTo: 1, paraIdx: 1, charOffset: 3 },
      6,
    ), true);
    assert.equal(cursor.getHeaderFooterSelectionOrdered()?.previewPage, 4);

    cursor.clearSelection();
    assert.equal(cursor.selectHeaderFooterRange(
      { sectionIdx: 2, isHeader: true, applyTo: 1, paraIdx: 0, charOffset: 2 },
      { sectionIdx: 2, isHeader: true, applyTo: 1, paraIdx: 1, charOffset: 99 },
      6,
    ), false);
    assert.equal(cursor.getHeaderFooterSelectionOrdered(), null, 'stale 범위는 선택 없이 복원한다');
  } finally {
    await vite.close();
  }
});

test('#4121 HF snapshot command는 undo/redo 문맥과 선택 정책을 분리한다', async () => {
  const vite = await createServer({
    root: rootDir, appType: 'custom', logLevel: 'silent', server: { middlewareMode: true },
  });
  try {
    const { SubmodeSelectionSnapshotCommand } = await vite.ssrLoadModule('/src/engine/command.ts');
    let nextSnapshot = 1;
    const wasm: any = {
      saveSnapshot: () => nextSnapshot++,
      restoreSnapshot: () => {},
      discardSnapshot: () => {},
    };
    const before = {
      mode: 'headerFooter', sectionIdx: 0, isHeader: true, applyTo: 0,
      paraIdx: 1, charOffset: 3, previewPage: 4,
    };
    const after = {
      mode: 'headerFooter', sectionIdx: 0, isHeader: true, applyTo: 0,
      paraIdx: 0, charOffset: 2, previewPage: 4,
    };
    const selection = {
      mode: 'headerFooter',
      start: { sectionIdx: 0, isHeader: true, applyTo: 0, paraIdx: 0, charOffset: 2 },
      end: { sectionIdx: 0, isHeader: true, applyTo: 0, paraIdx: 1, charOffset: 3 },
      previewPage: 4,
    };
    const body = { sectionIndex: 0, paragraphIndex: 0, charOffset: 0 };
    const cmd = new SubmodeSelectionSnapshotCommand(
      'deleteSelectionInHeaderFooter', body, body, () => body,
      before, () => after, selection, null,
    );

    cmd.execute(wasm);
    assert.deepEqual(cmd.editContext(), after);
    assert.equal(cmd.selectionAfter(), null);
    cmd.undo(wasm);
    assert.deepEqual(cmd.editContext(), before);
    assert.deepEqual(cmd.selectionBefore(), selection);
    cmd.execute(wasm);
    assert.deepEqual(cmd.editContext(), after);
  } finally {
    await vite.close();
  }
});

test('#4121 HF 선택 삭제와 치환은 범위 API 한 호출과 선택 history를 사용한다', () => {
  const handler = src('src/engine/input-handler.ts');
  const del = functionBodyFrom(handler, 'private deleteSelection(');
  assert.match(del, /getNonEmptyHeaderFooterSelection\(\)/);
  assert.match(del, /replaceHeaderFooterSelection/);
  assert.match(handler, /replaceRangeInHeaderFooter\(/);
  assert.match(handler, /selectionBefore:/);
  assert.match(handler, /editContextAfter:/);
});

test('#4121 일반 입력과 IME는 HF 선택을 별도 삭제하지 않고 원자 치환한다', () => {
  const text = src('src/engine/input-handler-text.ts');
  const input = functionBodyFrom(text, 'export function onInput(');
  const compositionStart = functionBodyFrom(text, 'export function onCompositionStart(');
  const compositionEnd = functionBodyFrom(text, 'export function onCompositionEnd(');
  assert.match(input, /replaceHeaderFooterSelection/);
  assert.match(compositionStart, /beginHeaderFooterSelectionComposition/);
  assert.match(compositionEnd, /headerFooterSelectionComposition/);
});

test('#4121 HF IME 시작은 남아 있는 본문 selection을 삭제하지 않는다', async () => {
  const vite = await createServer({
    root: rootDir, appType: 'custom', logLevel: 'silent', server: { middlewareMode: true },
  });
  try {
    const { onCompositionStart } = await vite.ssrLoadModule('/src/engine/input-handler-text.ts');
    let bodyDeleteCalls = 0;
    const handler: any = {
      resetRawTextMutationEffects: () => {},
      headerFooterSelectionComposition: false,
      getNonEmptyHeaderFooterSelection: () => null,
      cursor: {
        isInHeaderFooter: () => true,
        isInFootnote: () => false,
        hasSelection: () => true,
        getPosition: () => ({ sectionIndex: 0, paragraphIndex: 9, charOffset: 3 }),
        hfCharOffset: 2,
      },
      textarea: { value: '' },
      deleteSelection: () => { bodyDeleteCalls++; },
      canInsertTextInFormMode: () => true,
      isComposing: false,
      compositionAnchor: null,
      compositionLength: 0,
    };

    onCompositionStart.call(handler);

    assert.equal(bodyDeleteCalls, 0);
    assert.equal(handler.isComposing, true);
    assert.deepEqual(handler.compositionAnchor, {
      sectionIndex: 0, paragraphIndex: 9, charOffset: 2,
    });
  } finally {
    await vite.close();
  }
});

test('#4121 HF copy/cut/paste는 HF 전용 copy와 평문 범위 치환을 사용한다', () => {
  const keyboard = src('src/engine/input-handler-keyboard.ts');
  const copy = functionBodyFrom(keyboard, 'export function onCopy(');
  const cut = functionBodyFrom(keyboard, 'export function onCut(');
  const paste = functionBodyFrom(keyboard, 'export function onPaste(');
  assert.match(copy, /copyHeaderFooterSelection/);
  assert.match(cut, /copyHeaderFooterSelection[\s\S]*deleteSelection/);
  assert.match(paste, /isInHeaderFooter\(\)[\s\S]*replaceHeaderFooterSelection/);
  assert.match(keyboard, /copySelectionInHeaderFooter\(/);
});

test('#4121 HF 부분 글자 서식은 선택 범위 API와 선택 유지 history를 사용한다', () => {
  const handler = src('src/engine/input-handler.ts');
  const apply = functionBodyFrom(handler, 'private applyCharFormat(');
  const props = functionBodyFrom(handler, 'private getCharPropertiesAtCursor(');
  assert.match(apply, /applyCharFormatInHeaderFooterSelection/);
  assert.match(handler, /applyCharFormatInHeaderFooter\(/);
  assert.match(handler, /selectionAfter:/);
  assert.match(props, /getCharPropertiesInHeaderFooter/);
});

test('#4121 undo는 HF selectionBefore, redo는 format의 selectionAfter를 복원한다', () => {
  const handler = src('src/engine/input-handler.ts');
  const undo = functionBodyFrom(handler, 'private handleUndo()');
  const redo = functionBodyFrom(handler, 'private handleRedo()');
  const restore = functionBodyFrom(handler, 'private restoreSelectionAfterUndo');
  assert.match(undo, /restoreSelectionAfterUndo/);
  assert.match(redo, /restoreSelectionAfterRedo/);
  assert.match(restore, /selectHeaderFooterRange/);
});

test('#4121 새 HF WASM API는 생성 바인딩 타입을 우회하지 않고 탐색 실패를 기록한다', () => {
  const bridge = src('src/core/wasm-bridge.ts');
  const cursor = src('src/engine/cursor.ts');
  assert.doesNotMatch(bridge, /\(this\.doc as any\)\.(?:replaceRangeInHeaderFooter|copySelectionInHeaderFooter|getCharPropertiesInHeaderFooter|applyCharFormatInHeaderFooter|getSelectionRectsInHeaderFooter)/);
  assert.match(cursor, /moveToWordBoundaryInHf 실패/);
  assert.match(cursor, /moveToParagraphBoundaryInHf 실패/);
  assert.match(cursor, /moveToHeaderFooterBoundary 실패/);
});
