import assert from 'node:assert/strict';
import test from 'node:test';

import type { LayerGlyphOutlineOp, LayerPaintOp, LayerTextRunOp } from '../src/core/types.ts';
import {
  selectLayerTextVariantsForLeaf,
  staticSvgPathLayersAreReplayable,
} from '../src/view/canvaskit/text-variant-selection.ts';

function textFallback(group: string): LayerTextRunOp {
  return {
    type: 'textRun',
    bbox: { x: 0, y: 0, width: 16, height: 16 },
    text: 'A',
    variant: {
      equivalenceGroup: group,
      variantId: 'textRun',
      variantKind: 'textRun',
      isDefaultFallback: true,
    },
  };
}

function glyphSidecar(group: string, payloadKind: 'bitmapGlyph' | 'svgGlyph'): LayerGlyphOutlineOp {
  return {
    type: 'glyphOutline',
    bbox: { x: 0, y: 0, width: 16, height: 16 },
    payloadKind,
    variant: {
      equivalenceGroup: group,
      variantId: payloadKind,
      variantKind: 'glyphOutline',
      isDefaultFallback: false,
    },
  };
}

test('selects a decodable glyph sidecar exclusively', () => {
  const fallback = textFallback('text-0');
  const sidecar = glyphSidecar('text-0', 'bitmapGlyph');
  const selected = selectLayerTextVariantsForLeaf([fallback, sidecar], () => true);

  assert.deepEqual([...selected], [sidecar]);
});

test('keeps text fallback for corrupt, missing, oversized, or unparseable resources', () => {
  const failures = ['invalidBase64', 'decodeThrows', 'decodeNull', 'oversized', 'missingResource', 'unparseableSvg'];
  for (const failure of failures) {
    const fallback = textFallback(failure);
    const sidecar = glyphSidecar(failure, failure === 'unparseableSvg' ? 'svgGlyph' : 'bitmapGlyph');
    const ops: LayerPaintOp[] = [fallback, sidecar];
    const selected = selectLayerTextVariantsForLeaf(ops, () => false);
    assert.deepEqual([...selected], [fallback], failure);
  }
});

test('selects each equivalence group independently', () => {
  const firstFallback = textFallback('text-0');
  const firstSidecar = glyphSidecar('text-0', 'bitmapGlyph');
  const secondFallback = textFallback('text-1');
  const secondSidecar = glyphSidecar('text-1', 'svgGlyph');
  const selected = selectLayerTextVariantsForLeaf(
    [firstFallback, firstSidecar, secondFallback, secondSidecar],
    op => op.payloadKind === 'svgGlyph',
  );

  assert.deepEqual([...selected], [firstFallback, secondSidecar]);
});

test('rejects SVG layers when CanvasKit path construction throws or returns null', () => {
  const layers = [{ pathData: 'not-a-path', fill: '#000000', opacity: 1 }];
  assert.equal(staticSvgPathLayersAreReplayable(layers, () => null), false);
  assert.equal(staticSvgPathLayersAreReplayable(layers, () => { throw new Error('parse'); }), false);
  let deleted = false;
  assert.equal(staticSvgPathLayersAreReplayable(layers, () => ({ delete: () => { deleted = true; } })), true);
  assert.equal(deleted, true);
});
