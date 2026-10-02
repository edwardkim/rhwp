import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { handleDocumentSelectAllShortcut, isEditableShortcutTarget, isTextEditingTarget } from '../src/command/document-shortcut-guard.ts';

const rootDir = dirname(dirname(fileURLToPath(import.meta.url)));
const source = (path: string) => readFileSync(join(rootDir, path), 'utf8');

const keyboardSrc = source('src/engine/input-handler-keyboard.ts');
const handlerSrc = source('src/engine/input-handler.ts');
const cursorSrc = source('src/engine/cursor.ts');
const bridgeSrc = source('src/core/wasm-bridge.ts');

test('⌘A 는 셀 블록을 풀고 캐럿 셀 내용만 선택한다 — 본문 전체 선택보다 먼저', () => {
  const fn = keyboardSrc.slice(keyboardSrc.indexOf('export function handleSelectAll'));
  const body = fn.slice(0, fn.indexOf('\n}'));
  const exitIdx = body.indexOf('exitCellSelectionMode');
  const cellIdx = body.indexOf('selectAllInCell');
  const docStart = body.indexOf('moveTo({ sectionIndex: 0, paragraphIndex: 0');
  assert.notEqual(exitIdx, -1, 'handleSelectAll 이 셀 블록을 해제해야 한다');
  assert.notEqual(cellIdx, -1, 'handleSelectAll 이 셀 내용 선택을 해야 한다');
  assert.ok(cellIdx < docStart, '셀 분기가 본문 전체 선택보다 먼저 와야 한다');
});

test('⌘A 는 캐럿을 범위 끝으로 옮겨도 화면을 스크롤하지 않는다 (맨 아래로 튀는 결함)', () => {
  const fn = keyboardSrc.slice(keyboardSrc.indexOf('export function handleSelectAll'));
  const body = fn.slice(0, fn.indexOf('\n}'));
  const calls = body.match(/this\.updateCaret\([^)]*\)/g) ?? [];
  assert.equal(calls.length, 3, '머리말·셀·문서 세 경로가 캐럿을 갱신한다');
  for (const call of calls) assert.equal(call, 'this.updateCaret(true)', `스크롤 생략이어야 한다: ${call}`);
});

test('selectAllInCell 은 셀 첫 문단~마지막 문단 끝을 선택한다', () => {
  assert.match(cursorSrc, /selectAllInCell\(\):\s*boolean/);
  const fn = cursorSrc.slice(cursorSrc.indexOf('selectAllInCell(): boolean'));
  const body = fn.slice(0, fn.indexOf('\n  }\n'));
  assert.match(body, /getCellParagraphCountByPath/, '중첩 셀은 경로 기반 문단 수');
  assert.match(body, /getCellParagraphLengthByPath|getCellParagraphLength/, '마지막 문단 길이');
  assert.match(body, /setAnchor\(\)/, 'anchor 고정');
  assert.match(body, /isInTextBox/, '글상자는 flat 축 규약을 따라야 한다');
});

test('본문 선택 하이라이트가 범위 안 표의 rect 를 포함한다', () => {
  const fn = handlerSrc.slice(handlerSrc.indexOf('private updateSelection'));
  const branch = fn.slice(fn.indexOf('!startInCell && !endInCell'));
  assert.match(branch, /bodyTableRectsInRange/, '본문 선택 분기에서 표 rect를 얹어야 한다');
  const helper = handlerSrc.slice(handlerSrc.indexOf('private bodyTableAnchorCache'));
  assert.match(helper, /ctrlId !== 'tbl'/, '표 컨트롤만');
  assert.match(helper, /c\.list !== 0/, '본문 소속(최외곽) 표만 — 셀 안 표 제외');
  assert.match(helper, /getControlTextPositions/, '앵커 문자 위치로 범위 판정');
  assert.match(helper, /getTableCellBboxes/, '다중 페이지 표는 셀 bbox 합집합');
  // getControls 의 평탄 para 는 document.sections 경계로 나눈다 — SectionDef 표식
  // 기반 getSectionStarts 가 아니라 getSectionCount+getParagraphCount 누적이어야 한다.
  assert.match(helper, /getSectionCount/, '구역 경계는 document.sections 기준');
  assert.match(helper, /getParagraphCount/, '문단 수 누적으로 시작 경계를 세운다');
  assert.doesNotMatch(helper, /getSectionStarts/, '논리 구역(SectionDef) 시작과 혼동 금지');
  // 문서 변경마다 캐시를 비운다 — clearTableResizeRuntimeCache 는 afterEdit·undo·
  // 문서 교체의 기존 깔때기다.
  const clr = handlerSrc.slice(handlerSrc.indexOf('private clearTableResizeRuntimeCache'));
  assert.match(clr.slice(0, clr.indexOf('\n  }')), /bodyTableAnchorCache = null/);
});

type FakeElement = HTMLElement & { parentElement: FakeElement | null };

function element(tagName: string, parentElement: FakeElement | null = null, options: {
  contentEditable?: boolean;
  role?: string;
} = {}): FakeElement {
  return {
    tagName,
    parentElement,
    isContentEditable: options.contentEditable ?? false,
    getAttribute: (name: string) => name === 'role' ? options.role ?? null : null,
  } as FakeElement;
}

function invokeDocumentSelectAll(target: FakeElement, overlay: Pick<Element, 'isConnected'> | null = null) {
  let dispatches = 0;
  let focuses = 0;
  let prevented = 0;
  const handled = handleDocumentSelectAllShortcut(
    { target, preventDefault: () => { prevented++; } },
    overlay,
    () => { dispatches++; },
    () => { focuses++; },
  );
  return { handled, dispatches, focuses, prevented };
}

test('붙어 있는 modal-overlay에서는 문서 ⌘A를 dispatch하지 않는다', () => {
  const body = element('BODY');
  assert.deepEqual(invokeDocumentSelectAll(body, { isConnected: true }), {
    handled: false, dispatches: 0, focuses: 0, prevented: 0,
  });
  assert.equal(invokeDocumentSelectAll(body, { isConnected: false }).dispatches, 1);
});

test('select, contentEditable 자손, role=textbox에서는 문서 ⌘A를 dispatch하지 않는다', () => {
  const body = element('BODY');
  const targets = [
    element('SELECT', body),
    element('SPAN', element('DIV', body, { contentEditable: true })),
    element('SPAN', element('DIV', body, { role: 'textbox' })),
    element('TEXTAREA', body),
    element('INPUT', body),
  ];
  for (const target of targets) {
    assert.equal(isEditableShortcutTarget(target), true, `${target.tagName} 편집 대상`);
    assert.deepEqual(invokeDocumentSelectAll(target), {
      handled: false, dispatches: 0, focuses: 0, prevented: 0,
    });
  }
});

test('body 대상 ⌘A는 문서 명령을 한 번 dispatch하고 편집기로 포커스를 돌린다', () => {
  assert.deepEqual(invokeDocumentSelectAll(element('BODY')), {
    handled: true, dispatches: 1, focuses: 1, prevented: 1,
  });
});

test('select는 ⌘A만 막고 전역 문서 이동 키 가드에서는 글자 편집 대상이 아니다', () => {
  const body = element('BODY');
  const select = element('SELECT', body);
  assert.equal(isEditableShortcutTarget(select), true);
  assert.equal(isTextEditingTarget(select), false);
  assert.equal(isTextEditingTarget(element('SPAN', element('DIV', body, { contentEditable: true }))), true);
  assert.equal(isTextEditingTarget(body), false);
});
