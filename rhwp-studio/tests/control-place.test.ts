import test from 'node:test';
import assert from 'node:assert/strict';
import { controlPlaceOf, fieldPinKey, positionOfPlace } from '../src/engine/control-place.ts';

test('본문·한 겹 셀·글상자·중첩 표 주소를 커서 위치와 맞바꾼다', () => {
  const body = { section: 1, para: 4, cellPath: [] as [number, number, number][] };
  assert.deepEqual(positionOfPlace(body, 2), { sectionIndex: 1, paragraphIndex: 4, charOffset: 2 });
  assert.deepEqual(controlPlaceOf(positionOfPlace(body, 2)), body);

  const cell = { section: 0, para: 3, cellPath: [[1, 5, 2]] as [number, number, number][] };
  const inCell = positionOfPlace(cell, 7, ['table']);
  assert.deepEqual(inCell, {
    sectionIndex: 0, paragraphIndex: 2, charOffset: 7,
    parentParaIndex: 3, controlIndex: 1, cellIndex: 5, cellParaIndex: 2,
  });
  assert.deepEqual(controlPlaceOf(inCell), cell);
  assert.equal(positionOfPlace(cell, 0, ['textBox']).isTextBox, true);

  const nested = { section: 0, para: 3, cellPath: [[1, 5, 2], [0, 1, 4]] as [number, number, number][] };
  const inNested = positionOfPlace(nested, 1, ['table', 'table']);
  assert.equal(inNested.paragraphIndex, 4);
  assert.equal(inNested.cellPath?.length, 2);
  assert.deepEqual(controlPlaceOf(inNested), nested);
});

test('F11 누름틀 열쇠는 같은 주소의 같은 글자 자리에서만 같다', () => {
  const at = (charOffset: number, para = 0) => ({ sectionIndex: 0, paragraphIndex: para, charOffset });
  assert.equal(fieldPinKey(at(2)), fieldPinKey(at(2)));
  assert.notEqual(fieldPinKey(at(2)), fieldPinKey(at(3)));
  assert.notEqual(fieldPinKey(at(2)), fieldPinKey({ ...at(2), parentParaIndex: 0, controlIndex: 1, cellIndex: 0, cellParaIndex: 0 }));
});
