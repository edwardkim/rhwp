import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const rootDir = dirname(dirname(fileURLToPath(import.meta.url)));

function source(path: string): string {
  return readFileSync(join(rootDir, path), 'utf8');
}

// Issue #2037: 찾아 바꾸기/모두 바꾸기는 문서를 mutate 하므로 반드시 편집 라우터
// (executeOperation)를 통과해 undo 스택에 기록되어야 한다 (#1320 계약).
// 기존에는 wasm.replaceText/replaceAll 직접 호출 + document-changed emit 만
// 수행되어, 대량 치환이 Ctrl+Z 로 복구되지 않았다.

function methodBlock(src: string, methodName: string): string {
  const start = src.indexOf(`private ${methodName}`);
  assert.notEqual(start, -1, `${methodName} not found`);
  const next = src.indexOf('\n  private ', start + 1);
  return src.slice(start, next === -1 ? undefined : next);
}

test('바꾸기(doReplace)는 스냅샷으로 undo 기록된다', () => {
  const dialog = source('src/ui/find-dialog.ts');
  const block = methodBlock(dialog, 'doReplace');

  assert.match(
    block,
    /executeOperation\(\{ kind: 'snapshot', operationType: 'replaceText'/,
    '바꾸기는 편집 라우터의 snapshot 명령으로 기록되어야 함',
  );
});

test('모두 바꾸기(doReplaceAll)는 스냅샷으로 undo 기록된다', () => {
  const dialog = source('src/ui/find-dialog.ts');
  const block = methodBlock(dialog, 'doReplaceAll');

  assert.match(
    block,
    /executeOperation\(\{ kind: 'snapshot', operationType: 'replaceAll'/,
    '모두 바꾸기는 편집 라우터의 snapshot 명령으로 기록되어야 함',
  );
});
