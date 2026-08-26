import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const rootDir = dirname(dirname(fileURLToPath(import.meta.url)));
const source = (path: string): string => readFileSync(join(rootDir, path), 'utf8');

function slice(s: string, from: string, to: string): string {
  const start = s.indexOf(from);
  assert.notEqual(start, -1, `${from} not found`);
  const end = s.indexOf(to, start + from.length);
  return end === -1 ? s.slice(start) : s.slice(start, end);
}

test('비텍스트 포커스의 활성 편집기는 undo/redo를 dispatcher로 보완한다', () => {
  const main = source('src/main.ts');
  const setup = slice(main, 'function setupGlobalShortcuts(): void {', '\nfunction setupFileInput');

  assert.match(setup, /if \(target instanceof HTMLInputElement \|\| target instanceof HTMLTextAreaElement\) return/,
    'textarea가 받는 단축키는 InputHandler가 계속 소유');
  assert.match(setup, /matchShortcut\(e, defaultShortcuts\)/, '기존 shortcut map으로 판정');
  assert.match(setup, /commandId === 'edit:undo' \|\| commandId === 'edit:redo'/,
    'fallback 범위를 undo/redo로 제한');
  assert.match(setup, /dispatcher\.dispatchWithResult\(commandId\)/, '동일 dispatcher 경로로 실행');
});

test('개체 선택 C/X/Delete는 canonical edit command를 사용하고 native paste는 보존한다', () => {
  const keyboard = source('src/engine/input-handler-keyboard.ts');
  const picture = slice(keyboard, "if (this.cursor.isInPictureObjectSelection()) {", '\n  // ─── 표 객체 선택 모드');
  const table = slice(keyboard, "if (this.cursor.isInTableObjectSelection()) {", '\n  // ─── 본문 블록 선택 모드');

  for (const branch of [picture, table]) {
    assert.match(branch, /this\.dispatcher\?\.dispatch\('edit:delete'\)/, 'Delete/Backspace는 dispatcher');
    assert.match(branch, /this\.dispatcher\?\.dispatch\('edit:copy'\)/, 'Ctrl+C는 dispatcher');
    assert.match(branch, /this\.dispatcher\?\.dispatch\('edit:cut'\)/, 'Ctrl+X는 dispatcher');
    assert.doesNotMatch(branch, /deleteSelectedObject|copyControl\(|operationType: 'cut|operationType: 'delete/,
      'keyboard branch에 개체 mutation 구현을 두지 않음');
  }

  const picturePaste = slice(picture, "if ((e.ctrlKey || e.metaKey) && e.key === 'v') {", '\n    // 방향키');
  const tablePaste = slice(table, "if ((e.ctrlKey || e.metaKey) && e.key === 'v') {", '\n    // 방향키');
  assert.doesNotMatch(picturePaste, /preventDefault\(\)/, '그림 Ctrl+V는 native paste event 허용');
  assert.doesNotMatch(tablePaste, /preventDefault\(\)/, '표 Ctrl+V는 native paste event 허용');
});

test('canonical 표 cut/delete가 keyboard의 중첩 표 안전 계약을 유지한다', () => {
  const inputHandler = source('src/engine/input-handler.ts');
  const cut = slice(inputHandler, 'performCut(): void {', '\n  /** 선택 영역 삭제');
  const del = slice(inputHandler, 'performDelete(): void {', '\n  /** 전체 선택');

  const tableCut = slice(cut, "if (this.cursor.isInTableObjectSelection()) {", '\n    // 텍스트 선택');
  const guard = tableCut.indexOf('if (ref.cellPath && ref.cellPath.length > 1) return;');
  const copy = tableCut.indexOf('this.performCopy();');
  assert.ok(guard !== -1 && guard < copy, '중첩 표는 복사·삭제 전에 차단');

  const nestedDelete = slice(del, 'if (ref.cellPath && ref.cellPath.length > 1) {', '\n      }');
  assert.match(nestedDelete, /this\.updateCaret\(\)/, '중첩 표 delete 뒤 caret를 갱신');
});
