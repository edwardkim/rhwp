import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const rootDir = dirname(dirname(fileURLToPath(import.meta.url)));

function source(path: string): string {
  return readFileSync(join(rootDir, path), 'utf8');
}

// 그림 속성 다이얼로그와 그림 삽입 경로는 문서를 mutate 하므로 반드시
// 편집 라우터(executeOperation)를 통과해 undo 스택에 기록되어야 한다.
// (기존에는 wasm setter 직접 호출 + document-changed emit 만 수행되어,
// "본문과의 배치" 변경/그림 삽입이 Ctrl+Z 로 복구되지 않았다.)

test('그림 속성 다이얼로그 apply는 스냅샷으로 undo 기록된다', () => {
  const dialog = source('src/ui/picture-props-dialog.ts');

  const handleOkStart = dialog.indexOf('private handleOk');
  assert.notEqual(handleOkStart, -1, 'handleOk not found');
  const handleOk = dialog.slice(handleOkStart, dialog.indexOf('\n  private ', handleOkStart + 1));

  assert.match(
    handleOk,
    /executeOperation\(\{\s*kind: 'snapshot',\s*operationType: 'objectProps'/,
    '개체 속성 적용은 편집 라우터의 snapshot 명령으로 기록되어야 함',
  );
});

test('그림 속성 다이얼로그 생성자는 CommandServices를 전달받는다', () => {
  const insert = source('src/command/commands/insert.ts');
  const format = source('src/command/commands/format.ts');

  assert.match(
    insert,
    /new PicturePropsDialog\(services\.wasm, services\.eventBus, services\)/,
    'insert:picture-props 진입점에서 services 전달 필요',
  );
  assert.match(
    format,
    /new PicturePropsDialog\(services\.wasm, services\.eventBus, services\)/,
    'format:object-properties 진입점에서 services 전달 필요',
  );
});

test('배치모드 그림 삽입은 스냅샷으로 undo 기록된다', () => {
  const table = source('src/engine/input-handler-table.ts');

  const start = table.indexOf('export function finishImagePlacement');
  assert.notEqual(start, -1, 'finishImagePlacement not found');
  const block = table.slice(start, table.indexOf('\nexport function ', start + 1));

  assert.match(
    block,
    /executeOperation\(\{ kind: 'snapshot', operationType: 'insertPicture'/,
    '배치모드 그림 삽입은 snapshot 명령으로 기록되어야 함',
  );
});

test('드래그드롭 그림 삽입은 스냅샷으로 undo 기록된다', () => {
  const ih = source('src/engine/input-handler.ts');

  const start = ih.indexOf('insertDroppedImageAtClientPoint(');
  assert.notEqual(start, -1, 'insertDroppedImageAtClientPoint not found');
  const block = ih.slice(start, ih.indexOf('\n  private ', start + 1));

  assert.match(
    block,
    /executeOperation\(\{ kind: 'snapshot', operationType: 'insertPicture'/,
    '드래그드롭 그림 삽입은 snapshot 명령으로 기록되어야 함',
  );
});
