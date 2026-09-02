import test from 'node:test';
import assert from 'node:assert/strict';

import {
  cacheTableCellBboxes,
  ensureTableCellBboxCache,
  type TableBboxCacheHost,
  type PageScopedBbox,
} from '../src/engine/table-bbox-cache.ts';

interface Bbox extends PageScopedBbox {
  cellIdx: number;
}

function makeHost(
  impl: (sec: number, ppi: number, ci: number, pageHint?: number) => Bbox[],
): TableBboxCacheHost<Bbox> & { calls: Array<number | undefined> } {
  const host = {
    calls: [] as Array<number | undefined>,
    wasm: {
      getTableCellBboxes(sec: number, ppi: number, ci: number, pageHint?: number): Bbox[] {
        host.calls.push(pageHint);
        return impl(sec, ppi, ci, pageHint);
      },
    },
    cachedTableRef: null,
    cachedCellBboxes: null,
    tableBboxFetchFailures: new Set<string>(),
  };
  return host;
}

const T = { sec: 0, ppi: 3, ci: 1 };
const page0Cells: Bbox[] = [
  { pageIndex: 0, cellIdx: 0 },
  { pageIndex: 0, cellIdx: 1 },
];

test('첫 hover 는 현재 페이지를 hint 로 1회 조회하고 캐시를 채운다 (#4117)', () => {
  const host = makeHost(() => page0Cells);
  const got = ensureTableCellBboxCache(host, T, 0);
  assert.deepEqual(got, page0Cells);
  assert.deepEqual(host.calls, [0], 'pageHint 로 현재 페이지가 전달되어야 함');
  assert.deepEqual(host.cachedTableRef, { ...T, pageHint: 0, pageIndexes: new Set([0]) });
  assert.equal(host.cachedCellBboxes, page0Cells);
});

test('같은 표 위 이동은 엔진을 다시 부르지 않는다 — task 2010 계약 (#4117)', () => {
  const host = makeHost(() => page0Cells);
  ensureTableCellBboxCache(host, T, 0);
  for (let i = 0; i < 50; i++) {
    assert.equal(ensureTableCellBboxCache(host, T, 0), page0Cells);
  }
  assert.equal(host.calls.length, 1, '이동 50회에도 엔진 호출은 최초 1회여야 함');
});

test('여러 쪽에 걸친 표: 캐시가 요구 페이지를 담고 있으면 hint 가 달라도 재조회하지 않는다', () => {
  const spanning: Bbox[] = [
    { pageIndex: 0, cellIdx: 0 },
    { pageIndex: 1, cellIdx: 1 },
  ];
  const host = makeHost(() => spanning);
  ensureTableCellBboxCache(host, T, 0);
  assert.equal(ensureTableCellBboxCache(host, T, 1), spanning, '페이지 1도 캐시로 응답');
  assert.equal(host.calls.length, 1);
});

test('캐시에 없는 페이지를 요구하면 그 페이지를 hint 로 재조회한다 (뒤 페이지에서 시작한 표)', () => {
  const byHint = (hint?: number): Bbox[] =>
    hint === 3
      ? [{ pageIndex: 3, cellIdx: 0 }, { pageIndex: 4, cellIdx: 1 }]
      : [{ pageIndex: 4, cellIdx: 1 }];
  const host = makeHost((_s, _p, _c, hint) => byHint(hint));
  ensureTableCellBboxCache(host, T, 4);
  const got = ensureTableCellBboxCache(host, T, 3);
  assert.deepEqual(host.calls, [4, 3]);
  assert.equal(got?.some((b) => b.pageIndex === 3), true);
});

test('다른 표로 넘어가면 재조회하고 캐시를 교체한다', () => {
  const other = { sec: 0, ppi: 9, ci: 0 };
  const otherCells: Bbox[] = [{ pageIndex: 0, cellIdx: 7 }];
  const host = makeHost((_s, ppi) => (ppi === 9 ? otherCells : page0Cells));
  ensureTableCellBboxCache(host, T, 0);
  assert.equal(ensureTableCellBboxCache(host, other, 0), otherCells);
  assert.equal(host.calls.length, 2);
  assert.deepEqual(host.cachedTableRef, { ...other, pageHint: 0, pageIndexes: new Set([0]) });
});

test('빈 결과는 (표, 페이지)당 1회만 시도한다 — 이동마다 재시도 금지 (#4117)', () => {
  const host = makeHost(() => []);
  assert.equal(ensureTableCellBboxCache(host, T, 0), null);
  for (let i = 0; i < 20; i++) {
    assert.equal(ensureTableCellBboxCache(host, T, 0), null);
  }
  assert.equal(host.calls.length, 1, '실패 후 이동 20회에도 재시도 없음');
  assert.equal(host.tableBboxFetchFailures.size, 1);
});

test('예외도 실패 메모로 흡수한다', () => {
  const host = makeHost(() => { throw new Error('wasm 오류'); });
  assert.equal(ensureTableCellBboxCache(host, T, 0), null);
  assert.equal(ensureTableCellBboxCache(host, T, 0), null);
  assert.equal(host.calls.length, 1);
});

test('서로 다른 페이지의 실패를 각각 기억해 어느 페이지도 반복 조회하지 않는다', () => {
  const host = makeHost(() => []);
  assert.equal(ensureTableCellBboxCache(host, T, 0), null);
  assert.equal(ensureTableCellBboxCache(host, T, 1), null);
  assert.equal(ensureTableCellBboxCache(host, T, 0), null);
  assert.equal(ensureTableCellBboxCache(host, T, 1), null);
  assert.deepEqual(host.calls, [0, 1], '문서 변경 전에는 (표, 페이지)당 한 번만 조회해야 함');
});

test('다른 경로의 성공 cache는 같은 표·페이지의 과거 실패를 해제한다', () => {
  let empty = true;
  const host = makeHost(() => (empty ? [] : page0Cells));
  assert.equal(ensureTableCellBboxCache(host, T, 0), null);

  // 셀 선택 mousedown의 직접 성공 경로를 모사한다.
  empty = false;
  const direct = host.wasm.getTableCellBboxes(T.sec, T.ppi, T.ci, 0);
  cacheTableCellBboxes(host, T, 0, direct);

  // 표 밖 이동/resize cleanup 뒤 같은 페이지에 돌아오면 새 조회가 가능해야 한다.
  host.cachedTableRef = null;
  host.cachedCellBboxes = null;
  assert.equal(ensureTableCellBboxCache(host, T, 0), page0Cells);
  assert.deepEqual(host.calls, [0, 0, 0]);
});

test('문서 변경(clearTableResizeRuntimeCache 모사) 뒤에는 다시 시도한다', () => {
  let empty = true;
  const host = makeHost(() => (empty ? [] : page0Cells));
  assert.equal(ensureTableCellBboxCache(host, T, 0), null);
  // input-handler.clearTableResizeRuntimeCache 가 하는 일
  empty = false;
  host.cachedTableRef = null;
  host.cachedCellBboxes = null;
  host.tableBboxFetchFailures.clear();
  assert.equal(ensureTableCellBboxCache(host, T, 0), page0Cells);
  assert.equal(host.calls.length, 2);
});
