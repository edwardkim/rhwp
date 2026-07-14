// 원격 로드 문서의 바이트 시그니처(매직 넘버) 판정.
//
// URL 파라미터/다운로드로 받은 응답이 실제 HWP/HWPX/HML 문서인지, 아니면 로그인/오류
// 미리보기 HTML 페이지인지를 매직 우선으로 가린다. 서버가 Content-Type 을 부정확하게
// 보내도(예: HWP3 본문에 text/html 404 헤더 — 조달청 pps.go.kr) 본문 매직이 우선한다.

export type DocumentByteKind = 'hwp' | 'hwpx' | 'hml' | 'html' | 'xml' | 'unknown';

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

function decodeXmlPrefix(bytes: Uint8Array): string {
  const prefixLength = Math.min(bytes.length, 64 * 1024);
  if (startsWithBytes(bytes, [0xFF, 0xFE])) {
    return new TextDecoder('utf-16le').decode(bytes.subarray(2, prefixLength));
  }
  if (startsWithBytes(bytes, [0xFE, 0xFF])) {
    return new TextDecoder('utf-16be').decode(bytes.subarray(2, prefixLength));
  }
  return new TextDecoder('utf-8').decode(bytes.subarray(0, prefixLength));
}

function skipXmlWhitespace(prefix: string, offset: number): number {
  while (offset < prefix.length && /\s/.test(prefix[offset])) offset += 1;
  return offset;
}

function matchRootElement(prefix: string): RegExpMatchArray | null {
  let offset = skipXmlWhitespace(prefix, 0);

  if (prefix.slice(offset, offset + 5).toLowerCase() === '<?xml') {
    const declarationEnd = prefix.indexOf('?>', offset + 5);
    if (declarationEnd === -1) return null;
    offset = skipXmlWhitespace(prefix, declarationEnd + 2);
  }

  while (prefix.startsWith('<!--', offset)) {
    const commentEnd = prefix.indexOf('-->', offset + 4);
    if (commentEnd === -1) return null;
    offset = skipXmlWhitespace(prefix, commentEnd + 3);
  }

  const rootPattern = /<(?:[A-Za-z_][\w.-]*:)?([A-Za-z_][\w.-]*)\b([^>]*)>/iy;
  rootPattern.lastIndex = offset;
  return rootPattern.exec(prefix);
}

function isHmlRoot(root: RegExpMatchArray | null, prefix: string): boolean {
  if (!root || root[1] !== 'HWPML' || /\/\s*$/.test(root[2])) return false;
  const afterRoot = prefix.slice((root.index ?? 0) + root[0].length);
  if (/^\s*<\/HWPML\s*>\s*$/.test(afterRoot)) return false;
  const hasHancomNamespace = /\bxmlns(?::[A-Za-z_][\w.-]*)?\s*=\s*['"]https?:\/\/www\.hancom\.co\.kr\/hwpml(?:\/[^'"]*)?['"]/i
    .test(root[2]);
  if (hasHancomNamespace) return /<[A-Za-z_][\w.-]*(?::[A-Za-z_][\w.-]*)?(?:\s|\/?>)/.test(afterRoot);

  const hasVersion = /(?:^|\s)Version\s*=\s*['"]\d+(?:\.\d+)*['"]/.test(root[2]);
  const hasSubVersion = /(?:^|\s)SubVersion\s*=\s*['"]\d+(?:\.\d+)*['"]/.test(root[2]);
  const hasStyle = /(?:^|\s)Style\s*=\s*['"][^'"]+['"]/.test(root[2]);
  return hasVersion && hasSubVersion && hasStyle;
}

export function detectDocumentByteKind(
  bytes: Uint8Array,
  contentType?: string | null,
): DocumentByteKind {
  if (startsWithBytes(bytes, HWP_CFB_SIGNATURE)) return 'hwp';
  if (startsWithBytes(bytes, HWP3_SIGNATURE)) return 'hwp';
  if (ZIP_SIGNATURES.some(signature => startsWithBytes(bytes, signature))) return 'hwpx';

  const prefix = decodeXmlPrefix(bytes).trimStart();
  const root = matchRootElement(prefix);
  if (isHmlRoot(root, prefix)) return 'hml';

  const normalizedPrefix = prefix.toLowerCase();
  if (normalizedPrefix.startsWith('<!doctype html') || root?.[1].toLowerCase() === 'html') {
    return 'html';
  }
  if (normalizedPrefix.startsWith('<?xml') || root) return 'xml';

  const declaredContentType = contentType?.toLowerCase() ?? '';
  if (declaredContentType.includes('text/html')) return 'html';

  return 'unknown';
}

export function assertRemoteDocumentBytes(bytes: Uint8Array, contentType?: string | null): void {
  const kind = detectDocumentByteKind(bytes, contentType);
  if (kind === 'hwp' || kind === 'hwpx' || kind === 'hml') return;

  if (kind === 'html') {
    throw new Error('실제 HWP/HWPX/HML 파일이 아닙니다. 파일 미리보기/오류 페이지가 반환되었습니다.');
  }

  if (kind === 'xml') {
    throw new Error('HML 문서가 아닌 일반 XML 파일입니다.');
  }

  throw new Error('실제 HWP/HWPX/HML 파일이 아닙니다. 파일 시그니처를 확인할 수 없습니다.');
}
