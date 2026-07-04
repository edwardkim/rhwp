// 원격 로드 문서의 바이트 시그니처(매직 넘버) 판정.
//
// URL 파라미터/다운로드로 받은 응답이 실제 HWP/HWPX 문서인지, 아니면 로그인/오류
// 미리보기 HTML 페이지인지를 매직 우선으로 가린다. 서버가 Content-Type 을 부정확하게
// 보내도(예: HWP3 본문에 text/html 404 헤더 — 조달청 pps.go.kr) 본문 매직이 우선한다.

export type DocumentByteKind = 'hwp' | 'hwpx' | 'html' | 'unknown';

// HWP5 OLE 복합 파일(CFB) 매직.
const HWP_CFB_SIGNATURE = [0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1] as const;

// [#1657] HWP3 고전 바이너리 매직 "HWP Document File V3.00" (코어: src/parser/hwp3/mod.rs).
// 조달청(pps.go.kr) 등 공공기관이 아직 HWP3 문서를 배포한다. 코어는 HWP3 를 정식 파싱하나
// 이 게이트가 CFB/ZIP 만 알아 HWP3 를 'unknown' 으로 거부하던 것을 해소한다.
const HWP3_SIGNATURE = [
  0x48, 0x57, 0x50, 0x20, 0x44, 0x6F, 0x63, 0x75, 0x6D, 0x65, 0x6E, 0x74,
  0x20, 0x46, 0x69, 0x6C, 0x65, 0x20, 0x56, 0x33, 0x2E, 0x30, 0x30,
] as const; // "HWP Document File V3.00"

// HWPX(ZIP) 매직 — 로컬/empty/spanned 헤더.
const ZIP_SIGNATURES = [
  [0x50, 0x4B, 0x03, 0x04],
  [0x50, 0x4B, 0x05, 0x06],
  [0x50, 0x4B, 0x07, 0x08],
] as const;

function startsWithBytes(bytes: Uint8Array, signature: readonly number[]): boolean {
  if (bytes.length < signature.length) return false;
  return signature.every((byte, index) => bytes[index] === byte);
}

export function detectDocumentByteKind(
  bytes: Uint8Array,
  contentType?: string | null,
): DocumentByteKind {
  if (startsWithBytes(bytes, HWP_CFB_SIGNATURE)) return 'hwp';
  if (startsWithBytes(bytes, HWP3_SIGNATURE)) return 'hwp';
  if (ZIP_SIGNATURES.some(signature => startsWithBytes(bytes, signature))) return 'hwpx';

  const declaredContentType = contentType?.toLowerCase() ?? '';
  if (declaredContentType.includes('text/html')) return 'html';

  const prefix = new TextDecoder('utf-8')
    .decode(bytes.subarray(0, Math.min(bytes.length, 256)))
    .trimStart()
    .toLowerCase();

  if (prefix.startsWith('<!doctype') || prefix.startsWith('<html') || prefix.startsWith('<?xml')) {
    return 'html';
  }

  return 'unknown';
}

export function assertRemoteDocumentBytes(bytes: Uint8Array, contentType?: string | null): void {
  const kind = detectDocumentByteKind(bytes, contentType);
  if (kind === 'hwp' || kind === 'hwpx') return;

  if (kind === 'html') {
    throw new Error('실제 HWP/HWPX 파일이 아닙니다. 파일 미리보기/오류 페이지가 반환되었습니다.');
  }

  throw new Error('실제 HWP/HWPX 파일이 아닙니다. 파일 시그니처를 확인할 수 없습니다.');
}
