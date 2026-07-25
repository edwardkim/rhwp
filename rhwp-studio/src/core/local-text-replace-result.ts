export interface LocalBodyTextReplaceResult {
  ok: true;
  charOffset: number;
  documentPaginationPending: boolean;
  flowChanged: boolean;
}

export function parseLocalBodyTextReplaceResult(
  raw: string,
): LocalBodyTextReplaceResult {
  const parsed = JSON.parse(raw) as Partial<LocalBodyTextReplaceResult>;
  if (
    parsed.ok !== true ||
    !Number.isInteger(parsed.charOffset) ||
    typeof parsed.documentPaginationPending !== 'boolean' ||
    typeof parsed.flowChanged !== 'boolean' ||
    (parsed.flowChanged && parsed.documentPaginationPending)
  ) {
    throw new Error('잘못된 local body text replace 결과');
  }
  return parsed as LocalBodyTextReplaceResult;
}
