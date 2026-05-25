import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';

import {
  resolveCanvasKitRenderMode,
  resolveCanvasKitSurfaceRequest,
  resolveRenderBackend,
  resolveRenderBackendRequest,
  resolveRenderProfile,
} from '../src/view/render-backend.ts';
import {
  canvasKitImageCacheKey,
  canvasKitImageFillModeTiles,
  canvasKitImagePlacement,
  canvasKitImageSourceRect,
  HWPUNIT_PER_PIXEL,
} from '../src/view/canvaskit/image-replay.ts';
import { glyphOutlinePayloadStatus } from '../src/view/glyph-outline-payload-status.ts';

test('render backend resolver keeps Canvas2D as the default and accepts skia aliases', () => {
  assert.equal(resolveRenderBackend(''), 'canvas2d');
  assert.equal(resolveRenderBackend('?renderer=canvas'), 'canvas2d');
  assert.equal(resolveRenderBackend('?renderer=canvas2d'), 'canvas2d');
  assert.equal(resolveRenderBackend('?renderer=canvaskit'), 'canvaskit');
  assert.equal(resolveRenderBackend('?renderer=skia'), 'canvaskit');
});

test('render backend resolver reports invalid explicit values and keeps URL opt-ins ephemeral', () => {
  const originalStorage = (globalThis as { localStorage?: unknown }).localStorage;
  (globalThis as { localStorage?: unknown }).localStorage = {
    getItem: () => 'canvaskit',
    setItem: () => undefined,
  };
  try {
    assert.equal(resolveRenderBackend(''), 'canvas2d');
    assert.deepEqual(resolveRenderBackendRequest('?renderer=unknown'), {
      backend: 'canvas2d',
      requested: 'unknown',
      unsupportedReason: 'unsupportedRenderBackend',
    });
  } finally {
    (globalThis as { localStorage?: unknown }).localStorage = originalStorage;
  }
});

test('CanvasKit mode resolver exposes default and conservative compat direct modes', () => {
  assert.equal(resolveCanvasKitRenderMode(''), 'default');
  assert.equal(resolveCanvasKitRenderMode('?canvaskitMode=compat'), 'compat');
  assert.equal(resolveCanvasKitRenderMode('?skiaMode=compatibility'), 'compat');
  assert.equal(resolveCanvasKitRenderMode('?canvaskitMode=overlay'), 'default');
});

test('CanvasKit surface resolver records unsupported requests without throwing', () => {
  assert.deepEqual(resolveCanvasKitSurfaceRequest('?canvaskitSurface=webgpu'), {
    preference: 'webgpu',
    requested: 'webgpu',
  });
  assert.deepEqual(resolveCanvasKitSurfaceRequest('?canvaskitSurface=cpu'), {
    preference: 'software',
    requested: 'cpu',
  });
  assert.deepEqual(resolveCanvasKitSurfaceRequest('?canvaskitSurface=metal'), {
    preference: 'auto',
    requested: 'metal',
    unsupportedReason: 'unsupportedSurfaceBackend',
  });
});

test('render profile resolver keeps screen as the stable browser default', () => {
  assert.equal(resolveRenderProfile(''), 'screen');
  assert.equal(resolveRenderProfile('?renderProfile=fast-preview'), 'fastPreview');
  assert.equal(resolveRenderProfile('?profile=print'), 'print');
  assert.equal(resolveRenderProfile('?profile=highQuality'), 'highQuality');
});

test('CanvasKit renderer source does not introduce Canvas2D overlay replay', () => {
  const source = readFileSync(new URL('../src/view/canvaskit-renderer.ts', import.meta.url), 'utf8');
  assert.equal(source.includes("getContext('2d')"), false);
  assert.equal(source.includes('renderPageToCanvas'), false);
  assert.equal(source.includes('rhwpOverlay'), false);
});

test('CanvasKit replay bridge fallback keeps compat on direct replay contract', () => {
  const source = readFileSync(new URL('../src/core/wasm-bridge.ts', import.meta.url), 'utf8');
  const method = source.match(/getCanvasKitReplayPlan\([^)]*\): string \{(?<body>[\s\S]*?)\n  \}/);
  assert.ok(method?.groups?.body);
  const fallback = method.groups.body;
  assert.match(fallback, /hiddenCanvas2dOverlayAllowed:\s*false/);
  assert.match(fallback, /directReplayRequired:\s*true/);
  assert.equal(fallback.includes("mode === 'compat'"), false);
  assert.equal(fallback.includes("mode === 'default'"), false);
});

test('CanvasKit image replay cache key includes payload fingerprint with repeated image refs', () => {
  const first = canvasKitImageCacheKey({ imageRef: 7, mime: 'image/png', base64: 'AAAA' });
  const second = canvasKitImageCacheKey({ imageRef: 7, mime: 'image/png', base64: 'BBBB' });
  assert.notEqual(first, second);
  assert.ok((first ?? '').startsWith('ref:7|image/png:4:'));
});

test('CanvasKit image crop source follows the same HWPUNIT crop scale as SVG replay', () => {
  const crop = canvasKitImageSourceRect(2320, 354, { left: 0, top: 0, right: 102366, bottom: 26580 });
  assert.ok(crop);
  assert.equal(crop.x, 0);
  assert.equal(crop.y, 0);
  assert.ok(Math.abs(crop.width - (102366 / HWPUNIT_PER_PIXEL)) < 0.01);
  assert.equal(crop.height, 354);
  assert.equal(canvasKitImageSourceRect(2320, 354, { left: 0, top: 0, right: 174000, bottom: 26580 }), null);
});

test('CanvasKit image placement follows layer fill-mode anchors', () => {
  const bbox = { x: 10, y: 20, width: 100, height: 80 };
  assert.deepEqual(canvasKitImagePlacement('center', bbox, 40, 20), { x: 40, y: 50 });
  assert.deepEqual(canvasKitImagePlacement('rightBottom', bbox, 40, 20), { x: 70, y: 80 });
  assert.deepEqual(canvasKitImagePlacement('leftTop', bbox, 40, 20), { x: 10, y: 20 });
});

test('CanvasKit image fill-mode tiling detection stays explicit', () => {
  for (const mode of ['tileAll', 'tileHorzTop', 'tileHorzBottom', 'tileVertLeft', 'tileVertRight']) {
    assert.equal(canvasKitImageFillModeTiles(mode), true);
  }
  for (const mode of [undefined, 'fitToSize', 'none', 'center', 'leftTop', 'rightBottom']) {
    assert.equal(canvasKitImageFillModeTiles(mode), false);
  }
});

test('GlyphOutline advanced payload gates reject richer payloads by default', () => {
  assert.deepEqual(
    glyphOutlinePayloadStatus({
      type: 'glyphOutline',
      bbox: { x: 0, y: 0, width: 10, height: 10 },
      payloadKind: 'colorLayers',
      colorLayers: {
        colorFormat: 'colrV1',
        sourceRangeUtf8: { start: 0, end: 1 },
        glyphRange: { start: 0, end: 1 },
        paintGraph: {
          rootNodeId: 0,
          nodes: [{
            nodeId: 0,
            kind: 'solidPath',
            solidPath: {
              commands: [{ type: 'moveTo', x: 0, y: 0 }],
              fill: { rgba: [0, 0, 0, 1] },
              fillRule: 'nonzero',
            },
            sourceRangeUtf8: { start: 0, end: 1 },
            glyphRange: { start: 0, end: 1 },
            sourceFontRef: { faceKey: 'fixture-face', glyphId: 42, colorFormat: 'colrV1' },
          }],
        },
      },
    }).reason,
    'unsupportedColorGlyph',
  );
  assert.equal(
    glyphOutlinePayloadStatus({
      type: 'glyphOutline',
      bbox: { x: 0, y: 0, width: 10, height: 10 },
      payloadKind: 'bitmapGlyph',
      bitmapGlyph: {
        imageRef: 1,
        sourceRangeUtf8: { start: 0, end: 1 },
        glyphRange: { start: 0, end: 1 },
        placement: { x: 0, y: 0, width: 10, height: 10 },
        scalingPolicy: 'sourceExact',
        filtering: 'linear',
      },
    }).reason,
    'unsupportedBitmapGlyph',
  );
  assert.equal(
    glyphOutlinePayloadStatus({
      type: 'glyphOutline',
      bbox: { x: 0, y: 0, width: 10, height: 10 },
      payloadKind: 'svgGlyph',
      svgGlyph: {
        svgRef: 1,
        sourceRangeUtf8: { start: 0, end: 1 },
        glyphRange: { start: 0, end: 1 },
        viewBox: { x: 0, y: 0, width: 10, height: 10 },
        staticSanitized: true,
        scriptAllowed: false,
        animationAllowed: false,
        externalResourcesAllowed: false,
        interactivityAllowed: false,
      },
    }).reason,
    'unsupportedSvgGlyph',
  );
});

test('GlyphOutline COLRv1 gate reports unsupported graph node kind exactly', () => {
  const status = glyphOutlinePayloadStatus({
    type: 'glyphOutline',
    bbox: { x: 0, y: 0, width: 10, height: 10 },
    payloadKind: 'colorLayers',
    colorLayers: {
      colorFormat: 'colrV1',
      paintGraph: {
        rootNodeId: 0,
        nodes: [{ nodeId: 0, kind: 'composite' }],
      },
    },
  }, { allowColrv1Stage1ColorGraph: true });
  assert.equal(status.reason, 'unsupportedColorGlyph');
  assert.equal(status.detail, 'colrV1Node:composite');
});

test('GlyphOutline COLRv1 gradient graph subset can pass the explicit gate', () => {
  const commands = [{ type: 'moveTo', x: 0, y: 0 }, { type: 'lineTo', x: 10, y: 0 }, { type: 'closePath' }];
  const stops = [
    { offset: 0, color: { rgba: [1, 0, 0, 1] } },
    { offset: 1, color: { rgba: [0, 0, 1, 1] } },
  ];
  const cases = [
    {
      kind: 'linearGradientPath',
      field: 'linearGradientPath',
      value: { commands, gradient: { x0: 0, y0: 0, x1: 10, y1: 10, stops }, fillRule: 'nonzero' },
    },
    {
      kind: 'radialGradientPath',
      field: 'radialGradientPath',
      value: { commands, gradient: { cx: 5, cy: 5, radius: 5, stops }, fillRule: 'nonzero' },
    },
    {
      kind: 'sweepGradientPath',
      field: 'sweepGradientPath',
      value: { commands, gradient: { cx: 5, cy: 5, startAngleDegrees: 0, endAngleDegrees: 360, stops }, fillRule: 'nonzero' },
    },
  ];
  for (const entry of cases) {
    const status = glyphOutlinePayloadStatus({
      type: 'glyphOutline',
      bbox: { x: 0, y: 0, width: 10, height: 10 },
      payloadKind: 'colorLayers',
      colorLayers: {
        colorFormat: 'colrV1',
        sourceRangeUtf8: { start: 0, end: 1 },
        glyphRange: { start: 0, end: 1 },
        sourceFontRef: { faceKey: 'fixture-face', glyphId: 42, colorFormat: 'colrV1' },
        paintGraph: {
          rootNodeId: 0,
          nodes: [{
            nodeId: 0,
            kind: entry.kind,
            [entry.field]: entry.value,
            sourceRangeUtf8: { start: 0, end: 1 },
            glyphRange: { start: 0, end: 1 },
            sourceFontRef: { faceKey: 'fixture-face', glyphId: 42, colorFormat: 'colrV1' },
          }],
        },
      },
    }, { allowColrv1Stage1ColorGraph: true });
    assert.equal(status.supported, true, entry.kind);
  }
});

test('CanvasKit renderer diagnostics keep GlyphOutline payload reject reasons visible', () => {
  const source = readFileSync(new URL('../src/view/canvaskit-renderer.ts', import.meta.url), 'utf8');
  assert.match(source, /glyphOutlinePayloadStatus\(op, \{ allowColrv1Stage1ColorGraph: true \}\)/);
  assert.match(source, /glyphOutline:\$\{status\.reason\}/);
});
