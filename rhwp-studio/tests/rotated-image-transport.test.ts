import test from 'node:test';
import assert from 'node:assert/strict';
import { collectFlowImagePaintOps, flowImageOpsFromNarrowQuery } from '../src/view/flow-image-clip.ts';

// 합성 직렬화 필드의 전달 계약이다. 실물 페이지의 픽셀 좌표를 고정하는 검사가 아니다.
const frame = { x: 10, y: 20, width: 150, height: 140 };
const paint = { x: 35, y: 50, width: 100, height: 80 };

function transports(extra: Record<string, unknown>) {
  const fields = { bbox: frame, transform: { rotation: 34 }, mime: 'image/png', ...extra };
  const full = collectFlowImagePaintOps({ kind: 'leaf', ops: [{
    ...fields, type: 'image', base64: 'AA==',
  }] }, op => op.type === 'image');
  const narrow = flowImageOpsFromNarrowQuery(JSON.stringify({ cacheable: true, images: [{
    ...fields, sourceImageKey: 'bin:0:1:src',
  }] }), key => `blob:${key}`);
  assert.ok(narrow);
  assert.equal(full.length, 1);
  assert.equal(narrow.length, 1);
  return [full[0], narrow[0]];
}

test('두 그림 전달 경로는 외접 프레임과 회전 전 출력 상자를 각각 보존한다', () => {
  for (const image of transports({ paintBbox: paint })) {
    assert.deepEqual(image.bbox, frame);
    assert.deepEqual(image.paintBbox, paint);
    assert.equal(image.rotation, 34);
    assert.notDeepEqual(image.bbox, image.paintBbox);
  }
});

test('회전 전 출력 상자가 없는 이전 생산자의 계약을 유지한다', () => {
  for (const image of transports({})) {
    assert.deepEqual(image.bbox, frame);
    assert.equal(image.paintBbox, undefined);
  }
});

test('유효하지 않은 선택 필드는 외접 프레임을 덮어쓰지 않는다', () => {
  for (const paintBbox of [null, 'invalid', { ...paint, x: 'invalid' }]) {
    for (const image of transports({ paintBbox })) {
      assert.deepEqual(image.bbox, frame);
      assert.equal(image.paintBbox, undefined);
    }
  }
});
