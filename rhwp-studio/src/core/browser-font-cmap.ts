/** Repair only an invalid format 4 terminal U+FFFF mapping in a renderer copy. */
export function repairTerminalFontCmap(fontData: ArrayBuffer): ArrayBuffer {
  const source = new DataView(fontData);
  const has = (offset: number, length: number): boolean => (
    Number.isSafeInteger(offset) && offset >= 0 && length >= 0
    && offset <= source.byteLength - length
  );
  if (!has(0, 12) || source.getUint32(0) !== 0x0001_0000) return fontData;
  const count = source.getUint16(4);
  if (count === 0 || count > 4096 || !has(12, count * 16)) return fontData;
  const tables = new Map<number, { record: number; offset: number; length: number }>();
  for (let index = 0; index < count; index += 1) {
    const record = 12 + index * 16;
    const offset = source.getUint32(record + 8);
    const length = source.getUint32(record + 12);
    if (!has(offset, length)) return fontData;
    tables.set(source.getUint32(record), { record, offset, length });
  }
  const cmap = tables.get(0x636d_6170);
  const maxp = tables.get(0x6d61_7870);
  const head = tables.get(0x6865_6164);
  if (!cmap || cmap.length < 4 || !maxp || maxp.length < 6 || !head || head.length < 12) return fontData;
  if (source.getUint16(cmap.offset) !== 0) return fontData;
  const records = source.getUint16(cmap.offset + 2);
  if (4 + records * 8 > cmap.length) return fontData;
  const glyphCount = source.getUint16(maxp.offset + 4);
  if (glyphCount === 0) return fontData;
  let repaired: DataView | null = null;
  for (let index = 0; index < records; index += 1) {
    const relative = source.getUint32(cmap.offset + 4 + index * 8 + 4);
    if (relative > cmap.length - 8) continue;
    const offset = cmap.offset + relative;
    if (source.getUint16(offset) !== 4) continue;
    const length = source.getUint16(offset + 2);
    const twice = source.getUint16(offset + 6);
    if (twice === 0 || twice % 2 !== 0 || length > cmap.length - relative) continue;
    const segments = twice / 2;
    if (length < 16 + segments * 8) continue;
    const end = offset + 14 + (segments - 1) * 2;
    const start = end + segments * 2 + 2;
    const delta = start + segments * 2;
    const range = delta + segments * 2;
    if (
      source.getUint16(end) === 0xffff && source.getUint16(start) === 0xffff
      && source.getUint16(range) === 0
      && ((0xffff + source.getUint16(delta)) & 0xffff) >= glyphCount
    ) {
      repaired ??= new DataView(fontData.slice(0));
      repaired.setUint16(delta, 1);
    }
  }
  if (!repaired) return fontData;
  const bytes = new Uint8Array(repaired.buffer);
  const checksum = (offset: number, length: number): number => {
    let sum = 0;
    for (let index = 0; index < length; index += 4) {
      let word = 0;
      for (let byte = 0; byte < 4; byte += 1) {
        word = (word << 8) | (index + byte < length ? bytes[offset + index + byte] : 0);
      }
      sum = (sum + (word >>> 0)) >>> 0;
    }
    return sum;
  };
  repaired.setUint32(cmap.record + 4, checksum(cmap.offset, cmap.length));
  repaired.setUint32(head.offset + 8, 0);
  repaired.setUint32(head.offset + 8, (0xb1b0_afba - checksum(0, bytes.length)) >>> 0);
  return repaired.buffer as ArrayBuffer;
}
