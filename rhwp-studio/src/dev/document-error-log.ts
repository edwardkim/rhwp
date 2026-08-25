import { DOCUMENT_ERROR_LOG_PATH } from './pdf-twin-contract.ts';

export type DocumentErrorType = 'line-break' | 'page-count' | 'paint';
export interface LineBreakDiagnostic {
  textUtf16Length?: number;
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
    storedMismatchUtf16Start?: number | null;
    freshMismatchUtf16Start?: number | null;
    storedMismatchRowPart?: 'single' | 'first' | 'middle' | 'last' | null;
    freshMismatchRowPart?: 'single' | 'first' | 'middle' | 'last' | null;
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

const isIndex = (value: unknown): value is number => Number.isSafeInteger(value) && Number(value) >= 0;
const isRowPart = (value: unknown): value is string =>
  value === 'single' || value === 'first' || value === 'middle' || value === 'last';
/** Render one CLI-stable document error as `type: [flat document attributes]`. */
export function formatDocumentError(
  type: DocumentErrorType,
  attributes: readonly (readonly [name: string, value: string | number])[],
): string {
  return `${type}: [${attributes.map(([name, value]) => {
    if (!/^[a-z][a-zA-Z]*$/.test(name)) throw new Error(`invalid document error attribute: ${name}`);
    const rendered = String(value);
    if (!/^[\x21-\x5a\x5c\x5e-\x7e]+$/.test(rendered)) throw new Error(`invalid document error value: ${name}`);
    return `${name}=${rendered}`;
  }).join(' ')}]`;
}

export function formatFirstLineBreakError(
  page: number,
  diagnostics: readonly LineBreakDiagnostic[],
): string | null {
  if (!Number.isSafeInteger(page) || page < 1) return null;
  for (const { coordinates, comparison } of diagnostics) {
    const paragraph = coordinates?.parentParaIdx ?? coordinates?.paragraphIdx;
    const stored = comparison?.storedMismatchUtf16Start;
    const fresh = comparison?.freshMismatchUtf16Start;
    const storedPart = comparison?.storedMismatchRowPart;
    const freshPart = comparison?.freshMismatchRowPart;
    if (
      !coordinates
      || !comparison
      || comparison.comparable !== true
      || comparison.matches !== false
      || !isIndex(coordinates.sectionIdx)
      || !isIndex(paragraph)
      || !isIndex(comparison.firstMismatchIndex)
      || !(stored === null ? storedPart === null : isIndex(stored) && isRowPart(storedPart))
      || !(fresh === null ? freshPart === null : isIndex(fresh) && isRowPart(freshPart))
      || (stored === null && fresh === null)
      || !(coordinates.cellPath ?? []).every(entry =>
        isIndex(entry.controlIndex) && isIndex(entry.cellIndex) && isIndex(entry.cellParaIndex))
      || !(coordinates.groupPath ?? []).every(isIndex)
    ) continue;
    const target = [
      `s${coordinates.sectionIdx}`,
      `p${paragraph}`,
      ...(coordinates.cellPath ?? []).map(({ controlIndex, cellIndex, cellParaIndex }) =>
        `c${controlIndex}.${cellIndex}.${cellParaIndex}`),
      ...(coordinates.groupPath?.length ? [`g${coordinates.groupPath.join('.')}`] : []),
    ].join('/');
    const line = formatDocumentError('line-break', [
      ['page', page],
      ['target', target],
      ['at', comparison.firstMismatchIndex],
      ['expected', stored === null ? '-' : `${stored}:${storedPart}`],
      ['actual', fresh === null ? '-' : `${fresh}:${freshPart}`],
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
  let total: number | null = null;
  while (!visited.has(start) && (total === null || visited.size <= total)) {
    visited.add(start);
    const result = inspectBatch(start);
    const error = formatFirstLineBreakError(page, result?.items ?? []);
    if (error) return error;
    const reportedTotal = result?.total;
    total ??= isIndex(reportedTotal) ? reportedTotal : 0;
    const next = result?.nextOffset;
    if (!isIndex(next) || next <= start) return null;
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
  if (!/^[A-Za-z0-9_-]{43}$/.test(capability)) throw new Error('invalid document error capability');
  const response = await send(DOCUMENT_ERROR_LOG_PATH, {
    method: 'POST',
    headers: {
      'Content-Type': 'text/plain; charset=utf-8',
      'x-rhwp-harness-capability': capability,
    },
    body: line,
    keepalive: true,
  });
  if (!response.ok) throw new Error(`document error endpoint rejected (${response.status})`);
}
