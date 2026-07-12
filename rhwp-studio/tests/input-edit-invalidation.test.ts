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

test('raw IME/iOS 입력도 command 경로와 같은 page-local hint를 전달한다', () => {
  const inputHandlerSource = readFileSync(new URL('../src/engine/input-handler.ts', import.meta.url), 'utf8');
  const textSource = readFileSync(new URL('../src/engine/input-handler-text.ts', import.meta.url), 'utf8');

  assert.match(
    inputHandlerSource,
    /private afterTextInputEdit\(\s*beforePos: DocumentPosition,\s*afterPos: DocumentPosition,\s*pageLocalOptions: PageLocalTextEditOptions = \{\},\s*\): void \{\s*if \(this\.shouldUsePageLocalRefresh\('insertText', beforePos, afterPos, pageLocalOptions\)\)/,
  );
  assert.match(
    textSource,
    /this\.afterTextInputEdit\(anchor, afterPos, \{\s*insertedText: text,\s*beforePageIndex,\s*afterPageIndex,\s*\}\);/,
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
    /this\.afterTextInputEdit\(iosAnchor, iosAfterPos, \{\s*insertedText: text,\s*beforePageIndex,\s*afterPageIndex,\s*\}\);/,
  );
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
