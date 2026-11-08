// Both browser FontFace and CanvasKit consume the same selected SFNT face.
export { normalizeFontFaceData as canvasKitFontFaceData } from '../../core/sfnt-face.ts';

const MAX_SFNT_TABLE_COUNT = 4096;
const MAX_NORMALIZED_SFNT_BYTES = 64 * 1024 * 1024;

/** Read maxp from an already selected standalone face; container guesses are not proof. */
export function canvasKitSfntGlyphCount(fontData: ArrayBuffer): number | null {
  if (fontData.byteLength < 12 || fontData.byteLength > MAX_NORMALIZED_SFNT_BYTES) return null;
  const view = new DataView(fontData);
  if (![0x0001_0000, 0x4f54_544f, 0x7472_7565, 0x7479_7031].includes(view.getUint32(0, false))) return null;
  const count = view.getUint16(4, false), directoryEnd = 12 + count * 16;
  if (count === 0 || count > MAX_SFNT_TABLE_COUNT || directoryEnd > fontData.byteLength) return null;
  let glyphCount: number | null = null;
  for (let index = 0; index < count; index++) {
    const record = 12 + index * 16;
    if (view.getUint32(record, false) !== 0x6d61_7870) continue;
    const offset = view.getUint32(record + 8, false), length = view.getUint32(record + 12, false);
    if (glyphCount !== null || offset < directoryEnd || length < 6
      || offset > fontData.byteLength || length > fontData.byteLength - offset) return null;
    const version = view.getUint32(offset, false);
    if (version !== 0x0000_5000 && version !== 0x0001_0000) return null;
    glyphCount = view.getUint16(offset + 4, false);
    if (glyphCount === 0) return null;
  }
  return glyphCount;
}
