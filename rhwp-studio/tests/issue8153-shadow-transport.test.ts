// 공개 paint 표본의 소비 계약이다. 실제 문서·글꼴·시각 검증을 대신하지 않는다.
import test, { before, after } from 'node:test';
import assert from 'node:assert/strict';
import { fileURLToPath } from 'node:url';
import { createServer, type ViteDevServer } from 'vite';
import { textShadowOffsets } from '../src/view/canvaskit/text-shadow.ts';

let vite: ViteDevServer;
let prototype: Record<string, any>;
before(async () => {
  vite = await createServer({ root: fileURLToPath(new URL('..', import.meta.url)),
    appType: 'custom', logLevel: 'silent', server: { middlewareMode: true } });
  prototype = (await vite.ssrLoadModule('/src/view/canvaskit-renderer.ts')).CanvasKitLayerRenderer.prototype;
});
after(async () => { await vite?.close(); });

test('혼합 축·음수의 독립 Print 표본을 순서대로 재생하며 보간을 다시 하지 않는다', () => {
  const samples: [number, number][] = [[0, 0], [0, 1], [-1, 2], [-1, 3], [-1, 4], [-2, 5]];
  assert.deepEqual(textShadowOffsets({ shadowType: 2, shadowOffsetX: 99, shadowOffsets: samples }), samples);
  assert.deepEqual(samples, [[0, 0], [0, 1], [-1, 2], [-1, 3], [-1, 4], [-2, 5]]);
});

test('원본 없음·비연속은 남아 있는 연속 표본보다 우선한다', () => {
  const style = { shadowOffsets: [[1, 1], [2, 2]] as [number, number][], shadowOffsetX: 3, shadowOffsetY: -2 };
  assert.deepEqual(textShadowOffsets({ ...style, shadowType: 0 }), []);
  assert.deepEqual(textShadowOffsets({ ...style, shadowType: 1 }), [[3, -2]]);
  assert.deepEqual(textShadowOffsets({ shadowType: 2, shadowOffsetX: 3, shadowOffsetY: -2 }), [[3, -2]]);
});

test('유효하지 않은 표본을 다른 표본이나 임의 좌표로 바꾸지 않는다', () => {
  for (const shadowOffsets of [[], [[NaN, 0]], [[0, Infinity]], [[1]], 'invalid']) {
    assert.throws(() => textShadowOffsets({ shadowType: 2, shadowOffsets: shadowOffsets as any }), /Invalid resolved text shadow offsets/);
  }
});

for (const kind of [0, 1, 2]) {
  test(`실제 glyph 소비자는 종류 ${kind}의 그림자와 원문을 각각 정해진 횟수만 그린다`, () => {
    const renderer = Object.create(prototype);
    const calls: { x: number; y: number; color: string }[] = [];
    let depth = 0;
    Object.assign(renderer, { glyphRunFonts: { font: () => ({}) },
      currentFontResources: undefined, currentReplayFeatureCounts: { glyphRuns: 0 },
      unsupportedOps: new Set(), makeFillPaint: (color: string) => ({ color, delete() {} }) });
    const canvas = { save() { depth++; }, restore() { depth--; }, concat(_matrix: number[]) {},
      drawGlyphs(_ids: unknown, _positions: unknown, x: number, y: number, _font: unknown,
        paint: { color: string }) { calls.push({ x, y, color: paint.color }); } };
    const offsets = [[0, 0], [0, 1], [-1, 2], [-1, 3], [-1, 4], [-2, 5]];
    const op = { glyphIds: [1], positions: [{ x: 0, y: 0 }], placement: {
      runToPage: { a: 1, b: 0, c: 0, d: 1, e: 0, f: 0 }, baselineY: 10 },
      paintStyle: { fontSize: 12, color: '#000000', shadowType: kind, shadowColor: '#123456',
        shadowOffsetX: -2, shadowOffsetY: 5, shadowOffsets: offsets } };
    renderer.renderGlyphRun(canvas, op);
    const shadow = kind === 2 ? offsets : kind === 1 ? [[-2, 5]] : [];
    assert.deepEqual(calls, [...shadow.map(([x, y]) => ({ x, y: y + 10, color: '#123456' })),
      { x: 0, y: 10, color: '#000000' }]);
    assert.equal(depth, 0);
    assert.equal(renderer.unsupportedOps.size, 0);
    assert.equal(renderer.currentReplayFeatureCounts.glyphRuns, 1);
  });
}
