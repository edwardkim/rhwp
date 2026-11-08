// 관계형: 독립 도형과 그림의 소유 경로를 비교한다. 실물 px 좌표를 고정하지 않는다.
// 주석 갱신: 2026-11-05 18:52 KST. 합성 주소의 API 계약으로 Native/fresh WASM 시각 수치는 비대상이다.
import test from 'node:test';
import assert from 'node:assert/strict';
import { isNestedCellDescendantOfControl } from '../src/engine/picture-hit-policy.ts';

const owner = { secIdx: 0, paraIdx: 9 };
const cell = { controlIndex: 0, cellIndex: 2, cellParaIndex: 1 };

test('#7375 같은 셀의 번호 도형은 뒤 그림의 컨테이너가 아니다', () => {
  const shape = { ...owner, controlIdx: 0, cellPath: [cell] };
  const picture = { ...owner, cellPath: [cell] };
  assert.equal(isNestedCellDescendantOfControl(shape, picture), false);
});

test('#7375 셀 안의 실제 글상자는 자신의 다음 깊이 그림을 소유한다', () => {
  const shape = { ...owner, controlIdx: 4, cellPath: [cell] };
  const picture = {
    ...owner,
    cellPath: [cell, { controlIndex: 4, cellIndex: 0, cellParaIndex: 0 }],
  };
  assert.equal(isNestedCellDescendantOfControl(shape, picture), true);
});

test('#7375 다른 셀이나 셀 문단의 같은 control 번호는 조상 관계가 아니다', () => {
  const shape = { ...owner, controlIdx: 4, cellPath: [cell] };
  for (const sibling of [
    { ...cell, cellIndex: 3 },
    { ...cell, cellParaIndex: 2 },
  ]) {
    const picture = {
      ...owner,
      cellPath: [sibling, { controlIndex: 4, cellIndex: 0, cellParaIndex: 0 }],
    };
    assert.equal(isNestedCellDescendantOfControl(shape, picture), false);
  }
});

test('#7375 앞선 깊이에 같은 번호가 있어도 자신의 자식이 아니면 제외한다', () => {
  const shape = { ...owner, controlIdx: 0, cellPath: [cell] };
  const picture = {
    ...owner,
    cellPath: [cell, { controlIndex: 4, cellIndex: 0, cellParaIndex: 0 }],
  };
  assert.equal(isNestedCellDescendantOfControl(shape, picture), false);
});

test('#7375 셀 경로 필드 별칭도 동일한 소유 주소로 판정한다', () => {
  const shape = {
    ...owner, controlIdx: 4,
    cellPath: [{ controlIdx: 0, cellIdx: 2, cellParaIdx: 1 }],
  };
  const picture = {
    ...owner,
    cellPath: [cell, { controlIndex: 4, cellIndex: 0, cellParaIndex: 0 }],
  };
  assert.equal(isNestedCellDescendantOfControl(shape, picture), true);
});
