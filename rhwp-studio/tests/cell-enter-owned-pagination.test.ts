import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

// [#4031] pending pagination 중 셀 Enter의 중복 full pagination 제거 계약.
//
// 계약: admission이 확정된 셀 Enter는 before-navigation full flush 대신
// 계산 없는 취소(cancelDeferredPaginationForOwnedMutation)를 쓰고, 성공한
// SplitParagraphInCellCommand가 pagination 완료를 소유한다. 실패나 admission
// 불충족은 기존 full-flush barrier로 fail-closed한다.

const rootDir = dirname(dirname(fileURLToPath(import.meta.url)));

function source(path: string): string {
  return readFileSync(join(rootDir, path), 'utf8');
}

function sliceBetween(text: string, startNeedle: string, endNeedle: string): string {
  const start = text.indexOf(startNeedle);
  assert.notEqual(start, -1, `"${startNeedle}" not found`);
  const end = text.indexOf(endNeedle, start);
  assert.notEqual(end, -1, `"${endNeedle}" not found after "${startNeedle}"`);
  return text.slice(start, end);
}

test('boundary key flush는 admitted 셀 Enter에서만 계산 없는 취소로 대체된다', () => {
  const keyboard = source('src/engine/input-handler-keyboard.ts');
  const block = sliceBetween(
    keyboard,
    'const committedCellEnterSplit = PAGINATION_BOUNDARY_KEYS.has(e.key)',
    '// ─── 머리말/꼬리말 편집 모드 키보드 처리',
  );
  assert.ok(
    block.includes('isCommittedCellEnterSplit.call(this, e)'),
    'admission은 committed cell Enter 판정 helper를 거쳐야 한다',
  );
  assert.ok(
    block.includes('this.cancelDeferredPaginationForOwnedMutation()'),
    'admitted 경로는 계산 없는 취소를 호출해야 한다',
  );
  assert.ok(
    block.includes("this.flushDeferredPaginationIfNeeded('before-navigation', false)"),
    '비admitted boundary key는 기존 before-navigation full flush를 유지해야 한다',
  );
  assert.ok(
    !block.includes('wasm.flushDeferredPagination'),
    'admitted 경로에 직접 full flush가 있으면 안 된다',
  );
});

test('admission helper는 case Enter까지의 모든 조기 분기를 배제한다', () => {
  const keyboard = source('src/engine/input-handler-keyboard.ts');
  const helper = sliceBetween(
    keyboard,
    'function isCommittedCellEnterSplit',
    'function dispatchSubmodeGlobalShortcut',
  );
  for (const guard of [
    "e.key === 'Enter'",
    '!e.shiftKey',
    '!e.ctrlKey',
    '!e.metaKey',
    '!e.altKey',
    '!this.isComposing',
    '!this.isFormMode?.()',
    '!this.cursor.isInHeaderFooter()',
    '!this.cursor.isInFootnote()',
    '!this.cursor.isInPictureObjectSelection()',
    '!this.cursor.isInTableObjectSelection()',
    '!this.cursor.isInBlockSelectionMode()',
    '!this.cursor.isInCellSelectionMode()',
    '!this.cursor.hasSelection()',
    'this.cursor.isInCell()',
  ]) {
    assert.ok(helper.includes(guard), `admission guard 누락: ${guard}`);
  }
});

test('성공한 셀 split이 pagination 완료를 소유하고 실패는 full flush로 복귀한다', () => {
  const keyboard = source('src/engine/input-handler-keyboard.ts');
  const enterCase = sliceBetween(keyboard, "case 'Enter': {", "case 'ArrowLeft':");
  const successIdx = enterCase.indexOf('this.completePaginationOwnedBySyncMutation()');
  const splitIdx = enterCase.indexOf('new SplitParagraphInCellCommand');
  const fallbackIdx = enterCase.indexOf(
    "this.flushDeferredPaginationIfNeeded('cell-enter-split-fallback', false)",
  );
  assert.notEqual(splitIdx, -1, 'SplitParagraphInCellCommand 실행이 없다');
  assert.ok(successIdx > splitIdx, '완료 선언은 split 실행 뒤에 와야 한다');
  assert.ok(fallbackIdx > splitIdx, 'catch fallback full flush가 없다');
  assert.ok(
    enterCase.includes('if (committedCellEnterSplit)'),
    '완료/복귀는 admission이 확정된 경우에만 수행해야 한다',
  );
});

test('소유 취소는 pending을 유지하고 완료 선언만 pending을 해소한다', () => {
  const handler = source('src/engine/input-handler.ts');
  const cancel = sliceBetween(
    handler,
    'cancelDeferredPaginationForOwnedMutation(): void {',
    'completePaginationOwnedBySyncMutation(): void {',
  );
  assert.ok(
    !cancel.includes('flushDeferredPagination()'),
    '소유 취소는 wasm full flush를 호출하면 안 된다',
  );
  assert.ok(
    !cancel.includes('deferredPaginationPending = false'),
    '소유 취소는 fail-closed를 위해 pending을 유지해야 한다',
  );
  assert.ok(
    cancel.includes('this.deferredPaginationRunner.cancel()'),
    '소유 취소는 scheduled/stepping runner job을 취소해야 한다',
  );

  const complete = sliceBetween(
    handler,
    'completePaginationOwnedBySyncMutation(): void {',
    'afterTextInputEdit(',
  );
  assert.ok(
    complete.includes('this.deferredPaginationPending = false'),
    '완료 선언은 pending을 해소해야 한다',
  );
  assert.ok(
    complete.includes('this.cursor.invalidateFocusedCellCursorGeometry()'),
    '완료 선언은 cursor geometry를 invalidate해야 한다',
  );
});

test('IME 종료 후 예약 Enter 경로의 barrier flush는 유지된다', () => {
  // processPendingNav의 Enter는 조합 확정만 하고 structural command가 없으므로
  // 이 flush가 최신 모델의 유일한 pagination barrier다 (stage1 §5).
  const text = source('src/engine/input-handler-text.ts');
  const pendingNav = sliceBetween(
    text,
    'function processPendingNav',
    'function tryDeleteBodyFootnoteAtCursor',
  );
  assert.ok(
    pendingNav.includes("this.flushDeferredPaginationIfNeeded('before-navigation', false)"),
    'processPendingNav의 before-navigation flush가 제거되면 안 된다',
  );
});
