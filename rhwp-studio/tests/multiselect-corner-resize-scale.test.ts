import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

// [Issue #2823] 다중 선택 코너 리사이즈: 드래그 확정(finishPictureResizeDrag)이 라이브 프리뷰
// (updatePictureResizeDrag)와 동일하게 코너에서 scaleX/scaleY 를 독립적으로 적용해야 한다.
// 회귀 시 finish 단계에서 sy 에 scaleX 가 재사용되어(복붙 실수) 세로 크기가 가로 배율로
// 잘못 스냅되며, 라이브 프리뷰와 확정 결과가 어긋난다. 소스 가드로 재발을 막는다.

const rootDir = dirname(dirname(fileURLToPath(import.meta.url)));
const src = readFileSync(join(rootDir, 'src/engine/input-handler-picture.ts'), 'utf8');

test('finishPictureResizeDrag 다중 선택 코너 분기는 sy 에 scaleY 를 사용한다', () => {
  const start = src.indexOf('export function finishPictureResizeDrag');
  assert.notEqual(start, -1, 'finishPictureResizeDrag 가 존재해야 함');
  const idx = src.indexOf("const sy = isCorner ?", start);
  assert.notEqual(idx, -1, 'sy 계산식을 찾아야 함');
  const line = src.slice(idx, src.indexOf('\n', idx));
  assert.match(line, /isCorner \? scaleY/, 'sy 는 코너일 때 scaleY 를 사용해야 함(scaleX 재사용 회귀 방지)');
});
