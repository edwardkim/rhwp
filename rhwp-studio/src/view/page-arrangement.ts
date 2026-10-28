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
