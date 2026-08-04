import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

// [Task #2915] 문단 모양 다이얼로그 "테두리/배경" 탭: 굵기/색/선종류 컨트롤이 문서 값과
// 동기화되지 않는 결함의 소스 가드.
//
// populateFromProps()는 borderStates/bdSideToggles(및 미리보기)만 문서의 실제 테두리로
// 갱신하고 buildBorderTab()의 bdTypeSelect/bdWidthSelect/bdColorInput 은 하드코딩 기본값
// (선 없음/0/#000000)에 머무른다. onBorderControlChange()/applyBorderPreset()은 바로 이
// 컨트롤들의 "현재 값"을 읽어 borderStates 를 덮어쓰므로, 프리셋/토글을 재적용하면 기존
// 서식이 조용히 유실된다 (#2908/#2913과 동일 패턴). 회귀 방지를 위해 populateFromProps
// 안에서 세 컨트롤이 대표 테두리 값으로 동기화되는지 정적으로 핀한다.

const rootDir = dirname(dirname(fileURLToPath(import.meta.url)));
const src = readFileSync(join(rootDir, 'src/ui/para-shape-dialog.ts'), 'utf8');

test('populateFromProps 는 굵기/색/선종류 컨트롤을 문서 테두리 값으로 동기화한다(#2915)', () => {
  const start = src.indexOf('private populateFromProps');
  assert.notEqual(start, -1, 'populateFromProps 메서드를 찾을 수 없음');
  const end = src.indexOf('\n  }', start);
  const body = src.slice(start, end);

  assert.match(
    body,
    /this\.borderResult\.bdTypeSelect\.value\s*=/,
    'populateFromProps 안에서 bdTypeSelect.value 를 문서 값으로 동기화해야 함',
  );
  assert.match(
    body,
    /this\.borderResult\.bdWidthSelect\.value\s*=/,
    'populateFromProps 안에서 bdWidthSelect.value 를 문서 값으로 동기화해야 함',
  );
  assert.match(
    body,
    /this\.borderResult\.bdColorInput\.value\s*=/,
    'populateFromProps 안에서 bdColorInput.value 를 문서 값으로 동기화해야 함',
  );
});
