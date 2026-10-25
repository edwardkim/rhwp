/** 글꼴 바이트는 보존하고 Rust/Native와 같은 가시 기호 공급 범위를 사용한다. */
export async function fontSymbolDescriptors(bytes: ArrayBuffer,
  descriptors: Pick<FontFaceDescriptors, 'weight' | 'style'> = {}): Promise<FontFaceDescriptors> {
  if (bytes.byteLength < 4) return descriptors;
  const signature = new DataView(bytes).getUint32(0);
  // WOFF 등 해석하지 않은 컨테이너의 glyph를 빈 것으로 추정하지 않는다.
  if (![0x00010000, 0x74727565, 0x4f54544f].includes(signature)) return descriptors;
  const exports = await import('@wasm/rhwp.js') as unknown as {
    fontSymbolUnicodeRange(bytes: Uint8Array, faceIndex: number): string;
  };
  const unicodeRange = exports.fontSymbolUnicodeRange(new Uint8Array(bytes), 0);
  return unicodeRange ? { ...descriptors, unicodeRange } : descriptors;
}
