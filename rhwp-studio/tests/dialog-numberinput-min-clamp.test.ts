import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

// [Issue #2845] page-setup-dialog.ts / table-cell-props-dialog.ts 의 공유 numberInput()
// 헬퍼가 <input type="number" min="0"> 을 만들지만, HTML min 속성은 .value 를 자동
// clamp하지 않는다(브라우저는 checkValidity() 호출 시에만 검사). onConfirm() 이
// input.value 를 그대로 parseFloat 하여 wasm.setPageDef / setCellProperties /
// setTableProperties 로 넘기므로, 직접 타이핑한 음수 값이 검증 없이 WASM까지 전달되던
// 버그의 회귀 가드. numberInput() 자체가 change 시 min/max로 clamp해야 한다.

const rootDir = dirname(dirname(fileURLToPath(import.meta.url)));

function readNumberInputBody(relPath: string): string {
  const src = readFileSync(join(rootDir, relPath), 'utf8');
  const match = src.match(/private numberInput\(\): HTMLInputElement \{([\s\S]*?)\n  \}/);
  assert.ok(match, `${relPath}: numberInput() 정의를 찾을 수 없음`);
  return match![1];
}

test('page-setup-dialog.ts numberInput()은 change 시 min/max로 값을 clamp한다', () => {
  const body = readNumberInputBody('src/ui/page-setup-dialog.ts');
  assert.match(
    body,
    /addEventListener\('change'/,
    'numberInput()에 change 리스너가 있어야 함',
  );
  assert.match(
    body,
    /Math\.min\(max, Math\.max\(min, v\)\)/,
    'change 리스너가 min/max로 값을 clamp해야 함',
  );
});

test('table-cell-props-dialog.ts numberInput()은 change 시 min/max로 값을 clamp한다', () => {
  const body = readNumberInputBody('src/ui/table-cell-props-dialog.ts');
  assert.match(
    body,
    /addEventListener\('change'/,
    'numberInput()에 change 리스너가 있어야 함',
  );
  assert.match(
    body,
    /Math\.min\(max, Math\.max\(min, v\)\)/,
    'change 리스너가 min/max로 값을 clamp해야 함',
  );
});
