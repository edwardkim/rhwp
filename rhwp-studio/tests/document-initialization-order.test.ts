import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const rootDir = dirname(dirname(fileURLToPath(import.meta.url)));

function source(path: string): string {
  return readFileSync(join(rootDir, path), 'utf8');
}

function initializeDocumentSource(): string {
  const main = source('src/main.ts');
  const start = main.indexOf('async function initializeDocument');
  const end = main.indexOf('\nasync function promptLocalFontsIfNeeded', start);
  assert.ok(start >= 0 && end > start, 'initializeDocument 범위를 찾을 수 있어야 한다');
  return main.slice(start, end);
}

test('문서 초기화는 로컬 글꼴 확인 후에만 입력 핸들러를 활성화한다', () => {
  const initializeDocument = initializeDocumentSource();
  const promptIndex = initializeDocument.indexOf('await promptLocalFontsIfNeeded(docInfo, displayName);');
  const activateIndex = initializeDocument.indexOf('inputHandler?.activateWithCaretPosition();');
  const completeIndex = initializeDocument.indexOf("documentState.markClean('document-initialized');");

  assert.ok(promptIndex >= 0, '로컬 글꼴 확인 단계가 있어야 한다');
  assert.ok(activateIndex > promptIndex, '로컬 글꼴 확인 뒤에 캐럿을 활성화해야 한다');
  assert.ok(completeIndex > activateIndex, '편집 준비 뒤에 문서 초기화를 완료해야 한다');
  assert.doesNotMatch(
    initializeDocument,
    /updateLoadProgress\(100, '완료'\)/,
    '최종 파일명 전환 전에 불필요한 100% paint 대기를 두지 않는다',
  );
});

test('CanvasKit local face 등록은 문서 초기화 대신 현재 뷰 재그리기를 요청한다', () => {
  const main = source('src/main.ts');
  const start = main.indexOf('function prepareCanvasKitLocalFonts');
  const end = main.indexOf('\nasync function initialize()', start);
  assert.ok(start >= 0 && end > start, 'CanvasKit local face 준비 함수를 찾을 수 있어야 한다');
  const prepareLocalFonts = main.slice(start, end);

  assert.match(prepareLocalFonts, /eventBus\.emit\('document-view-changed'\);/);
  assert.doesNotMatch(prepareLocalFonts, /canvasView\?\.loadDocument\(\);/);
});

test('로컬 글꼴 감지는 Canvas2D 문서를 전체 재로딩하지 않는다', () => {
  const main = source('src/main.ts');
  assert.doesNotMatch(main, /eventBus\.on\('local-fonts-changed',[\s\S]*?canvasView\?\.loadDocument\(\);/);
});
