import assert from 'node:assert/strict';

// 표식이 가진 줄 내부 기준선을 소비하는 경로와 기존 fallback을 구분한다.
export function assertFootnoteMarkerBaselineReplay(Renderer) {
  const renderer = new Renderer({}, 'default', {}, {});
  const replayed = [];
  renderer.renderTextRun = (_canvas, op) => replayed.push(op);
  const bbox = { x: 12, y: 80, width: 10, height: 12 };
  const inputs = [
    { baseline: 4, fontSize: 10 },
    { baseline: 0, fontSize: 10 },
    { fontSize: 10 },
    {},
  ];
  for (const input of inputs) {
    renderer.renderOp({}, {
      type: 'footnoteMarker', bbox, text: '1)', color: '#123456', ...input,
    }, 'default');
  }
  assert.equal(replayed.length, inputs.length);
  assert.equal(replayed[0].baseline, inputs[0].baseline,
    '표식의 저장 기준선을 fontSize로 덮어쓰면 안 된다');
  assert.equal(replayed[1].baseline, inputs[1].baseline,
    '유효한 기준선 0을 fallback으로 바꾸면 안 된다');
  assert.equal(replayed[2].baseline, inputs[2].fontSize);
  assert.equal(replayed[3].baseline, 7, '기존 기본 기준선 계약을 보존한다');
  for (const op of replayed) {
    assert.deepEqual(op.bbox, bbox);
    assert.equal(op.text, '1)');
    assert.equal(op.style.color, '#123456');
  }
}
