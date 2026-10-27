// 요청 family와 실제 SFNT 프로그램의 소유가 확인된 경우만 자체 굵기를 보존한다.
const PREFIX = '__rhwp_intrinsic__';
export const intrinsicCanvasFontAlias = family => PREFIX + family;
export const intrinsicCanvasSourceFamily = family => family.startsWith(PREFIX) ? family.slice(PREFIX.length) : null;

/** 임의 별칭으로 연결한 대체 글꼴을 원 face로 인정하지 않는다. SFNT face 0의 이름을 읽는다. */
export function fontProgramOwnsFamily(bytes, family) {
  try {
    const view = bytes instanceof ArrayBuffer ? new DataView(bytes)
      : new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
    if (view.byteLength < 12 || view.byteLength > 32 * 1024 * 1024) return false;
    if (![0x00010000, 0x4f54544f, 0x74727565].includes(view.getUint32(0))) return false;
    const count = view.getUint16(4);
    if (12 + count * 16 > view.byteLength) return false;
    for (let i = 0; i < count; i++) {
      const record = 12 + i * 16;
      if (view.getUint32(record) !== 0x6e616d65) continue;
      const start = view.getUint32(record + 8), length = view.getUint32(record + 12);
      if (length < 6 || start + length > view.byteLength) return false;
      const names = view.getUint16(start + 2), strings = view.getUint16(start + 4);
      if (6 + names * 12 > length || strings < 6 + names * 12 || strings > length) return false;
      for (let j = 0; j < names; j++) {
        const entry = start + 6 + j * 12;
        const platform = view.getUint16(entry), encoding = view.getUint16(entry + 2), id = view.getUint16(entry + 6);
        if (![1, 4, 6, 16, 21].includes(id) || !(platform === 0 || (platform === 3 && [0, 1, 10].includes(encoding)))) continue;
        const size = view.getUint16(entry + 8), offset = strings + view.getUint16(entry + 10);
        if (size % 2 || offset + size > length) continue;
        const name = new TextDecoder('utf-16be', {fatal: true}).decode(new Uint8Array(view.buffer, view.byteOffset + start + offset, size));
        if (name.trim().toLowerCase() === family.trim().toLowerCase()) return true;
      }
    }
  } catch { /* 알 수 없는 프로그램은 기존 fallback 굵기를 유지한다. */ }
  return false;
}

/** 공급된 보통 face를 암묵적 굵기용 별칭에 연결한다. 명시적 bold 사다리는 바꾸지 않는다. */
export async function installIntrinsicCanvasFont(family, bytes, descriptors) {
  if (Number(descriptors.weight || 400) >= 600 || !fontProgramOwnsFamily(bytes, family)) return null;
  const font = new FontFace(intrinsicCanvasFontAlias(family), bytes, {...descriptors, weight: '700'});
  await font.load(); document.fonts.add(font);
  return font;
}
