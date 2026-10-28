import assert from 'node:assert/strict';
import test from 'node:test';
import type { LayerLineOp } from '../src/core/types.ts';
import { rasterLineGeometry as resolvedGeometry } from '../src/view/canvaskit/raster-stroke.ts';

const geometry = (
  line: LayerLineOp,
  matrix: Parameters<typeof resolvedGeometry>[1],
  profile: Parameters<typeof resolvedGeometry>[2],
) => resolvedGeometry(line, matrix, profile, Math.max(0.3, line.style?.width ?? 1));

const identity = [1, 0, 0, 0, 1, 0, 0, 0, 1];
const line: LayerLineOp = {
  type: 'line', bbox: { x: 3, y: 10, width: 24, height: 0.48 },
  x1: 3, y1: 10, x2: 27, y2: 10,
  style: { color: '#bfbfbf', width: 0.48, dash: 'solid', lineType: 'single' },
};

// PDFium RasterizeStroke의 최소 장치 폭과 같은 원본 Print를 먼저 대조했다.
// 온새미로 전46쪽 Native94.65583/WASM94.55536%, 선을 읽은19쪽은 양쪽100%다.
test('인쇄의 가는 실선은 장치 픽셀 중심과 최소 폭을 함께 소비하고 원본 속성을 보존한다', () => {
  assert.deepEqual(geometry(line, identity, 'print'), [3, 10.5, 27, 10.5, 1]);
  assert.deepEqual(geometry(line, [2, 0, 0, 0, 2, 0, 0, 0, 1], 'print'),
    [3, 10.25, 27, 10.25, 0.5]);
  assert.equal(line.y1, 10, '레이아웃 원본 좌표는 바꾸지 않는다');
  assert.equal(line.style?.width, 0.48, '원본 선폭 속성도 바꾸지 않는다');
});

test('장치 이동·반전·직교 회전은 같은 법선 배율의 최소 폭을 소비한다', () => {
  assert.deepEqual(geometry(line, [1, 0, 0, 0, 1, 0.25, 0, 0, 1], 'print'),
    [3, 10.25, 27, 10.25, 1]);
  const rotated = geometry(line, [0, -2, 40, 2, 0, 0, 0, 0, 1], 'print');
  assert.deepEqual(rotated, [3, 9.75, 27, 9.75, 0.5]);
  assert.equal(rotated[4] * 2, 1, '회전 후 장치 법선 폭은 한 픽셀이다');
  assert.deepEqual(geometry(line, [-1, 0, 0, 0, -1, 0, 0, 0, 1], 'print'),
    [3, 9.5, 27, 9.5, 1]);
});

test('화면·굵은 선·복합선·화살표·그림자·사선은 원래 paint 좌표와 폭을 유지한다', () => {
  const original = [3, 10, 27, 10, 0.48];
  assert.deepEqual(geometry(line, identity, 'screen'), original);
  assert.deepEqual(geometry(line, [3, 0, 0, 0, 3, 0, 0, 0, 1], 'print'), original);
  for (const style of [{ dash: 'dot' as const }, { lineType: 'double' },
    { compoundComponent: true }, { startArrow: 'arrow' }, { shadow: {} }]) {
    assert.deepEqual(geometry({ ...line, style: { ...line.style, ...style } },
      identity, 'print'), original);
  }
  assert.deepEqual(geometry({ ...line, y2: 11 }, identity, 'print'), [3, 10, 27, 11, 0.48]);
});

test('특이·원근·비유한 행렬과 누락된 변환은 새 비유한 좌표나 폭을 만들지 않는다', () => {
  for (const matrix of [undefined, [0, 0, 0, 0, 0, 0, 0, 0, 1],
    [1, 0, 0, 0, 1, 0, 0.01, 0, 1], [1, 0, Infinity, 0, 1, 0, 0, 0, 1]]) {
    assert.deepEqual(geometry(line, matrix, 'print'), [3, 10, 27, 10, 0.48]);
  }
});
