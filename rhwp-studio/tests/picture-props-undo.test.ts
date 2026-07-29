import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const rootDir = dirname(dirname(fileURLToPath(import.meta.url)));

function source(path: string): string {
  return readFileSync(join(rootDir, path), 'utf8');
}

function privateMethod(sourceText: string, name: string): string {
  const start = sourceText.indexOf(`private ${name}(`);
  assert.notEqual(start, -1, `${name} not found`);
  const end = sourceText.indexOf('\n  private ', start + 1);
  return sourceText.slice(start, end < 0 ? undefined : end);
}

// 그림 속성 다이얼로그와 그림 삽입 경로는 문서를 mutate 하므로 반드시
// 편집 라우터(executeOperation)를 통과해 undo 스택에 기록되어야 한다.
// (기존에는 wasm setter 직접 호출 + document-changed emit 만 수행되어,
// "본문과의 배치" 변경/그림 삽입이 Ctrl+Z 로 복구되지 않았다.)

test('handleOk는 non-empty patch만 apply orchestration에 위임한다', () => {
  const dialog = source('src/ui/picture-props-dialog.ts');
  const handleOk = privateMethod(dialog, 'handleOk');

  assert.match(
    handleOk,
    /if \(Object\.keys\(patch\)\.length > 0\) this\.applyPropertyPatch\(patch\);/,
    'empty patch는 apply orchestration을 호출하지 않아야 함',
  );
  assert.doesNotMatch(handleOk, /this\.wasm\./, 'handleOk가 WASM setter를 직접 호출하면 안 됨');
  assert.doesNotMatch(handleOk, /executeOperation/, 'handleOk가 undo 구현을 다시 소유하면 안 됨');
});

test('applyPropertyPatch는 스냅샷 undo와 no-service fallback을 유지한다', () => {
  const dialog = source('src/ui/picture-props-dialog.ts');
  const apply = privateMethod(dialog, 'applyPropertyPatch');

  assert.match(
    apply,
    /executeOperation\(\{\s*kind: 'snapshot',\s*operationType: 'objectProps'/,
    '개체 속성 적용은 편집 라우터의 snapshot 명령으로 기록되어야 함',
  );
  assert.match(
    apply,
    /operation: \(\) => \{\s*applyProps\(\);\s*return ih\.getCursorPosition\(\);/,
    'snapshot operation은 setter 적용 후 cursor를 반환해야 함',
  );
  assert.match(
    apply,
    /else \{\s*applyProps\(\);\s*this\.eventBus\.emit\('document-changed'\);/,
    'services 미주입 fallback은 직접 적용 후 document-changed를 emit해야 함',
  );
  assert.doesNotMatch(apply, /this\.wasm\./, 'undo orchestration이 WASM setter를 직접 소유하면 안 됨');
});

test('WASM mutation은 5개 target adapter에만 남는다', () => {
  const dialog = source('src/ui/picture-props-dialog.ts');
  const adapter = privateMethod(dialog, 'applyPropertyPatchToWasm');
  const setterCalls = [...adapter.matchAll(/\bthis\.wasm\./g)];

  assert.equal(setterCalls.length, 5, 'shape 2개와 picture 3개 setter를 유지해야 함');
  assert.match(
    adapter,
    /case 'cell-shape':[\s\S]*?setCellShapePropertiesByPath\(\s*target\.sec, target\.para, target\.cellPath, target\.innerControlIdx, patch,/,
    'cell-shape setter 인자 순서를 유지해야 함',
  );
  assert.match(
    adapter,
    /case 'body-shape':[\s\S]*?setShapeProperties\(target\.sec, target\.para, target\.ci, patch\)/,
    'body-shape setter 인자 순서를 유지해야 함',
  );
  assert.match(
    adapter,
    /case 'header-footer-picture':[\s\S]*?setHeaderFooterPictureProperties\(\s*target\.sec, target\.outerParaIdx, target\.outerControlIdx,\s*target\.para, target\.ci, patch,/,
    'header-footer picture의 5-tuple 인자 순서를 유지해야 함',
  );
  assert.match(
    adapter,
    /case 'cell-picture':[\s\S]*?setCellPicturePropertiesByPath\(\s*target\.sec, target\.para, target\.cellPath, target\.innerControlIdx, patch,/,
    'cell-picture setter 인자 순서를 유지해야 함',
  );
  assert.match(
    adapter,
    /case 'body-picture':[\s\S]*?setPictureProperties\(target\.sec, target\.para, target\.ci, patch\)/,
    'body-picture setter 인자 순서를 유지해야 함',
  );
  assert.equal(
    [...dialog.matchAll(/applyPropertyPatchToWasm\(/g)].length,
    2,
    'target adapter는 method 선언과 snapshot/fallback 공용 closure 호출만 존재해야 함',
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
