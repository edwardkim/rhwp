import test from 'node:test';
import assert from 'node:assert/strict';
import { atOtherSelectedField, controlPlaceOf, fieldInfoAt, positionOfPlace, selectedFieldAt } from '../src/engine/control-place.ts';

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

test('입력 후 선택한 필드의 현재 범위와 셀 주소를 따른다', () => {
  const place = { section: 1, para: 3, cellPath: [[0, 2, 0]] as [number, number, number][] };
  const pin = { startOffset: 2, place, controlIndex: 1, info: { fieldId: 73, fieldType: 'clickhere' } };
  const other = { inField: true, fieldId: 72 };
  const wasm = {
    getFieldInfoAt: () => other,
    getFieldInfoForControlAt: (address: typeof place, index: number, offset: number) => {
      assert.deepEqual(address, place);
      assert.equal(index, 1);
      return { inField: offset >= 2 && offset <= 4, fieldId: 73 };
    },
  };
  for (const offset of [2, 3, 4]) {
    assert.equal(fieldInfoAt(wasm, positionOfPlace(place, offset), pin).fieldId, 73);
  }
  const at = positionOfPlace(place, 3);
  assert.equal(fieldInfoAt(wasm, positionOfPlace(place, 5), pin), other);
  assert.equal(fieldInfoAt(wasm, { ...at, cellIndex: 0 }, pin), other);
  assert.equal(fieldInfoAt(wasm, { ...at, sectionIndex: 0 }, pin), other);
  assert.equal(selectedFieldAt(wasm, at, { ...pin, info: { ...pin.info, fieldId: 74 } }), null);
});

test('일반 필드에 머무는 동안 F11은 같은 컨트롤에서 이어 간다', () => {
  const place = { section: 0, para: 1, cellPath: [] };
  const pin = { startOffset: 3, place, controlIndex: 1, info: { fieldType: 'date' } };
  assert.equal(atOtherSelectedField(positionOfPlace(place, 3), pin), true);
  assert.equal(atOtherSelectedField(positionOfPlace(place, 4), pin), false);
  assert.equal(atOtherSelectedField(positionOfPlace({ ...place, para: 2 }, 3), pin), false);
});
