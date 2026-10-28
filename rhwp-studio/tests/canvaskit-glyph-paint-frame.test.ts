import test, { before, after } from 'node:test';
import assert from 'node:assert/strict';
import { fileURLToPath } from 'node:url';
import { createServer, type ViteDevServer } from 'vite';

let vite: ViteDevServer;
let prototype: Record<string, any>;
before(async () => {
  vite = await createServer({ root: fileURLToPath(new URL('..', import.meta.url)),
    appType: 'custom', logLevel: 'silent', server: { middlewareMode: true } });
  prototype = (await vite.ssrLoadModule('/src/view/canvaskit-renderer.ts')).CanvasKitLayerRenderer.prototype;
});
after(async () => { await vite?.close(); });

// 공개 paint descriptor의 소비 경계 검사다. 실제 글꼴/문서 판독을 대신하지 않는다.
// 논리 점(10,20)을 이 독립 affine에 넣으면(11,16)이다. 뒤의 표 선은 원래 점이다.
const frame = { a: .8, b: 0, c: 0, d: .9, e: 3, f: -2 };
function canvasFixture() {
  let matrix = [1, 0, 0, 1, 0, 0];
  const stack: number[][] = [];
  return {
    save() { stack.push([...matrix]); },
    restore() { assert.ok(stack.length); matrix = stack.pop()!; },
    concat(values: number[]) {
      const [a, c, e, b, d, f] = values;
      const [ma, mb, mc, md, me, mf] = matrix;
      matrix = [ma * a + mc * b, mb * a + md * b, ma * c + mc * d,
        mb * c + md * d, ma * e + mc * f + me, mb * e + md * f + mf];
    },
    point(x: number, y: number) { return [matrix[0] * x + matrix[2] * y + matrix[4], matrix[1] * x + matrix[3] * y + matrix[5]]; },
    depth() { return stack.length; },
  };
}
for (const type of ['textRun', 'glyphRun', 'glyphOutline', 'charOverlap']) {
  test(`${type}: producer frame is applied once and source/border stay unchanged`, () => {
    const canvas = canvasFixture();
    const calls: number[][] = [];
    const op = { type, bbox: { x: 10, y: 20, width: 30, height: 12 }, paintTransform: frame };
    const unchanged = JSON.stringify(op);
    const renderer = Object.create(prototype);
    Object.assign(renderer, { currentOp: null,
      renderOp() { calls.push(canvas.point(10, 20)); } });
    renderer.renderLeaf(canvas, { kind: 'leaf', ops: [op, { type: 'line' }] }, 'print', 'flow', null);
    assert.deepEqual(calls, [[11, 16], [10, 20]]);
    assert.equal(canvas.depth(), 0);
    assert.equal(renderer.currentOp, null);
    assert.equal(JSON.stringify(op), unchanged);
  });
}
test('glyph failure restores the shared canvas and clears active op', () => {
  const canvas = canvasFixture();
  const renderer = Object.create(prototype);
  Object.assign(renderer, { currentOp: null, renderOp() { throw new Error('payload draw failed'); } });
  assert.throws(() => renderer.renderLeaf(canvas, { kind: 'leaf', ops: [{ type: 'textRun', paintTransform: frame }] }, 'print', 'flow', null), /payload draw failed/);
  assert.deepEqual(canvas.point(10, 20), [10, 20]);
  assert.equal(canvas.depth(), 0);
  assert.equal(renderer.currentOp, null);
});
for (const invalid of [{ ...frame, a: NaN }, { ...frame, f: Infinity }, { ...frame, a: 1e100 }, { ...frame, a: 0 }]) {
  test(`invalid glyph frame fails before drawing: ${JSON.stringify(invalid)}`, () => {
    const canvas = canvasFixture();
    let drawn = false;
    const renderer = Object.create(prototype);
    Object.assign(renderer, { currentOp: null, renderOp() { drawn = true; } });
    assert.throws(() => renderer.renderLeaf(canvas, { kind: 'leaf', ops: [{ type: 'textRun', paintTransform: invalid }] }, 'print', 'flow', null), /Invalid glyph paint transform/);
    assert.equal(drawn, false);
    assert.equal(canvas.depth(), 0);
    assert.equal(renderer.currentOp, null);
  });
}
