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
