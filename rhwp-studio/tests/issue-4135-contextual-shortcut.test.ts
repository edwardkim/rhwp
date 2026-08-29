import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';

import * as contextualShortcut from '../src/command/contextual-shortcut.ts';

const { resolveCellBlockCtrlShiftS } = contextualShortcut;

type CellBlockLetterResolver = (
  event: KeyboardEvent,
  context: { inCellSelectionMode: boolean },
) => { kind: 'dispatch'; commandId: 'table:cell-split' | 'table:cell-merge' } | null;

type CellBlockLetterImeGuardLike = {
  arm: (event: KeyboardEvent) => boolean;
  consume: (
    eventType: 'compositionstart' | 'input' | 'compositionend',
    text?: string,
  ) => boolean;
  reset: () => void;
};

type CellBlockLetterImeGuardConstructor = new () => CellBlockLetterImeGuardLike;

function cellBlockLetterResolver(): CellBlockLetterResolver {
  const resolver = (contextualShortcut as unknown as {
    resolveCellBlockLetterShortcut?: CellBlockLetterResolver;
  }).resolveCellBlockLetterShortcut;
  assert.equal(
    typeof resolver,
    'function',
    'Recovery R1: 수정자 없는 셀 블록 물리 키 resolver가 필요하다',
  );
  return resolver;
}

function cellBlockLetterImeGuardConstructor(): CellBlockLetterImeGuardConstructor {
  const Guard = (contextualShortcut as unknown as {
    CellBlockLetterImeGuard?: CellBlockLetterImeGuardConstructor;
  }).CellBlockLetterImeGuard;
  assert.equal(
    typeof Guard,
    'function',
    'Recovery R4 corrective RED: 셀 문자 단축키의 후속 IME 입력 guard가 필요하다',
  );
  return Guard;
}

function key(input: Partial<KeyboardEvent>): KeyboardEvent {
  return {
    key: input.key ?? '',
    code: input.code ?? '',
    shiftKey: input.shiftKey ?? false,
    ctrlKey: input.ctrlKey ?? false,
    metaKey: input.metaKey ?? false,
    altKey: input.altKey ?? false,
    isComposing: input.isComposing ?? false,
    keyCode: input.keyCode ?? 0,
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

test('InputHandler는 IME보다 먼저 Ctrl/Cmd+Shift+S와 수정자 없는 S/M을 순서대로 처리한다', () => {
  const source = readFileSync(
    new URL('../src/engine/input-handler-keyboard.ts', import.meta.url),
    'utf8',
  );
  const contextual = source.indexOf('dispatchCellBlockCtrlShiftS.call(this, e)');
  const cellLetters = source.indexOf('dispatchCellBlockLetterShortcut.call(this, e)');
  const ime = source.indexOf('if (e.isComposing || e.keyCode === 229) {');

  assert.ok(contextual >= 0, '셀 블록 문맥 단축키 호출이 있어야 한다');
  assert.ok(cellLetters > contextual, 'Ctrl/Cmd+Shift+S 뒤에 수정자 없는 S/M을 판정해야 한다');
  assert.ok(ime > cellLetters, '두 셀 블록 resolver 모두 IME 조기 반환보다 먼저 처리해야 한다');
});

test('Recovery R1: 영문·한글·Process KeyS는 모두 셀 나누기를 소유한다', () => {
  const resolve = cellBlockLetterResolver();
  const context = { inCellSelectionMode: true };
  for (const event of [
    key({ key: 's', code: 'KeyS' }),
    key({ key: 'S', code: 'KeyS', shiftKey: true }),
    key({ key: 'ㄴ', code: 'KeyS' }),
    key({ key: 'Process', code: 'KeyS' }),
  ]) {
    assert.deepEqual(resolve(event, context), {
      kind: 'dispatch',
      commandId: 'table:cell-split',
    });
  }
});

test('Recovery R1: 영문·한글·Process KeyM은 모두 셀 합치기를 소유한다', () => {
  const resolve = cellBlockLetterResolver();
  const context = { inCellSelectionMode: true };
  for (const event of [
    key({ key: 'm', code: 'KeyM' }),
    key({ key: 'M', code: 'KeyM', shiftKey: true }),
    key({ key: 'ㅡ', code: 'KeyM' }),
    key({ key: 'Process', code: 'KeyM' }),
  ]) {
    assert.deepEqual(resolve(event, context), {
      kind: 'dispatch',
      commandId: 'table:cell-merge',
    });
  }
});

test('Recovery R1: 셀 블록 밖이거나 Ctrl/Meta/Alt 수정자가 있으면 S/M을 소유하지 않는다', () => {
  const resolve = cellBlockLetterResolver();
  assert.equal(resolve(key({ key: 's', code: 'KeyS' }), { inCellSelectionMode: false }), null);
  assert.equal(resolve(key({ key: 's', code: 'KeyS', ctrlKey: true }), { inCellSelectionMode: true }), null);
  assert.equal(resolve(key({ key: 'm', code: 'KeyM', metaKey: true }), { inCellSelectionMode: true }), null);
  assert.equal(resolve(key({ key: 'ㄴ', code: 'KeyS', altKey: true }), { inCellSelectionMode: true }), null);
});

test('Recovery R1: 물리 S/M 셀 명령은 IME 조기 반환보다 먼저 처리한다', () => {
  const source = readFileSync(
    new URL('../src/engine/input-handler-keyboard.ts', import.meta.url),
    'utf8',
  );
  const contextual = source.indexOf('dispatchCellBlockLetterShortcut.call(this, e)');
  const ime = source.indexOf('if (e.isComposing || e.keyCode === 229) {');

  assert.ok(contextual >= 0, '셀 블록 S/M 문맥 단축키 호출이 있어야 한다');
  assert.ok(ime > contextual, '한글 IME 조기 반환보다 먼저 S/M을 처리해야 한다');
});

test('Recovery R4 corrective RED: 한글 셀 문자 단축키의 후속 조합 입력을 끝까지 소비한다', () => {
  const Guard = cellBlockLetterImeGuardConstructor();
  const guard = new Guard();

  assert.equal(guard.arm(key({ key: 'ㄴ', code: 'KeyS' })), true);
  assert.equal(guard.consume('compositionstart'), true);
  assert.equal(guard.consume('input'), true);
  assert.equal(guard.consume('compositionend'), true);
  assert.equal(guard.consume('input'), false, '조합 종료 뒤의 정상 입력은 통과해야 한다');
});

test('Recovery R4 corrective RED: Process 입력은 input-only 폴백도 한 번만 소비한다', () => {
  const Guard = cellBlockLetterImeGuardConstructor();
  const guard = new Guard();

  assert.equal(guard.arm(key({ key: 'Process', code: 'KeyM', isComposing: true, keyCode: 229 })), true);
  assert.equal(guard.consume('input', 'ㅡ'), true);
  assert.equal(guard.consume('input', 'ㅡ'), false, 'composition 이벤트가 없으면 첫 input만 소비해야 한다');
});

test('Recovery R4 corrective RED: 영문 S/M은 후속 정상 입력을 억제하지 않는다', () => {
  const Guard = cellBlockLetterImeGuardConstructor();
  const guard = new Guard();

  assert.equal(guard.arm(key({ key: 's', code: 'KeyS' })), false);
  assert.equal(guard.consume('compositionstart'), false);
  assert.equal(guard.consume('input'), false);

  assert.equal(guard.arm(key({ key: 'M', code: 'KeyM', shiftKey: true })), false);
  assert.equal(guard.consume('input'), false);
});

test('Recovery R4 corrective: compositionend 뒤 같은 유령 input만 추가로 소비한다', () => {
  const Guard = cellBlockLetterImeGuardConstructor();
  const guard = new Guard();

  guard.arm(key({ key: 'ㄴ', code: 'KeyS' }));
  assert.equal(guard.consume('compositionstart'), true);
  assert.equal(guard.consume('input', 'ㄴ'), true);
  assert.equal(guard.consume('compositionend'), true);
  assert.equal(guard.consume('input', 'ㄴ'), true, '동일 조합 텍스트의 유령 input은 소비해야 한다');
  assert.equal(guard.consume('input', 'ㄴ'), false, '유령 input 억제는 한 번으로 끝나야 한다');
});

test('Recovery R4 corrective: guard가 입력 경로 세 지점과 dispatcher 전에 연결된다', () => {
  const keyboardSource = readFileSync(
    new URL('../src/engine/input-handler-keyboard.ts', import.meta.url),
    'utf8',
  );
  const textSource = readFileSync(
    new URL('../src/engine/input-handler-text.ts', import.meta.url),
    'utf8',
  );
  const arm = keyboardSource.indexOf('this._cellBlockLetterImeGuard?.arm(e)');
  const dispatch = keyboardSource.indexOf('this.dispatcher.dispatch(resolution.commandId)', arm);

  assert.ok(arm >= 0, '셀 문자 단축키가 후속 IME guard를 arm해야 한다');
  assert.ok(dispatch > arm, 'focus 이동 전에 guard를 arm하고 대화상자 명령을 실행해야 한다');
  assert.match(textSource, /\?\.consume\('compositionstart'\)/);
  assert.match(textSource, /\?\.consume\('input', this\.textarea\.value\)/);
  assert.match(textSource, /\?\.consume\('compositionend', this\.textarea\.value\)/);
});
