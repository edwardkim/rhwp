// [Issue #7189] 칸 안 중첩 표의 경계에서 표 크기 조절 가이드가 뜨지 않는다.
//
// hover 경로는 `hitTest` 의 평면 필드(`sectionIndex/parentParaIndex/controlIndex`)로
// `tableRef` 를 만들어 `getTableCellBboxes` 만 불렀다. 그 세 값은 `cellPath[0]`, 즉
// **최외곽** 표다. 그래서 중첩 표의 괘선 좌표는 가이드가 아는 목록에 애초에 들어오지
// 않았고, 바깥 경계 근처에서는 바깥 표의 괘선이 잡혀 엉뚱한 자리에 선이 떴다.
//
// 캐시 신원도 `sec:ppi:ci` 뿐이라 같은 쪽의 바깥 표와 안쪽 표가 한 칸을 다퉜다.
//
// # 이 시험이 잠그는 것
//
// - 깊이 2 경로는 **경로 API** 로 bbox 를 얻는다.
// - 깊이 1 과 경로 없음은 같은 표다 — 평면 질의를 쓰고 캐시를 쪼개지 않는다.
// - 같은 쪽의 바깥/안쪽 표가 서로의 캐시와 실패 메모를 **덮어쓰지 않는다**.
//
// # 잠그지 않는 것
//
// 엔진 좌표의 정확성(`tests/cases/issue_7189_nested_table_resize_by_path.rs` 가 잠근다)과
// 마커 렌더링은 범위 밖이다.

import test from 'node:test';
import assert from 'node:assert/strict';

interface Bbox { pageIndex: number; tag: string }

async function cacheModule() {
  return await import('../src/engine/table-bbox-cache.ts');
}

function host(outer: Bbox[], inner: Bbox[]) {
  const calls: string[] = [];
  return {
    calls,
    wasm: {
      getTableCellBboxes(sec: number, ppi: number, ci: number, pageHint?: number): Bbox[] {
        calls.push(`flat:${sec}:${ppi}:${ci}:${pageHint}`);
        return outer;
      },
      getTableCellBboxesByPath(sec: number, ppi: number, pathJson: string): Bbox[] {
        calls.push(`path:${sec}:${ppi}:${pathJson}`);
        return inner;
      },
    },
    cachedTableRef: null as any,
    cachedCellBboxes: null as Bbox[] | null,
    tableBboxFetchFailures: new Set<string>(),
  };
}

const OUTER: Bbox[] = [{ pageIndex: 0, tag: 'outer' }];
const INNER: Bbox[] = [{ pageIndex: 0, tag: 'inner' }];

const outerRef = { sec: 0, ppi: 2, ci: 0 };
const nestedRef = {
  sec: 0,
  ppi: 2,
  ci: 0,
  path: [
    { controlIndex: 0, cellIndex: 0, cellParaIndex: 9 },
    { controlIndex: 0, cellIndex: 3, cellParaIndex: 0 },
  ],
};

test('깊이 2 경로는 경로 API 로 bbox 를 얻는다', async () => {
  const { ensureTableCellBboxCache } = await cacheModule();
  const h = host(OUTER, INNER);

  const got = ensureTableCellBboxCache(h as any, nestedRef, 0);

  assert.deepEqual(got, INNER, '중첩 표에서는 안쪽 표의 bbox 가 나와야 한다');
  assert.equal(h.calls.length, 1);
  assert.ok(h.calls[0].startsWith('path:'), `경로 API 를 불러야 한다 (실제 ${h.calls[0]})`);
});

test('경로가 없으면 평면 질의를 쓴다', async () => {
  const { ensureTableCellBboxCache } = await cacheModule();
  const h = host(OUTER, INNER);

  const got = ensureTableCellBboxCache(h as any, outerRef, 0);

  assert.deepEqual(got, OUTER);
  assert.ok(h.calls[0].startsWith('flat:'), `평면 질의여야 한다 (실제 ${h.calls[0]})`);
});

test('깊이 1 경로는 최외곽 표 — 평면 질의를 쓰고 캐시를 쪼개지 않는다', async () => {
  const { ensureTableCellBboxCache, tableIdentity } = await cacheModule();
  const depthOne = { ...outerRef, path: [{ controlIndex: 0, cellIndex: 0, cellParaIndex: 0 }] };
  const h = host(OUTER, INNER);

  ensureTableCellBboxCache(h as any, depthOne, 0);
  assert.ok(h.calls[0].startsWith('flat:'), '깊이 1 은 평면 질의다');

  assert.equal(
    tableIdentity(depthOne as any),
    tableIdentity(outerRef as any),
    '깊이 1 과 경로 없음은 같은 표 — 신원이 갈리면 같은 표를 두 번 조회한다',
  );

  // 같은 표이므로 두 번째 질의는 캐시가 받아야 한다.
  ensureTableCellBboxCache(h as any, outerRef, 0);
  assert.equal(h.calls.length, 1, '같은 표를 다시 조회하면 안 된다');
});

test('같은 쪽의 바깥 표와 안쪽 표가 서로의 캐시를 덮어쓰지 않는다', async () => {
  const { ensureTableCellBboxCache } = await cacheModule();
  const h = host(OUTER, INNER);

  assert.deepEqual(ensureTableCellBboxCache(h as any, outerRef, 0), OUTER);
  assert.deepEqual(ensureTableCellBboxCache(h as any, nestedRef, 0), INNER);
  // 바깥으로 돌아오면 다시 바깥 bbox 여야 한다 — 신원이 같으면 안쪽 캐시를 그대로 준다.
  assert.deepEqual(
    ensureTableCellBboxCache(h as any, outerRef, 0),
    OUTER,
    '안쪽 표를 거친 뒤에도 바깥 표는 바깥 bbox 를 받아야 한다',
  );
});

test('실패 메모도 표마다 따로 남는다', async () => {
  const { ensureTableCellBboxCache } = await cacheModule();
  const h = host([], []); // 두 질의 모두 빈 결과 — 실패로 메모된다

  assert.equal(ensureTableCellBboxCache(h as any, outerRef, 0), null);
  assert.equal(h.tableBboxFetchFailures.size, 1);

  assert.equal(ensureTableCellBboxCache(h as any, nestedRef, 0), null);
  assert.equal(
    h.tableBboxFetchFailures.size,
    2,
    '바깥 표의 실패가 안쪽 표의 조회까지 막으면 안 된다',
  );
});
