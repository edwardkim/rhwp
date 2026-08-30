export interface LocalBodyTextFocusedPagePatch {
  pageIndex: number;
  x: number;
  y: number;
  width: number;
  height: number;
}

export interface LocalBodyTextReplaceResult {
  ok: true;
  charOffset: number;
  documentPaginationPending: boolean;
  flowChanged: boolean;
  /** 국소 조판이 쪽 트리 캐시를 고친 영역의 재페인트 사각형. */
  focusedPagePatch?: LocalBodyTextFocusedPagePatch;
}

function parseFocusedPagePatch(value: unknown): LocalBodyTextFocusedPagePatch | undefined {
  if (!value || typeof value !== 'object') return undefined;
  const candidate = value as Partial<LocalBodyTextFocusedPagePatch>;
  const numbers = [candidate.x, candidate.y, candidate.width, candidate.height];
  if (
    !Number.isSafeInteger(candidate.pageIndex)
    || (candidate.pageIndex as number) < 0
    || !numbers.every((item) => typeof item === 'number' && Number.isFinite(item))
    || (candidate.width as number) <= 0
    || (candidate.height as number) <= 0
  ) {
    return undefined;
  }
  return {
    pageIndex: candidate.pageIndex as number,
    x: candidate.x as number,
    y: candidate.y as number,
    width: candidate.width as number,
    height: candidate.height as number,
  };
}

export function parseLocalBodyTextReplaceResult(
  raw: string,
): LocalBodyTextReplaceResult {
  const parsed = JSON.parse(raw) as Partial<LocalBodyTextReplaceResult>;
  if (
    parsed.ok !== true ||
    typeof parsed.charOffset !== 'number' ||
    !Number.isInteger(parsed.charOffset) ||
    typeof parsed.documentPaginationPending !== 'boolean' ||
    typeof parsed.flowChanged !== 'boolean' ||
    (parsed.flowChanged && parsed.documentPaginationPending)
  ) {
    throw new Error('잘못된 local body text replace 결과');
  }
  const focusedPagePatch = parsed.flowChanged
    ? undefined
    : parseFocusedPagePatch(parsed.focusedPagePatch);
  return {
    ok: true,
    charOffset: parsed.charOffset,
    documentPaginationPending: parsed.documentPaginationPending,
    flowChanged: parsed.flowChanged,
    ...(focusedPagePatch ? { focusedPagePatch } : {}),
  };
}
