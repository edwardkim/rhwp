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

export interface TableRef {
  sec: number;
  ppi: number;
  ci: number;
}

export interface PageScopedBbox {
  pageIndex: number;
}

export interface TableBboxCacheHost<B extends PageScopedBbox = PageScopedBbox> {
  wasm: {
    getTableCellBboxes(sec: number, ppi: number, ci: number, pageHint?: number): B[];
  };
  cachedTableRef: (TableRef & { pageHint?: number }) | null;
  cachedCellBboxes: B[] | null;
  tableBboxFetchFailure: (TableRef & { pageIdx: number }) | null;
}

function sameTable(a: TableRef, b: TableRef): boolean {
  return a.sec === b.sec && a.ppi === b.ppi && a.ci === b.ci;
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
      host.cachedCellBboxes.some((b) => b.pageIndex === pageIdx))
  ) {
    return host.cachedCellBboxes;
  }

  const failed = host.tableBboxFetchFailure;
  if (failed && sameTable(failed, tableRef) && failed.pageIdx === pageIdx) {
    return null;
  }

  try {
    const bboxes = host.wasm.getTableCellBboxes(tableRef.sec, tableRef.ppi, tableRef.ci, pageIdx);
    if (bboxes && bboxes.length > 0) {
      host.cachedTableRef = {
        sec: tableRef.sec, ppi: tableRef.ppi, ci: tableRef.ci, pageHint: pageIdx,
      };
      host.cachedCellBboxes = bboxes;
      host.tableBboxFetchFailure = null;
      return bboxes;
    }
  } catch { /* 조회 실패 — 아래 실패 메모가 이동마다 재시도를 막는다 */ }

  host.tableBboxFetchFailure = {
    sec: tableRef.sec, ppi: tableRef.ppi, ci: tableRef.ci, pageIdx,
  };
  return null;
}
