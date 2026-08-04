import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

// [Issue #2938] section-settings-dialog.ts 의 columnSpacingInput / defaultTabSpacingInput 은
// numberInput() 헬퍼로 min='0'을 갖지만, HTML min 속성은 .value 를 자동 clamp하지 않는다
// (#2845/#2847과 동일 패턴). onConfirm() 이 clamp 없이 parseFloat 결과를 그대로
// wasm.setSectionDef 로 전달하면 직접 타이핑한 음수가 WASM까지 전달된다. 이 회귀 가드는
// onConfirm() 이 두 필드 모두 Math.max(0, ...)로 하한을 강제하는지 확인한다.

const rootDir = dirname(dirname(fileURLToPath(import.meta.url)));

function readOnConfirmBody(): string {
  const src = readFileSync(join(rootDir, 'src/ui/section-settings-dialog.ts'), 'utf8');
  const match = src.match(/protected onConfirm\(\): void \{([\s\S]*?)\n  \}/);
  assert.ok(match, 'onConfirm() 정의를 찾을 수 없음');
  return match![1];
}

test('section-settings-dialog.ts onConfirm()은 columnSpacing을 0 이상으로 clamp한다', () => {
  const body = readOnConfirmBody();
  assert.match(
    body,
    /columnSpacing:\s*Math\.max\(0,\s*ptToHwpunit\(parseFloat\(this\.columnSpacingInput\.value\)\s*\|\|\s*0\)\)/,
    'columnSpacing 계산이 Math.max(0, ...)로 하한 clamp되어야 함',
  );
});

test('section-settings-dialog.ts onConfirm()은 defaultTabSpacing을 0 이상으로 clamp한다', () => {
  const body = readOnConfirmBody();
  assert.match(
    body,
    /defaultTabSpacing:\s*Math\.max\(0,\s*ptToHwpunit\(parseFloat\(this\.defaultTabSpacingInput\.value\)\s*\|\|\s*0\)\)/,
    'defaultTabSpacing 계산이 Math.max(0, ...)로 하한 clamp되어야 함',
  );
});
