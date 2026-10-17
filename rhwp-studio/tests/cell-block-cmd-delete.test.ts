import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { stripTypeScriptTypes } from 'node:module';
import { createServer } from 'vite';
import { codeOnly, functionBodyFrom } from './support/source-guard.ts';

// 셀 블록 지우기 키 정합 (한컴 Mac 단축키표: 셀 내용 지우기 = ⌃E/Delete, 지우기 = ⌘E).
//
// macOS 에서 한컴의 "Delete" 는 delete 키로 브라우저에는 Backspace 로 보고된다 —
// 셀 블록의 내용 지우기는 Backspace 와 forward Delete 양쪽을 받아야 한다.
// ⌘⌫/⌘Delete 는 한컴 표에는 없지만 macOS "항목 지우기" 관례로 edit:delete 에 태운다.

const rootDir = dirname(dirname(fileURLToPath(import.meta.url)));
const source = (p: string) => readFileSync(join(rootDir, p), 'utf8');

/** 셀 선택 모드 분기 본문만 잘라낸다 (issue-6741 가드와 동일 경계). */
function cellSelectionBranch(kb: string): string {
  const at = kb.indexOf('if (this.cursor.isInCellSelectionMode()) {');
  assert.notEqual(at, -1, '셀 선택 모드 분기를 찾지 못했다');
  const end = kb.indexOf('handleNavigationShortcut.call(this, e)', at);
  assert.notEqual(end, -1, '분기 끝(내비게이션 단축키 처리)을 찾지 못했다');
  return kb.slice(at, end);
}

test('셀 블록에서 Backspace·Delete(무보조)는 내용 지우기 — 블록 해제보다 먼저', () => {
  const block = cellSelectionBranch(codeOnly(source('src/engine/input-handler-keyboard.ts')));
  const fallthroughAt = block.lastIndexOf('this.cursor.exitCellSelectionMode();');

  const clearAt = block.indexOf("e.key === 'Delete' || e.key === 'Backspace'");
  assert.notEqual(clearAt, -1, 'Backspace+Delete 무보조 내용 지우기 분기가 없다');
  assert.ok(clearAt < fallthroughAt, '내용 지우기가 블록 해제 폴백보다 뒤에 있다');
  assert.match(
    block.slice(clearAt, fallthroughAt),
    /!e\.ctrlKey && !e\.metaKey && !e\.altKey[\s\S]*this\.clearSelectedCellBlock\(\)/,
    '무보조 Backspace/Delete 가 clearSelectedCellBlock 로 가지 않는다',
  );
});

test('셀 블록에서 ⌘⌫·⌘Delete 는 edit:delete 로 태운다 — 블록 해제보다 먼저', () => {
  const block = cellSelectionBranch(codeOnly(source('src/engine/input-handler-keyboard.ts')));
  const fallthroughAt = block.lastIndexOf('this.cursor.exitCellSelectionMode();');

  assert.match(
    block.slice(0, fallthroughAt),
    /e\.metaKey && !e\.ctrlKey && !e\.altKey[\s\S]{0,300}this\.dispatcher\?\.dispatch\('edit:delete'\)/,
    'Meta+Backspace/Delete 가 edit:delete 로 디스패치되지 않는다',
  );
});

test('edit:delete 는 셀 블록을 해제하지 않고 통과한다 (⌘E 경로)', () => {
  const kb = codeOnly(source('src/engine/input-handler-keyboard.ts'));
  assert.match(
    kb,
    /CELL_BLOCK_GLOBAL_COMMANDS = new Set\(\[[^\]]*'edit:delete'[^\]]*\]\)/,
    'CELL_BLOCK_GLOBAL_COMMANDS 에 edit:delete 가 없다 — ⌘E 가 블록을 먼저 해제한다',
  );
});

test('edit:delete 는 셀 선택 모드에서도 실행 가능하다', () => {
  const edit = codeOnly(source('src/command/commands/edit.ts'));
  const cmd = edit.slice(edit.indexOf("id: 'edit:delete'"), edit.indexOf("id: 'edit:select-all'"));
  assert.match(cmd, /canExecute:[\s\S]*ctx\.inCellSelectionMode/,
    'edit:delete 의 canExecute 가 inCellSelectionMode 를 인정하지 않는다');
});

test('performDelete 는 셀 선택 모드를 deleteSelectedCellBlock 로 라우팅한다', () => {
  const ih = codeOnly(source('src/engine/input-handler.ts'));
  const fn = functionBodyFrom(ih, 'performDelete(): void');
  const routeAt = fn.indexOf('this.cursor.isInCellSelectionMode()');
  assert.notEqual(routeAt, -1, 'performDelete 에 셀 선택 모드 분기가 없다');
  assert.match(fn.slice(routeAt), /void this\.deleteSelectedCellBlock\(\)/,
    '셀 선택 모드가 deleteSelectedCellBlock 으로 가지 않는다');
});

test('구조 삭제는 전체 줄/칸 정렬·중첩·제외 셀·병합 걸침을 가린다', () => {
  const ih = codeOnly(source('src/engine/input-handler.ts'));
  const fn = functionBodyFrom(ih, 'private async deleteSelectedCellBlock()');
  assert.match(fn, /isProtectedCellSelectionMode/, '보호 셀 선택 가드가 없다');
  assert.match(fn, /cellPath/, '중첩 표 제외 가드가 없다');
  assert.match(fn, /getExcludedCells\(\)\.size === 0/, '제외 셀 가드가 없다');
  assert.match(fn, /getTableDimensions/, '표 차수를 읽지 않는다');
  assert.match(fn, /getTableCellBboxes/, '병합 셀 걸침 검사가 없다');
  assert.match(fn, /askCellBlockDelete/, '구조 삭제 확인 대화상자를 묻지 않는다');
  assert.match(fn, /kind: 'snapshot'/, '구조 삭제가 스냅샷으로 기록되지 않는다');
  assert.match(fn, /mode: 'cellBlock'/, '지우기 전 셀 블록을 selectionBefore 에 싣지 않는다');
});

test('셀 블록 Ctrl+E는 내용만, ⌘E는 구조 삭제로 라우팅한다 (일반·IME)', async () => {
  const vite = await createServer({
    root: rootDir, appType: 'custom', logLevel: 'silent', server: { middlewareMode: true },
  });
  try {
    const { onKeyDown } = await vite.ssrLoadModule('/src/engine/input-handler-keyboard.ts');
    for (const composing of [false, true]) {
      for (const modifier of ['ctrl', 'meta'] as const) {
        const calls: string[] = [];
        const cursor = {
          isInCellSelectionMode: () => { calls.push('cell-mode'); return true; },
          isProtectedCellSelectionMode: () => false,
          isInHeaderFooter: () => {
            assert.equal(composing, false, 'IME 경로는 일반 모드 분기로 내려가지 않는다');
            return false;
          },
          isInFootnote: () => false,
          isInPictureObjectSelection: () => false,
          isInTableObjectSelection: () => false,
          isInBlockSelectionMode: () => false,
        };
        const handler = {
          active: true,
          cursor,
          dispatcher: { dispatch: (id: string) => calls.push(id), isEnabled: () => false },
          clearSelectedCellBlock: () => calls.push('clear'),
          updateCellSelection: () => calls.push('update'),
        };
        const event = {
          key: composing ? 'Process' : 'e', code: 'KeyE',
          isComposing: composing, keyCode: composing ? 229 : 0,
          ctrlKey: modifier === 'ctrl', metaKey: modifier === 'meta',
          shiftKey: false, altKey: false,
          preventDefault: () => calls.push('prevent'),
        };
        onKeyDown.call(handler, event);
        assert.ok(calls.includes('prevent'), `${modifier}, IME=${composing}: 키 소비`);
        if (modifier === 'ctrl') {
          assert.ok(calls.includes('clear'), `${modifier}, IME=${composing}: 내용 지우기`);
          assert.ok(calls.includes('update'), `${modifier}, IME=${composing}: 셀 선택 갱신`);
          assert.ok(!calls.includes('edit:delete'), `${modifier}, IME=${composing}: 대화상자 명령 없음`);
        } else {
          assert.ok(calls.includes('edit:delete'), `${modifier}, IME=${composing}: 구조 삭제 명령`);
          assert.ok(!calls.includes('clear'), `${modifier}, IME=${composing}: 즉시 내용 지우지 않음`);
        }
        if (composing) assert.ok(calls.includes('cell-mode'), 'IME 조합 분기에서 셀 모드 확인');
      }
    }
  } finally { await vite.close(); }
});

test('두 번째 행 삭제 실패는 snapshot을 복원하고 원래 행 수를 보존한다', async () => {
  const vite = await createServer({
    root: rootDir, appType: 'custom', logLevel: 'silent', server: { middlewareMode: true },
  });
  try {
    const { SnapshotCommand } = await vite.ssrLoadModule('/src/engine/command.ts');
    const method = functionBodyFrom(source('src/engine/input-handler.ts'), 'private async deleteSelectedCellBlock()');
    const methodCode = stripTypeScriptTypes(`const holder = { ${method.replace('private async', 'async')} };`);
    const deleteSelectedCellBlock = new Function('askCellBlockDelete', 'clampedCellAfterDelete',
      `${methodCode} return holder.deleteSelectedCellBlock;`)(
        async () => 'delete', () => null,
      );
    const before = 3;
    let rows = before;
    let nextId = 0;
    let restoreId: number | null = null;
    const snapshots = new Map<number, number>();
    const wasm = {
      getTableDimensions: () => ({ rowCount: rows, colCount: 3 }),
      getTableCellBboxes: () => [],
      deleteTableRow: () => {
        if (rows === 2) return { ok: false, rowCount: rows, colCount: 3 };
        rows -= 1;
        return { ok: true, rowCount: rows, colCount: 3 };
      },
      saveSnapshot: () => { const id = ++nextId; snapshots.set(id, rows); return id; },
      restoreSnapshot: (id: number) => { restoreId = id; rows = snapshots.get(id)!; },
      discardSnapshot: (id: number) => snapshots.delete(id),
    };
    const pos = { sectionIndex: 0, paragraphIndex: 1, charOffset: 0 };
    const cursor = {
      isProtectedCellSelectionMode: () => false,
      getCellTableContext: () => ({ sec: 0, ppi: 1, ci: 0 }),
      getSelectedCellRange: () => ({ startRow: 1, endRow: 2, startCol: 0, endCol: 2 }),
      getExcludedCells: () => new Set(),
      captureCellSelection: () => null,
      getPosition: () => pos,
    };
    const handler = {
      cursor, wasm, focusTextarea: () => {},
      executeOperation: (desc: any) => new SnapshotCommand(
        desc.operationType, pos, pos, desc.operation, desc.selectionBefore,
      ).execute(wasm),
    };
    await assert.rejects(deleteSelectedCellBlock.call(handler), /셀 블록 행 삭제 실패/);
    assert.equal(restoreId, 1, '실패 시 before 스냅숏 복원');
    assert.equal(rows, before, '첫 삭제도 원복되어 행 수 유지');
  } finally { await vite.close(); }
});
