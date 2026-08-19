import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

// 시작 화면이 회색 작업 영역만 보이면 편집기가 준비되지 않은 것처럼 보인다. WASM 초기화가
// 끝나면 빈 문서를 열어 바로 편집할 수 있게 하되, 문서를 넘겨받는 진입점(?url=, 자동저장
// 복구)이 문서를 열 기회를 먼저 갖는다.

const rootDir = dirname(dirname(fileURLToPath(import.meta.url)));

function source(path: string): string {
  return readFileSync(join(rootDir, path), 'utf8');
}

function section(text: string, startMarker: string, endMarker: string): string {
  const start = text.indexOf(startMarker);
  const end = text.indexOf(endMarker, start);
  assert.ok(start >= 0 && end > start, `${startMarker} 범위를 찾을 수 있어야 한다`);
  return text.slice(start, end);
}

test('시작 진입점은 ?url= 로드와 자동저장 복구 뒤에 빈 문서를 연다', () => {
  const main = source('src/main.ts');
  const startup = section(main, '    setupGlobalShortcuts();', '\n    // E2E 테스트용 전역 노출');

  const urlIndex = startup.indexOf('await loadFromUrlParam();');
  const recoveryIndex = startup.indexOf('await offerAutosaveRecoveryIfIdle();');
  const blankIndex = startup.indexOf('await openBlankDocumentIfIdle();');

  assert.ok(urlIndex >= 0, '?url= 진입점이 먼저 문서를 열 기회를 가져야 한다');
  assert.ok(recoveryIndex > urlIndex, '자동저장 복구가 그다음 기회를 가져야 한다');
  assert.ok(blankIndex > recoveryIndex, '빈 문서는 마지막에만 연다');
});

test('빈 문서 자동 생성은 이미 열린 문서와 embed 프로파일을 건드리지 않는다', () => {
  const main = source('src/main.ts');
  const fn = section(main, 'async function openBlankDocumentIfIdle', '\nasync function canReplaceCurrentDocument');

  assert.match(fn, /if \(chromeMode === 'embed'\) return;/);
  assert.match(fn, /if \(wasm\.pageCount > 0 \|\| documentState\.isDirty\(\)\) return;/);
  assert.match(fn, /await createNewDocument\(\);/);
});

test('문서 열기는 파싱 전에 빈 쪽 상태로 만든다', () => {
  const main = source('src/main.ts');
  const loadBytes = section(main, 'async function loadBytes(', '\n/** 파일 메뉴 "최근 문서"');

  const blankIndex = loadBytes.indexOf('canvasView?.showBlankPage();');
  const parseIndex = loadBytes.indexOf('await loadDocumentForOpen(data, fileName);');

  assert.ok(blankIndex >= 0, '열기는 빈 쪽 상태부터 만들어야 한다');
  assert.ok(parseIndex > blankIndex, '빈 쪽 상태를 만든 뒤에 파싱해야 한다');
});

test('열기 실패는 이전 문서 뷰를 되살린다', () => {
  const main = source('src/main.ts');
  const restore = section(main, 'function restoreViewAfterFailedOpen', '\nfunction showLoadErrorUnlessCancelled');

  assert.match(restore, /if \(!canvasView \|\| wasm\.pageCount === 0\) return;/);
  assert.match(restore, /canvasView\.loadDocument\(\)/);

  const cancelled = section(main, 'function showLoadErrorUnlessCancelled', '\nasync function loadFile');
  assert.match(cancelled, /restoreViewAfterFailedOpen\(\);/);

  const failed = section(main, 'function showLoadError(', '\nconst initPromise');
  assert.match(failed, /restoreViewAfterFailedOpen\(\);/);
});
