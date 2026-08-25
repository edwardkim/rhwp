export type PageArrangement =
  | { kind: 'auto' }
  | { kind: 'single' }
  | { kind: 'double' }
  | { kind: 'facing' }
  | { kind: 'multiple'; columns: number; rows: number };

export const DEFAULT_PAGE_ARRANGEMENT: PageArrangement = { kind: 'auto' };
export const MIN_MULTIPLE_PAGES = 1;
export const MAX_MULTIPLE_PAGES = 8;
export const MIN_DOCUMENT_ZOOM = 0.1;
export const MAX_DOCUMENT_ZOOM = 5;

function normalizeMultipleCount(value: unknown): number {
  const number = typeof value === 'number' ? value : Number(value);
  if (!Number.isFinite(number)) return MIN_MULTIPLE_PAGES;
  return Math.min(
    MAX_MULTIPLE_PAGES,
    Math.max(MIN_MULTIPLE_PAGES, Math.round(number)),
  );
}

/** localStorage와 외부 입력에서 읽은 쪽 배치를 안전한 판별 합집합으로 복원한다. */
export function normalizePageArrangement(value: unknown): PageArrangement {
  if (!value || typeof value !== 'object') return { ...DEFAULT_PAGE_ARRANGEMENT };
  const candidate = value as { kind?: unknown; columns?: unknown; rows?: unknown };
  switch (candidate.kind) {
    case 'auto':
    case 'single':
    case 'double':
    case 'facing':
      return { kind: candidate.kind };
    case 'multiple':
      return {
        kind: 'multiple',
        columns: normalizeMultipleCount(candidate.columns),
        rows: normalizeMultipleCount(candidate.rows),
      };
    default:
      return { ...DEFAULT_PAGE_ARRANGEMENT };
  }
}

export function pageArrangementsEqual(a: PageArrangement, b: PageArrangement): boolean {
  if (a.kind !== b.kind) return false;
  if (a.kind !== 'multiple' || b.kind !== 'multiple') return true;
  return a.columns === b.columns && a.rows === b.rows;
}

export interface MultiplePagesZoomInput {
  viewportWidth: number;
  viewportHeight: number;
  pageWidth: number;
  pageHeight: number;
  columns: number;
  rows: number;
  pageGap?: number;
}

/** 선택한 가로×세로 쪽과 외곽/쪽 사이 간격이 뷰포트에 들어오는 수치 배율을 계산한다. */
export function calculateMultiplePagesZoom(input: MultiplePagesZoomInput): number {
  const columns = normalizeMultipleCount(input.columns);
  const rows = normalizeMultipleCount(input.rows);
  const gap = Number.isFinite(input.pageGap) ? Math.max(0, input.pageGap ?? 0) : 0;
  if (
    !Number.isFinite(input.viewportWidth)
    || !Number.isFinite(input.viewportHeight)
    || !Number.isFinite(input.pageWidth)
    || !Number.isFinite(input.pageHeight)
    || input.pageWidth <= 0
    || input.pageHeight <= 0
  ) {
    return 1;
  }

  const width = Math.max(0, input.viewportWidth - gap * (columns + 1));
  const height = Math.max(0, input.viewportHeight - gap * (rows + 1));
  const requested = Math.min(
    width / (input.pageWidth * columns),
    height / (input.pageHeight * rows),
  );
  if (!Number.isFinite(requested) || requested <= 0) return MIN_DOCUMENT_ZOOM;
  return Math.max(MIN_DOCUMENT_ZOOM, Math.min(MAX_DOCUMENT_ZOOM, requested));
}
