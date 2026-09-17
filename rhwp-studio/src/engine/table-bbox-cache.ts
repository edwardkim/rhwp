/**
 * [#4117] 표 셀 bbox 캐시의 단일 채움 지점.
 *
 * 셀 선택 모드 클릭 없이도 표 경계 hover/리사이즈가 동작하도록 hover 경로가
 * 이 함수로 캐시를 채운다. 엔진 질의(getTableCellBboxes)는 페이지 렌더 트리
 * 캐시를 읽는 warm 질의지만 마우스 이동마다 부를 만큼 싸지는 않으므로:
 *
 * - 같은 표(+ 해당 페이지 포함)가 캐시돼 있으면 질의 없이 캐시를 돌려준다.
 * - 실패(빈 결과·예외)도 (표, 페이지)당 1회만 시도한다 — 이동마다 재시도 금지.
 * - 문서가 바뀌면 input-handler 의 clearTableResizeRuntimeCache 가 캐시와
 *   실패 메모를 함께 비워 다음 hover 가 새로 채운다.
 *
 * input-handler 밖의 독립 모듈인 이유: import 의존이 없어 node --test 가
 * 직접 불러 채움·메모 계약을 검증할 수 있다.
 */

/** `hitTest` 가 주는 셀 경로 한 마디. */
export interface CellPathStep {
  controlIndex: number;
  cellIndex: number;
  cellParaIndex: number;
}

export interface TableRef {
  sec: number;
  ppi: number;
  ci: number;
  /**
   * [#7189] `hitTest` 의 셀 경로. 깊이 2 이상이면 **중첩 표**다.
   *
   * `sec/ppi/ci` 는 경로의 첫 마디, 즉 **최외곽** 표만 가리킨다. 그 값만으로 캐시를 열면
   * 같은 쪽의 바깥 표와 안쪽 표가 서로를 덮어써, 안쪽 경계 hover 가 바깥 표의 괘선을 본다.
   */
  path?: readonly CellPathStep[];
}

export interface PageScopedBbox {
  pageIndex: number;
}

export interface CachedTableRef extends TableRef {
  pageHint?: number;
  pageIndexes?: ReadonlySet<number>;
}

export interface TableBboxCacheHost<B extends PageScopedBbox = PageScopedBbox> {
  wasm: {
    getTableCellBboxes(sec: number, ppi: number, ci: number, pageHint?: number): B[];
    /** [#7189] 중첩 표 전용. 깊이 2 이상 경로에서만 부른다. */
    getTableCellBboxesByPath(sec: number, ppi: number, pathJson: string): B[];
  };
  cachedTableRef: CachedTableRef | null;
  cachedCellBboxes: B[] | null;
  tableBboxFetchFailures: Set<string>;
}

/**
 * [#7189] 캐시·실패 메모의 신원. 경로를 빼면 같은 쪽의 바깥/안쪽 표가 한 칸을 다툰다.
 *
 * 깊이 1 경로와 경로 없음은 **같은 표**다 — 평면 API 가 가리키는 최외곽 표이므로 같은
 * 키를 써야 캐시가 쪼개지지 않는다.
 */
export function tableIdentity(tableRef: TableRef): string {
  const base = `${tableRef.sec}:${tableRef.ppi}:${tableRef.ci}`;
  const path = tableRef.path;
  if (!path || path.length <= 1) return base;
  return `${base}|${path.map((s) => `${s.controlIndex}.${s.cellIndex}.${s.cellParaIndex}`).join('/')}`;
}

function sameTable(a: TableRef, b: TableRef): boolean {
  return tableIdentity(a) === tableIdentity(b);
}

function failureKey(tableRef: TableRef, pageIdx: number): string {
  return `${tableIdentity(tableRef)}:${pageIdx}`;
}

/** 성공한 bbox 조회를 한 번에 기록하고, 같은 범위의 과거 실패를 해제한다. */
export function cacheTableCellBboxes<B extends PageScopedBbox>(
  host: TableBboxCacheHost<B>,
  tableRef: TableRef,
  pageIdx: number,
  bboxes: B[],
): B[] {
  if (bboxes.length === 0) return bboxes;

  const pageIndexes = new Set(bboxes.map((bbox) => bbox.pageIndex));
  // 조회가 성공한 hint 페이지도 membership에 포함한다. 분할 표의 엔진 결과가
  // 인접 페이지 bbox만 담더라도 같은 성공 조회를 mousemove에서 반복하지 않는다.
  pageIndexes.add(pageIdx);
  host.cachedTableRef = { ...tableRef, pageHint: pageIdx, pageIndexes };
  host.cachedCellBboxes = bboxes;
  for (const cachedPageIdx of pageIndexes) {
    host.tableBboxFetchFailures.delete(failureKey(tableRef, cachedPageIdx));
  }
  return bboxes;
}

export function ensureTableCellBboxCache<B extends PageScopedBbox>(
  host: TableBboxCacheHost<B>,
  tableRef: TableRef,
  pageIdx: number,
): B[] | null {
  const cached = host.cachedTableRef;
  if (
    cached && host.cachedCellBboxes && host.cachedCellBboxes.length > 0 &&
    sameTable(cached, tableRef) &&
    // hint 일치가 셀 배열 스캔 없이 끝나는 빠른 길. 다르면 페이지 포함 검사 —
    // 여러 쪽에 걸친 표는 한 번의 조회가 걸친 쪽들을 함께 담아 오므로,
    // hint 가 달라도 요구 페이지가 이미 있으면 재조회하지 않는다.
    (cached.pageHint === pageIdx ||
      cached.pageIndexes?.has(pageIdx) === true)
  ) {
    return host.cachedCellBboxes;
  }

  const key = failureKey(tableRef, pageIdx);
  if (host.tableBboxFetchFailures.has(key)) {
    return null;
  }

  try {
    // [#7189] 중첩 표는 평면 좌표계로 가리킬 수 없다 — 경로 API 로 물어야 안쪽 표의
    // 괘선이 나온다. 깊이 1 은 평면 질의가 페이지 힌트까지 받으므로 그대로 둔다.
    const path = tableRef.path;
    const bboxes = path && path.length > 1
      ? host.wasm.getTableCellBboxesByPath(tableRef.sec, tableRef.ppi, JSON.stringify(path))
      : host.wasm.getTableCellBboxes(tableRef.sec, tableRef.ppi, tableRef.ci, pageIdx);
    if (bboxes && bboxes.length > 0) {
      return cacheTableCellBboxes(host, tableRef, pageIdx, bboxes);
    }
  } catch { /* 조회 실패 — 아래 실패 메모가 이동마다 재시도를 막는다 */ }

  host.tableBboxFetchFailures.add(key);
  return null;
}
