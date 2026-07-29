import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

import CanvasKitInit from 'canvaskit-wasm/bin/full/canvaskit.js';

const studioRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const fontPath = path.resolve(studioRoot, '../assets/fonts/NotoSansKR-Regular.woff2');
const symbolFontPath = path.resolve(studioRoot, '../assets/fonts/D2Coding-Regular.woff2');
const oldHangulFontPath = path.resolve(studioRoot, '../assets/fonts/SourceHanSerifK-OldHangul-subset.woff2');
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

console.log('CanvasKit Noto Sans KR, D2Coding, and old-Hangul shaping coverage passed');
