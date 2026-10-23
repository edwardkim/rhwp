/**
 * 쓰기 도구 — AI 가 문서를 바꾸는 유일한 통로.
 *
 * 좌표 대신 **찾을 글**로 자리를 정한다. 모델이 문단 번호를 맞히지 않아도 되고, 적용하는 순간
 * 다시 찾아 같은 자리인지 확인할 수 있다 — 제안을 본 뒤 사용자가 문서를 고쳤으면 낡은 자리에
 * 쓰지 않고 거절한다.
 *
 * 흐름: `prepareWrite` 가 제안(무엇이 어떻게 바뀌나)을 만들고, 사용자가 승인하면 `apply` 가
 * `host.transaction` 한 번으로 적용한다 — undo 한 단계다. 적용 경로는 찾기/바꾸기 대화상자와 같다
 * (본문 `replaceText`, 표 칸은 칸 전용 삭제·삽입, #3865).
 */
import type { AiToolCall, AiToolDef } from '../provider/types.ts';
import { validateToolInput, type ToolSchema } from './schema-validate.ts';
import { cleanDocumentText } from './untrusted.ts';

const MAX_FIND_CHARS = 500;
const MAX_NEW_PARAGRAPHS = 50;
const MAX_REPLACE_CHARS = 2000;
const CONTEXT_CHARS = 30;
/** 한 문단 안의 글만 바꾼다 — 줄바꿈·제어 문자로 문단을 나누거나 개체 자리를 만들지 않는다. */
const FORBIDDEN = /[\u0000-\u001f\u007f\u2028\u2029]/;

interface SearchHit {
  sec: number;
  para: number;
  charOffset: number;
  length: number;
  cellContext?: { parentPara: number; ctrlIdx: number; cellIdx: number; cellPara: number };
}

/** 쓰기 도구가 문서에서 쓰는 것의 전부 — `HwpDocument` 의 부분집합. */
export interface DocumentWriter {
  searchAllText(query: string, caseSensitive: boolean, includeCells: boolean): string;
  getParagraphLength(section: number, paragraph: number): number;
  getTextRange(section: number, paragraph: number, offset: number, count: number): string;
  getCellParagraphLength(
    section: number, parentPara: number, controlIndex: number, cellIndex: number, cellPara: number,
  ): number;
  getTextInCell(
    section: number, parentPara: number, controlIndex: number, cellIndex: number,
    cellPara: number, offset: number, count: number,
  ): string;
  replaceText(section: number, paragraph: number, offset: number, length: number, text: string): string;
  deleteTextInCell(
    section: number, parentPara: number, controlIndex: number, cellIndex: number,
    cellPara: number, offset: number, count: number,
  ): string;
  insertTextInCell(
    section: number, parentPara: number, controlIndex: number, cellIndex: number,
    cellPara: number, offset: number, text: string,
  ): string;
  getFieldList(): string;
  setFieldValue(fieldId: number, value: string): string;
  getSectionCount(): number;
  getParagraphCount(section: number): number;
  splitParagraph(section: number, paragraph: number, offset: number): string;
  insertText(section: number, paragraph: number, offset: number, text: string): string;
}

/** 쓰기 도구가 쓰는 호스트 표면 — `PluginHost` 의 부분집합. */
export interface WriteHost {
  read<T>(fn: (doc: DocumentWriter) => T): T;
  transaction<T>(label: string, fn: (tx: { doc(): DocumentWriter }) => T): T;
  /** 빈 새 문서로 바꾼다. 저장 안 된 변경 확인에서 사용자가 취소하면 false. */
  createBlankDocument(): Promise<boolean>;
}

export type WriteToolName = 'replace_text' | 'fill_field' | 'insert_paragraphs' | 'create_document';

/** 제안 하나. 화면은 `prefix + removed + suffix` 를 「전」, `prefix + inserted + suffix` 를 「후」로 보인다. */
export interface WriteProposal {
  tool: WriteToolName;
  /** 어디인가 — 화면 표시용 */
  location: {
    section?: number;
    paragraph?: number;
    inTable?: boolean;
    field?: string;
    /** 문단 추가 자리 */
    position?: 'start' | 'end' | 'after';
  };
  prefix: string;
  removed: string;
  inserted: string;
  suffix: string;
  /** 승인되면 부른다. 던지면 적용하지 않은 것이다. */
  apply(): void | Promise<void>;
}

/** 모델에 돌려줄 수 있는 쓰기 오류(메시지를 모델이 읽는다). */
export class WriteRefused extends Error {}

const WRITE_SCHEMAS: Record<WriteToolName, ToolSchema> = {
  replace_text: {
    type: 'object',
    properties: {
      find: {
        type: 'string', minLength: 1, maxLength: MAX_FIND_CHARS,
        description: 'Exact text currently in the document, copied from what you read (case and spacing must match). '
          + 'It must be inside one paragraph or one table cell paragraph.',
      },
      replace: {
        type: 'string', maxLength: MAX_REPLACE_CHARS,
        description: 'New text that replaces it. No line breaks. Empty string deletes the text.',
      },
      occurrence: {
        type: 'integer', minimum: 1,
        description: 'Which match to change, counting from 1 in reading order. Only needed when find occurs more than once.',
      },
    },
    required: ['find', 'replace'],
    additionalProperties: false,
  },
  fill_field: {
    type: 'object',
    properties: {
      name: { type: 'string', minLength: 1, maxLength: 200, description: 'Field name as listed by list_fields.' },
      value: { type: 'string', maxLength: MAX_REPLACE_CHARS, description: 'Value to put in the field. No line breaks.' },
      occurrence: {
        type: 'integer', minimum: 1,
        description: 'Which field with this name, counting from 1. Only needed when several fields share the name.',
      },
    },
    required: ['name', 'value'],
    additionalProperties: false,
  },
  insert_paragraphs: {
    type: 'object',
    properties: {
      position: {
        type: 'string', enum: ['start', 'end', 'after'],
        description: 'Where to add: start or end of the document, or after the body paragraph containing `after`.',
      },
      after: {
        type: 'string', minLength: 1, maxLength: MAX_FIND_CHARS,
        description: 'With position "after": exact text from the body paragraph to add after (must occur once; not in a table).',
      },
      paragraphs: {
        type: 'array', minItems: 1, maxItems: MAX_NEW_PARAGRAPHS,
        items: { type: 'string', maxLength: MAX_REPLACE_CHARS },
        description: 'New paragraphs in order, one string each, without line breaks. Use an empty string for a blank line.',
      },
    },
    required: ['position', 'paragraphs'],
    additionalProperties: false,
  },
  create_document: {
    type: 'object',
    properties: {},
    required: [],
    additionalProperties: false,
  },
};

export const WRITE_TOOL_DEFS: readonly AiToolDef[] = [
  {
    name: 'replace_text',
    description: 'Change text in the document (body or table cell). Each call is shown to the user, who may apply '
      + 'or reject it. find must match exactly one place; if it matches several, either make it longer or pass '
      + 'occurrence. Use one call per separate place. Formatting of the surrounding text is kept.',
    inputSchema: WRITE_SCHEMAS.replace_text as unknown as Record<string, unknown>,
  },
  {
    name: 'fill_field',
    description: 'Put a value into a form field (click-here field) found with list_fields. Shown to the user for '
      + 'approval like replace_text.',
    inputSchema: WRITE_SCHEMAS.fill_field as unknown as Record<string, unknown>,
  },
  {
    name: 'insert_paragraphs',
    description: 'Write new body paragraphs: at the start or end of the document, or after a paragraph you name by its '
      + 'text. Use this to write new content (including into a new document). New paragraphs take the style of the '
      + 'paragraph they are added next to. Shown to the user for approval.',
    inputSchema: WRITE_SCHEMAS.insert_paragraphs as unknown as Record<string, unknown>,
  },
  {
    name: 'create_document',
    description: 'Replace the open document with a new blank document, when the user asks for a new document. '
      + 'The user approves first and is asked to save unsaved changes. Then write content with insert_paragraphs '
      + '(position "end").',
    inputSchema: WRITE_SCHEMAS.create_document as unknown as Record<string, unknown>,
  },
];

const WRITE_TOOL_NAMES: readonly string[] = ['replace_text', 'fill_field', 'insert_paragraphs', 'create_document'];

export function isWriteTool(name: string): name is WriteToolName {
  return WRITE_TOOL_NAMES.includes(name);
}

function sameHit(a: SearchHit, b: SearchHit): boolean {
  return a.sec === b.sec && a.para === b.para && a.charOffset === b.charOffset && a.length === b.length
    && JSON.stringify(a.cellContext ?? null) === JSON.stringify(b.cellContext ?? null);
}

function paragraphText(doc: DocumentWriter, hit: SearchHit, from: number, to: number): string {
  const cell = hit.cellContext;
  const length = cell
    ? doc.getCellParagraphLength(hit.sec, cell.parentPara, cell.ctrlIdx, cell.cellIdx, cell.cellPara)
    : doc.getParagraphLength(hit.sec, hit.para);
  const start = Math.max(0, from);
  const stop = Math.min(length, to);
  if (stop <= start) return '';
  return cleanDocumentText(cell
    ? doc.getTextInCell(hit.sec, cell.parentPara, cell.ctrlIdx, cell.cellIdx, cell.cellPara, start, stop - start)
    : doc.getTextRange(hit.sec, hit.para, start, stop - start));
}

function pickHit(hits: SearchHit[], find: string, occurrence: number | undefined): SearchHit {
  if (hits.length === 0) {
    throw new WriteRefused(`"${find}" was not found. Read the document again and copy the exact text.`);
  }
  if (occurrence === undefined) {
    if (hits.length > 1) {
      throw new WriteRefused(`"${find}" occurs ${hits.length} times. Use a longer find text or pass occurrence (1-${hits.length}).`);
    }
    return hits[0];
  }
  if (occurrence > hits.length) {
    throw new WriteRefused(`occurrence must be between 1 and ${hits.length}.`);
  }
  return hits[occurrence - 1];
}

function prepareReplace(host: WriteHost, input: Record<string, unknown>): WriteProposal {
  const find = input.find as string;
  const replace = input.replace as string;
  const occurrence = input.occurrence as number | undefined;
  if (FORBIDDEN.test(find) || FORBIDDEN.test(replace)) {
    throw new WriteRefused('find and replace must not contain line breaks or control characters.');
  }
  if (find === replace) throw new WriteRefused('replace is the same as find; nothing to change.');

  const search = (doc: DocumentWriter) => JSON.parse(doc.searchAllText(find, true, true)) as SearchHit[];
  return host.read((doc) => {
    const hits = search(doc);
    const hit = pickHit(hits, find, occurrence);
    const prefix = paragraphText(doc, hit, hit.charOffset - CONTEXT_CHARS, hit.charOffset);
    const suffix = paragraphText(doc, hit, hit.charOffset + hit.length, hit.charOffset + hit.length + CONTEXT_CHARS);
    return {
      tool: 'replace_text',
      location: {
        section: hit.sec,
        paragraph: hit.cellContext ? hit.cellContext.parentPara : hit.para,
        inTable: hit.cellContext !== undefined,
      },
      prefix,
      removed: find,
      inserted: replace,
      suffix,
      apply: () => host.transaction('AI: 글 바꾸기', (tx) => {
        const target = tx.doc();
        // 제안 뒤에 문서가 바뀌었으면 같은 자리가 아닐 수 있다 — 다시 찾아 확인한다.
        const again = search(target);
        if (again.length !== hits.length || !sameHit(pickHit(again, find, occurrence), hit)) {
          throw new WriteRefused('The document changed after this was proposed. Read it again and propose a new change.');
        }
        const cell = hit.cellContext;
        if (cell) {
          const removed = JSON.parse(target.deleteTextInCell(
            hit.sec, cell.parentPara, cell.ctrlIdx, cell.cellIdx, cell.cellPara, hit.charOffset, hit.length,
          )) as { ok?: boolean };
          if (!removed.ok) throw new Error('deleteTextInCell failed');
          if (replace.length > 0) {
            const inserted = JSON.parse(target.insertTextInCell(
              hit.sec, cell.parentPara, cell.ctrlIdx, cell.cellIdx, cell.cellPara, hit.charOffset, replace,
            )) as { ok?: boolean };
            // 트랜잭션 본문이 던지면 진입 시점으로 롤백된다 — 반쯤 지운 상태를 남기지 않는다.
            if (!inserted.ok) throw new Error('insertTextInCell failed');
          }
        } else {
          const result = JSON.parse(target.replaceText(hit.sec, hit.para, hit.charOffset, hit.length, replace)) as { ok?: boolean };
          if (!result.ok) throw new Error('replaceText failed');
        }
      }),
    };
  });
}

function prepareFill(host: WriteHost, input: Record<string, unknown>): WriteProposal {
  const name = input.name as string;
  const value = input.value as string;
  const occurrence = input.occurrence as number | undefined;
  if (FORBIDDEN.test(value)) throw new WriteRefused('value must not contain line breaks or control characters.');

  return host.read((doc) => {
    const fields = (JSON.parse(doc.getFieldList()) as Array<{ fieldId: number; name?: string; value?: string }>)
      .filter((field) => field.name === name);
    if (fields.length === 0) throw new WriteRefused(`No field named "${name}". Call list_fields to see the names.`);
    if (occurrence === undefined && fields.length > 1) {
      throw new WriteRefused(`${fields.length} fields are named "${name}". Pass occurrence (1-${fields.length}).`);
    }
    if (occurrence !== undefined && occurrence > fields.length) {
      throw new WriteRefused(`occurrence must be between 1 and ${fields.length}.`);
    }
    const index = (occurrence ?? 1) - 1;
    const target = fields[index];
    const before = cleanDocumentText(target.value ?? '');
    if (before === value) throw new WriteRefused('The field already has this value.');
    return {
      tool: 'fill_field',
      location: { field: fields.length > 1 ? `${name} (${index + 1})` : name },
      prefix: '',
      removed: before,
      inserted: value,
      suffix: '',
      // 이름이 아니라 목록의 fieldId 로 지목한다 — 같은 이름의 누름틀이 여러 개여도 정확히 하나다.
      apply: () => host.transaction('AI: 누름틀 채우기', (tx) => {
        const writer = tx.doc();
        const now = (JSON.parse(writer.getFieldList()) as Array<{ fieldId: number; name?: string; value?: string }>)
          .find((field) => field.fieldId === target.fieldId);
        if (!now || now.name !== name || cleanDocumentText(now.value ?? '') !== before) {
          throw new WriteRefused('The field changed after this was proposed. Call list_fields again.');
        }
        const result = JSON.parse(writer.setFieldValue(target.fieldId, value)) as { ok?: boolean };
        if (!result.ok) throw new Error('setFieldValue failed');
      }),
    };
  });
}

function bodyText(doc: DocumentWriter, section: number, paragraph: number): string {
  const length = doc.getParagraphLength(section, paragraph);
  return length > 0 ? doc.getTextRange(section, paragraph, 0, length) : '';
}

function ok(json: string, what: string): void {
  if (!(JSON.parse(json) as { ok?: boolean }).ok) throw new Error(`${what} failed`);
}

/** 문단 `paragraph` 뒤에 새 문단들을 차례로 만든다. 나누기는 앞 문단의 모양을 이어받는다. */
function appendAfter(doc: DocumentWriter, section: number, paragraph: number, texts: readonly string[]): void {
  let at = paragraph;
  for (const text of texts) {
    ok(doc.splitParagraph(section, at, doc.getParagraphLength(section, at)), 'splitParagraph');
    at += 1;
    if (text) ok(doc.insertText(section, at, 0, text), 'insertText');
  }
}

function prepareInsert(host: WriteHost, input: Record<string, unknown>): WriteProposal {
  const position = input.position as 'start' | 'end' | 'after';
  const after = input.after as string | undefined;
  const texts = input.paragraphs as string[];
  if (texts.some((text) => FORBIDDEN.test(text))) {
    throw new WriteRefused('Each paragraph must be one line: put each paragraph in its own array item.');
  }
  if (position === 'after' && !after) throw new WriteRefused('position "after" needs the after text.');
  if (position !== 'after' && after !== undefined) throw new WriteRefused('after is only used with position "after".');

  const search = (doc: DocumentWriter) => (JSON.parse(doc.searchAllText(after!, true, true)) as SearchHit[]);
  return host.read((doc) => {
    let anchor: SearchHit | null = null;
    let anchorCount = 0;
    if (position === 'after') {
      const hits = search(doc);
      anchorCount = hits.length;
      anchor = pickHit(hits, after!, undefined);
      if (anchor.cellContext) throw new WriteRefused('Adding paragraphs inside a table cell is not supported yet.');
    }
    const lastSection = doc.getSectionCount() - 1;
    const location = position === 'after'
      ? { section: anchor!.sec, paragraph: anchor!.para, position }
      : position === 'start'
        ? { section: 0, paragraph: 0, position }
        : { section: lastSection, paragraph: doc.getParagraphCount(lastSection) - 1, position };
    return {
      tool: 'insert_paragraphs',
      location,
      prefix: '',
      removed: '',
      inserted: texts.join('\n'),
      suffix: '',
      apply: () => host.transaction('AI: 문단 추가', (tx) => {
        const target = tx.doc();
        if (position === 'after') {
          const again = search(target);
          if (again.length !== anchorCount || !sameHit(pickHit(again, after!, undefined), anchor!)) {
            throw new WriteRefused('The document changed after this was proposed. Read it again and propose a new change.');
          }
          appendAfter(target, anchor!.sec, anchor!.para, texts);
          return;
        }
        if (position === 'start') {
          // 첫 문단을 맨 앞에서 나누면 빈 문단이 앞에 생긴다 — 거기에 첫 글을 넣고 나머지를 잇는다.
          ok(target.splitParagraph(0, 0, 0), 'splitParagraph');
          if (texts[0]) ok(target.insertText(0, 0, 0, texts[0]), 'insertText');
          appendAfter(target, 0, 0, texts.slice(1));
          return;
        }
        const section = target.getSectionCount() - 1;
        const last = target.getParagraphCount(section) - 1;
        // 새 문서처럼 마지막 문단이 비어 있으면 그 문단부터 채운다 — 맨 앞에 빈 줄을 남기지 않는다.
        if (bodyText(target, section, last).length === 0) {
          if (texts[0]) ok(target.insertText(section, last, 0, texts[0]), 'insertText');
          appendAfter(target, section, last, texts.slice(1));
        } else {
          appendAfter(target, section, last, texts);
        }
      }),
    };
  });
}

function prepareCreate(host: WriteHost): WriteProposal {
  return {
    tool: 'create_document',
    location: {},
    prefix: '',
    removed: '',
    inserted: '',
    suffix: '',
    apply: async () => {
      if (!await host.createBlankDocument()) {
        throw new WriteRefused('The user cancelled creating a new document; the open document is unchanged.');
      }
    },
  };
}

/** 입력을 검증하고 제안을 만든다. 거절 사유는 `WriteRefused` 로 던진다. */
export function prepareWrite(host: WriteHost, call: AiToolCall): WriteProposal {
  if (!isWriteTool(call.name)) throw new WriteRefused(`Unknown tool: ${call.name}`);
  const tool = call.name;
  const checked = validateToolInput(WRITE_SCHEMAS[tool], call.input);
  if (!checked.ok) throw new WriteRefused(`Invalid input: ${checked.error}`);
  switch (tool) {
    case 'replace_text': return prepareReplace(host, checked.value);
    case 'fill_field': return prepareFill(host, checked.value);
    case 'insert_paragraphs': return prepareInsert(host, checked.value);
    case 'create_document': return prepareCreate(host);
  }
}
