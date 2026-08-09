import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';

import {
  isPageLocalTextEditCommand,
  MAX_PAGE_LOCAL_TEXT_EDIT_CHARS,
} from '../src/engine/input-edit-invalidation.ts';
import type { DocumentPosition } from '../src/core/types.ts';

const baseCellPos: DocumentPosition = {
  sectionIndex: 0,
  paragraphIndex: 2,
  charOffset: 3,
  parentParaIndex: 2,
  controlIndex: 0,
  cellIndex: 1,
  cellParaIndex: 0,
  cellPath: [{ controlIndex: 0, cellIndex: 1, cellParaIndex: 0 }],
};

test('isPageLocalTextEditCommand는 같은 셀 내부 insert/delete만 허용한다', () => {
  assert.equal(
    isPageLocalTextEditCommand('insertText', baseCellPos, { ...baseCellPos, charOffset: 4 }, { insertedText: '가' }),
    true,
  );
  assert.equal(
    isPageLocalTextEditCommand('deleteText', baseCellPos, baseCellPos, { deleteCount: 1 }),
    true,
  );
});

test('텍스트 command는 page-local 판정용 payload hint를 노출한다', () => {
  const source = readFileSync(new URL('../src/engine/command.ts', import.meta.url), 'utf8');

  assert.match(source, /getPageLocalTextEditOptions\?\(\): \{ insertedText\?: string; deleteCount\?: number \}/);
  assert.match(source, /getPageLocalTextEditOptions\(\): \{ insertedText: string \} \{\s*return \{ insertedText: this\.text \};\s*\}/);
  assert.match(source, /getPageLocalTextEditOptions\(\): \{ deleteCount: number \} \{\s*return \{ deleteCount: this\.count \};\s*\}/);
});

test('raw IME/iOS 입력은 flow effect를 cursor lookup 전에 소비하고 refresh에 전달한다', () => {
  const inputHandlerSource = readFileSync(new URL('../src/engine/input-handler.ts', import.meta.url), 'utf8');
  const textSource = readFileSync(new URL('../src/engine/input-handler-text.ts', import.meta.url), 'utf8');

  assert.match(
    inputHandlerSource,
    /private afterTextInputEdit\(\s*beforePos: DocumentPosition,\s*afterPos: DocumentPosition,\s*pageLocalOptions: PageLocalTextEditOptions = \{\},\s*boundaryHandled = false,\s*\): void \{\s*if \(boundaryHandled\) \{\s*this\.afterEdit\(false\);\s*return;\s*\}/,
  );
  assert.match(
    textSource,
    /this\.afterTextInputEdit\(anchor, afterPos, \{\s*insertedText: text,\s*beforePageIndex,\s*afterPageIndex,\s*\}, boundaryHandled\);/,
  );
  assert.match(
    textSource,
    /this\._iosBeforePageIndex = this\.cursor\.getRect\(\)\?\.pageIndex;/,
  );
  assert.match(
    textSource,
    /const beforePageIndex = this\._iosBeforePageIndex;/,
  );
  assert.match(
    textSource,
    /this\.afterTextInputEdit\(iosAnchor, iosAfterPos, \{\s*insertedText: text,\s*beforePageIndex,\s*afterPageIndex,\s*\}, requiresFullRefresh\);/,
  );

  const imeStart = textSource.indexOf('if (this.isComposing && this.compositionAnchor)');
  const iosStart = textSource.indexOf('if (this._isIOS && !this.isComposing)');
  const generalStart = textSource.indexOf('// 일반 입력 (비조합)');
  assert.ok(imeStart >= 0 && iosStart > imeStart && generalStart > iosStart);

  const imeSource = textSource.slice(imeStart, iosStart);
  const iosSource = textSource.slice(iosStart, generalStart);
  assert.ok(
    imeSource.indexOf('this.consumeRawTextMutationBeforeCursor()') < imeSource.indexOf('this.cursor.moveTo('),
    'IME effect는 cursor.moveTo 전에 소비해야 한다',
  );
  assert.ok(
    iosSource.indexOf('this.consumeRawTextMutationBeforeCursor()') < iosSource.indexOf('this.cursor.moveTo('),
    'iOS effect는 cursor.moveTo 전에 소비해야 한다',
  );
  assert.match(iosSource, /this\._iosRequiresFullRefresh = this\._iosRequiresFullRefresh \|\| boundaryHandled;/);
  assert.match(
    textSource,
    /this\.caret\.hideComposition\(\);\s*this\.updateCaret\(\);\s*this\.resetRawTextMutationEffects\(\);/,
    'compositionend는 일반 DOM caret를 exact cursor에서 다시 표시해야 한다',
  );

  assert.match(
    imeSource,
    /this\.replaceTextAtRaw\(anchor, this\.compositionLength, text\);/,
    'IME update는 이전 조합 삭제와 새 조합 삽입을 한 local replace로 보내야 한다',
  );
  assert.doesNotMatch(imeSource, /this\.deleteTextAt\(anchor/);
  assert.doesNotMatch(imeSource, /this\.insertTextAtRaw\(anchor/);
  assert.match(
    iosSource,
    /this\.replaceTextAtRaw\(this\._iosAnchor, this\._iosLength, text\);/,
    'iOS fallback도 조합 교체를 한 local replace로 보내야 한다',
  );
  assert.doesNotMatch(iosSource, /this\.deleteTextAt\(this\._iosAnchor/);
  assert.doesNotMatch(iosSource, /this\.insertTextAtRaw\(this\._iosAnchor/);
  assert.doesNotMatch(
    iosSource,
    /this\._iosInputTimer = setTimeout/,
    'iOS 현재 페이지 paint도 100ms debounce를 기다리면 안 된다',
  );
});

test('IME 조합 caret은 매 갱신마다 anchor의 현재 위치를 다시 조회한다', () => {
  // [#4150] 조합 시작 좌표를 compositionAnchorRect 필드에 캐시하고 조합 갱신마다
  // 재사용하던 이전 구현은, 캐시를 무효화해야 하는 호출부(afterPageLocalEdit) 중
  // 하나가 누락되면서 같은 페이지 안 reflow(줄바꿈) 뒤에도 옛 좌표에 조합
  // 오버레이가 그려져 실제 캔버스 텍스트와 겹쳤다(issue #4150). 캐시 자체를
  // 없애고 anchor의 실제 위치를 매번 다시 조회하면, 무효화를 잊는 호출부가
  // 늘어나도 이 버그 계열이 재발할 수 없다.
  const inputHandlerSource = readFileSync(new URL('../src/engine/input-handler.ts', import.meta.url), 'utf8');
  const textSource = readFileSync(new URL('../src/engine/input-handler-text.ts', import.meta.url), 'utf8');

  assert.doesNotMatch(
    inputHandlerSource,
    /compositionAnchorRect/,
    '조합 anchor 좌표는 캐시하지 않는다 — 캐시가 있으면 무효화를 잊는 호출부가 재발할 수 있다',
  );
  assert.match(
    inputHandlerSource,
    /let startRect: CursorRect;\s*if \(this\.cursor\.isInHeaderFooter\(\)\) \{[\s\S]*?this\.wasm\.getCursorRectInCell\(/,
    '조합 caret 갱신마다 anchor의 exact 위치를 다시 조회해야 한다(캐시 히트 분기가 없어야 함)',
  );
  assert.doesNotMatch(textSource, /captureCompositionAnchorRect|clearCompositionAnchorRect/);
});

test('raw 셀 입력은 command와 같은 typed mutation helper를 사용한다', () => {
  const textSource = readFileSync(new URL('../src/engine/input-handler-text.ts', import.meta.url), 'utf8');

  assert.match(
    textSource,
    /export function insertTextAtRaw\([\s\S]*?\): TextMutationEffects \{[\s\S]*?return insertTextWithMutationEffects\(this\.wasm, pos, text\);\s*\}/,
  );
  assert.match(
    textSource,
    /export function deleteTextAt\([\s\S]*?\): TextMutationEffects \{[\s\S]*?return deleteTextWithMutationEffects\(this\.wasm, pos, count\);\s*\}/,
  );
  const inputHandlerSource = readFileSync(new URL('../src/engine/input-handler.ts', import.meta.url), 'utf8');
  assert.match(
    inputHandlerSource,
    /private deleteTextAt\(pos: DocumentPosition, count: number\): void \{\s*this\.rawTextMutationEffects\.add\(_text\.deleteTextAt\.call\(this, pos, count\)\);\s*\}/,
    'IME 조합 치환의 delete와 insert effect를 한 accumulator에서 OR 누적해야 한다',
  );
});

test('depth-1 셀 IME replacement는 body fallback보다 먼저 atomic helper를 사용한다', () => {
  const textSource = readFileSync(
    new URL('../src/engine/input-handler-text.ts', import.meta.url),
    'utf8',
  );
  const replaceStart = textSource.indexOf('export function replaceTextAtRaw(');
  const deleteStart = textSource.indexOf('export function deleteTextAt(', replaceStart);
  const replaceSource = textSource.slice(replaceStart, deleteStart);

  assert.match(
    replaceSource,
    /canUseDeferredCellTextReplace\(pos, deleteCount, text\)/,
  );
  assert.match(
    replaceSource,
    /return replaceCellTextWithMutationEffects\(this\.wasm, pos, deleteCount, text\);/,
  );
  assert.ok(
    replaceSource.indexOf('canUseDeferredCellTextReplace') <
      replaceSource.indexOf('canUseLocalBodyTextReplace'),
    'cell atomic route must be checked before the body-only route',
  );
});

test('deferred pending이 실제로 있을 때만 page-local idle flush를 예약한다', () => {
  const inputHandlerSource = readFileSync(new URL('../src/engine/input-handler.ts', import.meta.url), 'utf8');
  const bridgeSource = readFileSync(new URL('../src/core/wasm-bridge.ts', import.meta.url), 'utf8');

  assert.match(
    inputHandlerSource,
    /if \(this\.deferredPaginationPending\) \{\s*this\.scheduleDeferredPaginationFlush\(\);\s*\}/,
  );
  assert.match(
    bridgeSource,
    /cellFlowChanged: paginationDeferred && parsed\.cellFlowChanged !== false/,
    '구형 deferred 결과의 누락 신호는 mutation 후 예외 대신 보수적 경계로 복구해야 한다',
  );
  assert.match(inputHandlerSource, /if \(!this\.deferredPaginationPending\) return false;/);
  assert.match(
    inputHandlerSource,
    /if \(effects\.paginationCompleted\) \{\s*this\.cancelDeferredPaginationFlush\(\);\s*this\.deferredPaginationRunner\.cancel\(\);\s*this\.deferredPaginationPending = false;\s*\}/,
  );
  assert.match(inputHandlerSource, /if \(effects\.flowChanged && effects\.paginationCompleted\) return true;/);
  assert.match(inputHandlerSource, /if \(!effects\.documentPaginationPending\) return false;/);
  assert.match(
    inputHandlerSource,
    /const replacesPendingJob = this\.deferredPaginationRunner\.hasPendingWork\(\);/,
  );
  assert.match(
    inputHandlerSource,
    /if \(!effects\.flowChanged && !replacesPendingJob\) return false;/,
  );
  assert.match(
    inputHandlerSource,
    /this\.deferredPaginationRunner\.requestStart\(\s*DOCUMENT_PAGINATION_RESTART_COALESCE_DELAY_MS,\s*DOCUMENT_PAGINATION_INITIAL_START_DELAY_MS,\s*DOCUMENT_PAGINATION_POST_FIRST_STEP_DELAY_MS,\s*\);/,
    'cell-flow 경계는 input stack 밖에서 latest-only resumable begin을 요청해야 한다',
  );
});

test('document pagination은 작은 문서의 120ms idle과 명시 boundary에서 flush된다', () => {
  const inputHandlerSource = readFileSync(new URL('../src/engine/input-handler.ts', import.meta.url), 'utf8');
  const keyboardSource = readFileSync(new URL('../src/engine/input-handler-keyboard.ts', import.meta.url), 'utf8');
  const textSource = readFileSync(new URL('../src/engine/input-handler-text.ts', import.meta.url), 'utf8');
  const fileSource = readFileSync(new URL('../src/command/commands/file.ts', import.meta.url), 'utf8');

  assert.match(
    inputHandlerSource,
    /const DOCUMENT_PAGINATION_IDLE_FLUSH_DELAY_MS = 120;/,
  );
  // [#3412] #3248 은 idle 병합을 도입하며 문서 크기 게이트를 지우고 그 부재를 여기서
  // 가드로 고정했다. 그러나 대형 문서에서는 그 flush 자체가 결함이다 — 115쪽 문서에서
  // 타이핑을 120ms 만 멈추면 동기 전체 pagination 이 839ms 동안 메인 스레드를 막고,
  // #2214 의 재개형 러너를 취소해 페이지-로컬 리페인트 계약(flush 0)을 깬다.
  // 그래서 idle 병합은 유지하되 대상은 작은 문서로 되돌린다. 큰 문서는 재개형 러너와
  // 명시 boundary flush(undo/redo/navigation/blur/저장·인쇄)로 마감한다.
  assert.match(inputHandlerSource, /const DOCUMENT_PAGINATION_IDLE_FLUSH_PAGE_LIMIT = 30;/);
  assert.match(
    inputHandlerSource,
    /if \(!this\.shouldAutoFlushDeferredPagination\(\)\) return;/,
    'idle flush 예약은 대상 판정을 통과한 문서에만 걸려야 한다',
  );
  assert.match(
    inputHandlerSource,
    /const DOCUMENT_PAGINATION_INITIAL_START_DELAY_MS = 100;/,
    '최초 begin은 입력 paint를 위한 고정 100ms timer target을 둔다',
  );
  assert.match(
    inputHandlerSource,
    /const DOCUMENT_PAGINATION_RESTART_COALESCE_DELAY_MS = 200;/,
    'active restart만 마지막 입력 뒤 200ms까지 합친다',
  );
  assert.match(
    inputHandlerSource,
    /const DOCUMENT_PAGINATION_POST_FIRST_STEP_DELAY_MS = 25;/,
    '첫 fragment 뒤 후속 step은 25ms settle gap 뒤 실행한다',
  );
  assert.match(
    inputHandlerSource,
    /private shouldAutoFlushDeferredPagination\(\): boolean \{\s*if \(this\.deferredPaginationRunner\.hasPendingWork\(\)\) return false;\s*return this\.wasm\.pageCount <= DOCUMENT_PAGINATION_IDLE_FLUSH_PAGE_LIMIT;\s*\}/,
    '예약 begin 또는 전진 중 job이 있으면 idle flush가 같은 일을 동기로 되풀이하면 안 된다',
  );
  assert.match(
    inputHandlerSource,
    /const shouldFlush = this\.deferredPaginationPending\s*\|\| this\.deferredPaginationFlushTimer !== null\s*\|\| this\.deferredPaginationRunner\.hasPendingWork\(\);/,
    '명시 barrier는 queued begin도 pending work로 취급해야 한다',
  );

  const undoStart = inputHandlerSource.indexOf('private handleUndo(): void {');
  const redoStart = inputHandlerSource.indexOf('private handleRedo(): void {');
  const restoreStart = inputHandlerSource.indexOf('private restoreEditContextAfterHistory', redoStart);
  const undoSource = inputHandlerSource.slice(undoStart, redoStart);
  const redoSource = inputHandlerSource.slice(redoStart, restoreStart);
  assert.ok(
    undoSource.indexOf("flushDeferredPaginationIfNeeded('before-undo', false)") <
      undoSource.indexOf('this.history.undo(this.wasm)'),
  );
  assert.ok(
    redoSource.indexOf("flushDeferredPaginationIfNeeded('before-redo', false)") <
      redoSource.indexOf('this.history.redo(this.wasm)'),
  );

  assert.match(keyboardSource, /PAGINATION_BOUNDARY_KEYS/);
  assert.match(
    keyboardSource,
    /this\.flushDeferredPaginationIfNeeded\('before-navigation', false\)/,
  );
  assert.match(
    textSource,
    /function processPendingNav[\s\S]*?this\.flushDeferredPaginationIfNeeded\('before-navigation', false\);/,
  );
  assert.match(inputHandlerSource, /private onInputBlurBound: \(\) => void;/);
  assert.match(
    inputHandlerSource,
    /this\.onInputBlurBound = \(\) => \{\s*this\.flushDeferredPaginationIfNeeded\('input-blur', false\);\s*\};/,
  );
  assert.match(inputHandlerSource, /this\.textarea\.addEventListener\('blur', this\.onInputBlurBound\);/);
  assert.match(inputHandlerSource, /this\.textarea\.removeEventListener\('blur', this\.onInputBlurBound\);/);
  assert.match(fileSource, /flushDeferredPaginationBeforeExplicitOutput/);
});

test('문서 전환은 deferred·IME·iOS 입력 세션 상태를 격리한다', () => {
  const inputHandlerSource = readFileSync(new URL('../src/engine/input-handler.ts', import.meta.url), 'utf8');
  const deactivateStart = inputHandlerSource.indexOf('deactivate(): void {');
  const disposeStart = inputHandlerSource.indexOf('dispose(): void {', deactivateStart);
  assert.ok(deactivateStart >= 0 && disposeStart > deactivateStart);
  const deactivateSource = inputHandlerSource.slice(deactivateStart, disposeStart);

  assert.ok(
    deactivateSource.indexOf("this.flushDeferredPaginationIfNeeded('before-deactivate', false)") <
      deactivateSource.indexOf('this.active = false'),
  );
  assert.match(deactivateSource, /this\.cancelDeferredPaginationFlush\(\);/);
  assert.match(deactivateSource, /this\.deferredPaginationRunner\.cancel\(\);/);
  assert.match(deactivateSource, /this\.deferredPaginationPending = false;/);
  assert.match(deactivateSource, /this\.resetRawTextMutationEffects\(\);/);
  assert.match(deactivateSource, /this\._lastComposedText = '';/);
  assert.match(deactivateSource, /this\._iosAnchor = null;/);
  assert.match(deactivateSource, /this\._iosRequiresFullRefresh = false;/);
  assert.match(deactivateSource, /this\.textarea\.value = '';/);
});

test('저장·다른 이름 저장·인쇄는 resumable job을 출력 전에 동기 barrier로 마감한다', () => {
  const fileCommandSource = readFileSync(
    new URL('../src/command/commands/file.ts', import.meta.url),
    'utf8',
  );
  assert.match(
    fileCommandSource,
    /function flushDeferredPaginationBeforeExplicitOutput\([\s\S]*?flushDeferredPaginationIfNeeded\(reason\);[\s\S]*?hasDeferredPaginationPending\(\)[\s\S]*?throw new Error/,
    'flush 실패 뒤 pending이 남으면 저장·인쇄를 중단해야 한다',
  );
  for (const reason of ['save', 'save-as', 'print']) {
    assert.match(
      fileCommandSource,
      new RegExp(`flushDeferredPaginationBeforeExplicitOutput\\(services, '${reason}'\\)`),
      `${reason} 경로는 export/render 전에 pending pagination을 마감해야 한다`,
    );
  }
});

test('isPageLocalTextEditCommand는 같은 본문 문단의 짧은 텍스트 편집을 허용한다', () => {
  const bodyPos: DocumentPosition = {
    sectionIndex: 0,
    paragraphIndex: 2,
    charOffset: 3,
  };

  assert.equal(
    isPageLocalTextEditCommand(
      'insertText',
      bodyPos,
      { ...bodyPos, charOffset: 4 },
      { insertedText: '가', beforePageIndex: 0, afterPageIndex: 0 },
    ),
    true,
  );
  assert.equal(
    isPageLocalTextEditCommand(
      'deleteText',
      bodyPos,
      bodyPos,
      { deleteCount: 1, beforePageIndex: 0, afterPageIndex: 0 },
    ),
    true,
  );
});

test('isPageLocalTextEditCommand는 본문 경계와 구조 변경을 full refresh로 남긴다', () => {
  const bodyPos: DocumentPosition = {
    sectionIndex: 0,
    paragraphIndex: 2,
    charOffset: 3,
  };

  assert.equal(
    isPageLocalTextEditCommand(
      'insertText',
      bodyPos,
      { ...bodyPos, paragraphIndex: 3, charOffset: 1 },
      { insertedText: '가' },
    ),
    false,
  );
  assert.equal(isPageLocalTextEditCommand('splitParagraphInCell', baseCellPos, baseCellPos), false);
  assert.equal(isPageLocalTextEditCommand('deleteSelection', baseCellPos, baseCellPos), false);
});

test('isPageLocalTextEditCommand는 셀 경로가 바뀌면 full refresh를 요구한다', () => {
  assert.equal(
    isPageLocalTextEditCommand('insertText', baseCellPos, {
      ...baseCellPos,
      cellPath: [{ controlIndex: 0, cellIndex: 2, cellParaIndex: 0 }],
      charOffset: 4,
    }),
    false,
  );
  assert.equal(
    isPageLocalTextEditCommand('insertText', baseCellPos, { ...baseCellPos, cellParaIndex: 1, charOffset: 4 }),
    false,
  );
});

test('isPageLocalTextEditCommand는 긴 단일 paste와 줄바꿈/탭 삽입을 full refresh로 남긴다', () => {
  const shortText = '가'.repeat(MAX_PAGE_LOCAL_TEXT_EDIT_CHARS);
  const longText = '가'.repeat(MAX_PAGE_LOCAL_TEXT_EDIT_CHARS + 1);

  assert.equal(
    isPageLocalTextEditCommand(
      'insertText',
      baseCellPos,
      { ...baseCellPos, charOffset: baseCellPos.charOffset + shortText.length },
      { insertedText: shortText },
    ),
    true,
  );
  assert.equal(
    isPageLocalTextEditCommand(
      'insertText',
      baseCellPos,
      { ...baseCellPos, charOffset: baseCellPos.charOffset + longText.length },
      { insertedText: longText },
    ),
    false,
  );
  assert.equal(
    isPageLocalTextEditCommand(
      'insertText',
      baseCellPos,
      { ...baseCellPos, charOffset: baseCellPos.charOffset + 3 },
      { insertedText: '가\n나' },
    ),
    false,
  );
  assert.equal(
    isPageLocalTextEditCommand(
      'insertText',
      baseCellPos,
      { ...baseCellPos, charOffset: baseCellPos.charOffset + 3 },
      { insertedText: '가\t나' },
    ),
    false,
  );
});

test('isPageLocalTextEditCommand는 큰 삭제와 페이지 이동을 full refresh로 남긴다', () => {
  assert.equal(
    isPageLocalTextEditCommand(
      'deleteText',
      baseCellPos,
      baseCellPos,
      { deleteCount: MAX_PAGE_LOCAL_TEXT_EDIT_CHARS + 1 },
    ),
    false,
  );
  assert.equal(
    isPageLocalTextEditCommand(
      'insertText',
      baseCellPos,
      { ...baseCellPos, charOffset: 4 },
      { insertedText: '가', beforePageIndex: 0, afterPageIndex: 1 },
    ),
    false,
  );
});
