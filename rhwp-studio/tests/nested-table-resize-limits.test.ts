import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { stripTypeScriptTypes } from 'node:module';

// 최종 mouse-up이 사용하는 실제 clamp 함수를 실행한다. DOM과 명령 모듈은 이 계산에 필요 없다.
const source = readFileSync(new URL('../src/engine/input-handler-table.ts', import.meta.url), 'utf8');
const start = source.indexOf('function clampCompensatedResizeDelta(');
const end = source.indexOf('\nexport function startResizeDrag', start);
assert.ok(start >= 0 && end > start);
const clamp = new Function('MIN_TABLE_CELL_SIZE_HWP',
  `${stripTypeScriptTypes(source.slice(start, end))}; return clampCompensatedResizeDelta;`)(200);
const path = [
  { controlIndex: 0, cellIndex: 0, cellParaIndex: 9 },
  { controlIndex: 0, cellIndex: 0, cellParaIndex: 0 },
];
const ref = { sec: 0, ppi: 2, ci: 0, path };
const pairs = [{ targetCellIdx: 0, neighborCellIdxs: [1] }];

for (const [label, requested, expected] of [['expand', 9000, 300], ['shrink', -9000, -3800]] as const) {
  test(`nested ${label}: both cells retain their own minimum size and total width`, () => {
    let flatReads = 0;
    const wasm = {
      getCellProperties() { flatReads++; return { width: 30000, height: 9000 }; },
      getCellPropertiesByPath(sec: number, ppi: number, json: string, cell: number) {
        assert.equal(sec, 0); assert.equal(ppi, 2); assert.deepEqual(JSON.parse(json), path);
        return { width: [4000, 500][cell], height: 3000 };
      },
    };
    const delta = clamp(wasm, ref, { type: 'col' }, pairs, requested);
    assert.equal(delta, expected);
    assert.equal(flatReads, 0);
    assert.ok(4000 + delta >= 200 && 500 - delta >= 200);
    assert.equal((4000 + delta) + (500 - delta), 4500);
  });
}

test('flat resize retains its original property API and row limit', () => {
  const wasm = {
    getCellProperties(_s: number, _p: number, _c: number, cell: number) {
      return { width: 4000, height: [1000, 250][cell] };
    },
    getCellPropertiesByPath() { assert.fail('flat resize must not require a nested API'); },
  };
  assert.equal(clamp(wasm, { sec: 0, ppi: 0, ci: 0 }, { type: 'row' }, pairs, 500), 50);
});
