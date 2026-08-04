import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const rootDir = dirname(dirname(fileURLToPath(import.meta.url)));

function source(path: string): string {
  return readFileSync(join(rootDir, path), 'utf8');
}

// 수식 속성 다이얼로그의 글자 크기 입력은 min=1/max=127 속성을 갖지만,
// 이는 브라우저 UI(스피너/키보드 화살표)에서만 강제되고 Enter 키 등으로
// 프로그램적으로 제출되는 값은 클램프되지 않았다. handleOk()가 입력값을
// 그대로 100배해 wasm.setEquationProperties 로 넘기면 범위를 벗어난
// fontSize(예: 음수, 수만pt)가 CTRL_DATA에 기록될 수 있다.
// (page-setup-dialog #2845, table-cell-props-dialog #2847 등과 동일한
// "min/max 속성만 있고 confirm 시 clamp 누락" 패턴.)
test('수식 속성 다이얼로그 handleOk는 글자 크기를 1~127 범위로 clamp 한다', () => {
  const dialog = source('src/ui/equation-props-dialog.ts');

  const handleOkStart = dialog.indexOf('private handleOk');
  assert.notEqual(handleOkStart, -1, 'handleOk not found');
  const handleOk = dialog.slice(handleOkStart, dialog.indexOf('\n  private ', handleOkStart + 1));

  assert.match(
    handleOk,
    /Math\.max\(1,\s*Math\.min\(127,\s*parseInt\(this\.fontSizeInput\.value,\s*10\)\s*\|\|\s*10\)\)/,
    'handleOk는 fontSizeInput 값을 1~127 사이로 clamp 해야 함',
  );
});
