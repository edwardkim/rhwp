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
});

test('raw 셀 입력은 command와 같은 typed mutation helper를 사용한다', () => {
  const textSource = readFileSync(new URL('../src/engine/input-handler-text.ts', import.meta.url), 'utf8');

  assert.match(
    textSource,
    /export function insertTextAtRaw\([\s\S]*?\): TextMutationEffects \{[\s\S]*?return insertTextWithMutationEffects\(this\.wasm, pos, text\);\s*\}/,
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
  assert.match(inputHandlerSource, /if \(effects\.paginationCompleted\) \{\s*this\.cancelDeferredPaginationFlush\(\);\s*this\.deferredPaginationPending = false;\s*\}/);
});

test('문서 전환은 deferred·IME·iOS 입력 세션 상태를 격리한다', () => {
  const inputHandlerSource = readFileSync(new URL('../src/engine/input-handler.ts', import.meta.url), 'utf8');
  const deactivateStart = inputHandlerSource.indexOf('deactivate(): void {');
  const disposeStart = inputHandlerSource.indexOf('dispose(): void {', deactivateStart);
  assert.ok(deactivateStart >= 0 && disposeStart > deactivateStart);
  const deactivateSource = inputHandlerSource.slice(deactivateStart, disposeStart);

  assert.match(deactivateSource, /this\.cancelDeferredPaginationFlush\(\);/);
  assert.match(deactivateSource, /this\.deferredPaginationPending = false;/);
  assert.match(deactivateSource, /this\.resetRawTextMutationEffects\(\);/);
  assert.match(deactivateSource, /this\._lastComposedText = '';/);
  assert.match(deactivateSource, /this\._iosAnchor = null;/);
  assert.match(deactivateSource, /this\._iosRequiresFullRefresh = false;/);
  assert.match(deactivateSource, /this\.textarea\.value = '';/);
});

test('isPageLocalTextEditCommand는 본문 텍스트와 구조 변경 명령을 full refresh로 남긴다', () => {
  const bodyPos: DocumentPosition = {
    sectionIndex: 0,
    paragraphIndex: 2,
    charOffset: 3,
  };

  assert.equal(isPageLocalTextEditCommand('insertText', bodyPos, { ...bodyPos, charOffset: 4 }), false);
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
