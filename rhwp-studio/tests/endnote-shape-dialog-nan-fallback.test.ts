import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

// [Task #3041] 미주 모양 대화상자 간격 입력값 NaN 폴백 가드.
//
// separatorLength/separatorMarginTop/noteSpacing/separatorMarginBottom 은
// mmToHwp(parseFloat(input.value)) 형태였다. 입력을 비운 채 확인하면
// parseFloat('') === NaN 이 그대로 mmToHwp에 전달되어 설정에 NaN이 저장됐다.
// column-settings-dialog.ts/page-setup-dialog.ts 등 다른 다이얼로그의 관례대로
// `parseFloat(...) || 0` 폴백을 적용한다.

const rootDir = dirname(dirname(fileURLToPath(import.meta.url)));
const src = readFileSync(join(rootDir, 'src/ui/endnote-shape-dialog.ts'), 'utf8');

const FIELDS = [
  'separatorLengthInput',
  'marginTopInput',
  'noteSpacingInput',
  'marginBottomInput',
];

for (const field of FIELDS) {
  test(`endnote-shape-dialog.ts: ${field} 파싱은 NaN 폴백을 갖춘다`, () => {
    assert.match(
      src,
      new RegExp(`parseFloat\\(this\\.${field}\\.value\\) \\|\\| 0`),
      `${field}: parseFloat(...) || 0 폴백 누락`,
    );
  });
}
