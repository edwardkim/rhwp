/** 글꼴 바이트는 보존하고 Rust/Native와 같은 가시 기호 공급 범위를 사용한다. */
export type FontSymbolInspector = (bytes: Uint8Array, faceIndex: number) => string;

export function fontSymbolDescriptors(bytes: ArrayBuffer,
  descriptors: Pick<FontFaceDescriptors, 'weight' | 'style'> = {},
  inspect?: FontSymbolInspector): FontFaceDescriptors {
  // 일반 글꼴 공급 도구는 엔진 없이도 사용할 수 있다. 실제 문서 bridge는
  // 초기화한 공통 Rust 검사기를 명시적으로 주입한다.
  if (!inspect || bytes.byteLength < 4) return descriptors;
  const signature = new DataView(bytes).getUint32(0);
  // WOFF 등 해석하지 않은 컨테이너의 glyph를 빈 것으로 추정하지 않는다.
  if (![0x00010000, 0x74727565, 0x4f54544f].includes(signature)) return descriptors;
  const unicodeRange = inspect(new Uint8Array(bytes), 0);
  return unicodeRange ? { ...descriptors, unicodeRange } : descriptors;
}
