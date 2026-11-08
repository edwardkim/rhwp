import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

import {
  AI_ALLOWED_COMMANDS,
  COMMAND_TOOL_DEFS,
  listAllowedCommands,
  prepareCommand,
  selectText,
  type CommandHost,
} from '../src/ai/tools/command-tools.ts';
import { WriteRefused } from '../src/ai/tools/write-tools.ts';

function fakeHost() {
  let selection: { text: string } | null = null;
  const executed: string[] = [];
  const selected: unknown[] = [];
  const host: CommandHost = {
    read: (fn) => fn({
      searchAllText: (q) => JSON.stringify(q === '없음' ? [] : q === '두 번'
        ? [{ sec: 0, para: 1, charOffset: 0, length: 3 }, { sec: 0, para: 2, charOffset: 0, length: 3 }]
        : [{ sec: 0, para: 4, charOffset: 2, length: q.length, cellContext: { parentPara: 4, ctrlIdx: 0, cellIdx: 3, cellPara: 0 } }]),
    }),
    selectTextRange: (range) => { selected.push(range); selection = { text: 'x²' }; return true; },
    getSelectionText: () => (selection ? { ...selection, truncated: false } as never : null),
    automation: {
      listCommands: () => [
        { id: 'format:superscript', label: '위 첨자', enabled: true },
        { id: 'table:insert-row-below', label: '아래에 줄 추가', enabled: false },
        { id: 'page:break', label: '쪽 나누기', enabled: true },
        { id: 'file:save', label: '저장', enabled: true },
      ],
      isEnabled: (id) => id !== 'table:insert-row-below',
      execute: (id) => { executed.push(id); return { ok: true }; },
    },
  };
  return { host, executed, selected, select: (text: string) => { selection = { text }; } };
}

const call = (name: string, input: unknown) => ({ id: 't', name, input });
const refused = (pattern: RegExp) => (error: unknown) => error instanceof WriteRefused && pattern.test(error.message);

test('명령 통로 도구 셋: 목록·선택은 strict 아님, 실행은 strict', () => {
  assert.deepEqual(COMMAND_TOOL_DEFS.map((t) => [t.name, t.strict !== false]),
    [['list_commands', false], ['select_text', false], ['run_command', true]]);
});

test('허용 목록에는 파일·보기·클립보드·대화상자 명령이 없다', () => {
  assert.ok(AI_ALLOWED_COMMANDS.every((id) => !/^(file|view|tool):/.test(id)));
  for (const id of ['edit:cut', 'edit:copy', 'edit:paste', 'edit:undo', 'edit:redo', 'insert:image', 'insert:shape', 'format:char-shape']) {
    assert.ok(!AI_ALLOWED_COMMANDS.includes(id), id);
  }
  // 대화상자를 여는 커맨드(opensDialog: true)가 허용 목록에 섞이지 않았는지 소스로 확인한다.
  const root = join(dirname(fileURLToPath(import.meta.url)), '..', 'src', 'command', 'commands');
  const dialogIds = new Set<string>();
  for (const file of ['format.ts', 'insert.ts', 'page.ts', 'table.ts', 'edit.ts']) {
    const source = readFileSync(join(root, file), 'utf8');
    for (const m of source.matchAll(/\n {4}id: '([a-z-]+:[a-z0-9-]+)',([\s\S]*?)\n {2}\},/g)) {
      if (m[2].includes('opensDialog: true')) dialogIds.add(m[1]);
    }
  }
  assert.ok(dialogIds.size > 10);
  assert.deepEqual(AI_ALLOWED_COMMANDS.filter((id) => dialogIds.has(id)), []);
});

test('목록은 허용 목록 중 레지스트리에 있는 것만, 지금 실행 가능 여부와 함께', () => {
  const { host } = fakeHost();
  const listed = listAllowedCommands(host);
  assert.deepEqual(listed.commands, [
    { id: 'format:superscript', label: '위 첨자', enabled: true },
    { id: 'page:break', label: '쪽 나누기', enabled: true },
    { id: 'table:insert-row-below', label: '아래에 줄 추가', enabled: false },
  ], '허용 목록 순서, 파일 명령 제외');
  assert.equal(listed.selection, null);
});

test('선택: 찾은 자리(표 칸 포함)를 편집기에서 선택하고, 여러 곳·없음은 거절', () => {
  const { host, selected } = fakeHost();
  assert.deepEqual(selectText(host, call('select_text', { find: 'x2' })), { selected: 'x2', cursor: 'select', inTable: true });
  assert.deepEqual(selected[0], {
    section: 0, paragraph: 4, charOffset: 2, length: 2,
    cell: { parentPara: 4, controlIndex: 0, cellIndex: 3, cellPara: 0 },
  });
  assert.throws(() => selectText(host, call('select_text', { find: '없음' })), refused(/not found/));
  assert.throws(() => selectText(host, call('select_text', { find: '두 번' })), refused(/occurs 2 times\. Keep find exactly/));
  selectText(host, call('select_text', { find: '두 번', occurrence: 2 }));
  assert.deepEqual((selected[1] as { paragraph: number }).paragraph, 2);
});

test('실행: 허용 목록·실행 가능 여부를 보고, 승인 시점에 선택이 같을 때만 실행한다', () => {
  const t = fakeHost();
  t.select('x2');
  assert.throws(() => prepareCommand(t.host, call('run_command', { command: 'file:save' })), refused(/not available/));
  assert.throws(() => prepareCommand(t.host, call('run_command', { command: 'table:insert-row-below' })), refused(/cannot run/));
  const proposal = prepareCommand(t.host, call('run_command', { command: 'format:superscript' }));
  assert.deepEqual([proposal.tool, proposal.inserted, proposal.format], ['run_command', 'x2', { command: '위 첨자' }]);
  t.select('다른 곳');
  assert.throws(() => proposal.apply(), refused(/selection changed/));
  assert.deepEqual(t.executed, []);
  t.select('x2');
  proposal.apply();
  assert.deepEqual(t.executed, ['format:superscript']);
});

test('끼워 넣는 명령은 선택이 있으면 거절하고, place before·after 는 길이 0 커서를 둔다', () => {
  const t = fakeHost();
  t.select('안건');
  assert.throws(() => prepareCommand(t.host, call('run_command', { command: 'page:break' })), refused(/would replace the selected text/));
  assert.deepEqual(selectText(t.host, call('select_text', { find: 'x2', place: 'after' })), { selected: null, cursor: 'after', inTable: true });
  assert.deepEqual([(t.selected[0] as { charOffset: number }).charOffset, (t.selected[0] as { length: number }).length], [4, 0]);
  selectText(t.host, call('select_text', { find: 'x2', place: 'before' }));
  assert.deepEqual([(t.selected[1] as { charOffset: number }).charOffset, (t.selected[1] as { length: number }).length], [2, 0]);
  t.select('');
  prepareCommand(t.host, call('run_command', { command: 'page:break' })).apply();
  assert.deepEqual(t.executed, ['page:break']);
});
