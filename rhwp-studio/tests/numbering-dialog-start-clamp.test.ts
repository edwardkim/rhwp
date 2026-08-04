import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

// [Issue #2838] numbering-dialog.ts '시작 번호' input(min=1,max=999)이 값 변경 시
// 범위를 clamp하지 않고 WASM createNumbering으로 그대로 전달되던 검증 누락 버그의 회귀 가드.
//
// 이 다이얼로그는 <form> submit 흐름이 없어 HTML min/max 속성이 강제되지 않으므로,
// input 이벤트 핸들러 자체가 1~999 범위로 clamp해야 한다.

const rootDir = dirname(dirname(fileURLToPath(import.meta.url)));
const src = readFileSync(join(rootDir, 'src/ui/numbering-dialog.ts'), 'utf8');

test('numbering-dialog.ts 시작 번호 input 핸들러는 1~999로 clamp한다', () => {
  const handlerMatch = src.match(
    /startInput\.addEventListener\('input', \(\) => \{([\s\S]*?)\}\);/,
  );
  assert.ok(handlerMatch, 'startInput input 핸들러를 찾을 수 없음');
  const body = handlerMatch![1];
  assert.match(
    body,
    /Math\.min\(\s*999\s*,\s*Math\.max\(\s*1\s*,/,
    'startNumber 대입 전에 Math.min(999, Math.max(1, ...)) clamp가 있어야 함',
  );
});
