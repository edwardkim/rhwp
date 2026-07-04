import test from 'node:test';
import assert from 'node:assert/strict';

import { defaultShortcuts, matchShortcut } from '../src/command/shortcut-map.ts';

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

function command(input: Partial<KeyboardEvent>, platform: 'mac' | 'other' = 'other'): string | null {
  return matchShortcut(key(input), defaultShortcuts, platform);
}

test('한컴 호환 장평 단축키를 영문 키로 매핑한다', () => {
  assert.equal(command({ key: 'j', code: 'KeyJ', altKey: true, shiftKey: true }), 'format:char-ratio-decrease');
  assert.equal(command({ key: 'k', code: 'KeyK', altKey: true, shiftKey: true }), 'format:char-ratio-increase');
});

test('한컴 호환 자간 단축키를 영문 키로 매핑한다', () => {
  assert.equal(command({ key: 'n', code: 'KeyN', altKey: true, shiftKey: true }), 'format:char-spacing-decrease');
  assert.equal(command({ key: 'w', code: 'KeyW', altKey: true, shiftKey: true }), 'format:char-spacing-increase');
});

test('한글 입력 모드 장평/자간 단축키를 매핑한다', () => {
  assert.equal(command({ key: 'ㅓ', altKey: true, shiftKey: true }), 'format:char-ratio-decrease');
  assert.equal(command({ key: 'ㅏ', altKey: true, shiftKey: true }), 'format:char-ratio-increase');
  assert.equal(command({ key: 'ㅜ', altKey: true, shiftKey: true }), 'format:char-spacing-decrease');
  assert.equal(command({ key: 'ㅈ', altKey: true, shiftKey: true }), 'format:char-spacing-increase');
});

test('IME pending 상태처럼 key가 Process여도 code로 장평/자간 단축키를 판별한다', () => {
  assert.equal(command({ key: 'Process', code: 'KeyJ', altKey: true, shiftKey: true }), 'format:char-ratio-decrease');
  assert.equal(command({ key: 'Process', code: 'KeyK', altKey: true, shiftKey: true }), 'format:char-ratio-increase');
  assert.equal(command({ key: 'Process', code: 'KeyN', altKey: true, shiftKey: true }), 'format:char-spacing-decrease');
  assert.equal(command({ key: 'Process', code: 'KeyW', altKey: true, shiftKey: true }), 'format:char-spacing-increase');
});

test('표 줄/칸 추가·지우기 단축키는 대화상자 명령으로 매핑한다', () => {
  assert.equal(command({ key: 'Enter', altKey: true }, 'mac'), 'table:insert-row-col');
  assert.equal(command({ key: 'enter', altKey: true }, 'mac'), 'table:insert-row-col');
  assert.equal(command({ key: 'Enter', altKey: true }, 'other'), 'table:insert-row-col');
  assert.equal(command({ key: 'enter', altKey: true }, 'other'), 'table:insert-row-col');
  assert.equal(command({ key: 'Insert', altKey: true }, 'mac'), null);
  assert.equal(command({ key: 'Help', altKey: true }, 'mac'), null);
  assert.equal(command({ key: 'Insert', altKey: true }, 'other'), null);
  assert.equal(command({ key: 'insert', altKey: true }, 'other'), null);
  assert.equal(command({ key: 'Help', altKey: true }, 'other'), null);
  assert.equal(command({ key: 'Process', code: 'Insert', altKey: true }, 'other'), null);
  assert.equal(command({ key: 'Process', code: 'Help', altKey: true }, 'other'), null);
  assert.equal(command({ key: 'Delete', altKey: true }), 'table:delete-row-col');
  assert.equal(command({ key: 'delete', altKey: true }), 'table:delete-row-col');
});
