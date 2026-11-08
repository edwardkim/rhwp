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
const MAX_TABLE_COLUMNS = 20;
const MAX_TABLE_CELLS = 400;
const COLOR = /^#[0-9a-fA-F]{6}$/;
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
  /**
   * 문단을 나눈다. **`offset` 은 논리 위치**(글자와 인라인 개체를 각각 1로 센다) — 다른 편집 API 의 글자 위치와
   * 축이 다르다(실측: 각주가 든 「…23일」을 글자 길이 12에서 나누면 「일」이 다음 문단으로 넘어감).
   */
  splitParagraph(section: number, paragraph: number, offset: number): string;
  /** 논리 길이 — 글자와 인라인 개체(각주·그림·수식·표 등)를 각각 1로 센다. 문단 끝에서 나눌 때 쓴다. */
  getLogicalLength(section: number, paragraph: number): number;
  insertText(section: number, paragraph: number, offset: number, text: string): string;
  applyCharFormat(section: number, paragraph: number, start: number, end: number, propsJson: string): string;
  applyCharFormatInCell(
    section: number, parentPara: number, controlIndex: number, cellIndex: number,
    cellPara: number, start: number, end: number, propsJson: string,
  ): string;
  applyParaFormat(section: number, paragraph: number, propsJson: string): string;
  applyParaFormatInCell(
    section: number, parentPara: number, controlIndex: number, cellIndex: number, cellPara: number, propsJson: string,
  ): string;
  findOrCreateFontId(name: string): number;
  createTable(section: number, paragraph: number, offset: number, rows: number, columns: number): string;
  deleteParagraph(section: number, paragraph: number): string;
  deleteText(section: number, paragraph: number, offset: number, count: number): string;
  /** 문단 안 컨트롤(구역 설정·머리말·표·각주 등)의 글자 위치 목록(JSON 배열) */
  getControlTextPositions(section: number, paragraph: number): string;
  ensureDefaultBullet(bulletChar: string): number;
  ensureDefaultNumbering(): number;
  getStyleList(): string;
  applyStyle(section: number, paragraph: number, styleId: number): string;
  getParaPropertiesAt(section: number, paragraph: number): string;
  getTableDimensions(section: number, parentPara: number, controlIndex: number): string;
  getCellInfo(section: number, parentPara: number, controlIndex: number, cellIndex: number): string;
  insertTableRow(section: number, parentPara: number, controlIndex: number, row: number, below: boolean): string;
  insertTableColumn(section: number, parentPara: number, controlIndex: number, column: number, right: boolean): string;
  deleteTableRow(section: number, parentPara: number, controlIndex: number, row: number): string;
  deleteTableColumn(section: number, parentPara: number, controlIndex: number, column: number): string;
  mergeTableCells(
    section: number, parentPara: number, controlIndex: number,
    startRow: number, startCol: number, endRow: number, endCol: number,
  ): string;
  splitParagraphInCell(
    section: number, parentPara: number, controlIndex: number, cellIndex: number, cellPara: number, offset: number,
  ): string;
  createHeaderFooter(section: number, isHeader: boolean, applyTo: number): string;
  getHeaderFooter(section: number, isHeader: boolean, applyTo: number): string;
  insertTextInHeaderFooter(
    section: number, isHeader: boolean, applyTo: number, hfPara: number, offset: number, text: string,
  ): string;
  deleteTextInHeaderFooter(
    section: number, isHeader: boolean, applyTo: number, hfPara: number, offset: number, count: number,
  ): string;
  insertFootnote(section: number, paragraph: number, offset: number): string;
  replaceAll(query: string, text: string, caseSensitive: boolean): string;
  getPageDef(section: number): string;
  setPageDef(section: number, json: string): string;
  setCellProperties(section: number, parentPara: number, controlIndex: number, cellIndex: number, json: string): string;
  insertHyperlinkEx(json: string): number;
  addBookmark(section: number, paragraph: number, offset: number, name: string): string;
  getBookmarks(): string;
  insertEquation(section: number, paragraph: number, offset: number, script: string, fontSize: number, color: number): string;
  setColumnDef(section: number, count: number, type: number, sameWidth: number, spacing: number): string;
  insertPicture(
    section: number, paragraph: number, offset: number, cellPathJson: string, data: Uint8Array,
    width: number, height: number, naturalWidth: number, naturalHeight: number, extension: string, description: string,
  ): string;
  setPictureProperties(section: number, paragraph: number, controlIndex: number, json: string): string;
  createShapeControl(json: string): string;
  getDocumentInfo(): string;
  /** 쪽에 그려진 글 조각(0부터 센 쪽). 개체를 놓을 자리를 실제 조판 위치로 정할 때 쓴다. */
  getPageTextLayout(page: number): string;
  insertNewNumber(section: number, paragraph: number, offset: number, start: number): string;
  getPageBorderFill(section: number): string;
  setPageBorderFill(section: number, json: string): string;
  insertClickHereField(
    section: number, paragraph: number, offset: number, guide: string, memo: string, name: string, editable: boolean,
  ): string;
  getChartDataByIndex(index: number): string;
  setChartDataByIndex(index: number, json: string): string;
  insertTextInFootnote(
    section: number, paragraph: number, controlIndex: number, fnPara: number, offset: number, text: string,
  ): string;
}

/** 쓰기 도구가 쓰는 호스트 표면 — `PluginHost` 의 부분집합. */
export interface WriteHost {
  read<T>(fn: (doc: DocumentWriter) => T): T;
  transaction<T>(label: string, fn: (tx: { doc(): DocumentWriter }) => T): T;
  /** 빈 새 문서로 바꾼다. 저장 안 된 변경 확인에서 사용자가 취소하면 false. */
  createBlankDocument(): Promise<boolean>;
}

export type WriteToolName =
  | 'replace_text' | 'fill_field' | 'insert_paragraphs' | 'create_document'
  | 'format_text' | 'format_paragraph' | 'insert_table'
  | 'delete_paragraph' | 'set_list' | 'apply_style' | 'edit_table' | 'set_cell_text'
  | 'set_header_footer' | 'insert_footnote'
  | 'replace_all' | 'format_cell' | 'page_setup'
  | 'set_table_border' | 'set_cell_size' | 'insert_hyperlink' | 'add_bookmark' | 'insert_equation' | 'set_columns'
  | 'insert_picture' | 'insert_textbox' | 'insert_shape' | 'new_page_number' | 'set_page_border' | 'insert_field'
  | 'set_chart_data';

/** 제안 하나. 화면은 `prefix + removed + suffix` 를 「전」, `prefix + inserted + suffix` 를 「후」로 보인다. */
export interface WriteProposal {
  /** `run_command` 는 명령 통로(`command-tools.ts`)의 제안이다. */
  tool: WriteToolName | 'run_command';
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
  /** 바꾸는 속성(화면 요약용). 키는 도구 입력 이름 그대로다. */
  format?: Record<string, string | number | boolean>;
  /** 적용 결과와 함께 모델에 알릴 말(요청과 다르게 처리한 이유 등) */
  note?: string;
  /**
   * 사용자가 파일을 골라야 적용할 수 있는 제안(그림). 화면은 승인 전에 파일을 받아 `provide` 로 넘긴다.
   * 자동 적용이어도 파일 없이는 적용하지 않는다.
   */
  file?: { accept: string; provide(file: PickedFile): void; provided(): boolean };
  /** 승인되면 부른다. 던지면 적용하지 않은 것이다. */
  apply(): void | Promise<void>;
}

/** 사용자가 고른 파일 — 그림이면 원본 픽셀 크기까지. */
export interface PickedFile {
  name: string;
  bytes: Uint8Array;
  extension: string;
  naturalWidth: number;
  naturalHeight: number;
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
        description: 'With position "after": exact text from the paragraph to add after (must occur once; may be inside a table cell).',
      },
      paragraphs: {
        type: 'array', minItems: 1, maxItems: MAX_NEW_PARAGRAPHS,
        items: { type: 'string', maxLength: MAX_REPLACE_CHARS },
        description: 'New paragraphs in order, one string each, without line breaks. Use an empty string for a blank line.',
      },
      style: {
        type: 'string', minLength: 1, maxLength: 100,
        description: 'Style for the new paragraphs (for example 바탕글 or 본문). Without it they continue the formatting of '
          + 'the paragraph they follow, like pressing Enter — so after a heading or list item, pass a body style.',
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
  format_text: {
    type: 'object',
    properties: {
      find: {
        type: 'string', minLength: 1, maxLength: MAX_FIND_CHARS,
        description: 'Exactly the text to format — only these characters change. Copy it from the document. If it occurs '
          + 'more than once, keep it exact and pass occurrence instead of adding neighbouring words.',
      },
      occurrence: { type: 'integer', minimum: 1, description: 'Which match, counting from 1, when find occurs more than once.' },
      bold: { type: 'boolean', description: 'Bold on or off.' },
      italic: { type: 'boolean', description: 'Italic on or off.' },
      underline: { type: 'boolean', description: 'Underline on or off.' },
      strikethrough: { type: 'boolean', description: 'Strikethrough on or off.' },
      fontSize: { type: 'number', minimum: 1, maximum: 300, description: 'Font size in points.' },
      textColor: { type: 'string', minLength: 7, maxLength: 7, description: 'Text color as #RRGGBB.' },
      highlightColor: { type: 'string', minLength: 7, maxLength: 7, description: 'Background (shade) color as #RRGGBB.' },
      fontName: { type: 'string', minLength: 1, maxLength: 100, description: 'Font face name, for example 맑은 고딕.' },
      charRatio: { type: 'integer', minimum: 50, maximum: 200, description: 'Character width ratio (장평) in percent; 100 is normal.' },
      charSpacing: { type: 'integer', minimum: -50, maximum: 50, description: 'Character spacing (자간) in percent; 0 is normal.' },
    },
    required: ['find'],
    additionalProperties: false,
  },
  format_paragraph: {
    type: 'object',
    properties: {
      find: {
        type: 'string', minLength: 1, maxLength: MAX_FIND_CHARS,
        description: 'Exact text inside the paragraph to format, copied from the document.',
      },
      occurrence: { type: 'integer', minimum: 1, description: 'Which match, counting from 1, when find occurs more than once.' },
      alignment: {
        type: 'string', enum: ['left', 'center', 'right', 'justify', 'distribute'], description: 'Paragraph alignment.',
      },
      lineSpacing: { type: 'integer', minimum: 50, maximum: 500, description: 'Line spacing in percent (160 is the usual default).' },
      leftMargin: { type: 'number', minimum: 0, maximum: 400, description: 'Left margin of the paragraph in points.' },
      rightMargin: { type: 'number', minimum: 0, maximum: 400, description: 'Right margin of the paragraph in points.' },
      firstLineIndent: {
        type: 'number', minimum: -400, maximum: 400,
        description: 'First line indent in points; negative for a hanging indent. 0 removes it.',
      },
      spaceBefore: { type: 'number', minimum: 0, maximum: 400, description: 'Space above the paragraph in points.' },
      spaceAfter: { type: 'number', minimum: 0, maximum: 400, description: 'Space below the paragraph in points.' },
    },
    required: ['find'],
    additionalProperties: false,
  },
  delete_paragraph: {
    type: 'object',
    properties: {
      find: { type: 'string', minLength: 1, maxLength: MAX_FIND_CHARS, description: 'Exact text inside the body paragraph to delete.' },
      occurrence: { type: 'integer', minimum: 1, description: 'Which match, counting from 1, when find occurs more than once.' },
    },
    required: ['find'],
    additionalProperties: false,
  },
  set_list: {
    type: 'object',
    properties: {
      find: { type: 'string', minLength: 1, maxLength: MAX_FIND_CHARS, description: 'Exact text inside the paragraph.' },
      occurrence: { type: 'integer', minimum: 1, description: 'Which match, counting from 1, when find occurs more than once.' },
      list: { type: 'string', enum: ['bullet', 'number', 'none'], description: 'Bullet list, numbered list, or remove the list.' },
    },
    required: ['find', 'list'],
    additionalProperties: false,
  },
  apply_style: {
    type: 'object',
    properties: {
      find: { type: 'string', minLength: 1, maxLength: MAX_FIND_CHARS, description: 'Exact text inside the body paragraph.' },
      occurrence: { type: 'integer', minimum: 1, description: 'Which match, counting from 1, when find occurs more than once.' },
      style: { type: 'string', minLength: 1, maxLength: 100, description: 'Style name in the document, for example 본문, 개요 1, 바탕글.' },
    },
    required: ['find', 'style'],
    additionalProperties: false,
  },
  edit_table: {
    type: 'object',
    properties: {
      find: {
        type: 'string', minLength: 1, maxLength: MAX_FIND_CHARS,
        description: 'Exact text inside a cell of the table; that cell is the reference cell.',
      },
      occurrence: { type: 'integer', minimum: 1, description: 'Which match, counting from 1, when find occurs more than once.' },
      action: {
        type: 'string',
        enum: ['insert_row_above', 'insert_row_below', 'insert_column_left', 'insert_column_right',
          'delete_row', 'delete_column', 'merge_right', 'merge_down'],
        description: 'What to do relative to the reference cell.',
      },
      span: {
        type: 'integer', minimum: 2, maximum: 20,
        description: 'For merge_right / merge_down: how many cells to merge, including the reference cell (default 2).',
      },
    },
    required: ['find', 'action'],
    additionalProperties: false,
  },
  set_cell_text: {
    type: 'object',
    properties: {
      find: {
        type: 'string', minLength: 1, maxLength: MAX_FIND_CHARS,
        description: 'Exact text inside any cell of the target table (to identify the table).',
      },
      occurrence: { type: 'integer', minimum: 1, description: 'Which match, counting from 1, when find occurs more than once.' },
      row: { type: 'integer', minimum: 1, description: 'Row number, 1-based.' },
      column: { type: 'integer', minimum: 1, description: 'Column number, 1-based.' },
      text: { type: 'string', maxLength: MAX_REPLACE_CHARS, description: 'New text for the whole cell (first paragraph). No line breaks.' },
    },
    required: ['find', 'row', 'column', 'text'],
    additionalProperties: false,
  },
  set_header_footer: {
    type: 'object',
    properties: {
      kind: { type: 'string', enum: ['header', 'footer'], description: 'Header (top of page) or footer (bottom of page).' },
      text: { type: 'string', maxLength: MAX_REPLACE_CHARS, description: 'Text for all pages of the first section. Empty clears it.' },
    },
    required: ['kind', 'text'],
    additionalProperties: false,
  },
  insert_footnote: {
    type: 'object',
    properties: {
      find: { type: 'string', minLength: 1, maxLength: MAX_FIND_CHARS, description: 'Exact body text; the footnote mark goes right after it.' },
      occurrence: { type: 'integer', minimum: 1, description: 'Which match, counting from 1, when find occurs more than once.' },
      text: { type: 'string', minLength: 1, maxLength: MAX_REPLACE_CHARS, description: 'Footnote text. No line breaks.' },
    },
    required: ['find', 'text'],
    additionalProperties: false,
  },
  replace_all: {
    type: 'object',
    properties: {
      find: { type: 'string', minLength: 1, maxLength: MAX_FIND_CHARS, description: 'Text to find everywhere in the document (including tables).' },
      replace: { type: 'string', maxLength: MAX_REPLACE_CHARS, description: 'Replacement text. No line breaks. Empty deletes every match.' },
      caseSensitive: { type: 'boolean', description: 'Match letter case exactly. Defaults to true.' },
    },
    required: ['find', 'replace'],
    additionalProperties: false,
  },
  format_cell: {
    type: 'object',
    properties: {
      find: { type: 'string', minLength: 1, maxLength: MAX_FIND_CHARS, description: 'Exact text inside a cell of the table (the reference cell).' },
      occurrence: { type: 'integer', minimum: 1, description: 'Which match, counting from 1, when find occurs more than once.' },
      scope: {
        type: 'string', enum: ['cell', 'row', 'column', 'table'],
        description: 'Apply to the reference cell only (default), its whole row, its whole column, or every cell of the table.',
      },
      backgroundColor: { type: 'string', minLength: 4, maxLength: 7, description: 'Cell background as #RRGGBB, or "none" to remove it.' },
      verticalAlign: { type: 'string', enum: ['top', 'middle', 'bottom'], description: 'Vertical alignment of text in the cells.' },
    },
    required: ['find'],
    additionalProperties: false,
  },
  page_setup: {
    type: 'object',
    properties: {
      paper: { type: 'string', enum: ['A4', 'A3', 'B4', 'B5', 'Letter', 'Legal'], description: 'Paper size.' },
      orientation: { type: 'string', enum: ['portrait', 'landscape'], description: 'Page orientation.' },
      marginTop: { type: 'number', minimum: 0, maximum: 100, description: 'Top margin in millimetres.' },
      marginBottom: { type: 'number', minimum: 0, maximum: 100, description: 'Bottom margin in millimetres.' },
      marginLeft: { type: 'number', minimum: 0, maximum: 100, description: 'Left margin in millimetres.' },
      marginRight: { type: 'number', minimum: 0, maximum: 100, description: 'Right margin in millimetres.' },
    },
    required: [],
    additionalProperties: false,
  },
  set_table_border: {
    type: 'object',
    properties: {
      find: { type: 'string', minLength: 1, maxLength: MAX_FIND_CHARS, description: 'Exact text inside a cell of the table (the reference cell).' },
      occurrence: { type: 'integer', minimum: 1, description: 'Which match, counting from 1, when find occurs more than once.' },
      scope: { type: 'string', enum: ['cell', 'row', 'column', 'table'], description: 'Cells to change (default cell).' },
      style: { type: 'string', enum: ['solid', 'dashed', 'dotted', 'double', 'none'], description: 'Line style for all four sides of each cell.' },
      thicknessMm: { type: 'number', minimum: 0.1, maximum: 5, description: 'Line thickness in millimetres (default 0.12).' },
      color: { type: 'string', minLength: 7, maxLength: 7, description: 'Line color as #RRGGBB (default black).' },
    },
    required: ['find', 'style'],
    additionalProperties: false,
  },
  set_cell_size: {
    type: 'object',
    properties: {
      find: { type: 'string', minLength: 1, maxLength: MAX_FIND_CHARS, description: 'Exact text inside a cell of the table (the reference cell).' },
      occurrence: { type: 'integer', minimum: 1, description: 'Which match, counting from 1, when find occurs more than once.' },
      columnWidthMm: { type: 'number', minimum: 5, maximum: 400, description: 'Width of the reference cell column in millimetres.' },
      rowHeightMm: { type: 'number', minimum: 3, maximum: 400, description: 'Height of the reference cell row in millimetres.' },
    },
    required: ['find'],
    additionalProperties: false,
  },
  insert_hyperlink: {
    type: 'object',
    properties: {
      find: { type: 'string', minLength: 1, maxLength: MAX_FIND_CHARS, description: 'Exact text that becomes the link.' },
      occurrence: { type: 'integer', minimum: 1, description: 'Which match, counting from 1, when find occurs more than once.' },
      url: { type: 'string', minLength: 1, maxLength: 2000, description: 'Link target: http(s):// or mailto: address.' },
    },
    required: ['find', 'url'],
    additionalProperties: false,
  },
  add_bookmark: {
    type: 'object',
    properties: {
      find: { type: 'string', minLength: 1, maxLength: MAX_FIND_CHARS, description: 'Exact body text; the bookmark goes at its start.' },
      occurrence: { type: 'integer', minimum: 1, description: 'Which match, counting from 1, when find occurs more than once.' },
      name: { type: 'string', minLength: 1, maxLength: 100, description: 'Bookmark name (unique in the document).' },
    },
    required: ['find', 'name'],
    additionalProperties: false,
  },
  insert_equation: {
    type: 'object',
    properties: {
      after: { type: 'string', minLength: 1, maxLength: MAX_FIND_CHARS, description: 'Exact body text; the equation goes right after it.' },
      occurrence: { type: 'integer', minimum: 1, description: 'Which match, counting from 1, when after occurs more than once.' },
      script: {
        type: 'string', minLength: 1, maxLength: 2000,
        description: 'Equation in Hancom equation script, for example "x = {-b +- sqrt {b^2 - 4ac}} over {2a}".',
      },
      fontSize: { type: 'number', minimum: 5, maximum: 100, description: 'Font size in points (default 10).' },
    },
    required: ['after', 'script'],
    additionalProperties: false,
  },
  set_columns: {
    type: 'object',
    properties: {
      count: { type: 'integer', minimum: 1, maximum: 4, description: 'Number of text columns (1 for a single column).' },
      gapMm: { type: 'number', minimum: 0, maximum: 50, description: 'Gap between columns in millimetres (default 8).' },
    },
    required: ['count'],
    additionalProperties: false,
  },
  insert_picture: {
    type: 'object',
    properties: {
      position: { type: 'string', enum: ['end', 'after'], description: 'At the end of the document, or right after the body text `after`.' },
      after: { type: 'string', minLength: 1, maxLength: MAX_FIND_CHARS, description: 'With position "after": exact body text the picture follows.' },
      widthMm: { type: 'number', minimum: 5, maximum: 400, description: 'Picture width in millimetres (default: original size, at most the text width).' },
      purpose: { type: 'string', maxLength: 200, description: 'What picture the user should choose (shown to the user).' },
    },
    required: ['position'],
    additionalProperties: false,
  },
  insert_textbox: {
    type: 'object',
    properties: {
      after: { type: 'string', minLength: 1, maxLength: MAX_FIND_CHARS, description: 'Exact body text; the text box goes just below its paragraph.' },
      occurrence: { type: 'integer', minimum: 1, description: 'Which match, counting from 1, when after occurs more than once.' },
      text: { type: 'string', maxLength: MAX_REPLACE_CHARS, description: 'Text inside the box (one line).' },
      widthMm: { type: 'number', minimum: 5, maximum: 400, description: 'Width in millimetres (default 60).' },
      heightMm: { type: 'number', minimum: 5, maximum: 400, description: 'Height in millimetres (default 20).' },
    },
    required: ['after', 'text'],
    additionalProperties: false,
  },
  insert_shape: {
    type: 'object',
    properties: {
      after: { type: 'string', minLength: 1, maxLength: MAX_FIND_CHARS, description: 'Exact body text; the shape goes just below its paragraph.' },
      occurrence: { type: 'integer', minimum: 1, description: 'Which match, counting from 1, when after occurs more than once.' },
      shape: { type: 'string', enum: ['rectangle', 'ellipse', 'line'], description: 'Shape kind.' },
      widthMm: { type: 'number', minimum: 1, maximum: 400, description: 'Width in millimetres (default 30).' },
      heightMm: { type: 'number', minimum: 0, maximum: 400, description: 'Height in millimetres (default 15; 0 for a horizontal line).' },
    },
    required: ['after', 'shape'],
    additionalProperties: false,
  },
  new_page_number: {
    type: 'object',
    properties: {
      find: { type: 'string', minLength: 1, maxLength: MAX_FIND_CHARS, description: 'Exact body text on the page where numbering restarts.' },
      occurrence: { type: 'integer', minimum: 1, description: 'Which match, counting from 1, when find occurs more than once.' },
      start: { type: 'integer', minimum: 1, maximum: 9999, description: 'Page number to restart from.' },
    },
    required: ['find', 'start'],
    additionalProperties: false,
  },
  set_page_border: {
    type: 'object',
    properties: {
      style: { type: 'string', enum: ['solid', 'dashed', 'dotted', 'double', 'none'], description: 'Line style around every page.' },
      thicknessMm: { type: 'number', minimum: 0.1, maximum: 5, description: 'Line thickness in millimetres (default 0.12).' },
      color: { type: 'string', minLength: 7, maxLength: 7, description: 'Line color as #RRGGBB (default black).' },
    },
    required: ['style'],
    additionalProperties: false,
  },
  insert_field: {
    type: 'object',
    properties: {
      after: { type: 'string', minLength: 1, maxLength: MAX_FIND_CHARS, description: 'Exact body text; the field goes right after it.' },
      occurrence: { type: 'integer', minimum: 1, description: 'Which match, counting from 1, when after occurs more than once.' },
      name: { type: 'string', minLength: 1, maxLength: 100, description: 'Field name (used by fill_field and mail merge).' },
      guide: { type: 'string', maxLength: 200, description: 'Guide text shown in the empty field, for example 이름을 입력하세요.' },
    },
    required: ['after', 'name'],
    additionalProperties: false,
  },
  set_chart_data: {
    type: 'object',
    properties: {
      chart: { type: 'integer', minimum: 1, description: 'Chart number from list_charts (1-based).' },
      series: { type: 'integer', minimum: 1, description: 'Series number within the chart (1-based).' },
      values: {
        type: 'array', minItems: 1, maxItems: 200, items: { type: 'string', maxLength: 50 },
        description: 'New values of the series in category order, as numbers written as text (e.g. "12.5").',
      },
      labels: {
        type: 'array', minItems: 1, maxItems: 200, items: { type: 'string', maxLength: 100 },
        description: 'Optional new category labels; same count as values.',
      },
    },
    required: ['chart', 'series', 'values'],
    additionalProperties: false,
  },
  insert_table: {
    type: 'object',
    properties: {
      position: {
        type: 'string', enum: ['start', 'end', 'after'],
        description: 'Where to add the table: start or end of the document, or after the body paragraph containing `after`.',
      },
      after: {
        type: 'string', minLength: 1, maxLength: MAX_FIND_CHARS,
        description: 'With position "after": exact text from the body paragraph to add the table after.',
      },
      columns: { type: 'integer', minimum: 1, maximum: MAX_TABLE_COLUMNS, description: 'Number of columns.' },
      cells: {
        type: 'array', minItems: 1, maxItems: MAX_TABLE_CELLS,
        items: { type: 'string', maxLength: MAX_REPLACE_CHARS },
        description: 'Cell texts row by row, left to right (header row first). Its length must be a multiple of columns.',
      },
    },
    required: ['position', 'columns', 'cells'],
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
    name: 'format_text',
    description: 'Change character formatting (bold, italic, underline, strikethrough, size, color, highlight, font, '
      + 'width ratio, spacing) of '
      + 'text found by its exact content. Set only the properties to change. Shown to the user for approval.',
    inputSchema: WRITE_SCHEMAS.format_text as unknown as Record<string, unknown>,
    // 선택 매개변수가 많아 strict 예산(전체 24개)을 넘긴다 — 클라이언트 검증만 한다.
    strict: false,
  },
  {
    name: 'format_paragraph',
    description: 'Change paragraph formatting of the paragraph containing the given text: alignment, line spacing, '
      + 'left/right margins, first-line or hanging indent, space before/after. Shown to the user for approval.',
    inputSchema: WRITE_SCHEMAS.format_paragraph as unknown as Record<string, unknown>,
    // 선택 매개변수가 많아 strict 예산(전체 24개)을 넘긴다 — 클라이언트 검증만 한다.
    strict: false,
  },
  {
    name: 'insert_table',
    description: 'Add a new table with the given cell texts at the start or end of the document, or after a body '
      + 'paragraph named by its text. Shown to the user for approval.',
    inputSchema: WRITE_SCHEMAS.insert_table as unknown as Record<string, unknown>,
  },
  {
    name: 'delete_paragraph',
    description: 'Delete a whole body paragraph (named by text inside it). To delete only some words, use replace_text '
      + 'with an empty replace. Shown to the user for approval.',
    inputSchema: WRITE_SCHEMAS.delete_paragraph as unknown as Record<string, unknown>,
  },
  {
    name: 'set_list',
    description: 'Make the paragraph containing the text a bullet list item or numbered list item, or remove its list '
      + 'marker. Call once per paragraph. Shown to the user for approval.',
    inputSchema: WRITE_SCHEMAS.set_list as unknown as Record<string, unknown>,
  },
  {
    name: 'apply_style',
    description: 'Apply a paragraph style of the document (such as 개요 1 or 본문) to the body paragraph containing the '
      + 'text. An unknown name returns the available style names. Shown to the user for approval.',
    inputSchema: WRITE_SCHEMAS.apply_style as unknown as Record<string, unknown>,
  },
  {
    name: 'edit_table',
    description: 'Change table structure around the cell containing the text: insert or delete a row or column, or merge '
      + 'cells to the right or downward. New cells are empty; fill them with set_cell_text. Shown to the user for approval.',
    inputSchema: WRITE_SCHEMAS.edit_table as unknown as Record<string, unknown>,
  },
  {
    name: 'set_cell_text',
    description: 'Set the text of the cell at row/column (1-based) of the table that contains the given text. Works '
      + 'for empty cells too. Shown to the user for approval.',
    inputSchema: WRITE_SCHEMAS.set_cell_text as unknown as Record<string, unknown>,
  },
  {
    name: 'set_header_footer',
    description: 'Set the page header or footer text (one line, all pages of the first section). Shown to the user for approval.',
    inputSchema: WRITE_SCHEMAS.set_header_footer as unknown as Record<string, unknown>,
  },
  {
    name: 'insert_footnote',
    description: 'Add a footnote after the given body text, with the footnote text. Shown to the user for approval.',
    inputSchema: WRITE_SCHEMAS.insert_footnote as unknown as Record<string, unknown>,
  },
  {
    name: 'replace_all',
    description: 'Replace every occurrence of a text in the whole document at once (find & replace all). Shows the '
      + 'number of matches for approval. Use replace_text to change a single place.',
    inputSchema: WRITE_SCHEMAS.replace_all as unknown as Record<string, unknown>,
  },
  {
    name: 'format_cell',
    description: 'Set table cell background color or vertical alignment for the cell containing the text, or for its '
      + 'whole row, column or table. Shown to the user for approval.',
    inputSchema: WRITE_SCHEMAS.format_cell as unknown as Record<string, unknown>,
  },
  {
    name: 'page_setup',
    description: 'Change paper size, orientation or page margins for the whole document (all sections). This reflows '
      + 'the document and can change the page count. Shown to the user for approval.',
    inputSchema: WRITE_SCHEMAS.page_setup as unknown as Record<string, unknown>,
    // 선택 매개변수가 많아 strict 예산(전체 24개)을 넘긴다 — 클라이언트 검증만 한다.
    strict: false,
  },
  {
    name: 'set_table_border',
    description: 'Set table border lines (style, thickness, color) on all four sides of the cell containing the text, or of every cell in its row, column or table. Shown to the user for approval.',
    inputSchema: WRITE_SCHEMAS.set_table_border as unknown as Record<string, unknown>,
    // strict 예산(도구 20개·선택 매개변수 24개)을 넘기지 않도록 클라이언트 검증만 한다.
    strict: false,
  },
  {
    name: 'set_cell_size',
    description: 'Set the width of the column and/or the height of the row of the cell containing the text, in millimetres. Shown to the user for approval.',
    inputSchema: WRITE_SCHEMAS.set_cell_size as unknown as Record<string, unknown>,
    // strict 예산(도구 20개·선택 매개변수 24개)을 넘기지 않도록 클라이언트 검증만 한다.
    strict: false,
  },
  {
    name: 'insert_hyperlink',
    description: 'Turn text into a hyperlink to a web or e-mail address. Shown to the user for approval.',
    inputSchema: WRITE_SCHEMAS.insert_hyperlink as unknown as Record<string, unknown>,
    // strict 예산(도구 20개·선택 매개변수 24개)을 넘기지 않도록 클라이언트 검증만 한다.
    strict: false,
  },
  {
    name: 'add_bookmark',
    description: 'Add a named bookmark at the start of the given body text. Shown to the user for approval.',
    inputSchema: WRITE_SCHEMAS.add_bookmark as unknown as Record<string, unknown>,
    // strict 예산(도구 20개·선택 매개변수 24개)을 넘기지 않도록 클라이언트 검증만 한다.
    strict: false,
  },
  {
    name: 'insert_equation',
    description: 'Insert an equation (Hancom equation script) right after the given body text. Shown to the user for approval.',
    inputSchema: WRITE_SCHEMAS.insert_equation as unknown as Record<string, unknown>,
    // strict 예산(도구 20개·선택 매개변수 24개)을 넘기지 않도록 클라이언트 검증만 한다.
    strict: false,
  },
  {
    name: 'set_columns',
    description: 'Set the number of text columns (다단) for the whole document. This reflows the document. Shown to the user for approval.',
    inputSchema: WRITE_SCHEMAS.set_columns as unknown as Record<string, unknown>,
    // strict 예산(도구 20개·선택 매개변수 24개)을 넘기지 않도록 클라이언트 검증만 한다.
    strict: false,
  },
  {
    name: 'insert_picture',
    description: 'Insert a picture that the user chooses from their computer (you cannot create images). The approval card asks the user to pick the file; say what picture to choose in purpose.',
    inputSchema: WRITE_SCHEMAS.insert_picture as unknown as Record<string, unknown>,
    strict: false,
  },
  {
    name: 'insert_textbox',
    description: 'Insert a text box with one line of text just below the paragraph containing the given body text (text '
      + 'flows below the box). Shown to the user for approval.',
    inputSchema: WRITE_SCHEMAS.insert_textbox as unknown as Record<string, unknown>,
    strict: false,
  },
  {
    name: 'insert_shape',
    description: 'Insert a rectangle, ellipse or line just below the paragraph containing the given body text. Shown to '
      + 'the user for approval.',
    inputSchema: WRITE_SCHEMAS.insert_shape as unknown as Record<string, unknown>,
    strict: false,
  },
  {
    name: 'new_page_number',
    description: 'Restart page numbering from the given number on the page containing the text. Shown to the user for approval.',
    inputSchema: WRITE_SCHEMAS.new_page_number as unknown as Record<string, unknown>,
    strict: false,
  },
  {
    name: 'set_page_border',
    description: 'Draw (or remove) a border line around every page. Shown to the user for approval.',
    inputSchema: WRITE_SCHEMAS.set_page_border as unknown as Record<string, unknown>,
    strict: false,
  },
  {
    name: 'insert_field',
    description: 'Create a new form field (click-here field) with a name and guide text right after the given body text. Shown to the user for approval.',
    inputSchema: WRITE_SCHEMAS.insert_field as unknown as Record<string, unknown>,
    strict: false,
  },
  {
    name: 'set_chart_data',
    description: 'Change the values (and optionally the category labels) of one series of a chart listed by list_charts. Shown to the user for approval.',
    inputSchema: WRITE_SCHEMAS.set_chart_data as unknown as Record<string, unknown>,
    strict: false,
  },
  {
    name: 'create_document',
    description: 'Replace the open document with a new blank document, when the user asks for a new document. '
      + 'The user approves first and is asked to save unsaved changes. Then write content with insert_paragraphs '
      + '(position "end").',
    inputSchema: WRITE_SCHEMAS.create_document as unknown as Record<string, unknown>,
  },
];

const WRITE_TOOL_NAMES: readonly string[] = [
  'replace_text', 'fill_field', 'insert_paragraphs', 'format_text', 'format_paragraph', 'insert_table',
  'delete_paragraph', 'set_list', 'apply_style', 'edit_table', 'set_cell_text', 'set_header_footer', 'insert_footnote',
  'replace_all', 'format_cell', 'page_setup',
  'set_table_border', 'set_cell_size', 'insert_hyperlink', 'add_bookmark', 'insert_equation', 'set_columns',
  'insert_picture', 'insert_textbox', 'insert_shape', 'new_page_number', 'set_page_border', 'insert_field', 'set_chart_data',
  'create_document',
];

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

/**
 * @param exactRange `find` 가 곧 적용 범위인 도구(글자 모양 등). 더 긴 글로 고르라고 하면 범위가
 *   넓어지므로(실측: 「RISE」 대신 「RISE사업」을 굵게) occurrence 만 안내한다.
 */
function pickHit(hits: SearchHit[], find: string, occurrence: number | undefined, exactRange = false): SearchHit {
  if (hits.length === 0) {
    throw new WriteRefused(`"${find}" was not found. Read the document again and copy the exact text.`);
  }
  if (occurrence === undefined) {
    if (hits.length > 1) {
      throw new WriteRefused(exactRange
        ? `"${find}" occurs ${hits.length} times. Keep find exactly the text to change and pass occurrence (1-${hits.length}); `
          + 'search_text shows the matches in reading order.'
        : `"${find}" occurs ${hits.length} times. Use a longer find text or pass occurrence (1-${hits.length}).`);
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
    // 문단 끝은 논리 길이다 — 글자 길이로 나누면 개체가 든 문단의 마지막 글자가 잘려 넘어간다.
    ok(doc.splitParagraph(section, at, doc.getLogicalLength(section, at)), 'splitParagraph');
    at += 1;
    if (text) ok(doc.insertText(section, at, 0, text), 'insertText');
  }
}

/** 문서의 스타일을 이름(또는 영문 이름)으로 찾는다. 없으면 목록을 알려 주며 거절한다. */
function findStyle(doc: DocumentWriter, name: string): { id: number; name: string } {
  const styles = JSON.parse(doc.getStyleList()) as Array<{ id: number; name: string; englishName?: string }>;
  const wanted = name.trim();
  const style = styles.find((s) => s.name === wanted)
    ?? styles.find((s) => s.englishName?.toLowerCase() === wanted.toLowerCase());
  if (!style) throw new WriteRefused(`No style named "${wanted}". Available: ${styles.slice(0, 40).map((s) => s.name).join(', ')}`);
  return style;
}

function prepareInsert(host: WriteHost, input: Record<string, unknown>): WriteProposal {
  const position = input.position as 'start' | 'end' | 'after';
  const styleName = input.style as string | undefined;
  const after = input.after as string | undefined;
  const texts = input.paragraphs as string[];
  if (texts.some((text) => FORBIDDEN.test(text))) {
    throw new WriteRefused('Each paragraph must be one line: put each paragraph in its own array item.');
  }
  if (position === 'after' && !after) throw new WriteRefused('position "after" needs the after text.');
  if (position !== 'after' && after !== undefined) throw new WriteRefused('after is only used with position "after".');

  const search = (doc: DocumentWriter) => (JSON.parse(doc.searchAllText(after!, true, true)) as SearchHit[]);
  return host.read((doc) => {
    const style = styleName !== undefined ? findStyle(doc, styleName) : null;
    let anchor: SearchHit | null = null;
    let anchorCount = 0;
    if (position === 'after') {
      const hits = search(doc);
      anchorCount = hits.length;
      anchor = pickHit(hits, after!, undefined);
    }
    const lastSection = doc.getSectionCount() - 1;
    const location = position === 'after'
      ? { ...hitLocation(anchor!), position }
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
      ...(style ? { format: { style: style.name } } : {}),
      apply: () => host.transaction('AI: 문단 추가', (tx) => {
        const target = tx.doc();
        // 새 문단은 항상 paragraphs 개수만큼 연속으로 생긴다 — 시작 자리만 정하면 된다.
        const sectionOf = position === 'after' ? anchor!.sec : position === 'start' ? 0 : target.getSectionCount() - 1;
        const lastBefore = target.getParagraphCount(sectionOf) - 1;
        const firstNew = position === 'after' ? anchor!.para + 1
          : position === 'start' ? 0
            : bodyText(target, sectionOf, lastBefore).length === 0 ? lastBefore : lastBefore + 1;
        const restyle = () => {
          if (!style || (position === 'after' && anchor!.cellContext)) return;
          for (let i = 0; i < texts.length; i += 1) ok(target.applyStyle(sectionOf, firstNew + i, style.id), 'applyStyle');
        };
        if (position === 'after') {
          const again = search(target);
          if (again.length !== anchorCount || !sameHit(pickHit(again, after!, undefined), anchor!)) {
            throw new WriteRefused('The document changed after this was proposed. Read it again and propose a new change.');
          }
          const cell = anchor!.cellContext;
          if (cell) {
            // 표 칸 안: 그 칸 문단 뒤에 칸 문단을 만든다.
            let at = cell.cellPara;
            for (const text of texts) {
              const length = target.getCellParagraphLength(anchor!.sec, cell.parentPara, cell.ctrlIdx, cell.cellIdx, at);
              ok(target.splitParagraphInCell(anchor!.sec, cell.parentPara, cell.ctrlIdx, cell.cellIdx, at, length), 'splitParagraphInCell');
              at += 1;
              if (text) ok(target.insertTextInCell(anchor!.sec, cell.parentPara, cell.ctrlIdx, cell.cellIdx, at, 0, text), 'insertTextInCell');
            }
            return;
          }
          appendAfter(target, anchor!.sec, anchor!.para, texts);
          restyle();
          return;
        }
        if (position === 'start') {
          // 첫 문단을 맨 앞에서 나누면 빈 문단이 앞에 생긴다 — 거기에 첫 글을 넣고 나머지를 잇는다.
          ok(target.splitParagraph(0, 0, 0), 'splitParagraph');
          if (texts[0]) ok(target.insertText(0, 0, 0, texts[0]), 'insertText');
          appendAfter(target, 0, 0, texts.slice(1));
          restyle();
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
        restyle();
      }),
    };
  });
}

/** 찾을 글로 자리를 정하고, 적용 때 같은 자리인지 다시 확인하는 짝. */
function locate(doc: DocumentWriter, find: string, occurrence: number | undefined, exactRange = false) {
  const hits = JSON.parse(doc.searchAllText(find, true, true)) as SearchHit[];
  return { count: hits.length, hit: pickHit(hits, find, occurrence, exactRange) };
}

function relocate(doc: DocumentWriter, find: string, occurrence: number | undefined, expected: { count: number; hit: SearchHit }): void {
  const hits = JSON.parse(doc.searchAllText(find, true, true)) as SearchHit[];
  if (hits.length !== expected.count || !sameHit(pickHit(hits, find, occurrence), expected.hit)) {
    throw new WriteRefused('The document changed after this was proposed. Read it again and propose a new change.');
  }
}

function hitLocation(hit: SearchHit) {
  return { section: hit.sec, paragraph: hit.cellContext ? hit.cellContext.parentPara : hit.para, inTable: hit.cellContext !== undefined };
}

const FORMAT_KEYS = [
  'bold', 'italic', 'underline', 'strikethrough', 'fontSize', 'textColor', 'highlightColor', 'fontName', 'charRatio', 'charSpacing',
] as const;

function prepareFormatText(host: WriteHost, input: Record<string, unknown>): WriteProposal {
  const find = input.find as string;
  const occurrence = input.occurrence as number | undefined;
  const format: Record<string, string | number | boolean> = {};
  for (const key of FORMAT_KEYS) if (input[key] !== undefined) format[key] = input[key] as string | number | boolean;
  if (Object.keys(format).length === 0) throw new WriteRefused('Set at least one formatting property.');
  for (const key of ['textColor', 'highlightColor'] as const) {
    if (format[key] !== undefined && !COLOR.test(format[key] as string)) throw new WriteRefused(`${key} must be #RRGGBB.`);
  }
  return host.read((doc) => {
    const where = locate(doc, find, occurrence, true);
    const hit = where.hit;
    return {
      tool: 'format_text',
      location: hitLocation(hit),
      prefix: paragraphText(doc, hit, hit.charOffset - CONTEXT_CHARS, hit.charOffset),
      removed: '',
      inserted: find,
      suffix: paragraphText(doc, hit, hit.charOffset + hit.length, hit.charOffset + hit.length + CONTEXT_CHARS),
      format,
      apply: () => host.transaction('AI: 글자 모양', (tx) => {
        const target = tx.doc();
        relocate(target, find, occurrence, where);
        // studio 의 글자 모양 명령과 같은 속성 이름·단위(크기는 HWPUNIT, 1pt = 100)로 보낸다.
        const props: Record<string, unknown> = {};
        for (const key of ['bold', 'italic', 'underline', 'strikethrough', 'textColor'] as const) {
          if (format[key] !== undefined) props[key] = format[key];
        }
        if (format.fontSize !== undefined) props.fontSize = Math.round((format.fontSize as number) * 100);
        if (format.highlightColor !== undefined) props.shadeColor = format.highlightColor;
        // 글자 모양 대화상자와 같이 일곱 언어 칸에 같은 값을 준다.
        if (format.charRatio !== undefined) props.ratios = Array(7).fill(format.charRatio);
        if (format.charSpacing !== undefined) props.spacings = Array(7).fill(format.charSpacing);
        if (format.fontName !== undefined) {
          const fontId = target.findOrCreateFontId(format.fontName as string);
          if (fontId < 0) throw new WriteRefused(`Font "${format.fontName}" could not be used.`);
          props.fontId = fontId;
        }
        const json = JSON.stringify(props);
        const cell = hit.cellContext;
        ok(cell
          ? target.applyCharFormatInCell(hit.sec, cell.parentPara, cell.ctrlIdx, cell.cellIdx, cell.cellPara,
            hit.charOffset, hit.charOffset + hit.length, json)
          : target.applyCharFormat(hit.sec, hit.para, hit.charOffset, hit.charOffset + hit.length, json), 'applyCharFormat');
      }),
    };
  });
}

function prepareFormatParagraph(host: WriteHost, input: Record<string, unknown>): WriteProposal {
  const find = input.find as string;
  const occurrence = input.occurrence as number | undefined;
  const format: Record<string, string | number> = {};
  for (const key of ['alignment', 'lineSpacing', 'leftMargin', 'rightMargin', 'firstLineIndent', 'spaceBefore', 'spaceAfter'] as const) {
    if (input[key] !== undefined) format[key] = input[key] as string | number;
  }
  if (Object.keys(format).length === 0) throw new WriteRefused('Set at least one paragraph property.');
  return host.read((doc) => {
    const where = locate(doc, find, occurrence);
    const hit = where.hit;
    const cell = hit.cellContext;
    const full = paragraphText(doc, hit, 0, Number.MAX_SAFE_INTEGER);
    return {
      tool: 'format_paragraph',
      location: hitLocation(hit),
      prefix: '',
      removed: '',
      inserted: full.length > 80 ? `${full.slice(0, 80)}…` : full,
      suffix: '',
      format,
      apply: () => host.transaction('AI: 문단 모양', (tx) => {
        const target = tx.doc();
        relocate(target, find, occurrence, where);
        const props: Record<string, unknown> = {};
        if (format.alignment !== undefined) props.alignment = format.alignment;
        if (format.lineSpacing !== undefined) {
          props.lineSpacing = format.lineSpacing;
          props.lineSpacingType = 'Percent';
        }
        // 문단 모양 대화상자와 같은 단위: 여백·들여쓰기는 pt×200(HWPUNIT 2배 저장값), 문단 간격은 pt×100.
        // 들여쓰기는 양수, 내어쓰기는 음수다.
        if (format.leftMargin !== undefined) props.marginLeft = Math.round((format.leftMargin as number) * 200);
        if (format.rightMargin !== undefined) props.marginRight = Math.round((format.rightMargin as number) * 200);
        if (format.firstLineIndent !== undefined) props.indent = Math.round((format.firstLineIndent as number) * 200);
        if (format.spaceBefore !== undefined) props.spacingBefore = Math.round((format.spaceBefore as number) * 100);
        if (format.spaceAfter !== undefined) props.spacingAfter = Math.round((format.spaceAfter as number) * 100);
        const json = JSON.stringify(props);
        ok(cell
          ? target.applyParaFormatInCell(hit.sec, cell.parentPara, cell.ctrlIdx, cell.cellIdx, cell.cellPara, json)
          : target.applyParaFormat(hit.sec, hit.para, json), 'applyParaFormat');
      }),
    };
  });
}

/**
 * 새 빈 문단 하나를 만들어 그 자리를 돌려준다(표를 놓을 자리). 문서 끝 문단이 비어 있으면 그 문단을 쓴다.
 * 자리 규칙은 `insert_paragraphs` 와 같다.
 */
function emptyParagraphAt(doc: DocumentWriter, position: 'start' | 'end' | 'after', anchor: SearchHit | null) {
  if (position === 'after') {
    appendAfter(doc, anchor!.sec, anchor!.para, ['']);
    return { section: anchor!.sec, paragraph: anchor!.para + 1 };
  }
  if (position === 'start') {
    ok(doc.splitParagraph(0, 0, 0), 'splitParagraph');
    return { section: 0, paragraph: 0 };
  }
  const section = doc.getSectionCount() - 1;
  const last = doc.getParagraphCount(section) - 1;
  if (bodyText(doc, section, last).length === 0) return { section, paragraph: last };
  appendAfter(doc, section, last, ['']);
  return { section, paragraph: last + 1 };
}

function prepareTable(host: WriteHost, input: Record<string, unknown>): WriteProposal {
  const position = input.position as 'start' | 'end' | 'after';
  const after = input.after as string | undefined;
  const columns = input.columns as number;
  const cells = input.cells as string[];
  if (cells.length % columns !== 0) throw new WriteRefused(`cells has ${cells.length} items; it must be a multiple of columns (${columns}).`);
  if (cells.some((text) => FORBIDDEN.test(text))) throw new WriteRefused('Cell texts must not contain line breaks.');
  if (position === 'after' && !after) throw new WriteRefused('position "after" needs the after text.');
  if (position !== 'after' && after !== undefined) throw new WriteRefused('after is only used with position "after".');
  const rows = cells.length / columns;

  return host.read((doc) => {
    const where = position === 'after' ? locate(doc, after!, undefined) : null;
    if (where?.hit.cellContext) throw new WriteRefused('Adding a table inside a table cell is not supported yet.');
    const lastSection = doc.getSectionCount() - 1;
    const location = where
      ? { section: where.hit.sec, paragraph: where.hit.para, position }
      : position === 'start'
        ? { section: 0, paragraph: 0, position }
        : { section: lastSection, paragraph: doc.getParagraphCount(lastSection) - 1, position };
    const preview: string[] = [];
    for (let r = 0; r < rows; r += 1) preview.push(cells.slice(r * columns, (r + 1) * columns).join(' | '));
    return {
      tool: 'insert_table',
      location,
      prefix: '',
      removed: '',
      inserted: preview.join('\n'),
      suffix: '',
      format: { rows, columns },
      apply: () => host.transaction('AI: 표 만들기', (tx) => {
        const target = tx.doc();
        if (where) relocate(target, after!, undefined, where);
        const spot = emptyParagraphAt(target, position, where?.hit ?? null);
        // 앞 문단을 나눠 만든 자리라 개요·번호·글머리표 모양을 물려받을 수 있다. 그대로 두면 표 칸과
        // 표 뒤 문단에 번호가 붙는다(실측: 개요 1 뒤의 표 → 「4. 항목」). 그때만 바탕글로 되돌린다.
        const head = (JSON.parse(target.getParaPropertiesAt(spot.section, spot.paragraph)) as { headType?: string }).headType;
        if (head && head !== 'None') ok(target.applyStyle(spot.section, spot.paragraph, 0), 'applyStyle');
        const created = JSON.parse(target.createTable(spot.section, spot.paragraph, 0, rows, columns)) as {
          ok?: boolean; paraIdx?: number; controlIdx?: number;
        };
        if (!created.ok) throw new Error('createTable failed');
        const parentPara = created.paraIdx ?? spot.paragraph;
        const control = created.controlIdx ?? 0;
        // 새 표의 칸은 행 우선 순서로 번호가 붙는다(병합 없음).
        cells.forEach((text, index) => {
          if (text) ok(target.insertTextInCell(spot.section, parentPara, control, index, 0, 0, text), 'insertTextInCell');
        });
      }),
    };
  });
}

function requireBody(hit: SearchHit, what: string): void {
  if (hit.cellContext) throw new WriteRefused(`${what} inside a table cell is not supported; name a body paragraph.`);
}

function wholeParagraph(doc: DocumentWriter, hit: SearchHit): string {
  return paragraphText(doc, hit, 0, Number.MAX_SAFE_INTEGER);
}

function prepareDeleteParagraph(host: WriteHost, input: Record<string, unknown>): WriteProposal {
  const find = input.find as string;
  const occurrence = input.occurrence as number | undefined;
  return host.read((doc) => {
    const where = locate(doc, find, occurrence);
    const hit = where.hit;
    requireBody(hit, 'Deleting a paragraph');
    if (doc.getParagraphCount(hit.sec) <= 1) throw new WriteRefused('The section has only this paragraph; it cannot be deleted.');
    // 구역 첫 문단은 구역 설정·머리말·꼬리말을, 다른 문단은 표·각주 등을 품을 수 있다. 문단을 통째로
    // 지우면 그것들이 함께 사라진다(실측: 제목 문단을 지우자 머리말이 없어짐) — 그런 문단은 글만 지운다.
    const holdsObjects = (JSON.parse(doc.getControlTextPositions(hit.sec, hit.para)) as number[]).length > 0;
    return {
      tool: 'delete_paragraph',
      location: hitLocation(hit),
      prefix: '', removed: wholeParagraph(doc, hit), inserted: '', suffix: '',
      ...(holdsObjects ? {
        format: { keepsObjects: true },
        note: 'The paragraph holds objects such as section settings, a header/footer or a table, so only its text was '
          + 'deleted and an empty paragraph remains.',
      } : {}),
      apply: () => host.transaction('AI: 문단 지우기', (tx) => {
        const target = tx.doc();
        relocate(target, find, occurrence, where);
        if (holdsObjects) {
          const length = target.getParagraphLength(hit.sec, hit.para);
          if (length > 0) ok(target.deleteText(hit.sec, hit.para, 0, length), 'deleteText');
        } else {
          ok(target.deleteParagraph(hit.sec, hit.para), 'deleteParagraph');
        }
      }),
    };
  });
}

function prepareSetList(host: WriteHost, input: Record<string, unknown>): WriteProposal {
  const find = input.find as string;
  const occurrence = input.occurrence as number | undefined;
  const list = input.list as 'bullet' | 'number' | 'none';
  return host.read((doc) => {
    const where = locate(doc, find, occurrence);
    const hit = where.hit;
    return {
      tool: 'set_list',
      location: hitLocation(hit),
      prefix: '', removed: '', inserted: wholeParagraph(doc, hit), suffix: '',
      format: { list },
      apply: () => host.transaction('AI: 글머리표·번호', (tx) => {
        const target = tx.doc();
        relocate(target, find, occurrence, where);
        // studio 의 글머리표·문단 번호 명령과 같은 속성으로 보낸다.
        const props = list === 'none'
          ? { headType: 'None', numberingId: 0 }
          : list === 'bullet'
            ? { headType: 'Bullet', numberingId: target.ensureDefaultBullet('●'), paraLevel: 0 }
            : { headType: 'Number', numberingId: target.ensureDefaultNumbering(), paraLevel: 0 };
        const json = JSON.stringify(props);
        const cell = hit.cellContext;
        ok(cell
          ? target.applyParaFormatInCell(hit.sec, cell.parentPara, cell.ctrlIdx, cell.cellIdx, cell.cellPara, json)
          : target.applyParaFormat(hit.sec, hit.para, json), 'applyParaFormat');
      }),
    };
  });
}

function prepareApplyStyle(host: WriteHost, input: Record<string, unknown>): WriteProposal {
  const find = input.find as string;
  const occurrence = input.occurrence as number | undefined;
  const name = (input.style as string).trim();
  return host.read((doc) => {
    const styles = JSON.parse(doc.getStyleList()) as Array<{ id: number; name: string; englishName?: string }>;
    const style = styles.find((s) => s.name === name)
      ?? styles.find((s) => s.englishName?.toLowerCase() === name.toLowerCase());
    if (!style) {
      throw new WriteRefused(`No style named "${name}". Available: ${styles.slice(0, 40).map((s) => s.name).join(', ')}`);
    }
    const where = locate(doc, find, occurrence);
    const hit = where.hit;
    requireBody(hit, 'Applying a style');
    return {
      tool: 'apply_style',
      location: hitLocation(hit),
      prefix: '', removed: '', inserted: wholeParagraph(doc, hit), suffix: '',
      format: { style: style.name },
      apply: () => host.transaction('AI: 스타일', (tx) => {
        const target = tx.doc();
        relocate(target, find, occurrence, where);
        ok(target.applyStyle(hit.sec, hit.para, style.id), 'applyStyle');
      }),
    };
  });
}

type TableAction = 'insert_row_above' | 'insert_row_below' | 'insert_column_left' | 'insert_column_right'
  | 'delete_row' | 'delete_column' | 'merge_right' | 'merge_down';

/** 찾은 글이 든 칸과 그 표. 첫 단계 표만 다룬다(중첩 표 칸은 거절). */
function cellOf(doc: DocumentWriter, hit: SearchHit) {
  const cell = hit.cellContext;
  if (!cell) throw new WriteRefused('find must be text inside a table cell.');
  const info = JSON.parse(doc.getCellInfo(hit.sec, cell.parentPara, cell.ctrlIdx, cell.cellIdx)) as {
    row: number; col: number; rowSpan: number; colSpan: number;
  };
  const dims = JSON.parse(doc.getTableDimensions(hit.sec, cell.parentPara, cell.ctrlIdx)) as {
    rowCount: number; colCount: number; cellCount: number;
  };
  return { cell, info, dims };
}

function prepareEditTable(host: WriteHost, input: Record<string, unknown>): WriteProposal {
  const find = input.find as string;
  const occurrence = input.occurrence as number | undefined;
  const action = input.action as TableAction;
  const span = (input.span as number | undefined) ?? 2;
  if (span !== 2 && action !== 'merge_right' && action !== 'merge_down') throw new WriteRefused('span is only used with merge actions.');
  return host.read((doc) => {
    const where = locate(doc, find, occurrence);
    const hit = where.hit;
    const { cell, info, dims } = cellOf(doc, hit);
    if (action === 'delete_row' && dims.rowCount <= 1) throw new WriteRefused('The table has only one row.');
    if (action === 'delete_column' && dims.colCount <= 1) throw new WriteRefused('The table has only one column.');
    if (action === 'merge_right' && info.col + span > dims.colCount) throw new WriteRefused(`Only ${dims.colCount - info.col} cells to the right, including this one.`);
    if (action === 'merge_down' && info.row + span > dims.rowCount) throw new WriteRefused(`Only ${dims.rowCount - info.row} cells downward, including this one.`);
    return {
      tool: 'edit_table',
      location: hitLocation(hit),
      prefix: '', removed: '', inserted: wholeParagraph(doc, hit), suffix: '',
      format: { tableAction: action, row: info.row + 1, column: info.col + 1, ...(action.startsWith('merge') ? { span } : {}) },
      apply: () => host.transaction('AI: 표 고치기', (tx) => {
        const target = tx.doc();
        relocate(target, find, occurrence, where);
        const args = [hit.sec, cell.parentPara, cell.ctrlIdx] as const;
        switch (action) {
          case 'insert_row_above': ok(target.insertTableRow(...args, info.row, false), 'insertTableRow'); break;
          case 'insert_row_below': ok(target.insertTableRow(...args, info.row + info.rowSpan - 1, true), 'insertTableRow'); break;
          case 'insert_column_left': ok(target.insertTableColumn(...args, info.col, false), 'insertTableColumn'); break;
          case 'insert_column_right': ok(target.insertTableColumn(...args, info.col + info.colSpan - 1, true), 'insertTableColumn'); break;
          case 'delete_row': ok(target.deleteTableRow(...args, info.row), 'deleteTableRow'); break;
          case 'delete_column': ok(target.deleteTableColumn(...args, info.col), 'deleteTableColumn'); break;
          case 'merge_right': ok(target.mergeTableCells(...args, info.row, info.col, info.row, info.col + span - 1), 'mergeTableCells'); break;
          case 'merge_down': ok(target.mergeTableCells(...args, info.row, info.col, info.row + span - 1, info.col), 'mergeTableCells'); break;
        }
      }),
    };
  });
}

function prepareSetCellText(host: WriteHost, input: Record<string, unknown>): WriteProposal {
  const find = input.find as string;
  const occurrence = input.occurrence as number | undefined;
  const row = (input.row as number) - 1;
  const column = (input.column as number) - 1;
  const text = input.text as string;
  if (FORBIDDEN.test(text)) throw new WriteRefused('text must not contain line breaks.');
  return host.read((doc) => {
    const where = locate(doc, find, occurrence);
    const hit = where.hit;
    const { cell, dims } = cellOf(doc, hit);
    if (row >= dims.rowCount || column >= dims.colCount) {
      throw new WriteRefused(`The table has ${dims.rowCount} rows and ${dims.colCount} columns.`);
    }
    // 병합된 표도 다루도록 칸 번호를 행·열로 찾는다(병합 칸은 시작 위치로 지목한다).
    let cellIndex = -1;
    for (let index = 0; index < dims.cellCount; index += 1) {
      const info = JSON.parse(doc.getCellInfo(hit.sec, cell.parentPara, cell.ctrlIdx, index)) as { row: number; col: number };
      if (info.row === row && info.col === column) { cellIndex = index; break; }
    }
    if (cellIndex < 0) throw new WriteRefused(`Row ${row + 1}, column ${column + 1} is covered by a merged cell; name its top-left cell.`);
    const length = doc.getCellParagraphLength(hit.sec, cell.parentPara, cell.ctrlIdx, cellIndex, 0);
    const before = cleanDocumentText(length > 0 ? doc.getTextInCell(hit.sec, cell.parentPara, cell.ctrlIdx, cellIndex, 0, 0, length) : '');
    if (before === text) throw new WriteRefused('The cell already has this text.');
    return {
      tool: 'set_cell_text',
      location: { ...hitLocation(hit), inTable: true },
      prefix: '', removed: before, inserted: text, suffix: '',
      format: { row: row + 1, column: column + 1 },
      apply: () => host.transaction('AI: 칸 글', (tx) => {
        const target = tx.doc();
        relocate(target, find, occurrence, where);
        const now = target.getCellParagraphLength(hit.sec, cell.parentPara, cell.ctrlIdx, cellIndex, 0);
        if (now > 0) ok(target.deleteTextInCell(hit.sec, cell.parentPara, cell.ctrlIdx, cellIndex, 0, 0, now), 'deleteTextInCell');
        if (text) ok(target.insertTextInCell(hit.sec, cell.parentPara, cell.ctrlIdx, cellIndex, 0, 0, text), 'insertTextInCell');
      }),
    };
  });
}

/** 머리말·꼬리말은 첫 구역의 「양 쪽」(applyTo 0) 하나를 다룬다. */
const HEADER_FOOTER_APPLY_TO = 0;

function prepareHeaderFooter(host: WriteHost, input: Record<string, unknown>): WriteProposal {
  const isHeader = input.kind === 'header';
  const text = input.text as string;
  if (FORBIDDEN.test(text)) throw new WriteRefused('text must be one line.');
  return host.read((doc) => {
    const current = JSON.parse(doc.getHeaderFooter(0, isHeader, HEADER_FOOTER_APPLY_TO)) as { exists?: boolean; text?: string };
    const before = current.exists ? cleanDocumentText(current.text ?? '') : '';
    if (before === text) throw new WriteRefused(`The ${isHeader ? 'header' : 'footer'} already has this text.`);
    return {
      tool: 'set_header_footer',
      location: {},
      prefix: '', removed: before, inserted: text, suffix: '',
      format: { kind: isHeader ? 'header' : 'footer' },
      apply: () => host.transaction(isHeader ? 'AI: 머리말' : 'AI: 꼬리말', (tx) => {
        const target = tx.doc();
        const now = JSON.parse(target.getHeaderFooter(0, isHeader, HEADER_FOOTER_APPLY_TO)) as { exists?: boolean; text?: string };
        if (!now.exists) {
          ok(target.createHeaderFooter(0, isHeader, HEADER_FOOTER_APPLY_TO), 'createHeaderFooter');
        } else if ((now.text ?? '').length > 0) {
          // 첫 문단만 바꾼다. 여러 문단인 머리말은 나머지를 그대로 둔다.
          const first = (now.text ?? '').split(/\r?\n/)[0];
          if (first.length > 0) {
            ok(target.deleteTextInHeaderFooter(0, isHeader, HEADER_FOOTER_APPLY_TO, 0, 0, first.length), 'deleteTextInHeaderFooter');
          }
        }
        if (text) ok(target.insertTextInHeaderFooter(0, isHeader, HEADER_FOOTER_APPLY_TO, 0, 0, text), 'insertTextInHeaderFooter');
      }),
    };
  });
}

function prepareFootnote(host: WriteHost, input: Record<string, unknown>): WriteProposal {
  const find = input.find as string;
  const occurrence = input.occurrence as number | undefined;
  const text = input.text as string;
  if (FORBIDDEN.test(text)) throw new WriteRefused('text must be one line.');
  return host.read((doc) => {
    const where = locate(doc, find, occurrence);
    const hit = where.hit;
    requireBody(hit, 'Adding a footnote');
    return {
      tool: 'insert_footnote',
      location: hitLocation(hit),
      prefix: paragraphText(doc, hit, hit.charOffset - CONTEXT_CHARS, hit.charOffset + hit.length),
      removed: '',
      inserted: text,
      suffix: '',
      apply: () => host.transaction('AI: 각주', (tx) => {
        const target = tx.doc();
        relocate(target, find, occurrence, where);
        const created = JSON.parse(target.insertFootnote(hit.sec, hit.para, hit.charOffset + hit.length)) as {
          ok?: boolean; paraIdx?: number; controlIdx?: number;
        };
        if (!created.ok || created.controlIdx === undefined) throw new Error('insertFootnote failed');
        ok(target.insertTextInFootnote(hit.sec, created.paraIdx ?? hit.para, created.controlIdx, 0, 0, text), 'insertTextInFootnote');
      }),
    };
  });
}

function prepareReplaceAll(host: WriteHost, input: Record<string, unknown>): WriteProposal {
  const find = input.find as string;
  const replace = input.replace as string;
  const caseSensitive = input.caseSensitive !== false;
  if (FORBIDDEN.test(find) || FORBIDDEN.test(replace)) throw new WriteRefused('find and replace must be one line.');
  if (find === replace) throw new WriteRefused('replace is the same as find; nothing to change.');
  return host.read((doc) => {
    const count = (JSON.parse(doc.searchAllText(find, caseSensitive, true)) as unknown[]).length;
    if (count === 0) throw new WriteRefused(`"${find}" was not found.`);
    return {
      tool: 'replace_all',
      location: {},
      prefix: '', removed: find, inserted: replace, suffix: '',
      format: { count },
      apply: () => host.transaction('AI: 모두 바꾸기', (tx) => {
        const target = tx.doc();
        const now = (JSON.parse(target.searchAllText(find, caseSensitive, true)) as unknown[]).length;
        if (now !== count) {
          throw new WriteRefused(`The number of matches changed (${count} → ${now}). Propose the change again.`);
        }
        ok(target.replaceAll(find, replace, caseSensitive), 'replaceAll');
      }),
    };
  });
}

const VALIGN = { top: 0, middle: 1, bottom: 2 } as const;

function prepareFormatCell(host: WriteHost, input: Record<string, unknown>): WriteProposal {
  const find = input.find as string;
  const occurrence = input.occurrence as number | undefined;
  const scope = (input.scope as 'cell' | 'row' | 'column' | 'table' | undefined) ?? 'cell';
  const background = input.backgroundColor as string | undefined;
  const valign = input.verticalAlign as keyof typeof VALIGN | undefined;
  if (background === undefined && valign === undefined) throw new WriteRefused('Set backgroundColor or verticalAlign.');
  if (background !== undefined && background !== 'none' && !COLOR.test(background)) {
    throw new WriteRefused('backgroundColor must be #RRGGBB or "none".');
  }
  return host.read((doc) => {
    const where = locate(doc, find, occurrence);
    const hit = where.hit;
    const { cell, info, dims } = cellOf(doc, hit);
    const targets: number[] = [];
    for (let index = 0; index < dims.cellCount; index += 1) {
      const other = JSON.parse(doc.getCellInfo(hit.sec, cell.parentPara, cell.ctrlIdx, index)) as { row: number; col: number };
      if (scope === 'table' || (scope === 'cell' && index === cell.cellIdx)
        || (scope === 'row' && other.row === info.row) || (scope === 'column' && other.col === info.col)) {
        targets.push(index);
      }
    }
    const format: Record<string, string | number> = { scope, cells: targets.length };
    if (background !== undefined) format.backgroundColor = background;
    if (valign !== undefined) format.verticalAlign = valign;
    return {
      tool: 'format_cell',
      location: { ...hitLocation(hit), inTable: true },
      prefix: '', removed: '', inserted: wholeParagraph(doc, hit), suffix: '',
      format,
      apply: () => host.transaction('AI: 칸 속성', (tx) => {
        const target = tx.doc();
        relocate(target, find, occurrence, where);
        // 표/칸 속성 대화상자와 같은 속성 이름(배경은 fillType·fillColor, 세로 정렬 0 위 1 가운데 2 아래).
        const props: Record<string, unknown> = {};
        if (background !== undefined) {
          if (background === 'none') props.fillType = 'none';
          else { props.fillType = 'solid'; props.fillColor = background.toLowerCase(); }
        }
        if (valign !== undefined) props.verticalAlign = VALIGN[valign];
        const json = JSON.stringify(props);
        for (const index of targets) ok(target.setCellProperties(hit.sec, cell.parentPara, cell.ctrlIdx, index, json), 'setCellProperties');
      }),
    };
  });
}

/** 용지 크기(세로 기준, mm). HWPUNIT = mm × 7200 / 25.4. */
const PAPER_MM: Record<string, [number, number]> = {
  A4: [210, 297], A3: [297, 420], B4: [257, 364], B5: [182, 257], Letter: [215.9, 279.4], Legal: [215.9, 355.6],
};
const mmToHwp = (mm: number) => Math.round((mm * 7200) / 25.4);
/** 여백을 빼고 남는 본문이 이보다 좁으면 거절한다(대화상자는 본문 소멸만 막는다 — AI 는 여유를 둔다). */
const MIN_BODY_MM = 30;

function preparePageSetup(host: WriteHost, input: Record<string, unknown>): WriteProposal {
  const keys = ['paper', 'orientation', 'marginTop', 'marginBottom', 'marginLeft', 'marginRight'] as const;
  const format: Record<string, string | number> = {};
  for (const key of keys) if (input[key] !== undefined) format[key] = input[key] as string | number;
  if (Object.keys(format).length === 0) throw new WriteRefused('Set paper, orientation or a margin.');

  type PageDef = {
    width: number; height: number; landscape: boolean;
    marginTop: number; marginBottom: number; marginLeft: number; marginRight: number;
    marginHeader: number; marginFooter: number; marginGutter: number;
  };
  const next = (current: PageDef): PageDef => {
    const def = { ...current };
    if (format.paper !== undefined) {
      const [w, h] = PAPER_MM[format.paper as string];
      // 쪽 정의는 가로 방향이어도 세로 기준 크기를 저장한다(쪽 설정 대화상자와 같음).
      def.width = mmToHwp(w);
      def.height = mmToHwp(h);
    }
    if (format.orientation !== undefined) def.landscape = format.orientation === 'landscape';
    for (const key of ['marginTop', 'marginBottom', 'marginLeft', 'marginRight'] as const) {
      if (format[key] !== undefined) def[key] = mmToHwp(format[key] as number);
    }
    const [pw, ph] = def.landscape ? [def.height, def.width] : [def.width, def.height];
    const bodyW = pw - def.marginLeft - def.marginRight - def.marginGutter;
    const bodyH = ph - def.marginTop - def.marginBottom - def.marginHeader - def.marginFooter;
    if (bodyW < mmToHwp(MIN_BODY_MM) || bodyH < mmToHwp(MIN_BODY_MM)) {
      throw new WriteRefused(`The margins leave less than ${MIN_BODY_MM} mm for the body text. Use smaller margins.`);
    }
    return def;
  };

  return host.read((doc) => {
    const sections = doc.getSectionCount();
    for (let section = 0; section < sections; section += 1) next(JSON.parse(doc.getPageDef(section)) as PageDef);
    return {
      tool: 'page_setup',
      location: {},
      prefix: '', removed: '', inserted: '', suffix: '',
      format,
      apply: () => host.transaction('AI: 쪽 설정', (tx) => {
        const target = tx.doc();
        for (let section = 0; section < target.getSectionCount(); section += 1) {
          const def = next(JSON.parse(target.getPageDef(section)) as PageDef);
          ok(target.setPageDef(section, JSON.stringify(def)), 'setPageDef');
        }
      }),
    };
  });
}

/** 테두리 선 종류(셀 테두리/배경 대화상자 번호) */
const LINE_TYPE = { none: 0, solid: 1, dashed: 2, dotted: 3, double: 8 } as const;
/** 테두리 굵기 번호 → mm (같은 대화상자의 표) */
const LINE_WIDTH_MM = [0.1, 0.12, 0.15, 0.2, 0.25, 0.3, 0.4, 0.5, 0.6, 0.7, 1.0, 1.5, 2.0, 3.0, 4.0, 5.0];

function nearestWidthIndex(mm: number): number {
  let best = 0;
  LINE_WIDTH_MM.forEach((value, index) => { if (Math.abs(value - mm) < Math.abs(LINE_WIDTH_MM[best] - mm)) best = index; });
  return best;
}

/** 기준 칸과 범위에 드는 칸 번호들. */
function cellsInScope(doc: DocumentWriter, hit: SearchHit, scope: 'cell' | 'row' | 'column' | 'table') {
  const { cell, info, dims } = cellOf(doc, hit);
  const targets: number[] = [];
  for (let index = 0; index < dims.cellCount; index += 1) {
    const other = JSON.parse(doc.getCellInfo(hit.sec, cell.parentPara, cell.ctrlIdx, index)) as { row: number; col: number };
    if (scope === 'table' || (scope === 'cell' && index === cell.cellIdx)
      || (scope === 'row' && other.row === info.row) || (scope === 'column' && other.col === info.col)) {
      targets.push(index);
    }
  }
  return { cell, info, dims, targets };
}

function prepareTableBorder(host: WriteHost, input: Record<string, unknown>): WriteProposal {
  const find = input.find as string;
  const occurrence = input.occurrence as number | undefined;
  const scope = (input.scope as 'cell' | 'row' | 'column' | 'table' | undefined) ?? 'cell';
  const style = input.style as keyof typeof LINE_TYPE;
  const thickness = (input.thicknessMm as number | undefined) ?? 0.12;
  const color = (input.color as string | undefined) ?? '#000000';
  if (!COLOR.test(color)) throw new WriteRefused('color must be #RRGGBB.');
  return host.read((doc) => {
    const where = locate(doc, find, occurrence);
    const hit = where.hit;
    const { cell, targets } = cellsInScope(doc, hit, scope);
    const widthIndex = nearestWidthIndex(thickness);
    return {
      tool: 'set_table_border',
      location: { ...hitLocation(hit), inTable: true },
      prefix: '', removed: '', inserted: wholeParagraph(doc, hit), suffix: '',
      format: { scope, cells: targets.length, borderStyle: style, ...(style === 'none' ? {} : { thicknessMm: LINE_WIDTH_MM[widthIndex], color }) },
      apply: () => host.transaction('AI: 표 테두리', (tx) => {
        const target = tx.doc();
        relocate(target, find, occurrence, where);
        const line = { type: LINE_TYPE[style], width: widthIndex, color: color.toLowerCase() };
        const json = JSON.stringify({ borderLeft: line, borderRight: line, borderTop: line, borderBottom: line });
        for (const index of targets) ok(target.setCellProperties(hit.sec, cell.parentPara, cell.ctrlIdx, index, json), 'setCellProperties');
      }),
    };
  });
}

function prepareCellSize(host: WriteHost, input: Record<string, unknown>): WriteProposal {
  const find = input.find as string;
  const occurrence = input.occurrence as number | undefined;
  const widthMm = input.columnWidthMm as number | undefined;
  const heightMm = input.rowHeightMm as number | undefined;
  if (widthMm === undefined && heightMm === undefined) throw new WriteRefused('Set columnWidthMm or rowHeightMm.');
  return host.read((doc) => {
    const where = locate(doc, find, occurrence);
    const hit = where.hit;
    // 칸 하나의 폭만 바꾸면 표 격자가 어긋난다(실측: 칸 0 만 10000, 나머지 20977) — 같은 열·줄의
    // 한 칸짜리 칸 모두에 같은 값을 준다. 여러 열에 걸친 병합 칸은 건드리지 않는다.
    const { cell, info, dims } = cellOf(doc, hit);
    const columnCells: number[] = [];
    const rowCells: number[] = [];
    for (let index = 0; index < dims.cellCount; index += 1) {
      const other = JSON.parse(doc.getCellInfo(hit.sec, cell.parentPara, cell.ctrlIdx, index)) as {
        row: number; col: number; rowSpan: number; colSpan: number;
      };
      if (other.col === info.col && other.colSpan === 1) columnCells.push(index);
      if (other.row === info.row && other.rowSpan === 1) rowCells.push(index);
    }
    const format: Record<string, number> = {};
    if (widthMm !== undefined) format.columnWidthMm = widthMm;
    if (heightMm !== undefined) format.rowHeightMm = heightMm;
    return {
      tool: 'set_cell_size',
      location: { ...hitLocation(hit), inTable: true },
      prefix: '', removed: '', inserted: wholeParagraph(doc, hit), suffix: '',
      format,
      apply: () => host.transaction('AI: 칸 크기', (tx) => {
        const target = tx.doc();
        relocate(target, find, occurrence, where);
        if (widthMm !== undefined) {
          const json = JSON.stringify({ width: mmToHwp(widthMm) });
          for (const index of columnCells) ok(target.setCellProperties(hit.sec, cell.parentPara, cell.ctrlIdx, index, json), 'setCellProperties');
        }
        if (heightMm !== undefined) {
          const json = JSON.stringify({ height: mmToHwp(heightMm) });
          for (const index of rowCells) ok(target.setCellProperties(hit.sec, cell.parentPara, cell.ctrlIdx, index, json), 'setCellProperties');
        }
      }),
    };
  });
}

function prepareHyperlink(host: WriteHost, input: Record<string, unknown>): WriteProposal {
  const find = input.find as string;
  const occurrence = input.occurrence as number | undefined;
  const url = (input.url as string).trim();
  if (!/^(https?:\/\/[^\s]+|mailto:[^\s@]+@[^\s@]+)$/i.test(url)) {
    throw new WriteRefused('url must be an http(s):// address or a mailto: e-mail address without spaces.');
  }
  return host.read((doc) => {
    const where = locate(doc, find, occurrence, true);
    const hit = where.hit;
    return {
      tool: 'insert_hyperlink',
      location: hitLocation(hit),
      prefix: paragraphText(doc, hit, hit.charOffset - CONTEXT_CHARS, hit.charOffset),
      removed: '', inserted: find,
      suffix: paragraphText(doc, hit, hit.charOffset + hit.length, hit.charOffset + hit.length + CONTEXT_CHARS),
      format: { url },
      apply: () => host.transaction('AI: 하이퍼링크', (tx) => {
        const target = tx.doc();
        relocate(target, find, occurrence, where);
        const cell = hit.cellContext;
        // 하이퍼링크 대화상자와 같은 대상 형식: 본문이면 빈 cellPath, 칸이면 [표, 칸, 칸 문단] 한 단.
        const linkTarget = cell
          ? { section: hit.sec, para: cell.parentPara, cellPath: [[cell.ctrlIdx, cell.cellIdx, cell.cellPara]] }
          : { section: hit.sec, para: hit.para, cellPath: [] };
        const fieldId = target.insertHyperlinkEx(JSON.stringify({
          target: linkTarget, start: hit.charOffset, end: hit.charOffset + hit.length, uri: url,
        }));
        if (!(fieldId > 0)) throw new Error('insertHyperlinkEx failed');
      }),
    };
  });
}

function prepareBookmark(host: WriteHost, input: Record<string, unknown>): WriteProposal {
  const find = input.find as string;
  const occurrence = input.occurrence as number | undefined;
  const name = (input.name as string).trim();
  if (FORBIDDEN.test(name) || name.length === 0) throw new WriteRefused('name must be one line.');
  return host.read((doc) => {
    const existing = JSON.parse(doc.getBookmarks()) as Array<{ name: string }>;
    if (existing.some((bookmark) => bookmark.name === name)) throw new WriteRefused(`A bookmark named "${name}" already exists.`);
    const where = locate(doc, find, occurrence);
    const hit = where.hit;
    requireBody(hit, 'Adding a bookmark');
    return {
      tool: 'add_bookmark',
      location: hitLocation(hit),
      prefix: '', removed: '', inserted: find,
      suffix: paragraphText(doc, hit, hit.charOffset + hit.length, hit.charOffset + hit.length + CONTEXT_CHARS),
      format: { name },
      apply: () => host.transaction('AI: 책갈피', (tx) => {
        const target = tx.doc();
        relocate(target, find, occurrence, where);
        ok(target.addBookmark(hit.sec, hit.para, hit.charOffset, name), 'addBookmark');
      }),
    };
  });
}

function prepareEquation(host: WriteHost, input: Record<string, unknown>): WriteProposal {
  const after = input.after as string;
  const occurrence = input.occurrence as number | undefined;
  const script = input.script as string;
  const fontSize = (input.fontSize as number | undefined) ?? 10;
  if (FORBIDDEN.test(script)) throw new WriteRefused('script must be one line; use the equation script\'s own line commands (#) instead.');
  return host.read((doc) => {
    const where = locate(doc, after, occurrence);
    const hit = where.hit;
    requireBody(hit, 'Inserting an equation');
    return {
      tool: 'insert_equation',
      location: hitLocation(hit),
      prefix: paragraphText(doc, hit, hit.charOffset - CONTEXT_CHARS, hit.charOffset + hit.length),
      removed: '', inserted: script, suffix: '',
      format: { fontSize },
      apply: () => host.transaction('AI: 수식', (tx) => {
        const target = tx.doc();
        relocate(target, after, occurrence, where);
        // 수식 넣기 명령과 같은 인자(글자 크기 HWPUNIT, 색 0 = 검정).
        ok(target.insertEquation(hit.sec, hit.para, hit.charOffset + hit.length, script, Math.round(fontSize * 100), 0), 'insertEquation');
      }),
    };
  });
}

function prepareColumns(host: WriteHost, input: Record<string, unknown>): WriteProposal {
  const count = input.count as number;
  const gapMm = (input.gapMm as number | undefined) ?? 8;
  return {
    tool: 'set_columns',
    location: {},
    prefix: '', removed: '', inserted: '', suffix: '',
    format: { columnCount: count, gapMm },
    apply: () => host.transaction('AI: 단 설정', (tx) => {
      const target = tx.doc();
      // 단 설정 명령과 같은 인자: 보통 단(0), 너비 같게(1), 간격 HWPUNIT. 모든 구역에 준다.
      for (let section = 0; section < target.getSectionCount(); section += 1) {
        ok(target.setColumnDef(section, count, 0, 1, mmToHwp(gapMm)), 'setColumnDef');
      }
    }),
  };
}

/** 본문 폭(HWPUNIT) — 그림을 이 폭 안에 맞춘다. */
function bodyWidth(doc: DocumentWriter, section: number): number {
  const def = JSON.parse(doc.getPageDef(section)) as {
    width: number; height: number; landscape: boolean; marginLeft: number; marginRight: number; marginGutter: number;
  };
  const pageWidth = def.landscape ? def.height : def.width;
  return Math.max(mmToHwp(10), pageWidth - def.marginLeft - def.marginRight - def.marginGutter);
}

/** 화면 1px = 75 HWPUNIT(96dpi) — studio 의 그림 끌어놓기와 같다. */
const PX_TO_HWP = 75;

function preparePicture(host: WriteHost, input: Record<string, unknown>): WriteProposal {
  const position = input.position as 'end' | 'after';
  const after = input.after as string | undefined;
  const widthMm = input.widthMm as number | undefined;
  const purpose = (input.purpose as string | undefined) ?? '';
  if (position === 'after' && !after) throw new WriteRefused('position "after" needs the after text.');
  let picked: PickedFile | null = null;
  return host.read((doc) => {
    const where = position === 'after' ? locate(doc, after!, undefined) : null;
    if (where) requireBody(where.hit, 'Inserting a picture');
    return {
      tool: 'insert_picture',
      location: where ? hitLocation(where.hit) : { position: 'end' as const },
      prefix: '', removed: '', inserted: purpose, suffix: '',
      ...(widthMm !== undefined ? { format: { widthMm } } : {}),
      file: {
        accept: 'image/png,image/jpeg,image/gif,image/bmp',
        provide: (file) => { picked = file; },
        provided: () => picked !== null,
      },
      apply: () => host.transaction('AI: 그림', (tx) => {
        if (!picked) throw new WriteRefused('The user did not choose a picture file.');
        const target = tx.doc();
        if (where) relocate(target, after!, undefined, where);
        const spot = where
          ? { section: where.hit.sec, paragraph: where.hit.para, offset: where.hit.charOffset + where.hit.length }
          : { ...emptyParagraphAt(target, 'end', null), offset: 0 };
        const naturalW = picked.naturalWidth * PX_TO_HWP;
        const naturalH = picked.naturalHeight * PX_TO_HWP;
        const wanted = widthMm !== undefined ? mmToHwp(widthMm) : naturalW;
        const width = Math.max(1, Math.min(wanted, bodyWidth(target, spot.section)));
        const height = Math.max(1, Math.round((naturalH * width) / Math.max(1, naturalW)));
        const description = `그림입니다.\r\n원본 그림의 이름: ${picked.name}\r\n원본 그림의 크기: 가로 ${picked.naturalWidth}pixel, 세로 ${picked.naturalHeight}pixel`;
        const result = JSON.parse(target.insertPicture(spot.section, spot.paragraph, spot.offset, '', picked.bytes,
          width, height, picked.naturalWidth, picked.naturalHeight, picked.extension, description)) as {
          ok?: boolean; paraIdx?: number; controlIdx?: number; error?: string;
        };
        if (!result.ok || result.controlIdx === undefined) throw new WriteRefused(`The picture could not be inserted${result.error ? `: ${result.error}` : ''}.`);
        // 끌어놓기와 같이 글자처럼 취급해 본문 흐름에 둔다.
        ok(target.setPictureProperties(spot.section, result.paraIdx ?? spot.paragraph, result.controlIdx, JSON.stringify({ treatAsChar: true })), 'setPictureProperties');
      }),
    };
  });
}

/** 본문 문단의 마지막 줄 아래 끝(쪽 위에서 px)과 그 쪽. 조판 결과의 글 조각에서 읽는다. */
function paragraphBottom(doc: DocumentWriter, section: number, paragraph: number): { page: number; bottomPx: number } {
  const pages = (JSON.parse(doc.getDocumentInfo()) as { pageCount?: number }).pageCount ?? 1;
  let found: { page: number; bottomPx: number } | null = null;
  for (let page = 0; page < pages; page += 1) {
    const runs = (JSON.parse(doc.getPageTextLayout(page)) as {
      runs?: Array<{ y: number; h: number; secIdx?: number; paraIdx?: number; cellPath?: unknown[] }>;
    }).runs ?? [];
    for (const run of runs) {
      if (run.secIdx === section && run.paraIdx === paragraph && !(run.cellPath && run.cellPath.length > 0)) {
        const bottom = run.y + run.h;
        // 여러 쪽에 걸친 문단은 마지막 쪽의 마지막 줄을 쓴다.
        if (!found || page > found.page || (page === found.page && bottom > found.bottomPx)) found = { page, bottomPx: bottom };
      }
    }
  }
  if (!found) throw new WriteRefused('Could not find where that paragraph is laid out.');
  return found;
}

function prepareShape(host: WriteHost, input: Record<string, unknown>, kind: 'textbox' | 'rectangle' | 'ellipse' | 'line'): WriteProposal {
  const after = input.after as string;
  const occurrence = input.occurrence as number | undefined;
  const text = kind === 'textbox' ? (input.text as string) : '';
  if (FORBIDDEN.test(text)) throw new WriteRefused('text must be one line.');
  const widthMm = (input.widthMm as number | undefined) ?? (kind === 'textbox' ? 60 : 30);
  const heightMm = (input.heightMm as number | undefined) ?? (kind === 'textbox' ? 20 : kind === 'line' ? 0 : 15);
  return host.read((doc) => {
    const where = locate(doc, after, occurrence);
    const hit = where.hit;
    requireBody(hit, 'Inserting a shape');
    return {
      tool: kind === 'textbox' ? 'insert_textbox' : 'insert_shape',
      location: hitLocation(hit),
      prefix: paragraphText(doc, hit, hit.charOffset - CONTEXT_CHARS, hit.charOffset + hit.length),
      removed: '', inserted: text, suffix: '',
      format: { shape: kind, widthMm, heightMm },
      apply: () => host.transaction(kind === 'textbox' ? 'AI: 글상자' : 'AI: 도형', (tx) => {
        const target = tx.doc();
        relocate(target, after, occurrence, where);
        // 글자처럼 취급한 글상자는 엔진이 줄 밖(위쪽)에 그린다(실측: 줄 y132 인데 상자 y77) — studio 의 글상자
        // 그리기처럼 떠 있는 개체로, 앵커 문단 바로 아래에 「위아래」 배치로 둔다. 그러면 뒤 글이 상자 아래로 밀려
        // 겹치지 않는다(실측: 문단 끝 y146 → 상자 y149, 다음 문단 y233). 위치는 그 쪽의 실제 조판 결과에서 읽는다.
        // 같은 문단에 「위아래」 개체를 두면 엔진이 그 문단의 글자처럼 취급한 그림을 개체 아래로 내려 그린다
        // (실측: 줄 y132, 그림 y215). 그래서 앵커 문단 뒤에 빈 문단을 만들어 그 문단에 붙인다.
        appendAfter(target, hit.sec, hit.para, ['']);
        const spot = paragraphBottom(target, hit.sec, hit.para);
        const def = JSON.parse(target.getPageDef(hit.sec)) as { marginLeft: number; marginGutter: number };
        const created = JSON.parse(target.createShapeControl(JSON.stringify({
          sectionIdx: hit.sec, paraIdx: hit.para + 1, charOffset: 0,
          width: mmToHwp(widthMm), height: mmToHwp(heightMm),
          horzOffset: def.marginLeft + def.marginGutter,
          vertOffset: Math.round(spot.bottomPx * PX_TO_HWP) + mmToHwp(1),
          treatAsChar: false, textWrap: 'TopAndBottom', shapeType: kind,
        }))) as { ok?: boolean; paraIdx?: number; controlIdx?: number };
        if (!created.ok || created.controlIdx === undefined) throw new Error('createShapeControl failed');
        // 글상자의 글은 칸과 같은 경로(칸 번호 0)로 넣는다.
        if (text) ok(target.insertTextInCell(hit.sec, created.paraIdx ?? hit.para + 1, created.controlIdx, 0, 0, 0, text), 'insertTextInCell');
      }),
    };
  });
}

function prepareNewPageNumber(host: WriteHost, input: Record<string, unknown>): WriteProposal {
  const find = input.find as string;
  const occurrence = input.occurrence as number | undefined;
  const start = input.start as number;
  return host.read((doc) => {
    const where = locate(doc, find, occurrence);
    const hit = where.hit;
    requireBody(hit, 'Restarting page numbers');
    return {
      tool: 'new_page_number',
      location: hitLocation(hit),
      prefix: '', removed: '', inserted: wholeParagraph(doc, hit), suffix: '',
      format: { start },
      apply: () => host.transaction('AI: 새 쪽 번호', (tx) => {
        const target = tx.doc();
        relocate(target, find, occurrence, where);
        ok(target.insertNewNumber(hit.sec, hit.para, hit.charOffset, start), 'insertNewNumber');
      }),
    };
  });
}

function preparePageBorder(host: WriteHost, input: Record<string, unknown>): WriteProposal {
  const style = input.style as keyof typeof LINE_TYPE;
  const thickness = (input.thicknessMm as number | undefined) ?? 0.12;
  const color = (input.color as string | undefined) ?? '#000000';
  if (!COLOR.test(color)) throw new WriteRefused('color must be #RRGGBB.');
  const widthIndex = nearestWidthIndex(thickness);
  return {
    tool: 'set_page_border',
    location: {},
    prefix: '', removed: '', inserted: '', suffix: '',
    format: { borderStyle: style, ...(style === 'none' ? {} : { thicknessMm: LINE_WIDTH_MM[widthIndex], color }) },
    apply: () => host.transaction('AI: 쪽 테두리', (tx) => {
      const target = tx.doc();
      const line = { type: LINE_TYPE[style], width: widthIndex, color: color.toLowerCase() };
      // 쪽 테두리 대화상자와 같이 현재 설정을 읽어 네 변만 바꾼다. 모든 구역에 준다.
      for (let section = 0; section < target.getSectionCount(); section += 1) {
        const current = JSON.parse(target.getPageBorderFill(section)) as Record<string, unknown>;
        ok(target.setPageBorderFill(section, JSON.stringify({
          ...current, borderLeft: line, borderRight: line, borderTop: line, borderBottom: line,
        })), 'setPageBorderFill');
      }
    }),
  };
}

function prepareField(host: WriteHost, input: Record<string, unknown>): WriteProposal {
  const after = input.after as string;
  const occurrence = input.occurrence as number | undefined;
  const name = (input.name as string).trim();
  const guide = (input.guide as string | undefined) ?? '';
  if (FORBIDDEN.test(name) || FORBIDDEN.test(guide)) throw new WriteRefused('name and guide must be one line.');
  return host.read((doc) => {
    const where = locate(doc, after, occurrence);
    const hit = where.hit;
    requireBody(hit, 'Creating a field');
    return {
      tool: 'insert_field',
      location: hitLocation(hit),
      prefix: paragraphText(doc, hit, hit.charOffset - CONTEXT_CHARS, hit.charOffset + hit.length),
      removed: '', inserted: guide || name, suffix: '',
      format: { name },
      apply: () => host.transaction('AI: 누름틀', (tx) => {
        const target = tx.doc();
        relocate(target, after, occurrence, where);
        ok(target.insertClickHereField(hit.sec, hit.para, hit.charOffset + hit.length, guide, '', name, true), 'insertClickHereField');
      }),
    };
  });
}

/** 문서의 차트를 순번으로 읽는다(차트 데이터 대화상자와 같은 순번). */
export function listCharts(doc: Pick<DocumentWriter, 'getChartDataByIndex'>) {
  const charts: Array<{ chart: number; plot?: string; labels?: string[]; series: Array<{ name?: string | null; values: string[] }> }> = [];
  for (let index = 0; index < 50; index += 1) {
    let data: { ok?: boolean; plot?: string; labels?: string[]; series?: Array<{ name?: string | null; values: string[] }> };
    try { data = JSON.parse(doc.getChartDataByIndex(index)); } catch { break; }
    if (!data.ok) break;
    charts.push({ chart: index + 1, plot: data.plot, labels: data.labels, series: data.series ?? [] });
  }
  return charts;
}

function prepareChartData(host: WriteHost, input: Record<string, unknown>): WriteProposal {
  const chartNo = input.chart as number;
  const seriesNo = input.series as number;
  const values = input.values as string[];
  const labels = input.labels as string[] | undefined;
  if (values.some((value) => !/^-?\d+(\.\d+)?$/.test(value.trim()))) throw new WriteRefused('values must be numbers written as text, like "12.5".');
  return host.read((doc) => {
    const charts = listCharts(doc);
    const chart = charts[chartNo - 1];
    if (!chart) throw new WriteRefused(`There are ${charts.length} charts; call list_charts.`);
    const series = chart.series[seriesNo - 1];
    if (!series) throw new WriteRefused(`Chart ${chartNo} has ${chart.series.length} series.`);
    if (labels && labels.length !== values.length) throw new WriteRefused('labels must have the same count as values.');
    if (!labels && chart.labels && chart.labels.length !== values.length) {
      throw new WriteRefused(`The chart has ${chart.labels.length} categories; give that many values or new labels.`);
    }
    const edits = {
      series: chart.series.map((s, i) => ({ values: i === seriesNo - 1 ? values.map((v) => v.trim()) : [...s.values] })),
      ...(labels ? { labels } : {}),
    };
    return {
      tool: 'set_chart_data',
      location: {},
      prefix: '', removed: series.values.join(', '), inserted: values.join(', '), suffix: '',
      format: { chart: chartNo, series: series.name ?? String(seriesNo) },
      apply: () => host.transaction('AI: 차트 데이터', (tx) => {
        const result = JSON.parse(tx.doc().setChartDataByIndex(chartNo - 1, JSON.stringify(edits))) as { ok?: boolean };
        if (!result.ok) throw new Error('setChartDataByIndex failed');
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
    case 'format_text': return prepareFormatText(host, checked.value);
    case 'format_paragraph': return prepareFormatParagraph(host, checked.value);
    case 'insert_table': return prepareTable(host, checked.value);
    case 'delete_paragraph': return prepareDeleteParagraph(host, checked.value);
    case 'set_list': return prepareSetList(host, checked.value);
    case 'apply_style': return prepareApplyStyle(host, checked.value);
    case 'edit_table': return prepareEditTable(host, checked.value);
    case 'set_cell_text': return prepareSetCellText(host, checked.value);
    case 'set_header_footer': return prepareHeaderFooter(host, checked.value);
    case 'insert_footnote': return prepareFootnote(host, checked.value);
    case 'replace_all': return prepareReplaceAll(host, checked.value);
    case 'format_cell': return prepareFormatCell(host, checked.value);
    case 'page_setup': return preparePageSetup(host, checked.value);
    case 'set_table_border': return prepareTableBorder(host, checked.value);
    case 'set_cell_size': return prepareCellSize(host, checked.value);
    case 'insert_hyperlink': return prepareHyperlink(host, checked.value);
    case 'add_bookmark': return prepareBookmark(host, checked.value);
    case 'insert_equation': return prepareEquation(host, checked.value);
    case 'set_columns': return prepareColumns(host, checked.value);
    case 'insert_picture': return preparePicture(host, checked.value);
    case 'insert_textbox': return prepareShape(host, checked.value, 'textbox');
    case 'insert_shape': return prepareShape(host, checked.value, checked.value.shape as 'rectangle' | 'ellipse' | 'line');
    case 'new_page_number': return prepareNewPageNumber(host, checked.value);
    case 'set_page_border': return preparePageBorder(host, checked.value);
    case 'insert_field': return prepareField(host, checked.value);
    case 'set_chart_data': return prepareChartData(host, checked.value);
  }
}
