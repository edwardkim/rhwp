/**
 * 상태 표시줄 쪽 표시 문자열 조립.
 *
 * 한글과 같은 규칙이다 — 현재 쪽은 **문서가 매기는 쪽번호**(`쪽 > 새 번호로 시작` 반영)를,
 * 전체는 **물리 쪽수**를 보여준다. 앞 2쪽 뒤에 1쪽부터 다시 시작하는 문서라면 세 번째 쪽에서
 * 한글도 rhwp 도 `1` 을 보여야 한다 (#5749).
 *
 * 문서 쪽번호를 모르는 경우(구 WASM, 조회 실패)에는 물리 순번으로 물러난다 — 표시가 비거나
 * `NaN` 이 되는 것보다 예전 동작이 낫다.
 */

export interface PageIndicatorInput {
  /** 현재 쪽의 물리 순번 (0-based) */
  pageIndex: number;
  /** 전체 물리 쪽수 */
  totalPages: number;
  /** 문서가 매기는 쪽번호 (1-based). 모르면 null/undefined */
  documentPageNumber?: number | null;
}

/** 문서 쪽번호로 쓸 수 있는 값인지 — 1 이상의 유한 정수만 받는다. */
function usableDocumentPageNumber(value: number | null | undefined): value is number {
  return typeof value === 'number' && Number.isFinite(value) && value >= 1;
}

/** 상태 표시줄에 보일 현재 쪽 번호 */
export function currentPageLabel(input: PageIndicatorInput): number {
  return usableDocumentPageNumber(input.documentPageNumber)
    ? Math.floor(input.documentPageNumber)
    : input.pageIndex + 1;
}

/** 상태 표시줄 문자열 (`1 / 33 쪽`) */
export function formatPageIndicator(input: PageIndicatorInput): string {
  return `${currentPageLabel(input)} / ${input.totalPages} 쪽`;
}
