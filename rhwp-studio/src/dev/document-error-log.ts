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

export interface LayoutTraceEntry {
  id: number;
  parentId: number | null;
  function: string;
  args: Record<string, string | number | boolean>;
  durationMs: number;
  depth: number;
}

const MAX_DOCUMENT_ERROR_LENGTH = 16_384;
export const MAX_LAYOUT_TRACE_ENTRIES = 64;

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

function isLayoutTraceEntry(entry: unknown): entry is LayoutTraceEntry {
  if (!entry || typeof entry !== 'object') return false;
  const value = entry as Partial<LayoutTraceEntry>;
  return isIndex(value.id)
    && (value.parentId === null || isIndex(value.parentId) && value.parentId < value.id)
    && typeof value.function === 'string'
    && /^[a-z][a-z0-9_]*$/.test(value.function)
    && !!value.args
    && typeof value.args === 'object'
    && Object.keys(value.args).length <= 10
    && Object.entries(value.args).every(([name, field]) =>
      /^[a-z][a-z0-9_]*$/.test(name)
      && (typeof field === 'number' && Number.isFinite(field)
        || typeof field === 'boolean'
        || typeof field === 'string' && field.length <= 256))
    && typeof value.durationMs === 'number'
    && Number.isFinite(value.durationMs)
    && value.durationMs >= 0
    && isIndex(value.depth)
    && value.depth <= 16;
}

export function parseLayoutTrace(serialized: string): LayoutTraceEntry[] {
  try {
    const trace = JSON.parse(serialized);
    if (!Array.isArray(trace)) return [];
    return trace
      .filter(isLayoutTraceEntry)
      .slice(-MAX_LAYOUT_TRACE_ENTRIES)
      .map(entry => ({ ...entry, durationMs: Number(entry.durationMs.toFixed(3)) }));
  } catch {
    return [];
  }
}

export function attachDocumentErrorTrace(
  line: string,
  trace: readonly LayoutTraceEntry[],
): string {
  const bounded: LayoutTraceEntry[] = [];
  for (const entry of trace.slice(-MAX_LAYOUT_TRACE_ENTRIES).reverse()) {
    if (!isLayoutTraceEntry(entry)) continue;
    const candidate = [entry, ...bounded];
    if (`${line} trace=${JSON.stringify(candidate)}`.length > MAX_DOCUMENT_ERROR_LENGTH) break;
    bounded.unshift(entry);
  }
  return bounded.length ? `${line} trace=${JSON.stringify(bounded)}` : line;
}

export function isDocumentErrorLine(line: string): boolean {
  if (line.length === 0 || line.length > MAX_DOCUMENT_ERROR_LENGTH || /[\r\n]/.test(line)) return false;
  const match = /^(line-break|page-count|paint): \[([^\[\]]+)\](?: trace=(.*))?$/.exec(line);
  if (!match) return false;
  const attributes = match[2].split(' ');
  if (!attributes.some(attribute => /^page=\d+$/.test(attribute))
    || !attributes.every(attribute => /^[a-z][a-zA-Z]*=[\x21-\x5a\x5c\x5e-\x7e]+$/.test(attribute))) {
    return false;
  }
  if (match[3] === undefined) return true;
  try {
    const trace = JSON.parse(match[3]);
    return Array.isArray(trace)
      && trace.length > 0
      && trace.length <= MAX_LAYOUT_TRACE_ENTRIES
      && trace.every(isLayoutTraceEntry);
  } catch {
    return false;
  }
}

export function formatDocumentErrorForTerminal(line: string): string | null {
  if (!isDocumentErrorLine(line)) return null;
  const traceAt = line.indexOf(' trace=');
  if (traceAt < 0) return line;
  const error = line.slice(0, traceAt);
  const trace = JSON.parse(line.slice(traceAt + ' trace='.length)) as LayoutTraceEntry[];
  const field = (value: string | number | boolean): string => typeof value === 'string'
    ? JSON.stringify(value)
    : typeof value === 'number' ? String(Number(value.toFixed(3))) : String(value);
  return [
    error,
    'trace:',
    ...trace.map(entry => {
      const fields = Object.entries(entry.args);
      const args = fields
        .filter(([name]) => !name.startsWith('result_'))
        .map(([name, value]) => `${name}=${field(value)}`)
        .join(', ');
      const results = fields
        .filter(([name]) => name.startsWith('result_'))
        .map(([name, value]) => `${name.slice(7)}=${field(value)}`)
        .join(', ');
      const indent = '  '.repeat(entry.depth + 1);
      return `${indent}#${entry.id} ${entry.function}(${args})${results ? ` => ${results}` : ''} ${entry.durationMs}ms`;
    }),
  ].join('\n');
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
