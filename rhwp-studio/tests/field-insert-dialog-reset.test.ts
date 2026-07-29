import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const rootDir = dirname(dirname(fileURLToPath(import.meta.url)));

function source(path: string): string {
  return readFileSync(join(rootDir, path), 'utf8');
}

test('필드 입력 대화상자는 열릴 때마다 메모/이름/편집가능 값을 초기 상태로 리셋한다', () => {
  const dialog = source('src/ui/field-insert-dialog.ts');
  const showStart = dialog.indexOf('override show(): void');
  assert.notEqual(showStart, -1, 'show() 오버라이드를 찾을 수 있어야 한다');

  const showBody = dialog.slice(showStart);

  // 싱글턴으로 재사용되므로(commands/insert.ts), 매번 열릴 때 이전 삽입 값이 남지 않아야 한다.
  assert.match(showBody, /this\.memoInput\.value\s*=\s*''/);
  assert.match(showBody, /this\.nameInput\.value\s*=\s*''/);
  assert.match(showBody, /this\.editableCheckbox\.checked\s*=\s*true/);
});
