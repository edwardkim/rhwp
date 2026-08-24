import {
  DOCUMENT_ERROR_CAPABILITY_HEADER,
  DOCUMENT_ERROR_LOG_PATH,
} from './pdf-twin-contract.ts';

export type DocumentErrorType = 'line-break' | 'page-count' | 'paint';
export interface LineBreakDiagnostic {
  coordinates?: {
    sectionIdx?: number;
    paragraphIdx?: number | null;
    parentParaIdx?: number | null;
    cellPath?: Array<{ controlIndex: number; cellIndex: number; cellParaIndex: number }>;
    groupPath?: number[];
  };
  comparison?: {
    comparable?: boolean;
    matches?: boolean | null;
    firstMismatchIndex?: number | null;
    storedStartsTruncated?: boolean;
    storedUtf16Starts?: number[];
    freshStartsTruncated?: boolean;
    freshUtf16Starts?: number[];
  };
}
export interface LineBreakVisibleResult {
  total?: number;
  nextOffset?: number | null;
  items?: LineBreakDiagnostic[];
}

/** Render one CLI-stable document error as `type: [flat document attributes]`. */
export function formatDocumentError(
  type: DocumentErrorType,
  attributes: readonly (readonly [name: string, value: string | number])[],
): string {
  const body = attributes.map(([name, value]) => {
    if (!/^[a-z][a-zA-Z]*$/.test(name)) throw new Error(`invalid document error attribute: ${name}`);
    const rendered = String(value);
    if (!/^[\x21-\x5a\x5c\x5e-\x7e]+$/.test(rendered)) {
      throw new Error(`invalid document error value: ${name}`);
    }
    return `${name}=${rendered}`;
  }).join(' ');
  return `${type}: [${body}]`;
}

export function formatFirstLineBreakError(
  page: number,
  diagnostics: readonly LineBreakDiagnostic[],
): string | null {
  const isIndex = (value: unknown): value is number =>
    Number.isSafeInteger(value) && Number(value) >= 0;
  if (!Number.isSafeInteger(page) || page < 1) return null;
  for (const mismatch of diagnostics) {
    const coordinates = mismatch.coordinates;
    const comparison = mismatch.comparison;
    const paragraph = coordinates?.parentParaIdx ?? coordinates?.paragraphIdx;
    const stored = comparison?.storedUtf16Starts;
    const fresh = comparison?.freshUtf16Starts;
    if (
      !coordinates
      || !comparison
      || comparison.comparable !== true
      || comparison.matches !== false
      || comparison.storedStartsTruncated !== false
      || comparison.freshStartsTruncated !== false
      || !isIndex(coordinates.sectionIdx)
      || !isIndex(paragraph)
      || !isIndex(comparison.firstMismatchIndex)
      || !Array.isArray(stored)
      || !Array.isArray(fresh)
      || stored.length + fresh.length === 0
      || stored.length > 2_048
      || fresh.length > 2_048
      || !stored.every(isIndex)
      || !fresh.every(isIndex)
      || comparison.firstMismatchIndex >= Math.max(stored.length, fresh.length)
      || !(coordinates.cellPath ?? []).every(entry =>
        isIndex(entry.controlIndex) && isIndex(entry.cellIndex) && isIndex(entry.cellParaIndex))
      || !(coordinates.groupPath ?? []).every(isIndex)
    ) continue;
    const cells = (coordinates.cellPath ?? [])
      .map(entry => `c${entry.controlIndex}.${entry.cellIndex}.${entry.cellParaIndex}`)
      .join('/');
    const groups = coordinates.groupPath?.length ? `g${coordinates.groupPath.join('.')}` : '';
    const target = [`s${coordinates.sectionIdx}`, `p${paragraph}`, cells, groups]
      .filter(Boolean)
      .join('/');
    const line = formatDocumentError('line-break', [
      ['page', page],
      ['target', target],
      ['at', comparison.firstMismatchIndex],
      ['expected', stored.join(',') || '-'],
      ['actual', fresh.join(',') || '-'],
    ]);
    return isDocumentErrorLine(line) ? line : null;
  }
  return null;
}

export function findFirstLineBreakError(
  page: number,
  inspectBatch: (start: number) => LineBreakVisibleResult | undefined,
): string | null {
  const visited = new Set<number>();
  let start = 0;
  let targetCount: number | null = null;
  while (!visited.has(start) && (targetCount === null || visited.size <= targetCount)) {
    visited.add(start);
    const result = inspectBatch(start);
    const error = formatFirstLineBreakError(page, result?.items ?? []);
    if (error) return error;
    if (targetCount === null) {
      const total = result?.total;
      targetCount = typeof total === 'number' && Number.isSafeInteger(total) && total >= 0
        ? total
        : 0;
    }
    const next = result?.nextOffset;
    if (typeof next !== 'number' || !Number.isSafeInteger(next) || next <= start) return null;
    start = next;
  }
  return null;
}

export function isDocumentErrorLine(line: string): boolean {
  if (line.length === 0 || line.length > 4_096 || /[\r\n]/.test(line)) return false;
  const match = /^(line-break|page-count|paint): \[([^\[\]]+)\]$/.exec(line);
  if (!match) return false;
  const attributes = match[2].split(' ');
  return attributes.some(attribute => /^page=\d+$/.test(attribute))
    && attributes.every(attribute => /^[a-z][a-zA-Z]*=[\x21-\x5a\x5c\x5e-\x7e]+$/.test(attribute));
}

export async function sendDocumentErrorLine(
  line: string,
  capability: string,
  send: typeof fetch = fetch,
): Promise<void> {
  if (!isDocumentErrorLine(line)) throw new Error('invalid document error line');
  const response = await send(DOCUMENT_ERROR_LOG_PATH, {
    method: 'POST',
    headers: {
      'Content-Type': 'text/plain; charset=utf-8',
      [DOCUMENT_ERROR_CAPABILITY_HEADER]: capability,
    },
    body: line,
    keepalive: true,
  });
  if (!response.ok) {
    throw new Error(`document error endpoint rejected (${response.status})`);
  }
}
