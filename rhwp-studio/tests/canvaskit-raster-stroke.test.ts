import assert from 'node:assert/strict';
import test from 'node:test';
import type { LayerLineOp } from '../src/core/types.ts';
import { rasterLineEndpoints as resolvedRasterLineEndpoints } from '../src/view/canvaskit/raster-stroke.ts';

const rasterLineEndpoints = (
  line: LayerLineOp,
  matrix: Parameters<typeof resolvedRasterLineEndpoints>[1],
  profile: Parameters<typeof resolvedRasterLineEndpoints>[2],
) => resolvedRasterLineEndpoints(line, matrix, profile, Math.max(0.3, line.style?.width ?? 1));

const identity = [1, 0, 0, 0, 1, 0, 0, 0, 1];
const line: LayerLineOp = {
  type: 'line', bbox: { x: 3, y: 10, width: 24, height: 0.48 },
  x1: 3, y1: 10, x2: 27, y2: 10,
  style: { color: '#bfbfbf', width: 0.48, dash: 'solid', lineType: 'single' },
};

test('인쇄의 subpixel 실선은 폭·길이를 유지하고 한 장치 픽셀 중심에 놓인다', () => {
  assert.deepEqual(rasterLineEndpoints(line, identity, 'print'), [3, 10.5, 27, 10.5]);
  assert.deepEqual(rasterLineEndpoints(line, [2, 0, 0, 0, 2, 0, 0, 0, 1], 'print'),
    [3, 10.25, 27, 10.25]);
  assert.equal(line.y1, 10, '원본 모델은 변경하지 않는다');
});

test('장치 변환의 이동·반전·직교 회전도 같은 물리 픽셀 중심을 소비한다', () => {
  assert.deepEqual(rasterLineEndpoints(line, [1, 0, 0, 0, 1, 0.25, 0, 0, 1], 'print'),
    [3, 10.25, 27, 10.25]);
  const result = rasterLineEndpoints(line, [0, -1, 40, 1, 0, 0, 0, 0, 1], 'print');
  assert.equal(40 - result[1], 30.5, '회전 뒤 법선의 물리 중심');
  assert.equal(result[2] - result[0], 24, '접선 길이 보존');
});

test('화면·굵은 선·복합선·화살표·그림자·사선은 원래 paint 좌표를 유지한다', () => {
  const original = [3, 10, 27, 10];
  assert.deepEqual(rasterLineEndpoints(line, identity, 'screen'), original);
  assert.deepEqual(rasterLineEndpoints(line, [3, 0, 0, 0, 3, 0, 0, 0, 1], 'print'), original);
  for (const style of [{ dash: 'dot' as const }, { lineType: 'double' },
    { startArrow: 'arrow' }, { shadow: {} }]) {
    assert.deepEqual(rasterLineEndpoints({ ...line, style: { ...line.style, ...style } },
      identity, 'print'), original);
  }
  const diagonal = { ...line, y2: 11 };
  assert.deepEqual(rasterLineEndpoints(diagonal, identity, 'print'), [3, 10, 27, 11]);
});

test('특이·원근·비유한 행렬과 누락된 변환은 새 비유한 좌표를 만들지 않는다', () => {
  for (const matrix of [undefined, [0, 0, 0, 0, 0, 0, 0, 0, 1],
    [1, 0, 0, 0, 1, 0, 0.01, 0, 1], [1, 0, Infinity, 0, 1, 0, 0, 0, 1]]) {
    assert.deepEqual(rasterLineEndpoints(line, matrix, 'print'), [3, 10, 27, 10]);
  }
});
