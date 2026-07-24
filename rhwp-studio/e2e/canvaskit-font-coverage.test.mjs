import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

import CanvasKitInit from 'canvaskit-wasm/bin/full/canvaskit.js';

const studioRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const fontPath = path.resolve(studioRoot, '../assets/fonts/NotoSansKR-Regular.woff2');
const symbolFontPath = path.resolve(studioRoot, '../assets/fonts/D2Coding-Regular.woff2');
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

console.log('CanvasKit Noto Sans KR and D2Coding symbol coverage passed');
