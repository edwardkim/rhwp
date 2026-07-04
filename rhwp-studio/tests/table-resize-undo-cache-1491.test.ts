import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const rootDir = dirname(dirname(fileURLToPath(import.meta.url)));

function source(path: string): string {
  return readFileSync(join(rootDir, path), 'utf8');
}

function methodBlock(methodName: string): string {
  const inputHandler = source('src/engine/input-handler.ts');
  const start = inputHandler.indexOf(`private ${methodName}`);
  assert.notEqual(start, -1, `${methodName} not found`);
  const next = inputHandler.indexOf('\n  private ', start + 1);
  return inputHandler.slice(start, next === -1 ? undefined : next);
}

function constructorEventsBlock(): string {
  const inputHandler = source('src/engine/input-handler.ts');
  const start = inputHandler.indexOf("eventBus.on('document-changed'");
  assert.notEqual(start, -1, 'document-changed event block not found');
  const end = inputHandler.indexOf('// [Task #394]', start);
  assert.notEqual(end, -1, 'event block end not found');
  return inputHandler.slice(start, end);
}

// #1491 Stage 1: undo 후 stale local resize segment가 다음 resize에 재적용되면 안 된다.
test('undo/redo는 표 로컬 resize 런타임 캐시를 비운다', () => {
  const undo = methodBlock('handleUndo');
  const redo = methodBlock('handleRedo');

  assert.match(undo, /clearTableResizeRuntimeCache\(\)/, 'undo 후 local resize 캐시를 비워야 함');
  assert.match(redo, /clearTableResizeRuntimeCache\(\)/, 'redo 후 local resize 캐시를 비워야 함');
});

test('표 resize 런타임 캐시 정리는 local segment와 bbox 캐시를 함께 비운다', () => {
  const clear = methodBlock('clearTableResizeRuntimeCache');

  assert.match(clear, /tableLocalResizeSegments\.clear\(\)/, 'local resize segment 캐시 삭제 필요');
  assert.match(clear, /cachedTableRef = null/, '표 ref 캐시 삭제 필요');
  assert.match(clear, /cachedCellBboxes = null/, 'bbox 캐시 삭제 필요');
  assert.match(clear, /tableResizeRenderer\?\.clear\(\)/, 'hover/drag marker 삭제 필요');
});

test('문서 전환 이벤트도 표 resize 런타임 캐시 정리 helper를 사용한다', () => {
  const events = constructorEventsBlock();

  assert.match(events, /create-new-document'[\s\S]*clearTableResizeRuntimeCache\(\)/, '새 문서에서 캐시 정리 필요');
  assert.match(events, /open-document-bytes'[\s\S]*clearTableResizeRuntimeCache\(\)/, '문서 열기에서 캐시 정리 필요');
});
