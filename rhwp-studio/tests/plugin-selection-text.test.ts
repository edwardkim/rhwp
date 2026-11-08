import test from 'node:test';
import assert from 'node:assert/strict';

import type { DocumentPosition } from '../src/core/types.ts';
import { readSelectionText, type SelectionTextReader } from '../src/plugin/selection-text.ts';

const body = ['첫째 문단', '둘째 문단입니다', '', '넷째'];
const cell = ['칸 하나', '칸 둘째 줄'];

const reader: SelectionTextReader = {
  getParagraphLength: (_sec, para) => body[para].length,
  getTextRange: (_sec, para, offset, count) => body[para].slice(offset, offset + count),
  getCellParagraphLengthByPath: (_sec, _parent, pathJson) =>
    cell[JSON.parse(pathJson).at(-1).cellParaIndex].length,
  getTextInCellByPath: (_sec, _parent, pathJson, offset, count) =>
    cell[JSON.parse(pathJson).at(-1).cellParaIndex].slice(offset, offset + count),
};

const bodyPos = (paragraphIndex: number, charOffset: number): DocumentPosition =>
  ({ sectionIndex: 0, paragraphIndex, charOffset });
const cellPos = (cellParaIndex: number, charOffset: number, cellIndex = 0): DocumentPosition => ({
  sectionIndex: 0, paragraphIndex: 5, charOffset, parentParaIndex: 5,
  cellPath: [{ controlIndex: 0, cellIndex, cellParaIndex }],
});

test('본문 한 문단 안의 선택', () => {
  assert.deepEqual(readSelectionText(reader, bodyPos(1, 3), bodyPos(1, 5), 100),
    { text: '문단', truncated: false });
});

test('본문 여러 문단은 줄바꿈으로 잇고 빈 문단도 보존한다', () => {
  assert.deepEqual(readSelectionText(reader, bodyPos(0, 3), bodyPos(3, 1), 100),
    { text: '문단\n둘째 문단입니다\n\n넷', truncated: false });
});

test('한 칸 안의 칸 문단 범위', () => {
  assert.deepEqual(readSelectionText(reader, cellPos(0, 2), cellPos(1, 4), 100),
    { text: '하나\n칸 둘째', truncated: false });
});

test('레거시 평면 셀 좌표도 같은 칸으로 읽는다', () => {
  const legacy = (cellParaIndex: number, charOffset: number): DocumentPosition => ({
    sectionIndex: 0, paragraphIndex: 5, charOffset,
    parentParaIndex: 5, controlIndex: 0, cellIndex: 0, cellParaIndex,
  });
  assert.equal(readSelectionText(reader, legacy(0, 0), legacy(0, 3), 100)?.text, '칸 하');
});

test('빈 선택·다른 칸·본문과 칸 혼합·글상자·다른 구역은 null', () => {
  assert.equal(readSelectionText(reader, bodyPos(1, 2), bodyPos(1, 2), 100), null);
  assert.equal(readSelectionText(reader, cellPos(0, 0, 0), cellPos(0, 1, 1), 100), null);
  assert.equal(readSelectionText(reader, bodyPos(0, 0), cellPos(0, 1), 100), null);
  assert.equal(readSelectionText(reader, { ...bodyPos(0, 0), isTextBox: true }, bodyPos(0, 2), 100), null);
  assert.equal(readSelectionText(reader, bodyPos(0, 0), { ...bodyPos(1, 1), sectionIndex: 1 }, 100), null);
});

test('상한을 넘으면 앞쪽만 담고 truncated 를 켠다', () => {
  assert.deepEqual(readSelectionText(reader, bodyPos(0, 0), bodyPos(1, 8), 8),
    { text: '첫째 문단\n둘째', truncated: true });
});

test('대리 쌍 한가운데서 자르지 않는다', () => {
  const emoji: SelectionTextReader = {
    ...reader,
    getParagraphLength: () => 4,
    getTextRange: () => 'a😀b',
  };
  assert.deepEqual(readSelectionText(emoji, bodyPos(0, 0), bodyPos(0, 4), 2),
    { text: 'a', truncated: true });
});
