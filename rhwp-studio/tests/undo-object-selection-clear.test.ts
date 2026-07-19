import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const rootDir = dirname(dirname(fileURLToPath(import.meta.url)));

function source(path: string): string {
  return readFileSync(join(rootDir, path), 'utf8');
}

// Issue #2303: undo/redo 는 문단 컨트롤 구성을 되돌릴 수 있으므로, 위치 기반
// 개체/표 선택 ref({sec, ppi, ci})를 유지하면 stale 이 되어 이후 개체 속성
// (format:object-properties)이 WASM 예외("지정된 컨트롤이 그림이 아닙니다")로
// 실패한다. 히스토리 점프가 실제 수행될 때 선택 모드를 해제해야 한다.
// (실동작 검증은 e2e/undo-object-selection.test.mjs)

function methodBlock(src: string, methodName: string): string {
  const start = src.indexOf(`private ${methodName}`);
  assert.notEqual(start, -1, `${methodName} not found`);
  const next = src.indexOf('\n  private ', start + 1);
  return src.slice(start, next === -1 ? undefined : next);
}

test('handleUndo 는 히스토리 점프 시 개체/표 선택을 해제한다', () => {
  const handler = source('src/engine/input-handler.ts');
  const block = methodBlock(handler, 'handleUndo');

  assert.match(
    block,
    /resetDerivedStateAfterHistoryJump\(\)/,
    'undo 는 stale 선택 ref 를 남기지 않도록 선택 해제를 호출해야 함',
  );
});

test('handleRedo 는 히스토리 점프 시 개체/표 선택을 해제한다', () => {
  const handler = source('src/engine/input-handler.ts');
  const block = methodBlock(handler, 'handleRedo');

  assert.match(
    block,
    /resetDerivedStateAfterHistoryJump\(\)/,
    'redo 는 stale 선택 ref 를 남기지 않도록 선택 해제를 호출해야 함',
  );
});

test('선택 해제 헬퍼는 개체·표 선택 모두 정리하고 상태 이벤트를 발행한다', () => {
  const handler = source('src/engine/input-handler.ts');
  const block = methodBlock(handler, 'resetDerivedStateAfterHistoryJump');

  assert.match(block, /exitPictureObjectSelection\(\)/, '그림/도형 선택 해제');
  assert.match(block, /pictureObjectRenderer\?\.clear\(\)/, '선택 핸들 렌더러 정리');
  assert.match(
    block,
    /emit\('picture-object-selection-changed', false\)/,
    '개체 선택 상태 이벤트 발행 (도구 상자/메뉴 활성화 갱신)',
  );
  assert.match(block, /exitTableObjectSelection\(\)/, '표 선택 해제');
  assert.match(
    block,
    /emit\('table-object-selection-changed', false\)/,
    '표 선택 상태 이벤트 발행',
  );
});
