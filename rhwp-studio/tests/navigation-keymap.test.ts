import test from 'node:test';
import assert from 'node:assert/strict';

import {
  detectPlatformKind,
  formatShortcutLabel,
  getNavigationAction,
  shouldSuppressUnmappedNavigation,
  type NavigationKeyInput,
  type PlatformKind,
} from '../src/engine/navigation-keymap.ts';

function key(input: Partial<NavigationKeyInput>): NavigationKeyInput {
  return {
    key: input.key ?? '',
    code: input.code,
    shiftKey: input.shiftKey ?? false,
    ctrlKey: input.ctrlKey ?? false,
    metaKey: input.metaKey ?? false,
    altKey: input.altKey ?? false,
  };
}

function action(input: Partial<NavigationKeyInput>, platform: PlatformKind) {
  return getNavigationAction(key(input), platform);
}

test('detectPlatformKind는 macOS 계열 platform/userAgent를 mac으로 판별한다', () => {
  assert.equal(detectPlatformKind({ platform: 'MacIntel', userAgent: '' }), 'mac');
  assert.equal(detectPlatformKind({ platform: 'iPad', userAgent: '' }), 'mac');
  assert.equal(detectPlatformKind({ platform: '', userAgent: 'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7)' }), 'mac');
});

test('detectPlatformKind는 Windows/Linux 계열을 other로 판별한다', () => {
  assert.equal(detectPlatformKind({ platform: 'Win32', userAgent: '' }), 'other');
  assert.equal(detectPlatformKind({ platform: 'Linux x86_64', userAgent: '' }), 'other');
});

test('detectPlatformKind는 테스트 override를 우선한다', () => {
  const globalForTest = globalThis as typeof globalThis & { __rhwpTestPlatformKind?: PlatformKind };
  globalForTest.__rhwpTestPlatformKind = 'mac';
  assert.equal(detectPlatformKind({ platform: 'Win32', userAgent: '' }), 'mac');
  globalForTest.__rhwpTestPlatformKind = 'other';
  assert.equal(detectPlatformKind({ platform: 'MacIntel', userAgent: '' }), 'other');
  delete globalForTest.__rhwpTestPlatformKind;
});

test('macOS keymap은 Option+Arrow를 단어 이동으로 처리한다', () => {
  assert.equal(action({ key: 'ArrowLeft', altKey: true }, 'mac'), 'wordBackward');
  assert.equal(action({ key: 'ArrowRight', altKey: true }, 'mac'), 'wordForward');
  assert.equal(action({ key: 'ArrowLeft', altKey: true, shiftKey: true }, 'mac'), 'wordBackward');
});

test('macOS keymap은 Command+ArrowLeft/Right를 줄 처음/끝으로 처리한다', () => {
  assert.equal(action({ key: 'ArrowLeft', metaKey: true }, 'mac'), 'lineStart');
  assert.equal(action({ key: 'ArrowRight', metaKey: true }, 'mac'), 'lineEnd');
  assert.equal(action({ key: 'ArrowRight', metaKey: true, shiftKey: true }, 'mac'), 'lineEnd');
});

test('macOS keymap은 Ctrl+Arrow와 Command+ArrowUp/Down을 이번 범위에서 처리하지 않는다', () => {
  assert.equal(action({ key: 'ArrowLeft', ctrlKey: true }, 'mac'), null);
  assert.equal(action({ key: 'ArrowRight', ctrlKey: true }, 'mac'), null);
  assert.equal(action({ key: 'ArrowUp', metaKey: true }, 'mac'), null);
  assert.equal(action({ key: 'ArrowDown', metaKey: true }, 'mac'), null);
});

test('Windows/Linux keymap은 Ctrl+ArrowLeft/Right를 단어 이동으로 처리한다', () => {
  assert.equal(action({ key: 'ArrowLeft', ctrlKey: true }, 'other'), 'wordBackward');
  assert.equal(action({ key: 'ArrowRight', ctrlKey: true }, 'other'), 'wordForward');
  assert.equal(action({ key: 'ArrowLeft', ctrlKey: true, shiftKey: true }, 'other'), 'wordBackward');
});

test('Windows/Linux keymap은 Ctrl+ArrowUp/Down을 문단 이동으로 처리한다', () => {
  assert.equal(action({ key: 'ArrowUp', ctrlKey: true }, 'other'), 'paragraphBackward');
  assert.equal(action({ key: 'ArrowDown', ctrlKey: true }, 'other'), 'paragraphForward');
});

test('Windows/Linux keymap은 Alt+Arrow 단어 이동을 처리하지 않는다', () => {
  assert.equal(action({ key: 'ArrowLeft', altKey: true }, 'other'), null);
  assert.equal(action({ key: 'ArrowRight', altKey: true }, 'other'), null);
  assert.equal(action({ key: 'ArrowLeft', altKey: true, shiftKey: true }, 'other'), null);
  assert.equal(shouldSuppressUnmappedNavigation(key({ key: 'ArrowLeft', altKey: true }), 'other'), true);
  assert.equal(shouldSuppressUnmappedNavigation(key({ key: 'ArrowRight', altKey: true }), 'other'), true);
  assert.equal(shouldSuppressUnmappedNavigation(key({ key: 'ArrowUp', altKey: true }), 'other'), false);
  assert.equal(shouldSuppressUnmappedNavigation(key({ key: 'ArrowLeft', altKey: true }), 'mac'), false);
});

test('Home/End는 플랫폼 공통 줄 처음/끝으로 처리하고 Ctrl/Meta 조합은 기존 경로에 남긴다', () => {
  assert.equal(action({ key: 'Home' }, 'mac'), 'lineStart');
  assert.equal(action({ key: 'End' }, 'mac'), 'lineEnd');
  assert.equal(action({ key: 'Home', shiftKey: true }, 'other'), 'lineStart');
  assert.equal(action({ key: 'End', shiftKey: true }, 'other'), 'lineEnd');
  assert.equal(action({ key: 'Home', ctrlKey: true }, 'other'), null);
  assert.equal(action({ key: 'End', metaKey: true }, 'mac'), null);
});

test('일반 command shortcut 입력은 navigation helper가 처리하지 않는다', () => {
  assert.equal(action({ key: 's', ctrlKey: true }, 'other'), null);
  assert.equal(action({ key: 's', metaKey: true }, 'mac'), null);
  assert.equal(action({ key: 'c', ctrlKey: true }, 'other'), null);
  assert.equal(action({ key: 'c', metaKey: true }, 'mac'), null);
});

test('IME pending nav처럼 key가 Process여도 code로 navigation을 판별한다', () => {
  assert.equal(action({ key: 'Process', code: 'ArrowLeft', ctrlKey: true }, 'other'), 'wordBackward');
  assert.equal(action({ key: 'Process', code: 'ArrowRight', altKey: true }, 'mac'), 'wordForward');
});

test('formatShortcutLabel은 macOS에서 Ctrl을 Command로 치환한다', () => {
  assert.equal(formatShortcutLabel('Ctrl+S', 'mac'), 'Command+S');
  assert.equal(formatShortcutLabel('Ctrl+Shift+Z', 'mac'), 'Command+Shift+Z');
  assert.equal(formatShortcutLabel('Ctrl+Alt+C', 'mac'), 'Command+Alt+C');
  assert.equal(formatShortcutLabel('Ctrl+M,K', 'mac'), 'Command+M,K');
  assert.equal(formatShortcutLabel('Ctrl+Enter', 'mac'), 'Command+Enter');
});

test('formatShortcutLabel은 Windows/Linux에서 원본을 유지한다', () => {
  assert.equal(formatShortcutLabel('Ctrl+S', 'other'), 'Ctrl+S');
  assert.equal(formatShortcutLabel('Ctrl+Shift+Z', 'other'), 'Ctrl+Shift+Z');
});

test('formatShortcutLabel은 Ctrl이 없는 라벨을 변경하지 않는다', () => {
  assert.equal(formatShortcutLabel('F7', 'mac'), 'F7');
  assert.equal(formatShortcutLabel('Alt+G', 'mac'), 'Alt+G');
  assert.equal(formatShortcutLabel('Shift+Num +', 'mac'), 'Shift+Num +');
  assert.equal(formatShortcutLabel('H', 'mac'), 'H');
  assert.equal(formatShortcutLabel('Alt+Shift+V', 'mac'), 'Alt+Shift+V');
});

test('formatShortcutLabel은 테스트 override를 존중한다', () => {
  const globalForTest = globalThis as typeof globalThis & { __rhwpTestPlatformKind?: PlatformKind };
  try {
    globalForTest.__rhwpTestPlatformKind = 'mac';
    assert.equal(formatShortcutLabel('Ctrl+S'), 'Command+S');
    globalForTest.__rhwpTestPlatformKind = 'other';
    assert.equal(formatShortcutLabel('Ctrl+S'), 'Ctrl+S');
  } finally {
    delete globalForTest.__rhwpTestPlatformKind;
  }
});
