import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const rootDir = dirname(dirname(fileURLToPath(import.meta.url)));

function source(path: string): string {
  return readFileSync(join(rootDir, path), 'utf8');
}

// 수식 속성 다이얼로그는 문서를 mutate 하므로 그림 속성 다이얼로그(#2028)와 동일하게
// 편집 라우터(executeOperation)를 통과해 undo 스택에 기록되어야 한다.
// (기존에는 wasm setEquationProperties 직접 호출 + document-changed emit 만 수행되어,
// 수식 속성 변경이 Ctrl+Z 로 복구되지 않았다.)

test('수식 속성 다이얼로그 apply는 스냅샷으로 undo 기록된다', () => {
  const dialog = source('src/ui/equation-props-dialog.ts');

  const handleOkStart = dialog.indexOf('private handleOk');
  assert.notEqual(handleOkStart, -1, 'handleOk not found');
  const handleOk = dialog.slice(handleOkStart, dialog.indexOf('\n  private ', handleOkStart + 1));

  assert.match(
    handleOk,
    /executeOperation\(\{\s*kind: 'snapshot',\s*operationType: 'objectProps'/,
    '수식 속성 적용은 편집 라우터의 snapshot 명령으로 기록되어야 함',
  );
});

test('수식 속성 다이얼로그 생성자는 CommandServices를 전달받는다', () => {
  const insert = source('src/command/commands/insert.ts');
  const format = source('src/command/commands/format.ts');

  assert.match(
    insert,
    /new EquationPropertiesDialog\(services\.wasm, services\.eventBus, services\)/,
    'insert 진입점에서 services 전달 필요',
  );
  assert.match(
    format,
    /new EquationPropertiesDialog\(services\.wasm, services\.eventBus, services\)/,
    'format:object-properties 진입점에서 services 전달 필요',
  );
});
