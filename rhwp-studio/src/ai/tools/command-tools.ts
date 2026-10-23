/**
 * studio 명령 통로 — 전용 도구가 없는 편집 기능을 메뉴·단축키와 같은 커맨드로 실행한다.
 *
 * 흐름: `select_text` 로 대상 글을 편집기에서 선택 → `run_command` 로 커맨드 실행(사용자 승인).
 * 커맨드는 메뉴·툴바와 같은 `CommandDispatcher` 경로를 지나므로 undo·양식 모드 차단 같은 기존
 * 게이트가 그대로 적용된다.
 *
 * **허용 목록으로만 연다.** 대화상자를 여는 커맨드, 그리기처럼 화면 조작을 기다리는 커맨드,
 * 파일·보기·클립보드 커맨드, 클릭으로 고른 개체에 쓰는 커맨드는 넣지 않는다 — AI 가 결과를
 * 끝까지 만들 수 없거나 문서 밖의 일을 하기 때문이다.
 */
import type { PluginSelectionText, PluginTextRange } from '@/plugin/types';
import type { AiToolCall, AiToolDef } from '../provider/types.ts';
import { validateToolInput, type ToolSchema } from './schema-validate.ts';
import { cleanDocumentText } from './untrusted.ts';
import { WriteRefused, type WriteProposal } from './write-tools.ts';

/** AI 가 실행할 수 있는 커맨드. 모두 현재 선택 또는 캐럿 자리에 작용하고 대화상자가 없다. */
export const AI_ALLOWED_COMMANDS: readonly string[] = [
  // 글자 모양
  'format:bold', 'format:italic', 'format:underline', 'format:strikethrough',
  'format:emboss', 'format:engrave', 'format:outline', 'format:superscript', 'format:subscript',
  'format:font-size-increase', 'format:font-size-decrease',
  'format:char-ratio-increase', 'format:char-ratio-decrease',
  'format:char-spacing-increase', 'format:char-spacing-decrease',
  // 문단 모양
  'format:align-left', 'format:align-center', 'format:align-right', 'format:align-justify',
  'format:align-distribute', 'format:align-split',
  'format:line-spacing-increase', 'format:line-spacing-decrease',
  'format:toggle-numbering', 'format:toggle-bullet', 'format:level-increase', 'format:level-decrease',
  // 쪽·나누기·필드
  'page:break', 'page:column-break',
  'page:insert-field-pagenum', 'page:insert-field-totalpage', 'page:insert-field-filename',
  // 주석
  'insert:endnote',
  // 표(캐럿이 칸 안에 있을 때)
  'table:insert-row-above', 'table:insert-row-below', 'table:insert-col-left', 'table:insert-col-right',
  'table:delete-row', 'table:delete-col', 'table:delete',
  // 선택 영역 지우기
  'edit:delete',
];

/**
 * 커서 자리에 무언가를 끼워 넣는 커맨드. 선택 영역이 있으면 글자 입력처럼 **선택한 글을 지우고**
 * 들어간다(실측: 「안건」을 선택한 채 쪽 나누기 → 「안건」이 사라짐). 그래서 선택이 있으면 실행하지 않는다.
 */
const INSERTS_AT_CURSOR: readonly string[] = [
  'page:break', 'page:column-break',
  'page:insert-field-pagenum', 'page:insert-field-totalpage', 'page:insert-field-filename',
  'insert:endnote',
];

/** 명령 통로가 쓰는 호스트 표면 — `PluginHost` 의 부분집합. */
export interface CommandHost {
  read<T>(fn: (doc: { searchAllText(query: string, caseSensitive: boolean, includeCells: boolean): string }) => T): T;
  selectTextRange(range: PluginTextRange): boolean;
  getSelectionText(): PluginSelectionText | null;
  automation: {
    listCommands(): Array<{ id: string; label: string; enabled: boolean }>;
    isEnabled(id: string): boolean;
    execute(id: string): { ok: true } | { ok: false; reason: string; message?: string };
  };
}

const SELECT_SCHEMA: ToolSchema = {
  type: 'object',
  properties: {
    find: {
      type: 'string', minLength: 1, maxLength: 500,
      description: 'Exactly the text to select (the command acts on exactly this). If it occurs more than once, pass '
        + 'occurrence instead of adding neighbouring words.',
    },
    occurrence: { type: 'integer', minimum: 1, description: 'Which match, counting from 1, when find occurs more than once.' },
    place: {
      type: 'string', enum: ['select', 'before', 'after'],
      description: 'select (default) selects the text; before / after only puts the cursor just before or after it '
        + '(use this for commands that insert something, such as page break).',
    },
  },
  required: ['find'],
  additionalProperties: false,
};

const RUN_SCHEMA: ToolSchema = {
  type: 'object',
  properties: {
    command: { type: 'string', minLength: 1, maxLength: 100, description: 'Command id from list_commands, for example format:superscript.' },
  },
  required: ['command'],
  additionalProperties: false,
};

const LIST_SCHEMA: ToolSchema = { type: 'object', properties: {}, required: [], additionalProperties: false };

/** 읽기·선택 도구(문서를 바꾸지 않음)와 실행 도구(승인 필요). */
export const COMMAND_TOOL_DEFS: readonly AiToolDef[] = [
  {
    name: 'list_commands',
    description: 'List editor commands you can run with run_command (id, label, enabled now). Use them for editing '
      + 'features that have no dedicated tool, such as superscript/subscript, emboss, character spacing, page break, '
      + 'column break, page number fields, endnotes, or table rows at the cursor.',
    inputSchema: LIST_SCHEMA as unknown as Record<string, unknown>,
    strict: false,
  },
  {
    name: 'select_text',
    description: 'Select text in the editor, or put the cursor just before or after it, so that run_command acts there. '
      + 'Does not change the document.',
    inputSchema: SELECT_SCHEMA as unknown as Record<string, unknown>,
    strict: false,
  },
  {
    name: 'run_command',
    description: 'Run one editor command on the current selection or cursor position, as if from the menu. Select the '
      + 'target with select_text first. Shown to the user for approval.',
    inputSchema: RUN_SCHEMA as unknown as Record<string, unknown>,
  },
];

export function isCommandTool(name: string): boolean {
  return name === 'list_commands' || name === 'select_text' || name === 'run_command';
}

interface SearchHit {
  sec: number;
  para: number;
  charOffset: number;
  length: number;
  cellContext?: { parentPara: number; ctrlIdx: number; cellIdx: number; cellPara: number };
}

/** `list_commands` — 허용 목록과 지금 실행 가능 여부. */
export function listAllowedCommands(host: CommandHost) {
  const registry = new Map(host.automation.listCommands().map((command) => [command.id, command]));
  return {
    commands: AI_ALLOWED_COMMANDS
      .filter((id) => registry.has(id))
      .map((id) => ({ id, label: registry.get(id)!.label, enabled: registry.get(id)!.enabled })),
    selection: describeSelection(host.getSelectionText()),
  };
}

function describeSelection(selection: PluginSelectionText | null) {
  if (!selection) return null;
  const text = cleanDocumentText(selection.text);
  return { text: text.length > 200 ? `${text.slice(0, 200)}…` : text };
}

/** `select_text` — 찾은 글을 편집기에서 선택한다. 거절 사유는 `WriteRefused` 로 던진다. */
export function selectText(host: CommandHost, call: AiToolCall) {
  const checked = validateToolInput(SELECT_SCHEMA, call.input);
  if (!checked.ok) throw new WriteRefused(`Invalid input: ${checked.error}`);
  const find = checked.value.find as string;
  const occurrence = checked.value.occurrence as number | undefined;
  const place = (checked.value.place as 'select' | 'before' | 'after' | undefined) ?? 'select';
  const hits = host.read((doc) => JSON.parse(doc.searchAllText(find, true, true)) as SearchHit[]);
  if (hits.length === 0) throw new WriteRefused(`"${find}" was not found.`);
  if (occurrence === undefined && hits.length > 1) {
    // 선택 범위가 곧 명령의 대상이다 — 더 긴 글로 고르면 대상이 넓어지므로 occurrence 를 안내한다.
    throw new WriteRefused(`"${find}" occurs ${hits.length} times. Keep find exactly the text to act on and pass occurrence (1-${hits.length}).`);
  }
  if (occurrence !== undefined && occurrence > hits.length) throw new WriteRefused(`occurrence must be between 1 and ${hits.length}.`);
  const hit = hits[(occurrence ?? 1) - 1];
  const cell = hit.cellContext;
  const selected = host.selectTextRange({
    section: hit.sec,
    paragraph: cell ? cell.parentPara : hit.para,
    // before·after 는 길이 0 — 선택 없이 커서만 둔다.
    charOffset: place === 'after' ? hit.charOffset + hit.length : hit.charOffset,
    length: place === 'select' ? hit.length : 0,
    ...(cell ? { cell: { parentPara: cell.parentPara, controlIndex: cell.ctrlIdx, cellIndex: cell.cellIdx, cellPara: cell.cellPara } } : {}),
  });
  if (!selected) throw new WriteRefused('Selecting text is not available here.');
  return { selected: place === 'select' ? find : null, cursor: place, inTable: cell !== undefined };
}

/**
 * `run_command` 제안. 승인 시점에 선택이 제안 때와 같은지 다시 본다 — 사용자가 그 사이 다른 곳을
 * 선택했으면 엉뚱한 곳에 적용하지 않는다.
 */
export function prepareCommand(host: CommandHost, call: AiToolCall): WriteProposal {
  const checked = validateToolInput(RUN_SCHEMA, call.input);
  if (!checked.ok) throw new WriteRefused(`Invalid input: ${checked.error}`);
  const id = checked.value.command as string;
  if (!AI_ALLOWED_COMMANDS.includes(id)) {
    throw new WriteRefused(`Command "${id}" is not available to the assistant. Call list_commands for the allowed ids.`);
  }
  const command = host.automation.listCommands().find((entry) => entry.id === id);
  if (!command) throw new WriteRefused(`Command "${id}" does not exist in this editor.`);
  if (!host.automation.isEnabled(id)) {
    throw new WriteRefused(`Command "${id}" cannot run at the current cursor position (for example, table commands need the cursor in a table). Use select_text first.`);
  }
  const before = describeSelection(host.getSelectionText())?.text ?? '';
  if (INSERTS_AT_CURSOR.includes(id) && before.length > 0) {
    throw new WriteRefused(`"${id}" inserts at the cursor and would replace the selected text. `
      + 'Call select_text with place "before" or "after" first.');
  }
  return {
    tool: 'run_command',
    location: {},
    prefix: '',
    removed: '',
    inserted: before,
    suffix: '',
    format: { command: command.label },
    apply: () => {
      const now = describeSelection(host.getSelectionText())?.text ?? '';
      if (now !== before) {
        throw new WriteRefused('The selection changed after this was proposed. Select the text again and propose the command again.');
      }
      const result = host.automation.execute(id);
      if (!result.ok) {
        throw new WriteRefused(`The command did not run (${result.reason}${result.message ? `: ${result.message}` : ''}).`);
      }
    },
  };
}
