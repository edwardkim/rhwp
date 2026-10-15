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
const fontPath = path.resolve(studioRoot, '../assets/fonts/NotoSansKR-Regular.woff2');
const symbolFontPath = path.resolve(studioRoot, '../assets/fonts/D2Coding-Regular.woff2');
const oldHangulFontPath = path.resolve(studioRoot, '../assets/fonts/RHWPOldHangul-Regular.woff2');
const exactFaceFontPath = path.resolve(studioRoot, '../tests/fixtures/fonts/RHWPExactFaceSmoke.ttc');
const canvasKitBundle = path.resolve(studioRoot, 'node_modules/canvaskit-wasm/bin/full');
const CanvasKit = await CanvasKitInit({
  locateFile: (file) => path.join(canvasKitBundle, file),
});
const typeface = CanvasKit.Typeface.MakeFreeTypeFaceFromData(fs.readFileSync(fontPath));
assert.ok(typeface, 'Noto Sans KR Regular typeface를 만들 수 있어야 한다');

const fontManager = CanvasKit.FontMgr.FromData(fs.readFileSync(fontPath));
assert.equal(fontManager?.getFamilyName(0), 'Noto Sans KR', 'Regular 번들은 올바른 family name을 노출해야 한다');
fontManager?.delete();

const font = new CanvasKit.Font(typeface, 16);
try {
  for (const [character, codepoint] of [
    ['■', 'U+25A0'],
    ['▪', 'U+25AA'],
    ['□', 'U+25A1'],
    ['○', 'U+25CB'],
    ['─', 'U+2500'],
    ['가', 'U+AC00'],
  ]) {
    const glyphId = font.getGlyphIDs(character, 1)[0];
    assert.notEqual(glyphId, 0, `${codepoint} ${character}는 Noto Sans KR Regular에 있어야 한다`);
  }
} finally {
  font.delete();
  typeface.delete();
}

const symbolTypeface = CanvasKit.Typeface.MakeFreeTypeFaceFromData(fs.readFileSync(symbolFontPath));
assert.ok(symbolTypeface, 'D2Coding Regular 기호 폴백 typeface를 만들 수 있어야 한다');
const symbolFont = new CanvasKit.Font(symbolTypeface, 16);
try {
  for (const [character, codepoint] of [
    ['❖', 'U+2756'],
    ['⇩', 'U+21E9'],
    ['☑', 'U+2611'],
    ['☞', 'U+261E'],
    ['①', 'U+2460'],
    ['★', 'U+2605'],
  ]) {
    const glyphId = symbolFont.getGlyphIDs(character, 1)[0];
    assert.notEqual(glyphId, 0, `${codepoint} ${character}는 D2Coding 기호 폴백에 있어야 한다`);
  }
} finally {
  symbolFont.delete();
  symbolTypeface.delete();
}

const oldHangulTypeface = CanvasKit.Typeface.MakeFreeTypeFaceFromData(fs.readFileSync(oldHangulFontPath));
assert.ok(oldHangulTypeface, 'Source Han Serif K 옛한글 subset typeface를 만들 수 있어야 한다');
const oldHangulFontManager = CanvasKit.FontMgr.FromData(fs.readFileSync(oldHangulFontPath));
assert.ok(oldHangulFontManager?.countFamilies(), '옛한글 subset은 CanvasKit font manager를 제공해야 한다');

const oldHangulFont = new CanvasKit.Font(oldHangulTypeface, 40);
try {
  for (const [character, codepoint] of [
    ['ᄒ', 'U+1112'],
    ['ᆞ', 'U+119E'],
    ['ᆫ', 'U+11AB'],
  ]) {
    const glyphId = oldHangulFont.getGlyphIDs(character, 1)[0];
    assert.notEqual(glyphId, 0, `${codepoint} ${character}는 옛한글 subset에 있어야 한다`);
  }

  const paragraphStyle = new CanvasKit.ParagraphStyle({
    textStyle: {
      color: CanvasKit.BLACK,
      fontSize: 40,
      fontFamilies: [oldHangulFontManager.getFamilyName(0)],
    },
  });
  const builder = CanvasKit.ParagraphBuilder.Make(paragraphStyle, oldHangulFontManager);
  try {
    builder.addText('ᄒᆞᆫ');
    const paragraph = builder.build();
    try {
      paragraph.layout(400);
      assert.ok(paragraph.getLongestLine() > 0, '옛한글 cluster가 폭을 가져야 한다');
      assert.equal(
        paragraph.getRectsForRange(0, 3, CanvasKit.RectHeightStyle.Tight, CanvasKit.RectWidthStyle.Tight).length,
        1,
        'ᄒᆞᆫ은 CanvasKit paragraph shaping에서 하나의 glyph cluster여야 한다',
      );
    } finally {
      paragraph.delete();
    }
  } finally {
    builder.delete();
  }
} finally {
  oldHangulFont.delete();
  oldHangulTypeface.delete();
  oldHangulFontManager?.delete();
}

const exactFaceBytes = fs.readFileSync(exactFaceFontPath);
const exactFaceDigest = bytesToHex(blake3(exactFaceBytes));
const exactFaceResourceKey = `font:blake3:${exactFaceBytes.byteLength}:${exactFaceDigest}`;
const glyphRunFontResources = {
  blobs: [{
    id: exactFaceResourceKey,
    source: 'embedded',
    portability: 'portableBlob',
    digest: { algorithm: 'blake3', value: exactFaceDigest },
    dataRef: { kind: 'fontBlob', id: exactFaceResourceKey },
  }],
  faces: [{
    id: `${exactFaceResourceKey}:face:1`,
    blobKey: exactFaceResourceKey,
    faceIndex: 1,
  }],
};
const glyphRunResources = {
  tableId: 1,
  fontBlobs: [exactFaceBytes.toString('base64')],
  fontBlobKeys: [exactFaceResourceKey],
};
const glyphRunOp = {
  type: 'glyphRun',
  bbox: { x: 0, y: 0, width: 64, height: 64 },
  source: { id: 0, utf8Range: { start: 0, end: 1 }, utf16Range: { start: 0, end: 1 } },
  variant: {
    equivalenceGroup: 'exact-face',
    variantId: 'glyphRun',
    variantKind: 'glyphRun',
  },
  paintStyle: { fontFamily: 'RHWP Exact Face One', fontSize: 32, color: '#000000' },
  shapeKey: {
    fontInstance: {
      faceKey: `${exactFaceResourceKey}:face:1`,
      sizePx: 32,
      syntheticBold: false,
      syntheticItalic: false,
    },
    direction: 'ltr',
    writingMode: 'horizontal-tb',
    shapingEngine: 'ttf-parser-nominal-v1',
    fallbackPolicy: 'none',
  },
  placement: { runToPage: { a: 1, b: 0, c: 0, d: 1, e: 0, f: 32 }, baselineY: 0 },
  glyphIds: [1],
  positions: [{ x: 0, y: 0 }],
  advances: [{ dx: 32, dy: 0 }],
  clusters: [{
    sourceRangeUtf8: { start: 0, end: 1 },
    sourceRangeUtf16: { start: 0, end: 1 },
    textRangeUtf8: { start: 0, end: 1 },
    glyphRange: { start: 0, end: 1 },
  }],
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
  },
};
const glyphRunFontCache = new CanvasKitGlyphRunFontCache(CanvasKit);
glyphRunFontCache.registerResources(glyphRunFontResources, glyphRunResources);
const exactFaceStatus = glyphRunFontCache.replayStatus(glyphRunOp, glyphRunFontResources);
assert.equal(exactFaceStatus.replayable, true, 'TTC face 1 strict GlyphRun을 선택할 수 있어야 한다');
const exactFaceFont = glyphRunFontCache.font(glyphRunOp, glyphRunFontResources);
assert.ok(exactFaceFont, 'TTC face 1에서 CanvasKit Font를 만들 수 있어야 한다');
const exactFaceGlyphId = exactFaceFont.getGlyphIDs('\uE104', 1)[0];
assert.notEqual(exactFaceGlyphId, 0, '선택한 TTC face에서 U+E104 glyph를 찾을 수 있어야 한다');
const malformedRunStatus = glyphRunFontCache.replayStatus({}, glyphRunFontResources);
assert.equal(malformedRunStatus.replayable, false);
assert.equal(malformedRunStatus.reason, 'glyphRunMalformed');
const floatOverflowStatus = glyphRunFontCache.replayStatus({
  ...glyphRunOp,
  positions: [{ x: 1e308, y: 0 }],
}, glyphRunFontResources);
assert.equal(floatOverflowStatus.replayable, false);
assert.equal(floatOverflowStatus.reason, 'positionNotFinite');
const oversizedResourceTableStatus = glyphRunFontCache.replayStatus(glyphRunOp, {
  ...glyphRunFontResources,
  blobs: Array.from({ length: 257 }, () => glyphRunFontResources.blobs[0]),
});
assert.equal(oversizedResourceTableStatus.replayable, false);
assert.equal(oversizedResourceTableStatus.reason, 'fontResourceTableTooLarge');

for (const [name, mutate, expectedReason] of [
  [
    'synthetic',
    op => ({
      ...op,
      shapeKey: {
        ...op.shapeKey,
        fontInstance: { ...op.shapeKey.fontInstance, syntheticBold: true },
      },
    }),
    'syntheticStyleAuthorityPending',
  ],
  [
    'bidi-direction',
    op => ({ ...op, direction: 'rtl' }),
    'bidiDirectionAuthorityPending',
  ],
  [
    'bidi-level',
    op => ({ ...op, bidiLevel: 1 }),
    'bidiLevelAuthorityPending',
  ],
  [
    'writing-mode',
    op => ({ ...op, writingMode: 'vertical-rl' }),
    'writingModeAuthorityPending',
  ],
]) {
  const rejectedRun = mutate({
    ...glyphRunOp,
    variant: { ...glyphRunOp.variant, equivalenceGroup: `exact-face-${name}` },
  });
  const status = glyphRunFontCache.replayStatus(rejectedRun, glyphRunFontResources);
  assert.equal(status.replayable, false, `${name} strict GlyphRun은 direct replay하면 안 된다`);
  assert.equal(status.reason, expectedReason);
  const fallback = {
    type: 'textRun',
    bbox: glyphRunOp.bbox,
    text: '\uE104',
    variant: {
      equivalenceGroup: `exact-face-${name}`,
      variantId: 'textRun',
      variantKind: 'textRun',
      isDefaultFallback: true,
    },
  };
  const selected = selectLayerTextVariantsForLeaf(
    [fallback, rejectedRun],
    () => false,
    op => glyphRunFontCache.replayStatus(op, glyphRunFontResources).replayable,
  );
  assert.deepEqual([...selected], [fallback], `${name} strict GlyphRun은 TextRun으로 fallback해야 한다`);
}

const glyphSurface = CanvasKit.MakeSurface(64, 64);
assert.ok(glyphSurface, 'exact-face GlyphRun smoke surface를 만들 수 있어야 한다');
const glyphPaint = new CanvasKit.Paint();
let glyphSnapshot = null;
try {
  const glyphCanvas = glyphSurface.getCanvas();
  glyphCanvas.clear(CanvasKit.TRANSPARENT);
  glyphPaint.setColor(CanvasKit.BLACK);
  glyphPaint.setStyle(CanvasKit.PaintStyle.Fill);
  const replayGlyphRun = {
    ...glyphRunOp,
    glyphIds: [exactFaceGlyphId],
    placement: { runToPage: { a: 1, b: 0, c: 0, d: 1, e: 8, f: 48 }, baselineY: 0 },
  };
  const fallback = {
    type: 'textRun',
    bbox: glyphRunOp.bbox,
    text: '\uE104',
    variant: {
      equivalenceGroup: 'exact-face',
      variantId: 'textRun',
      variantKind: 'textRun',
      isDefaultFallback: true,
    },
  };
  const selected = selectLayerTextVariantsForLeaf(
    [fallback, replayGlyphRun],
    () => false,
    op => glyphRunFontCache.replayStatus(op, glyphRunFontResources).replayable,
  );
  assert.deepEqual([...selected], [replayGlyphRun]);
  assert.equal(
    drawCanvasKitGlyphRun(glyphCanvas, replayGlyphRun, exactFaceFont, glyphPaint),
    true,
    '선택한 GlyphRun을 renderer helper가 transform/baseline과 함께 그려야 한다',
  );
  glyphSurface.flush();
  glyphSnapshot = glyphSurface.makeImageSnapshot();
  const pixels = glyphSnapshot.readPixels(0, 0, {
    width: 64,
    height: 64,
    colorType: CanvasKit.ColorType.RGBA_8888,
    alphaType: CanvasKit.AlphaType.Unpremul,
    colorSpace: CanvasKit.ColorSpace.SRGB,
  });
  assert.ok(pixels, 'exact-face GlyphRun pixels를 읽을 수 있어야 한다');
  assert.ok(
    Array.from({ length: 64 * 64 }, (_, index) => pixels[index * 4 + 3])
      .some(alpha => alpha !== 0),
    'TTC face 1 GlyphRun은 실제 ink pixel을 만들어야 한다',
  );
} finally {
  glyphSnapshot?.delete();
  glyphPaint.delete();
  glyphSurface.delete();
}

const invalidFaceResources = {
  ...glyphRunFontResources,
  faces: [{ ...glyphRunFontResources.faces[0], id: `${exactFaceResourceKey}:face:2`, faceIndex: 2 }],
};
const invalidFaceOp = {
  ...glyphRunOp,
  shapeKey: {
    ...glyphRunOp.shapeKey,
    fontInstance: { ...glyphRunOp.shapeKey.fontInstance, faceKey: `${exactFaceResourceKey}:face:2` },
  },
};
const invalidFaceStatus = glyphRunFontCache.replayStatus(invalidFaceOp, invalidFaceResources);
assert.equal(invalidFaceStatus.replayable, false);
assert.equal(invalidFaceStatus.reason, 'faceIndexUnsupported');
const missingBlobTableStatus = glyphRunFontCache.replayStatus(glyphRunOp, {
  faces: glyphRunFontResources.faces,
});
assert.equal(missingBlobTableStatus.replayable, false);
assert.equal(missingBlobTableStatus.reason, 'fontResourceTableMalformed');
glyphRunFontCache.clear();
assert.deepEqual(
  glyphRunFontCache.diagnostics(),
  { blobs: 0, typefaces: 0, fonts: 0, bytes: 0 },
  '문서 resource reset은 GlyphRun native 객체와 blob을 모두 비워야 한다',
);

console.log('CanvasKit bundled fonts, old-Hangul shaping, and exact TTC GlyphRun coverage passed');
