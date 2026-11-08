import { normalizeFontFaceData } from './sfnt-face.ts';

export interface OrdinaryTextFontSource {
  ordinaryTextFontProgram?(bytes: Uint8Array, faceIndex: number): Uint8Array | undefined;
}

/** 일반 텍스트만 공통 Rust의 기본 윤곽 프로그램을 사용한다. 컬러 자원은 이 경로를 쓰지 않는다. */
export function ordinaryTextFontFaceData(
  data: ArrayBuffer, faceIndex: number, owner: OrdinaryTextFontSource | null = null,
): ArrayBuffer | null {
  const selected = normalizeFontFaceData(data, faceIndex);
  if (!selected) return null;
  const prepared = owner?.ordinaryTextFontProgram?.(new Uint8Array(selected), 0);
  // WASM 메모리·임시 view를 FontFace의 수명에 연결하지 않는다.
  return prepared ? Uint8Array.from(prepared).buffer : selected;
}
