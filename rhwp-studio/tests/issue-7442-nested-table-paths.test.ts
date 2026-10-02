// [Issue #7442] 중첩 표(칸 안 칸)의 크기 조절·칸 블록 선택·테두리 클릭 개체 선택.
//
// hitTest 가 중첩 칸의 빈 영역·괘선에서 바깥 칸의 깊이 1 cellPath 를 돌려주던
// 엔진 결함(수정됨) 때문에 Studio 의 평면 API 경로가 전부 바깥 표를 가리켰다.
// 이 시험이 잠그는 것:
//
// - hitTestCellRowCol 의 같은-표 규칙: 중첩 ctx 에서 깊이 1 · 형제 표 hit 는
//   (row, col) 으로 해석하지 않는다.
// - applyKeyboardResize / resizeTableProportional 이 중첩 ctx 에서 경로 API
//   (getTableCellBboxesByPath + resizeTableCellsByPath)를 쓰고, 깊이 1 에서는
//   평면 API 를 그대로 쓴다.
// - isSameNestedTablePath 의 접두사·마지막 controlIndex 계약.

import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { stripTypeScriptTypes } from 'node:module';
import {
  isSameNestedTablePath,
  ensureTableCellBboxCache,
  type CellPathStep,
} from '../src/engine/table-bbox-cache.ts';

const OUTER: CellPathStep[] = [
  { controlIndex: 0, cellIndex: 10, cellParaIndex: 0 },
];
const INNER_A: CellPathStep[] = [
  { controlIndex: 0, cellIndex: 10, cellParaIndex: 0 },
  { controlIndex: 0, cellIndex: 0, cellParaIndex: 0 },
];
const INNER_B_SAME_TABLE: CellPathStep[] = [
  { controlIndex: 0, cellIndex: 10, cellParaIndex: 0 },
  { controlIndex: 0, cellIndex: 5, cellParaIndex: 0 },
];
const SIBLING_TABLE: CellPathStep[] = [
  { controlIndex: 0, cellIndex: 10, cellParaIndex: 0 },
  { controlIndex: 1, cellIndex: 0, cellParaIndex: 0 },
];
const DEEPER: CellPathStep[] = [
  ...INNER_A,
  { controlIndex: 0, cellIndex: 0, cellParaIndex: 0 },
];

test('isSameNestedTablePath: 같은 안쪽 표의 다른 칸을 인정한다', () => {
  assert.equal(isSameNestedTablePath(INNER_A, INNER_B_SAME_TABLE), true);
});

test('isSameNestedTablePath: 깊이 1 hit(바깥 칸)은 거부한다', () => {
  assert.equal(isSameNestedTablePath(INNER_A, OUTER), false);
});

test('isSameNestedTablePath: 형제 표(마지막 controlIndex 다름)는 거부한다', () => {
  assert.equal(isSameNestedTablePath(INNER_A, SIBLING_TABLE), false);
});

test('isSameNestedTablePath: 더 깊은 경로와 깊이 1 ctx 는 거부한다', () => {
  assert.equal(isSameNestedTablePath(INNER_A, DEEPER), false);
  assert.equal(isSameNestedTablePath(OUTER, INNER_A), false);
  assert.equal(isSameNestedTablePath(INNER_A, undefined), false);
});

test('isSameNestedTablePath: 중간 마디의 칸이 다르면 거부한다', () => {
  const otherCell = [
    { controlIndex: 0, cellIndex: 11, cellParaIndex: 0 },
    { controlIndex: 0, cellIndex: 0, cellParaIndex: 0 },
  ];
  assert.equal(isSameNestedTablePath(INNER_A, otherCell), false);
});

// ─── hitTestCellRowCol / 키보드 리사이즈의 경로 선택 ───────────────────
// InputHandler 의 private 메서드는 DOM 어셈블리를 피하기 위해 소스를 잘라
// 필요한 의존(isSameNestedTablePath)만 주입해 실행한다.

const tableSource = readFileSync(
  new URL('../src/engine/input-handler-table.ts', import.meta.url),
  'utf8',
);

function sliceFn(source: string, startMarker: string, endMarker: string): string {
  const start = source.indexOf(startMarker);
  const end = source.indexOf(endMarker, start);
  assert.ok(start >= 0 && end > start, `slice ${startMarker}`);
  return stripTypeScriptTypes(source.slice(start, end));
}

// applyKeyboardResize 본문 — build 함수는 인자로 주입된다.
const applyKeyboardResize = new Function(
  `${sliceFn(tableSource, 'function applyKeyboardResize(', '\n/** Ctrl/Cmd+방향키')}
   return applyKeyboardResize;`,
)();

const resizeTableProportional = new Function(
  `${stripTypeScriptTypes(
    tableSource.slice(tableSource.indexOf('export function resizeTableProportional')),
  ).replace('export function', 'function')}
   return resizeTableProportional;`,
)();

function mockHost(ctx: any) {
  const calls: string[] = [];
  const updatesSeen: unknown[] = [];
  const host: any = {
    calls,
    updatesSeen,
    cursor: {
      getCellTableContext: () => ctx,
      getSelectedCellRange: () => ({ startRow: 0, startCol: 0, endRow: 0, endCol: 0 }),
      getPosition: () => ({ sectionIndex: 0, paragraphIndex: 0, charOffset: 0 }),
    },
    wasm: {
      getTableCellBboxes() {
        calls.push('flat:bboxes');
        return [{ cellIdx: 0, row: 0, col: 0, rowSpan: 1, colSpan: 1, pageIndex: 0, x: 0, y: 0, w: 40, h: 20 }];
      },
      getTableCellBboxesByPath(_sec: number, _ppi: number, pathJson: string) {
        calls.push(`path:bboxes:${pathJson}`);
        return [{ cellIdx: 0, row: 0, col: 0, rowSpan: 1, colSpan: 1, pageIndex: 0, x: 0, y: 0, w: 40, h: 20 }];
      },
      resizeTableCells(_s: number, _p: number, _c: number, updates: unknown) {
        calls.push('flat:resize');
        updatesSeen.push(updates);
      },
      resizeTableCellsByPath(_s: number, _p: number, pathJson: string, updates: unknown) {
        calls.push(`path:resize:${pathJson}`);
        updatesSeen.push(updates);
      },
    },
    executeOperation(op: any) {
      calls.push('exec');
      op.operation(host.wasm);
    },
    updateCellSelection() {},
  };
  return host;
}

test('resizeTableProportional: 중첩 ctx 는 경로 API 로 조회·적용한다', () => {
  const host = mockHost({ sec: 0, ppi: 5, ci: 0, cellPath: INNER_A });
  resizeTableProportional.call(host, 'ArrowRight');
  const pathJson = JSON.stringify(INNER_A);
  assert.deepEqual(host.calls, [
    `path:bboxes:${pathJson}`,
    'exec',
    `path:resize:${pathJson}`,
  ]);
});

test('resizeTableProportional: 깊이 1 ctx 는 평면 API 를 유지한다', () => {
  const host = mockHost({ sec: 0, ppi: 5, ci: 0, cellPath: OUTER });
  resizeTableProportional.call(host, 'ArrowRight');
  assert.deepEqual(host.calls, ['flat:bboxes', 'exec', 'flat:resize']);
});

test('applyKeyboardResize: 중첩 ctx 는 경로 API 로 조회·적용한다', () => {
  const host = mockHost({ sec: 0, ppi: 5, ci: 0, cellPath: INNER_A });
  applyKeyboardResize.call(
    host, 'ArrowRight', 'resizeCellByKeyboard',
    (bboxes: unknown[]) => bboxes.map(() => ({ cellIdx: 0, widthDelta: 200 })),
  );
  const pathJson = JSON.stringify(INNER_A);
  assert.deepEqual(host.calls, [
    `path:bboxes:${pathJson}`,
    'exec',
    `path:resize:${pathJson}`,
  ]);
});

test('applyKeyboardResize: 깊이 1 ctx 는 평면 API 를 유지한다', () => {
  const host = mockHost({ sec: 0, ppi: 5, ci: 0, cellPath: OUTER });
  applyKeyboardResize.call(
    host, 'ArrowRight', 'resizeCellByKeyboard',
    () => [{ cellIdx: 0, widthDelta: 200 }],
  );
  assert.deepEqual(host.calls, ['flat:bboxes', 'exec', 'flat:resize']);
});

// hitTestCellRowCol — isSameNestedTablePath 를 주입해 같은-표 규칙만 검증한다.
const inputSource = readFileSync(
  new URL('../src/engine/input-handler.ts', import.meta.url),
  'utf8',
);
const hitSliceStart = inputSource.indexOf('private hitTestCellRowCol(');
const hitSliceEnd = inputSource.indexOf('\n  /** F5 셀 선택', hitSliceStart);
assert.ok(hitSliceStart >= 0 && hitSliceEnd > hitSliceStart);
const hitTestCellRowCol = new Function(
  'isSameNestedTablePath',
  `${stripTypeScriptTypes(
    inputSource
      .slice(hitSliceStart, hitSliceEnd)
      .replace('private hitTestCellRowCol', 'function hitTestCellRowCol'),
  )}
   return hitTestCellRowCol;`,
)(isSameNestedTablePath);

function mockClickHost(ctx: any, hit: any) {
  const calls: string[] = [];
  const host: any = {
    calls,
    cursor: { getCellTableContext: () => ctx },
    viewportManager: { getZoom: () => 1 },
    container: {
      querySelector: () => ({ getBoundingClientRect: () => ({ left: 0, top: 0 }) }),
    },
    virtualScroll: {
      getPageAtPoint: () => 0,
      getPageOffset: () => 0,
      getPageWidth: () => 800,
      getPageLeftResolved: () => 0,
    },
    wasm: {
      hitTest: () => hit,
      getCellInfoByPath: () => {
        calls.push('getCellInfoByPath');
        return { row: 3, col: 2 };
      },
      getCellInfo: () => {
        calls.push('getCellInfo');
        return { row: 0, col: 0 };
      },
    },
  };
  return host;
}

const fakeEvent = { clientX: 10, clientY: 10 } as MouseEvent;

test('hitTestCellRowCol: 같은 중첩 표 hit 는 경로 API 로 (row,col)을 얻는다', () => {
  const ctx = { sec: 0, ppi: 5, ci: 0, cellPath: INNER_A };
  const hit = {
    sectionIndex: 0, parentParaIndex: 5, controlIndex: 0, cellIndex: 5,
    cellPath: INNER_B_SAME_TABLE,
  };
  const host = mockClickHost(ctx, hit);
  assert.deepEqual(hitTestCellRowCol.call(host, fakeEvent), { row: 3, col: 2 });
  assert.deepEqual(host.calls, ['getCellInfoByPath']);
});

test('hitTestCellRowCol: 중첩 ctx 의 깊이 1 hit 은 null 이다', () => {
  const ctx = { sec: 0, ppi: 5, ci: 0, cellPath: INNER_A };
  const hit = {
    sectionIndex: 0, parentParaIndex: 5, controlIndex: 0, cellIndex: 10,
    cellPath: OUTER,
  };
  const host = mockClickHost(ctx, hit);
  assert.equal(hitTestCellRowCol.call(host, fakeEvent), null);
  assert.deepEqual(host.calls, []);
});

test('hitTestCellRowCol: 중첩 ctx 의 형제 표 hit 은 null 이다', () => {
  const ctx = { sec: 0, ppi: 5, ci: 0, cellPath: INNER_A };
  const hit = {
    sectionIndex: 0, parentParaIndex: 5, controlIndex: 0, cellIndex: 0,
    cellPath: SIBLING_TABLE,
  };
  const host = mockClickHost(ctx, hit);
  assert.equal(hitTestCellRowCol.call(host, fakeEvent), null);
  assert.deepEqual(host.calls, []);
});

test('hitTestCellRowCol: 다른 구역의 같은 번호 표는 조회하지 않는다', () => {
  for (const cellPath of [OUTER, INNER_A]) {
    const host = mockClickHost(
      { sec: 0, ppi: 5, ci: 0, cellPath },
      { sectionIndex: 1, parentParaIndex: 5, controlIndex: 0, cellIndex: 0, cellPath },
    );
    assert.equal(hitTestCellRowCol.call(host, fakeEvent), null);
    assert.deepEqual(host.calls, []);
  }
});

test('중첩 표 캐시는 마지막 셀·문단 이동에도 재조회하지 않는다', () => {
  let queries = 0;
  const host: any = {
    wasm: {
      getTableCellBboxes() { assert.fail('중첩 표의 평면 조회'); },
      getTableCellBboxesByPath() { queries++; return [{ pageIndex: 0 }]; },
    },
    cachedTableRef: null, cachedCellBboxes: null, tableBboxFetchFailures: new Set(),
  };
  const otherParagraph = [INNER_A[0], { ...INNER_B_SAME_TABLE[1], cellParaIndex: 3 }];
  for (const path of [INNER_A, otherParagraph, INNER_A]) {
    ensureTableCellBboxCache(host, { sec: 0, ppi: 5, ci: 0, path }, 0);
  }
  assert.equal(queries, 1);
  // 같은 바깥 셀의 다른 문단에 놓인 표와 형제 control은 별개다.
  for (const path of [
    [{ ...INNER_A[0], cellParaIndex: 1 }, INNER_A[1]],
    SIBLING_TABLE,
  ]) {
    ensureTableCellBboxCache(host, { sec: 0, ppi: 5, ci: 0, path }, 0);
  }
  assert.equal(queries, 3);
});

test('중첩 표 조회 실패도 같은 표의 셀 이동에서 반복하지 않는다', () => {
  let queries = 0;
  const host: any = {
    wasm: {
      getTableCellBboxes() { assert.fail('중첩 표의 평면 조회'); },
      getTableCellBboxesByPath() { queries++; return []; },
    },
    cachedTableRef: null, cachedCellBboxes: null, tableBboxFetchFailures: new Set(),
  };
  for (const path of [INNER_A, INNER_B_SAME_TABLE, INNER_A]) {
    assert.equal(ensureTableCellBboxCache(host, { sec: 0, ppi: 5, ci: 0, path }, 0), null);
  }
  assert.equal(queries, 1);
});

test('hitTestCellRowCol: 깊이 1 ctx 는 평면 getCellInfo 를 유지한다', () => {
  const ctx = { sec: 0, ppi: 5, ci: 0, cellPath: OUTER };
  const hit = {
    sectionIndex: 0, parentParaIndex: 5, controlIndex: 0, cellIndex: 4,
    cellPath: OUTER,
  };
  const host = mockClickHost(ctx, hit);
  assert.deepEqual(hitTestCellRowCol.call(host, fakeEvent), { row: 0, col: 0 });
  assert.deepEqual(host.calls, ['getCellInfo']);
});
