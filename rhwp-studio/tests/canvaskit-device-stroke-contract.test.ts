import test, { before, after } from 'node:test';
import assert from 'node:assert/strict';
import { createRequire } from 'node:module';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { createServer, type ViteDevServer } from 'vite';
import CanvasKitInit from 'canvaskit-wasm';

let vite: ViteDevServer;
let prototype: Record<string, any>;
let kit: Awaited<ReturnType<typeof CanvasKitInit>>;
before(async () => {
  vite = await createServer({ root: fileURLToPath(new URL('..', import.meta.url)),
    appType: 'custom', logLevel: 'silent', server: { middlewareMode: true } });
  prototype = (await vite.ssrLoadModule('/src/view/canvaskit-renderer.ts')).CanvasKitLayerRenderer.prototype;
  const directory = dirname(createRequire(import.meta.url).resolve('canvaskit-wasm'));
  kit = await CanvasKitInit({ locateFile: (name) => join(directory, name) });
});
after(async () => { await vite?.close(); });

function renderer() {
  return Object.assign(Object.create(prototype), {
    canvasKit: kit, currentDeviceStrokeFloor: 1, recordingStrokeDeviceMatrix: null,
    unsupportedOps: new Set(), currentReplayFeatureCounts: { dashedStrokes: 0 },
  });
}

test('Print clip은 전달된 세대 정책에 따라 원본 또는 장치 경계를 선택한다', () => {
  const value = renderer();
  value.renderNode = () => {};
  let seen: number[] = [];
  const canvas = {
    save() {}, restore() {},
    clipRect(rect: ArrayLike<number>) { seen = Array.from(rect); },
    getTotalMatrix() { throw new Error('장치 경계 변환 호출'); },
  };
  const node = { kind: 'clipRect', clip: { x: 10.25, y: 20.5, width: 30.25, height: 40.5 }, child: {} };
  value.currentRasterClipCoverage = false;
  value.renderClipNode(canvas, node, 'print', 'flow', null);
  assert.deepEqual(seen, [10.25, 20.5, 40.5, 61]);
  value.currentRasterClipCoverage = true;
  assert.throws(() => value.renderClipNode(canvas, node, 'print', 'flow', null), /장치 경계 변환 호출/);
});

for (const [label, matrix, expected] of [
  ['identity', [1, 0, 0, 0, 1, 0, 0, 0, 1], 1],
  ['half-scale group', [.5, 0, 0, 0, .5, 0, 0, 0, 1], 2],
  ['double-scale group', [2, 0, 0, 0, 2, 0, 0, 0, 1], .5],
  ['rotated half-scale group', [0, -.5, 8, .5, 0, 12, 0, 0, 1], 2],
] as const) {
  test(`stroke floor uses the active canvas transform: ${label}`, () => {
    const value = renderer().deviceStrokeWidth({ getTotalMatrix: () => matrix }, .2);
    assert.equal(value, expected);
  });
}

test('a screen recording includes its external device transform', () => {
  const value = renderer();
  value.currentDeviceStrokeFloor = .5;
  value.recordingStrokeDeviceMatrix = [2, 0, 0, 0, 2, 0, 0, 0, 1];
  // A half-scale child recorded locally is replayed at double scale: one local unit is one device pixel.
  assert.equal(value.deviceStrokeWidth({ getTotalMatrix: () => [.5, 0, 0, 0, .5, 0, 0, 0, 1] }, .2), 1);
});

test('screen recording restores the external stroke transform after a child fails', () => {
  const value = renderer();
  Object.assign(value, {
    currentDeviceStrokeFloor: .5, currentPictureContext: 'stroke-test', currentTreeFingerprint: 'tree',
    staticPictures: { get: () => null }, staticPictureBounds: () => ({ x: 0, y: 0, width: 64, height: 64 }),
    equationRoutes: { values: [] }, textRecoveries: { values: [] },
    textFailures: { values: [] }, imageRecoveries: { values: [] }, patternReplayOps: 0,
  });
  const localCanvas = { getTotalMatrix: () => [.5, 0, 0, 0, .5, 0, 0, 0, 1] };
  value.canvasKit = {
    PictureRecorder: class {
      beginRecording() { return localCanvas; }
      finishRecordingAsPicture() { return { delete() {} }; }
      delete() {}
    },
    XYWHRect: kit.XYWHRect,
  };
  let width: number | undefined;
  value.renderNode = (canvas: unknown) => {
    width = value.deviceStrokeWidth(canvas, .2);
    throw new Error('child failed');
  };
  assert.throws(() => value.renderNodeContents(
    { getTotalMatrix: () => [2, 0, 0, 0, 2, 0, 0, 0, 1] },
    { kind: 'group', cacheHint: 'staticSubtree', children: [{}] }, 'screen', 'flow', null,
  ), /child failed/);
  assert.equal(width, 1);
  assert.equal(value.recordingStrokeDeviceMatrix, null);
  assert.equal(value.recordingStaticPicture, false);
});

test('a half-scale shape fills the one device pixel on the real CanvasKit raster', () => {
  const surface = kit.MakeSurface(64, 64)!;
  const canvas = surface.getCanvas();
  let image;
  try {
    canvas.clear(kit.WHITE);
    canvas.scale(.5, .5);
    renderer().drawStyledShape(canvas, { x: 21, y: 21, width: 40, height: 40 },
      { strokeColor: '#000000', strokeWidth: .2 },
      (paint: any) => canvas.drawRect(kit.XYWHRect(21, 21, 40, 40), paint));
    surface.flush();
    image = surface.makeImageSnapshot();
    const pixel = image.readPixels(20, 10, { width: 1, height: 1,
      colorType: kit.ColorType.RGBA_8888, alphaType: kit.AlphaType.Unpremul,
      colorSpace: kit.ColorSpace.SRGB })!;
    // The border is centered at device y=10.5. One pixel covers [10,11] completely.
    assert.deepEqual(Array.from(pixel), [0, 0, 0, 255]);
  } finally {
    image?.delete();
    surface.delete();
  }
});

test('HWP dash periods use the nominal width before the raster floor', () => {
  const surface = kit.MakeSurface(64, 64)!;
  const canvas = surface.getCanvas();
  const makeDash = kit.PathEffect.MakeDash;
  const intervals: number[][] = [];
  try {
    kit.PathEffect.MakeDash = (values, phase) => {
      intervals.push(Array.from(values));
      return makeDash(values, phase);
    };
    canvas.scale(.5, .5);
    renderer().drawStyledShape(canvas, { x: 21, y: 21, width: 40, height: 40 },
      { strokeColor: '#000000', strokeWidth: .48, strokeDash: 'hwpDash' },
      (paint: any) => canvas.drawRect(kit.XYWHRect(21, 21, 40, 40), paint));
    // 96/600=.16 outline; .48-.16=.32 pen. HWP dash: 7.5*.32+.16, 4.5*.32-.16.
    assert.equal(intervals.length, 1);
    for (const [index, expected] of [2.56, 1.28].entries()) {
      assert.ok(Math.abs(intervals[0][index] - expected) < 1e-12);
    }
  } finally {
    kit.PathEffect.MakeDash = makeDash;
    surface.delete();
  }
});

// 관계형 검사: 합성 도형의 빈 선 패턴은 실선과 같은 실제 CanvasKit 픽셀을 그려야 한다.
// 독립 문서 좌표나 PDF 일치율을 고정하는 검사가 아니며, 명시적 물리 패턴 검증은 유지한다.
for (const shape of ['line', 'rectangle'] as const) {
  test(`빈 일반 선 패턴은 실선으로 소비한다: ${shape}`, () => {
    function raster(dash: number[] | 'solid') {
      const surface = kit.MakeSurface(64, 64)!;
      const canvas = surface.getCanvas();
      let image;
      try {
        canvas.clear(kit.WHITE);
        const value = renderer();
        const paint = value.makeStrokePaint('#000000', 1);
        try {
          value.drawStrokeWithDash(dash, paint, () => {
            if (shape === 'line') canvas.drawLine(5, 32, 59, 32, paint);
            else canvas.drawRect(kit.XYWHRect(5, 5, 54, 54), paint);
          });
        } finally { paint.delete(); }
        assert.equal(value.unsupportedOps.size, 0);
        surface.flush();
        image = surface.makeImageSnapshot();
        return Array.from(image.readPixels(0, 0, { width: 64, height: 64,
          colorType: kit.ColorType.RGBA_8888, alphaType: kit.AlphaType.Unpremul,
          colorSpace: kit.ColorSpace.SRGB })!);
      } finally {
        image?.delete();
        surface.delete();
      }
    }
    const solid = raster('solid');
    assert.ok(solid.some((channel, i) => i % 4 !== 3 && channel < 255));
    assert.deepEqual(raster([]), solid);
    assert.notDeepEqual(raster([6, 3]), solid);
  });
}

test('명시적 물리 선 패턴의 빈 배열은 일반 실선으로 우회하지 않는다', () => {
  const value = renderer();
  const paint = value.makeStrokePaint('#000000', 1);
  let drawCount = 0;
  try {
    value.drawStrokeWithDash([], paint, () => { drawCount += 1; }, []);
    assert.equal(drawCount, 0);
    assert.deepEqual([...value.unsupportedOps], ['strokeDash:invalidPhysicalPattern']);
  } finally { paint.delete(); }
});
