import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

// [Task #2908] 표/셀 속성 다이얼로그 "테두리" 탭: 굵기/색/선종류 컨트롤이 문서 값과
// 동기화되지 않는 결함의 소스 가드.
//
// populateBorderFromTarget()은 borderEdits 배열(및 SVG 미리보기)만 문서의 실제 테두리로
// 갱신하고 borderWidthSelect/borderColorInput/borderSelectedLineType은 buildBorderTab()의
// 하드코딩 기본값(0.1mm/#000000/실선)에 머무른다. applyBorderToDirection()은 바로 이
// 컨트롤들의 "현재 값"을 읽어 wasm.set*Properties에 보낼 borderEdits를 덮어쓰므로, 방향
// 버튼을 다시 누르면 기존 서식이 조용히 유실된다. 회귀 방지를 위해 populateBorderFromTarget
// 안에서 세 컨트롤이 대표 테두리 값으로 동기화되는지 정적으로 핀한다.

const rootDir = dirname(dirname(fileURLToPath(import.meta.url)));
const src = readFileSync(join(rootDir, 'src/ui/table-cell-props-dialog.ts'), 'utf8');

test('populateBorderFromTarget 은 굵기/색/선종류 컨트롤을 문서 테두리 값으로 동기화한다(#2908)', () => {
  const start = src.indexOf('private populateBorderFromTarget');
  assert.notEqual(start, -1, 'populateBorderFromTarget 메서드를 찾을 수 없음');
  const end = src.indexOf('\n  }', start);
  const body = src.slice(start, end);

  assert.match(
    body,
    /this\.borderWidthSelect\.value\s*=/,
    'populateBorderFromTarget 안에서 borderWidthSelect.value 를 문서 값으로 동기화해야 함',
  );
  assert.match(
    body,
    /this\.borderColorInput\.value\s*=/,
    'populateBorderFromTarget 안에서 borderColorInput.value 를 문서 값으로 동기화해야 함',
  );
  assert.match(
    body,
    /this\.borderSelectedLineType\s*=/,
    'populateBorderFromTarget 안에서 borderSelectedLineType 을 문서 값으로 동기화해야 함',
  );
});
