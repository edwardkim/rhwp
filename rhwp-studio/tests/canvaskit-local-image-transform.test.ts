import test, { before, after } from 'node:test';
import assert from 'node:assert/strict';
import { fileURLToPath } from 'node:url';
import { createServer, type ViteDevServer } from 'vite';

let vite: ViteDevServer;
let prototype: Record<string, any>;
before(async () => {
  vite = await createServer({
    root: fileURLToPath(new URL('..', import.meta.url)),
    appType: 'custom', logLevel: 'silent', server: { middlewareMode: true },
  });
  prototype = (await vite.ssrLoadModule('/src/view/canvaskit-renderer.ts')).CanvasKitLayerRenderer.prototype;
});
after(async () => { await vite?.close(); });

// 실제 문서의 좌표가 아닌 행렬 계약 검사다. 원점 이동과 두 기저 벡터를 독립적으로 대조한다.
type Matrix = [number, number, number, number, number, number];
const identity = (): Matrix => [1, 0, 0, 1, 0, 0];
function multiply(a: Matrix, b: Matrix): Matrix {
  return [a[0] * b[0] + a[2] * b[1], a[1] * b[0] + a[3] * b[1],
    a[0] * b[2] + a[2] * b[3], a[1] * b[2] + a[3] * b[3],
    a[0] * b[4] + a[2] * b[5] + a[4], a[1] * b[4] + a[3] * b[5] + a[5]];
}

for (const [horzFlip, vertFlip] of [[false, false], [true, false], [false, true], [true, true]]) {
  test(`그림은 로컬 대칭 뒤 회전한다: 가로=${horzFlip}, 세로=${vertFlip}`, () => {
    let matrix = identity();
    const stack: Matrix[] = [];
    const concat = (m: Matrix) => { matrix = multiply(matrix, m); };
    const canvas = {
      save: () => { stack.push([...matrix]); },
      restore: () => { matrix = stack.pop()!; },
      translate: (x: number, y: number) => concat([1, 0, 0, 1, x, y]),
      scale: (x: number, y: number) => concat([x, 0, 0, y, 0, 0]),
      rotate: (degrees: number, x: number, y: number) => {
        const angle = degrees * Math.PI / 180;
        concat([1, 0, 0, 1, x, y]);
        concat([Math.cos(angle), Math.sin(angle), -Math.sin(angle), Math.cos(angle), 0, 0]);
        concat([1, 0, 0, 1, -x, -y]);
      },
    };
    const box = { x: 2, y: 3, width: 4, height: 6 };
    const center = [4, 6];
    const angle = 34 * Math.PI / 180;
    let calls = 0;
    prototype.withImageTransform.call({}, canvas, box, { rotation: 34, horzFlip, vertFlip }, () => {
      calls += 1;
      const fx = horzFlip ? -1 : 1;
      const fy = vertFlip ? -1 : 1;
      for (const [x, y] of [[0, 0], [1, 0], [0, 1]]) {
        const expected = [center[0] + Math.cos(angle) * fx * x - Math.sin(angle) * fy * y,
          center[1] + Math.sin(angle) * fx * x + Math.cos(angle) * fy * y];
        const input = [center[0] + x, center[1] + y];
        const actual = [matrix[0] * input[0] + matrix[2] * input[1] + matrix[4],
          matrix[1] * input[0] + matrix[3] * input[1] + matrix[5]];
        expected.forEach((value, i) => assert.ok(Math.abs(actual[i] - value) < 1e-10));
      }
    });
    assert.equal(calls, 1);
    assert.deepEqual(matrix, identity());
  });
}
