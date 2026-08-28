import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

import { blake3 } from '@noble/hashes/blake3.js';
import { bytesToHex } from '@noble/hashes/utils.js';
import CanvasKitInit from 'canvaskit-wasm/bin/full/canvaskit.js';
import {
  CanvasKitGlyphRunFontCache,
  drawCanvasKitGlyphRun,
} from '../src/view/canvaskit/glyph-run-fonts.ts';
import { selectLayerTextVariantsForLeaf } from '../src/view/canvaskit/text-variant-selection.ts';

const studioRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const repoRoot = path.resolve(studioRoot, '..');
const fontPath = path.join(repoRoot, 'ttfs/opensource/SourceHanSerifK-OldHangul-subset.otf');
const canvasKitBundle = path.join(studioRoot, 'node_modules/canvaskit-wasm/bin/full');
const CanvasKit = await CanvasKitInit({
  locateFile: file => path.join(canvasKitBundle, file),
});

const fontBytes = fs.readFileSync(fontPath);
const digest = bytesToHex(blake3(fontBytes));
const blobKey = `font:blake3:${fontBytes.byteLength}:${digest}`;
const faceKey = `${blobKey}:face:0`;
const fontResources = {
  blobs: [{
    id: blobKey,
    source: 'embedded',
    portability: 'portableBlob',
    digest: { algorithm: 'blake3', value: digest },
    dataRef: { kind: 'fontBlob', id: blobKey },
  }],
  faces: [{ id: faceKey, blobKey, faceIndex: 0 }],
};
const resources = {
  tableId: 1,
  fontBlobs: [fontBytes.toString('base64')],
  fontBlobKeys: [blobKey],
};

const ratio = 0.8;
const drawScale = Math.sqrt(ratio);
const modelFontSize = 40;
const drawFontSize = modelFontSize * drawScale;
const pagePositions = [0, 7.728 * 4, 15.456 * 4, 15.456 * 4];
const pageAdvances = [7.728 * 4, 7.728 * 4, 0, 0];
const localPositions = [0, 8.640166665059187 * 4, 17.280333330118374 * 4, 17.280333330118374 * 4];
const localAdvances = [8.640166665059187 * 4, 8.640166665059187 * 4, 0, 0];
const glyphIds = [614, 1230, 1497, 2085];
const equivalenceGroup = 'issue-4969-q2-d4-common-shaping';
const fallback = {
  type: 'textRun',
  bbox: { x: 12, y: 12, width: 15.456 * 4, height: 52 },
  text: 'ᄒᆞᆫ글',
  variant: {
    equivalenceGroup,
    variantId: 'textRun',
    variantKind: 'textRun',
    isDefaultFallback: true,
  },
};
const glyphRun = {
  type: 'glyphRun',
  bbox: fallback.bbox,
  source: {
    id: 0,
    utf8Range: { start: 0, end: 12 },
    utf16Range: { start: 0, end: 4 },
  },
  variant: {
    equivalenceGroup,
    variantId: 'glyphRun',
    variantKind: 'glyphRun',
    isDefaultFallback: false,
    requires: ['fontResources', 'text.glyphRun'],
    quality: 'exact',
  },
  paintStyle: {
    fontFamily: 'Source Han Serif K',
    fontSize: drawFontSize,
    ratio: 1,
    color: '#000000',
  },
  shapeKey: {
    fontInstance: {
      faceKey,
      sizePx: drawFontSize,
      syntheticBold: false,
      syntheticItalic: false,
    },
    direction: 'ltr',
    writingMode: 'horizontal-tb',
    shapingEngine: 'rustybuzz-q2-v1',
    fallbackPolicy: 'none',
  },
  placement: {
    runToPage: { a: drawScale, b: 0, c: 0, d: 1, e: 12, f: 52 },
    baselineY: 0,
  },
  glyphIds,
  positions: localPositions.map(x => ({ x, y: 0 })),
  advances: localAdvances.map(dx => ({ dx, dy: 0 })),
  clusters: [
    {
      sourceRangeUtf8: { start: 0, end: 9 },
      sourceRangeUtf16: { start: 0, end: 3 },
      textRangeUtf8: { start: 0, end: 9 },
      glyphRange: { start: 0, end: 1 },
    },
    {
      sourceRangeUtf8: { start: 9, end: 12 },
      sourceRangeUtf16: { start: 3, end: 4 },
      textRangeUtf8: { start: 9, end: 12 },
      glyphRange: { start: 1, end: 4 },
    },
  ],
  direction: 'ltr',
  bidiLevel: 0,
  writingMode: 'horizontal-tb',
  orientation: 'horizontal',
  diagnostics: {
    quality: 'exact',
    replayEligibility: 'portable',
    strictVisualEligible: true,
    maxOriginDeltaPx: 0,
    maxAdvanceDeltaPx: 0,
    maxResidualAfterAdjustmentPx: 0,
    clusterMismatchCount: 0,
    missingGlyphCount: 0,
    usedFallbackFontCount: 0,
    reason: 'q2CommonShapingCondensedDrawProjectionV1',
  },
};

function alphaBounds(pixels, width, height) {
  let left = width;
  let top = height;
  let right = -1;
  let bottom = -1;
  for (let y = 0; y < height; y += 1) {
    for (let x = 0; x < width; x += 1) {
      if (pixels[(y * width + x) * 4 + 3] === 0) continue;
      left = Math.min(left, x);
      top = Math.min(top, y);
      right = Math.max(right, x);
      bottom = Math.max(bottom, y);
    }
  }
  assert.ok(right >= left && bottom >= top, 'GlyphRun은 실제 ink pixel을 만들어야 한다');
  return { left, top, right, bottom, width: right - left + 1, height: bottom - top + 1 };
}

function rasterize(run, font) {
  const width = 128;
  const height = 80;
  const surface = CanvasKit.MakeSurface(width, height);
  assert.ok(surface, 'CanvasKit software surface를 만들 수 있어야 한다');
  const paint = new CanvasKit.Paint();
  let snapshot = null;
  try {
    const canvas = surface.getCanvas();
    canvas.clear(CanvasKit.TRANSPARENT);
    paint.setColor(CanvasKit.BLACK);
    paint.setStyle(CanvasKit.PaintStyle.Fill);
    assert.equal(drawCanvasKitGlyphRun(canvas, run, font, paint), true);
    surface.flush();
    snapshot = surface.makeImageSnapshot();
    const pixels = snapshot.readPixels(0, 0, {
      width,
      height,
      colorType: CanvasKit.ColorType.RGBA_8888,
      alphaType: CanvasKit.AlphaType.Unpremul,
      colorSpace: CanvasKit.ColorSpace.SRGB,
    });
    assert.ok(pixels, 'CanvasKit GlyphRun pixel을 읽을 수 있어야 한다');
    return alphaBounds(pixels, width, height);
  } finally {
    snapshot?.delete();
    paint.delete();
    surface.delete();
  }
}

const cache = new CanvasKitGlyphRunFontCache(CanvasKit);
try {
  cache.registerResources(fontResources, resources);
  const status = cache.replayStatus(glyphRun, fontResources);
  assert.equal(status.replayable, true, 'D4 common GlyphRun은 strict CanvasKit replay 대상이어야 한다');
  const selected = selectLayerTextVariantsForLeaf(
    [fallback, glyphRun],
    () => false,
    op => cache.replayStatus(op, fontResources).replayable,
  );
  assert.deepEqual([...selected], [glyphRun], 'strict CanvasKit은 TextRun 대신 common GlyphRun을 선택해야 한다');
  assert.deepEqual(
    [...selectLayerTextVariantsForLeaf([fallback, glyphRun], () => false, () => false)],
    [fallback],
    'Canvas2D·legacy·미지원 backend는 TextRun fallback을 유지해야 한다',
  );

  assert.deepEqual(glyphRun.glyphIds, glyphIds);
  assert.deepEqual(glyphRun.clusters.map(cluster => cluster.glyphRange), [
    { start: 0, end: 1 },
    { start: 1, end: 4 },
  ]);
  for (const [index, position] of glyphRun.positions.entries()) {
    assert.ok(Math.abs(position.x * drawScale - pagePositions[index]) <= 1e-9);
    assert.ok(Math.abs(glyphRun.advances[index].dx * drawScale - pageAdvances[index]) <= 1e-9);
  }
  assert.ok(Math.abs(
    glyphRun.advances.reduce((total, advance) => total + advance.dx, 0) * drawScale
      - fallback.bbox.width,
  ) <= 1e-9, 'affine advance와 layout bbox 폭은 같아야 한다');

  const font = cache.font(glyphRun, fontResources);
  assert.ok(font, 'exact Source Han face에서 replay Font를 만들어야 한다');
  const currentInk = rasterize(glyphRun, font);

  // #5821 이전 계약: glyph는 원래 font size로 두고 x만 ratio만큼 축소했다.
  // page advance는 같지만 세로 ink가 더 커지는 잘못을 실제 CanvasKit pixel로 대조한다.
  const oldContract = {
    ...glyphRun,
    paintStyle: { ...glyphRun.paintStyle, fontSize: modelFontSize },
    shapeKey: {
      ...glyphRun.shapeKey,
      fontInstance: { ...glyphRun.shapeKey.fontInstance, sizePx: modelFontSize },
    },
    placement: {
      ...glyphRun.placement,
      runToPage: { ...glyphRun.placement.runToPage, a: ratio },
    },
    positions: pagePositions.map(x => ({ x: x / ratio, y: 0 })),
    advances: pageAdvances.map(dx => ({ dx: dx / ratio, dy: 0 })),
  };
  const oldFont = cache.font(oldContract, fontResources);
  assert.ok(oldFont, 'historical comparison Font를 만들어야 한다');
  const oldInk = rasterize(oldContract, oldFont);
  assert.ok(
    currentInk.height < oldInk.height,
    `현재 √ratio glyph 높이 ${currentInk.height}px는 이전 계약 ${oldInk.height}px보다 작아야 한다`,
  );
  assert.ok(
    Math.abs(currentInk.width - oldInk.width) <= 2,
    `같은 page advance의 ink 폭 차이는 2px 이하여야 한다: ${currentInk.width}/${oldInk.width}`,
  );

  for (const rejected of [
    { ...glyphRun, diagnostics: { ...glyphRun.diagnostics, strictVisualEligible: false } },
    { ...glyphRun, placement: { ...glyphRun.placement, runToPage: { ...glyphRun.placement.runToPage, a: Number.NaN } } },
  ]) {
    const rejectedSelection = selectLayerTextVariantsForLeaf(
      [fallback, rejected],
      () => false,
      op => cache.replayStatus(op, fontResources).replayable,
    );
    assert.deepEqual([...rejectedSelection], [fallback]);
  }

  console.log(JSON.stringify({
    status: 'pass',
    selector: 'commonGlyphRun',
    pageAdvancePx: fallback.bbox.width,
    drawFontSize,
    drawScale,
    currentInk,
    oldContractInk: oldInk,
    fontCache: cache.diagnostics(),
  }));
} finally {
  cache.clear();
}
