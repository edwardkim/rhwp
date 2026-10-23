/**
 * P0 읽기 도구 — 모델이 문서를 필요한 만큼만 읽는 통로.
 *
 * 문서 전체를 대화에 넣지 않는다. 모델은 개요·쪽·문단·검색으로 필요한 부분을 골라 읽는다.
 * 모든 도구는 `host.read()` 안에서만 문서를 보고, 문서를 바꾸지 않는다.
 *
 * 좌표 규약: 구역·문단은 0부터, 쪽은 1부터(사람에게 보이는 쪽 번호). 결과의 문서 텍스트가
 * 상한을 넘으면 조용히 자르지 않고 `truncated` 와 이어 읽을 위치를 함께 돌려준다.
 */
import type { PluginCaret, PluginSelectionText } from '@/plugin/types';
import type { AiToolCall, AiToolDef, AiToolResult } from '../provider/types.ts';
import { validateToolInput, type ToolSchema } from './schema-validate.ts';
import { cleanDocumentText, wrapDocumentContent } from './untrusted.ts';
import { listCharts } from './write-tools.ts';

/** 한 결과에 담는 문서 텍스트 글자 수 상한 */
export const MAX_RESULT_CHARS = 12000;
export const MAX_PARAGRAPHS_PER_CALL = 50;
export const MAX_SEARCH_HITS = 50;
const SEARCH_CONTEXT_CHARS = 40;
const MAX_FIELD_VALUE_CHARS = 500;

/** 도구가 문서에서 읽는 것의 전부 — `HwpDocument` 의 부분집합. 테스트는 이것만 흉내 낸다. */
export interface DocumentReader {
  getDocumentInfo(): string;
  getSectionCount(): number;
  getParagraphCount(section: number): number;
  getParagraphLength(section: number, paragraph: number): number;
  getTextRange(section: number, paragraph: number, offset: number, count: number): string;
  getOutlineNavigation(): string;
  /** 쪽에 그려진 글 조각들(표 칸 포함). `page` 는 0부터. */
  getPageTextLayout(page: number): string;
  searchAllText(query: string, caseSensitive: boolean, includeCells: boolean): string;
  getFieldList(): string;
  getTextInCell(
    section: number, parentPara: number, controlIndex: number, cellIndex: number,
    cellPara: number, offset: number, count: number,
  ): string;
  getCellParagraphLength(
    section: number, parentPara: number, controlIndex: number, cellIndex: number, cellPara: number,
  ): number;
  getChartDataByIndex(index: number): string;
}

/** 도구가 쓰는 호스트 표면 — `PluginHost` 의 부분집합. */
export interface ToolHost {
  read<T>(fn: (doc: DocumentReader) => T): T;
  getSelectionText(): PluginSelectionText | null;
  getCaret(): PluginCaret | null;
}

/** 도구가 모델에 돌려줄 수 있는 오류. 메시지는 모델이 읽는다. */
export class ToolInputError extends Error {}

interface ReadTool {
  name: string;
  description: string;
  schema: ToolSchema;
  run(host: ToolHost, input: Record<string, unknown>): unknown;
}

function pageCountOf(doc: DocumentReader): number {
  const info = JSON.parse(doc.getDocumentInfo()) as { pageCount?: number };
  return Number.isSafeInteger(info.pageCount) ? info.pageCount! : 0;
}

/** 상한 안에서 자른다. 대리 쌍 한가운데서 자르지 않는다. */
function clip(text: string, max: number): { text: string; truncated: boolean } {
  if (text.length <= max) return { text, truncated: false };
  let cut = text.slice(0, Math.max(0, max));
  if (/[\uD800-\uDBFF]$/.test(cut)) cut = cut.slice(0, -1);
  return { text: cut, truncated: true };
}

interface LayoutRun {
  text: string;
  secIdx?: number;
  paraIdx?: number;
  cellPath?: Array<{ controlIndex: number; cellIndex: number; cellParaIndex: number }>;
  parentParaIdx?: number;
}

/**
 * 쪽 렌더 결과(글 조각 목록)를 읽는 순서의 평문으로 잇는다.
 *
 * `getPageText` 는 본문 문단만 담아 표 안의 글이 빠진다 — 서식 문서는 대부분 표라서
 * 모델이 내용을 못 읽는다. 렌더 조각은 표 칸까지 읽는 순서로 오고 소유 좌표(문단·칸 경로)를
 * 함께 가지므로, 소유(문단·칸 문단)가 바뀌면 줄을 나누고 칸에 들어설 때 표·칸 표시를 붙인다.
 */
export function pageTextFromLayout(layoutJson: string): string {
  const parsed = JSON.parse(layoutJson) as { runs?: LayoutRun[] };
  const lines: string[] = [];
  const tableNumbers = new Map<string, number>();
  let current = '';
  let ownerKey: string | null = null;

  const flush = () => {
    if (current.trim().length > 0) lines.push(current.replace(/\s+$/, ''));
    current = '';
  };

  for (const run of parsed.runs ?? []) {
    const path = run.cellPath ?? [];
    const inCell = path.length > 0;
    // 칸 문단까지 같아야 같은 소유다. 칸이 같고 문단만 다르면 칸 안의 줄바꿈이다.
    const cellKey = inCell
      ? `${run.secIdx ?? 0}:${run.parentParaIdx ?? run.paraIdx ?? 0}:${path.map((e) => `${e.controlIndex}.${e.cellIndex}`).join('/')}`
      : null;
    const key = inCell
      ? `${cellKey}#${path[path.length - 1].cellParaIndex}`
      : `${run.secIdx ?? 0}:${run.paraIdx ?? 0}`;
    // 같은 문단(칸 문단) 안의 화면 줄바꿈은 잇는다 — 낱말 한가운데서 끊긴 줄을 나누면 뜻이 깨진다.
    const newLine = key !== ownerKey;
    if (newLine) {
      flush();
      const enteringCell = inCell && (ownerKey === null || !ownerKey.startsWith(`${cellKey}#`));
      if (enteringCell) {
        const tableKey = cellKey!.slice(0, cellKey!.lastIndexOf('.'));
        if (!tableNumbers.has(tableKey)) tableNumbers.set(tableKey, tableNumbers.size + 1);
        current = `[table ${tableNumbers.get(tableKey)} / cell ${path[path.length - 1].cellIndex}] `;
      } else if (inCell) {
        current = '  ';
      }
      ownerKey = key;
    }
    current += cleanDocumentText(run.text);
  }
  flush();
  return lines.join('\n');
}

const NO_INPUT: ToolSchema = { type: 'object', properties: {}, required: [], additionalProperties: false };

const READ_TOOLS: readonly ReadTool[] = [
  {
    name: 'get_document_info',
    description: 'Get basic facts about the open document: page count, section count and paragraph count '
      + 'per section. Call this first to plan what to read.',
    schema: NO_INPUT,
    run: (host) => host.read((doc) => {
      const info = JSON.parse(doc.getDocumentInfo()) as { version?: string; encrypted?: boolean };
      const sectionCount = doc.getSectionCount();
      const paragraphCounts: number[] = [];
      for (let section = 0; section < sectionCount; section += 1) {
        paragraphCounts.push(doc.getParagraphCount(section));
      }
      return { pageCount: pageCountOf(doc), sectionCount, paragraphCounts, version: info.version ?? null };
    }),
  },
  {
    name: 'get_outline',
    description: 'Get the document outline (numbered outline headings) with level, number, title, '
      + 'page (1-based), section and paragraph (0-based). Empty when the document uses no outline numbering.',
    schema: NO_INPUT,
    run: (host) => host.read((doc) => {
      const parsed = JSON.parse(doc.getOutlineNavigation()) as {
        outline?: Array<{ level: number; number: string; title: string; page: number; section: number; paragraph: number }>;
      };
      const items: Array<Record<string, unknown>> = [];
      let used = 0;
      let truncated = false;
      for (const item of parsed.outline ?? []) {
        const title = cleanDocumentText(item.title);
        if (used + title.length > MAX_RESULT_CHARS) { truncated = true; break; }
        used += title.length;
        items.push({
          level: item.level, number: item.number, title,
          page: item.page, section: item.section, paragraph: item.paragraph,
        });
      }
      return { items, truncated };
    }),
  },
  {
    name: 'read_page_text',
    description: 'Read the text of one page as laid out (1-based page number), including table cells. '
      + 'Table cell text is marked "[table N / cell M]" (N counts tables on the page, M is the cell index in '
      + 'reading order). Long pages are returned in parts: when truncated is true, call again with offset set '
      + 'to nextOffset.',
    schema: {
      type: 'object',
      properties: {
        page: { type: 'integer', description: 'Page number, 1-based.', minimum: 1 },
        offset: { type: 'integer', description: 'Character offset into the page text to continue from.', minimum: 0 },
      },
      required: ['page'],
      additionalProperties: false,
    },
    run: (host, input) => host.read((doc) => {
      const page = input.page as number;
      const offset = (input.offset as number | undefined) ?? 0;
      const pageCount = pageCountOf(doc);
      if (page > pageCount) throw new ToolInputError(`page must be between 1 and ${pageCount}.`);
      const full = pageTextFromLayout(doc.getPageTextLayout(page - 1));
      if (offset > full.length) throw new ToolInputError(`offset must be at most ${full.length}.`);
      const part = clip(full.slice(offset), MAX_RESULT_CHARS);
      return {
        page, pageCount, offset, text: part.text, truncated: part.truncated,
        ...(part.truncated ? { nextOffset: offset + part.text.length } : {}),
      };
    }),
  },
  {
    name: 'read_paragraphs',
    description: `Read body paragraphs of a section in order (0-based section and paragraph indexes, up to `
      + `${MAX_PARAGRAPHS_PER_CALL} per call). Table contents are not included; use read_page_text for tables. `
      + 'When truncated is true, continue from nextFrom.',
    schema: {
      type: 'object',
      properties: {
        section: { type: 'integer', description: 'Section index, 0-based.', minimum: 0 },
        from: { type: 'integer', description: 'First paragraph index, 0-based.', minimum: 0 },
        count: {
          type: 'integer', description: 'How many paragraphs to read.', minimum: 1, maximum: MAX_PARAGRAPHS_PER_CALL,
        },
      },
      required: ['section', 'from', 'count'],
      additionalProperties: false,
    },
    run: (host, input) => host.read((doc) => {
      const section = input.section as number;
      const from = input.from as number;
      const count = input.count as number;
      const sectionCount = doc.getSectionCount();
      if (section >= sectionCount) throw new ToolInputError(`section must be less than ${sectionCount}.`);
      const paragraphCount = doc.getParagraphCount(section);
      if (from >= paragraphCount) throw new ToolInputError(`from must be less than ${paragraphCount}.`);
      const end = Math.min(paragraphCount, from + count);
      const paragraphs: Array<{ index: number; text: string; truncated?: true }> = [];
      let used = 0;
      // 상한에 걸려 요청보다 적게 담았으면 truncated. 남은 문단이 있으면 nextFrom 을 준다.
      let truncated = false;
      let nextFrom = end;
      for (let index = from; index < end; index += 1) {
        const length = doc.getParagraphLength(section, index);
        const text = cleanDocumentText(length > 0 ? doc.getTextRange(section, index, 0, length) : '');
        const room = MAX_RESULT_CHARS - used;
        if (text.length > room) {
          truncated = true;
          if (paragraphs.length === 0) {
            // 한 문단이 혼자 상한을 넘는다 — 그 문단만 잘라 담고 다음 문단부터 이어 읽게 한다.
            paragraphs.push({ index, text: clip(text, room).text, truncated: true });
            nextFrom = index + 1;
          } else {
            nextFrom = index;
          }
          break;
        }
        used += text.length;
        paragraphs.push({ index, text });
      }
      return {
        section, paragraphCount, paragraphs, truncated,
        ...(nextFrom < paragraphCount ? { nextFrom } : {}),
      };
    }),
  },
  {
    name: 'search_text',
    description: `Find text in the document, including table cells. Returns up to ${MAX_SEARCH_HITS} hits `
      + 'with section/paragraph (0-based), whether the hit is inside a table cell, and surrounding context.',
    schema: {
      type: 'object',
      properties: {
        query: { type: 'string', description: 'Text to find.', minLength: 1, maxLength: 200 },
        caseSensitive: { type: 'boolean', description: 'Match letter case exactly. Defaults to false.' },
      },
      required: ['query'],
      additionalProperties: false,
    },
    run: (host, input) => host.read((doc) => {
      const query = input.query as string;
      const hits = JSON.parse(doc.searchAllText(query, input.caseSensitive === true, true)) as Array<{
        sec: number; para: number; charOffset: number; length: number;
        cellContext?: { parentPara: number; ctrlIdx: number; cellIdx: number; cellPara: number };
      }>;
      const out: Array<Record<string, unknown>> = [];
      let used = 0;
      let truncated = hits.length > MAX_SEARCH_HITS;
      for (const hit of hits.slice(0, MAX_SEARCH_HITS)) {
        const cell = hit.cellContext;
        const length = cell
          ? doc.getCellParagraphLength(hit.sec, cell.parentPara, cell.ctrlIdx, cell.cellIdx, cell.cellPara)
          : doc.getParagraphLength(hit.sec, hit.para);
        const start = Math.max(0, hit.charOffset - SEARCH_CONTEXT_CHARS);
        const stop = Math.min(length, hit.charOffset + hit.length + SEARCH_CONTEXT_CHARS);
        const raw = stop > start
          ? (cell
            ? doc.getTextInCell(hit.sec, cell.parentPara, cell.ctrlIdx, cell.cellIdx, cell.cellPara, start, stop - start)
            : doc.getTextRange(hit.sec, hit.para, start, stop - start))
          : '';
        const context = cleanDocumentText(raw);
        if (used + context.length > MAX_RESULT_CHARS) { truncated = true; break; }
        used += context.length;
        out.push({
          section: hit.sec,
          paragraph: cell ? cell.parentPara : hit.para,
          charOffset: hit.charOffset,
          inTable: cell !== undefined,
          context,
        });
      }
      return { query, totalHits: hits.length, hits: out, truncated };
    }),
  },
  {
    name: 'get_selection',
    description: 'Get the text the user has selected in the editor, with its position. '
      + 'selection is null when nothing is selected; caret is the cursor position.',
    schema: NO_INPUT,
    run: (host) => {
      const selection = host.getSelectionText();
      const caret = host.getCaret();
      if (!selection) return { selection: null, caret };
      const part = clip(cleanDocumentText(selection.text), MAX_RESULT_CHARS);
      return {
        selection: {
          text: part.text,
          truncated: part.truncated || selection.truncated,
          start: selection.start,
          end: selection.end,
        },
        caret,
      };
    },
  },
  {
    name: 'list_fields',
    description: 'List the form fields (click-here fields) in the document with name, type, current value '
      + 'and guide text.',
    schema: NO_INPUT,
    run: (host) => host.read((doc) => {
      const fields = JSON.parse(doc.getFieldList()) as Array<{
        name?: string; fieldType?: string; value?: string; guide?: string;
      }>;
      const out: Array<Record<string, unknown>> = [];
      let used = 0;
      let truncated = false;
      for (const field of fields) {
        const value = clip(cleanDocumentText(field.value ?? ''), MAX_FIELD_VALUE_CHARS);
        const entry = {
          name: cleanDocumentText(field.name ?? ''),
          type: field.fieldType ?? '',
          value: value.text,
          ...(value.truncated ? { valueTruncated: true } : {}),
          guide: cleanDocumentText(field.guide ?? ''),
        };
        const size = entry.name.length + entry.value.length + entry.guide.length;
        if (used + size > MAX_RESULT_CHARS) { truncated = true; break; }
        used += size;
        out.push(entry);
      }
      return { fields: out, totalFields: fields.length, truncated };
    }),
  },
  {
    name: 'list_charts',
    description: 'List the charts in the document (1-based chart number, chart kind, category labels, and each series '
      + 'with its values). Use before set_chart_data.',
    schema: NO_INPUT,
    run: (host) => host.read((doc) => ({ charts: listCharts(doc) })),
  },
];

/** 모델에 알리는 도구 목록. 순서를 바꾸지 않는다 — 요청 접두가 같아야 캐시가 유지된다. */
export const READ_TOOL_DEFS: readonly AiToolDef[] = READ_TOOLS.map((tool) => ({
  name: tool.name,
  description: tool.description,
  inputSchema: tool.schema as unknown as Record<string, unknown>,
  // strict 도구 수 상한(20) 안에 쓰기 도구를 두기 위해 읽기 도구는 끈다. 입력은 실행 전에 검증한다.
  strict: false,
}));

/** 문서 해제·미로드 오류인가 — 플러그인 호스트가 `DOCUMENT_RELEASED` 로 던진다. */
function isDocumentReleased(error: unknown): boolean {
  return (error as { code?: unknown } | null)?.code === 'DOCUMENT_RELEASED';
}

/**
 * 도구 호출 하나를 실행해 결과를 돌려준다. 던지지 않는다 — 실패는 `isError` 결과로 모델에 알린다.
 * 성공 결과는 문서 내용 경계로 감싼다.
 */
export function executeReadTool(host: ToolHost, call: AiToolCall, boundaryId: string): AiToolResult {
  const tool = READ_TOOLS.find((candidate) => candidate.name === call.name);
  if (!tool) return { id: call.id, content: `Unknown tool: ${call.name}`, isError: true };
  const checked = validateToolInput(tool.schema, call.input);
  if (!checked.ok) return { id: call.id, content: `Invalid input: ${checked.error}`, isError: true };
  try {
    return { id: call.id, content: wrapDocumentContent(tool.run(host, checked.value), boundaryId), isError: false };
  } catch (error) {
    if (error instanceof ToolInputError) return { id: call.id, content: error.message, isError: true };
    if (isDocumentReleased(error)) return { id: call.id, content: 'No document is open.', isError: true };
    console.error('[AI] 도구 실행 실패', call.name, error);
    return { id: call.id, content: 'Failed to read the document.', isError: true };
  }
}
