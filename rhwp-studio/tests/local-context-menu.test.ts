import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

import { functionBodyFrom } from './support/source-guard.ts';

// [#6053] LocalContextMenu 는 DOM 셸이라 node 로 인스턴스화할 수 없다 — 여기서는
// 이 메뉴가 존재하는 이유(전역 커맨드 표면을 늘리지 않는다 · CSS 신규 0줄 ·
// 모달 위 키 처리)를 소스에서 못 박는다. chart-data-dialog.test.ts 선례.

const rootDir = dirname(dirname(fileURLToPath(import.meta.url)));
const src = readFileSync(join(rootDir, 'src/ui/local-context-menu.ts'), 'utf8');

/**
 * 주석을 지운 코드만. 이 파일의 헤더 주석은 "왜 레지스트리를 안 쓰는가"를 설명하느라
 * `registry`·`dispatcher` 를 그대로 인용한다 — 주석을 세면 그 설명이 곧 위반이 된다.
 */
const code = src.replace(/\/\*[\s\S]*?\*\//g, '').replace(/(^|[^:])\/\/.*$/gm, '$1');

test('전역 커맨드 표면을 늘리지 않는다 — 레지스트리·디스패처를 쓰지 않는다', () => {
  assert.doesNotMatch(code, /from '@\/command\//, '@/command/* 를 import 하면 안 된다');
  assert.doesNotMatch(code, /\bregistry\b|\bdispatcher\b/, '레지스트리 구동이면 커맨드 등재가 딸려온다');
  // `[data-cmd]` 는 automation e2e 의 마크업↔레지스트리 드리프트 스캔 표면이다.
  assert.doesNotMatch(code, /dataset\.cmd|data-cmd/, '그리드 항목을 커맨드 표면에 노출하지 않는다');
});

test('CSS 는 기존 메뉴 것을 그대로 재사용한다 — 신규 CSS 0줄', () => {
  for (const cls of ['context-menu', 'md-item', 'md-sep']) {
    assert.match(src, new RegExp(`'${cls}'`), `${cls} 를 재사용해야 한다`);
  }
  // 새 접두어를 만들면 UI 규칙 문서(rhwp_studio_ui_conventions.md) 표 갱신 의무가 생긴다.
  assert.doesNotMatch(src, /className = '(?!context-menu|md-item|md-sep)/, '새 클래스 접두어 금지');
});

test('ESC·Enter 는 window capture 에서 잡고 전파를 끊는다 — 모달과의 경합', () => {
  const body = functionBodyFrom(src, 'show(');
  // ModalDialog 는 document capture 에 ESC=닫기, 비-입력 Enter=[확인] 을 건다.
  // 캡처는 window → document 순이라 window 라야 메뉴가 먼저 먹는다.
  assert.match(body, /window\.addEventListener\(\s*'keydown'[\s\S]*?,\s*true\s*\)/,
    'keydown 은 window 에 capture 로 달아야 한다');
  assert.doesNotMatch(body, /document\.addEventListener\(\s*'keydown'/,
    'document 에 달면 ModalDialog 가 먼저 stopPropagation 해 도달하지 못한다');

  const handler = src.slice(src.indexOf('this.keyHandler = '), src.indexOf('window.addEventListener'));
  assert.match(handler, /'Escape'/, 'ESC 를 다뤄야 한다');
  assert.match(handler, /'Enter'/, 'Enter 도 다뤄야 한다 — 안 막으면 [확인] 이 눌린다');
  assert.match(handler, /stopPropagation\(\)/, '먹은 키는 다이얼로그로 넘기지 않는다');
});

test('닫을 때 window 리스너를 되돌린다 — 다이얼로그가 남아도 새는 핸들러가 없다', () => {
  // `hide(` 첫 등장은 show() 안의 호출이다 — 선언부를 지목해야 한다.
  const body = functionBodyFrom(src, 'hide(): void');
  assert.match(body, /window\.removeEventListener\(\s*'keydown'[\s\S]*?,\s*true\s*\)/);
  assert.match(body, /this\.el\?\.remove\(\)/);
});
