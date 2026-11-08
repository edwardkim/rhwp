import assert from 'node:assert/strict';
import fs from 'node:fs';
import test from 'node:test';

import { blake3 } from '@noble/hashes/blake3.js';
import { bytesToHex } from '@noble/hashes/utils.js';
import type { CanvasKit } from 'canvaskit-wasm';

import type { LayerFontResources, LayerGlyphRunOp, LayerTextRunOp } from '../src/core/types.ts';
import { CanvasKitGlyphRunFontCache } from '../src/view/canvaskit/glyph-run-fonts.ts';
import { selectLayerTextVariantsForLeaf } from '../src/view/canvaskit/text-variant-selection.ts';

const bytes = new Uint8Array(fs.readFileSync(new URL(
  '../../tests/fixtures/fonts/RHWPBitmapSvgGlyphSmoke.ttf', import.meta.url,
)));

function fixture(sizePx: number) {
  const instantiatedSizes: number[] = [];
  const digest = bytesToHex(blake3(bytes));
  const key = `font:blake3:${bytes.byteLength}:${digest}`;
  const resources: LayerFontResources = {
    blobs: [{ id: key, source: 'embedded', portability: 'portableBlob',
      digest: { algorithm: 'blake3', value: digest }, dataRef: { kind: 'fontBlob', id: key } }],
    faces: [{ id: 'face', blobKey: key, faceIndex: 0 }],
  };
  const cache = new CanvasKitGlyphRunFontCache({
    Typeface: { MakeTypefaceFromData: () => ({ delete() {} }) },
    Font: class {
      constructor(_typeface: unknown, size: number) { instantiatedSizes.push(size); }
      setSubpixel() {}
      setEmbolden() {}
      setSkewX() {}
      delete() {}
    },
  } as unknown as CanvasKit);
  cache.registerResources(resources, { fontBlobs: [bytes], fontBlobKeys: [key] });
  const fallback: LayerTextRunOp = {
    type: 'textRun', bbox: { x: 0, y: 0, width: 16, height: 16 }, text: 'A',
    variant: { equivalenceGroup: 'wire-size', variantId: 'textRun', variantKind: 'textRun', isDefaultFallback: true },
  };
  const glyph: LayerGlyphRunOp = {
    type: 'glyphRun', bbox: fallback.bbox,
    source: { id: 0, utf8Range: { start: 0, end: 1 }, utf16Range: { start: 0, end: 1 } },
    variant: { equivalenceGroup: 'wire-size', variantId: 'glyphRun', variantKind: 'glyphRun', partIndex: 0, partCount: 1 },
    paintStyle: { fontSize: 16, color: '#000000' },
    shapeKey: {
      fontInstance: { faceKey: 'face', sizePx, syntheticBold: false, syntheticItalic: false },
      direction: 'ltr', writingMode: 'horizontal-tb', shapingEngine: 'test', fallbackPolicy: 'none',
    },
    placement: { runToPage: { a: 1, b: 0, c: 0, d: 1, e: 0, f: 0 }, baselineY: 0 },
    glyphIds: [1], positions: [{ x: 0, y: 0 }], clusters: [], direction: 'ltr', bidiLevel: 0,
    writingMode: 'horizontal-tb', orientation: 'horizontal',
    diagnostics: {
      quality: 'exact', replayEligibility: 'portable', strictVisualEligible: true,
      maxOriginDeltaPx: 0, maxAdvanceDeltaPx: 0, maxResidualAfterAdjustmentPx: 0,
      clusterMismatchCount: 0, missingGlyphCount: 0, usedFallbackFontCount: 0,
    },
  };
  return { cache, resources, fallback, glyph, instantiatedSizes };
}

test('GlyphRun wire font-size collapse keeps the complete TextRun fallback', () => {
  for (const size of [Number.MIN_VALUE, 0.00000049]) {
    const { cache, resources, fallback, glyph, instantiatedSizes } = fixture(size);
    try {
      const wire: LayerGlyphRunOp = JSON.parse(JSON.stringify(glyph));
      // write_shape_key serializes sizePx with six fixed decimal places.
      wire.shapeKey.fontInstance.sizePx = Number(size.toFixed(6));
      assert.equal(wire.shapeKey.fontInstance.sizePx, 0);
      for (const candidate of [glyph, wire]) {
        const status = cache.replayStatus(candidate, resources);
        assert.equal(status.replayable, false);
        if (!status.replayable) assert.equal(status.reason, 'fontInstanceInvalid');
        const selected = selectLayerTextVariantsForLeaf(
          [fallback, candidate], () => false, op => cache.replayStatus(op, resources).replayable,
        );
        assert.deepEqual([...selected], [fallback]);
      }
      assert.deepEqual(instantiatedSizes, []);
    } finally {
      cache.clear();
    }
  }
});

test('GlyphRun admits the smallest positive six-decimal wire font size', () => {
  for (const size of [0.000001, 16, 4096]) {
    const { cache, resources, fallback, glyph, instantiatedSizes } = fixture(size);
    try {
      assert.equal(cache.replayStatus(glyph, resources).replayable, true);
      assert.deepEqual(instantiatedSizes, [size]);
      assert.equal(cache.diagnostics().fonts, 1);
      const selected = selectLayerTextVariantsForLeaf(
        [fallback, glyph], () => false, op => cache.replayStatus(op, resources).replayable,
      );
      assert.deepEqual([...selected], [glyph]);
    } finally {
      cache.clear();
    }
  }
});
