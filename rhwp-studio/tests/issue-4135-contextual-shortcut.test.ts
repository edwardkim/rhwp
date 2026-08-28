import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';

import { resolveCellBlockCtrlShiftS } from '../src/command/contextual-shortcut.ts';

function key(input: Partial<KeyboardEvent>): KeyboardEvent {
  return {
    key: input.key ?? '',
    code: input.code ?? '',
    shiftKey: input.shiftKey ?? false,
    ctrlKey: input.ctrlKey ?? false,
    metaKey: input.metaKey ?? false,
    altKey: input.altKey ?? false,
  } as KeyboardEvent;
}

const fullCellBlock = {
  inCellSelectionMode: true,
  blockSumEnabled: true,
  saveAsEnabled: true,
};

test('full 셀 블록의 Ctrl/Cmd+Shift+S는 블록 합계를 우선한다', () => {
  assert.deepEqual(
    resolveCellBlockCtrlShiftS(key({ key: 'S', code: 'KeyS', ctrlKey: true, shiftKey: true }), fullCellBlock),
    { kind: 'dispatch', commandId: 'table:block-sum' },
  );
  assert.deepEqual(
    resolveCellBlockCtrlShiftS(key({ key: 's', code: 'KeyS', metaKey: true, shiftKey: true }), fullCellBlock),
    { kind: 'dispatch', commandId: 'table:block-sum' },
  );
});

test('한글 IME 자모와 조합 중 Process도 물리 KeyS로 블록 합계를 찾는다', () => {
  assert.deepEqual(
    resolveCellBlockCtrlShiftS(key({ key: 'ㄴ', code: 'KeyS', ctrlKey: true, shiftKey: true }), fullCellBlock),
    { kind: 'dispatch', commandId: 'table:block-sum' },
  );
  assert.deepEqual(
    resolveCellBlockCtrlShiftS(key({ key: 'Process', code: 'KeyS', ctrlKey: true, shiftKey: true }), fullCellBlock),
    { kind: 'dispatch', commandId: 'table:block-sum' },
  );
});

test('셀 블록이 아니면 일반 shortcut-map이 Save As를 소유하도록 양보한다', () => {
  assert.equal(
    resolveCellBlockCtrlShiftS(
      key({ key: 'S', code: 'KeyS', ctrlKey: true, shiftKey: true }),
      { ...fullCellBlock, inCellSelectionMode: false },
    ),
    null,
  );
});

test('블록 합계가 차단되고 Save As가 활성인 full 상태에서는 Save As로 양보한다', () => {
  assert.deepEqual(
    resolveCellBlockCtrlShiftS(
      key({ key: 'S', code: 'KeyS', ctrlKey: true, shiftKey: true }),
      { ...fullCellBlock, blockSumEnabled: false },
    ),
    { kind: 'dispatch', commandId: 'file:save-as' },
  );
});

test('embed처럼 Save As 소유권이 없으면 후순위 블록 합계로 폴스루하지 않고 소비한다', () => {
  assert.deepEqual(
    resolveCellBlockCtrlShiftS(
      key({ key: 'S', code: 'KeyS', ctrlKey: true, shiftKey: true }),
      { ...fullCellBlock, saveAsEnabled: false },
    ),
    { kind: 'consume' },
  );
});

test('수정자 없는 S와 다른 물리 키는 문맥 라우터가 소유하지 않는다', () => {
  assert.equal(resolveCellBlockCtrlShiftS(key({ key: 's', code: 'KeyS' }), fullCellBlock), null);
  assert.equal(
    resolveCellBlockCtrlShiftS(key({ key: 'A', code: 'KeyA', ctrlKey: true, shiftKey: true }), fullCellBlock),
    null,
  );
});

test('InputHandler는 IME와 수정자 없는 S 분기보다 먼저 셀 블록 문맥 단축키를 처리한다', () => {
  const source = readFileSync(
    new URL('../src/engine/input-handler-keyboard.ts', import.meta.url),
    'utf8',
  );
  const contextual = source.indexOf('dispatchCellBlockCtrlShiftS.call(this, e)');
  const ime = source.indexOf('if (e.isComposing || e.keyCode === 229) {');
  const plainS = source.indexOf("this.dispatcher?.dispatch('table:cell-split')");

  assert.ok(contextual >= 0, '셀 블록 문맥 단축키 호출이 있어야 한다');
  assert.ok(ime > contextual, 'IME 조기 반환보다 먼저 처리해야 한다');
  assert.ok(plainS > contextual, '수정자 없는 S 셀 나누기보다 먼저 처리해야 한다');
  assert.match(
    source,
    /if \(!e\.ctrlKey && !e\.metaKey && !e\.altKey && \(e\.key === 's' \|\| e\.key === 'S'\)\)/,
  );
});
