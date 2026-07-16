import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

import CanvasKitInit from 'canvaskit-wasm/bin/full/canvaskit.js';

const studioRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const fontPath = path.resolve(studioRoot, '../assets/fonts/NotoSansKR-Regular.woff2');
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

console.log('CanvasKit Noto Sans KR symbol coverage passed');
