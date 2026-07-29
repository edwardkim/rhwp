// rhwp 문서 형식 검증 모듈
'use strict';

/** HWP 5.0 (OLE2 Compound Document) 매직 넘버 */
const HWP_SIGNATURE = [0xD0, 0xCF, 0x11, 0xE0];

/** HWPX (ZIP) 매직 넘버 */
const HWPX_SIGNATURE = [0x50, 0x4B, 0x03, 0x04];

/** HML 루트 판별에 사용할 최대 앞부분 크기 */
const HML_PREFIX_LIMIT = 64 * 1024;

const UTF8_BOM = [0xEF, 0xBB, 0xBF];
const UTF16LE_BOM = [0xFF, 0xFE];
const UTF16BE_BOM = [0xFE, 0xFF];

/**
 * 바이트 배열의 시작이 주어진 시그니처와 일치하는지 확인한다.
 * @param {Uint8Array} bytes
 * @param {number[]} signature
 * @returns {boolean}
 */
function matchesSignature(bytes, signature) {
  if (bytes.length < signature.length) return false;
  for (let i = 0; i < signature.length; i++) {
    if (bytes[i] !== signature[i]) return false;
  }
  return true;
}

/**
 * HML 앞부분을 Rust HML 판별기와 같은 문자 인코딩 규칙으로 복호화한다.
 * @param {Uint8Array} bytes
 * @returns {string|null}
 */
function decodeHmlPrefix(bytes) {
  const prefix = bytes.subarray(0, Math.min(bytes.length, HML_PREFIX_LIMIT));

  try {
    if (matchesSignature(prefix, UTF8_BOM)) {
      return new TextDecoder('utf-8').decode(prefix.subarray(UTF8_BOM.length));
    }
    if (matchesSignature(prefix, UTF16LE_BOM)) {
      return new TextDecoder('utf-16le').decode(prefix.subarray(UTF16LE_BOM.length));
    }
    if (matchesSignature(prefix, UTF16BE_BOM)) {
      return new TextDecoder('utf-16be').decode(prefix.subarray(UTF16BE_BOM.length));
    }
    return new TextDecoder('utf-8').decode(prefix);
  } catch {
    return null;
  }
}

/**
 * 큰따옴표/작은따옴표 속성 안의 `>`를 건너뛰고 시작 태그의 끝을 찾는다.
 * @param {string} text
 * @param {number} offset
 * @returns {number}
 */
function findStartTagEnd(text, offset) {
  let quote = null;
  for (let index = offset; index < text.length; index += 1) {
    const character = text[index];
    if (quote) {
      if (character === quote) quote = null;
      continue;
    }
    if (character === '"' || character === "'") {
      quote = character;
    } else if (character === '>') {
      return index;
    }
  }
  return -1;
}

/**
 * XML 선언/주석/공백 뒤의 HWPML Version 루트만 HML로 판정한다.
 * @param {string|null} xml
 * @returns {boolean}
 */
function hasHwpmlRoot(xml) {
  if (typeof xml !== 'string') return false;

  let offset = 0;
  while (offset < xml.length) {
    while (offset < xml.length && /\s/.test(xml[offset])) offset += 1;
    if (xml.startsWith('<?', offset)) {
      const end = xml.indexOf('?>', offset + 2);
      if (end < 0) return false;
      offset = end + 2;
      continue;
    }
    if (xml.startsWith('<!--', offset)) {
      const end = xml.indexOf('-->', offset + 4);
      if (end < 0) return false;
      offset = end + 3;
      continue;
    }
    break;
  }

  if (!xml.startsWith('<HWPML', offset)) return false;
  const nameEnd = offset + '<HWPML'.length;
  if (!/[\s/>]/.test(xml[nameEnd] || '')) return false;

  const tagEnd = findStartTagEnd(xml, nameEnd);
  if (tagEnd < 0) return false;
  const attributes = xml.slice(nameEnd, tagEnd);
  return /(?:^|\s)Version\s*=\s*(["'])(?=[^"'])[^"']+\1/.test(attributes);
}

/**
 * 파일 데이터가 HWP, HWPX 또는 HML 문서인지 판정한다.
 * @param {ArrayBuffer|Uint8Array} data
 * @returns {{ isDocument: boolean, format: 'hwp'|'hwpx'|'hml'|null }}
 */
function verifyDocumentSignature(data) {
  const bytes = data instanceof Uint8Array ? data : new Uint8Array(data);
  if (bytes.length < 4) return { isDocument: false, format: null };

  if (matchesSignature(bytes, HWP_SIGNATURE)) {
    return { isDocument: true, format: 'hwp' };
  }
  if (matchesSignature(bytes, HWPX_SIGNATURE)) {
    return { isDocument: true, format: 'hwpx' };
  }
  if (hasHwpmlRoot(decodeHmlPrefix(bytes))) {
    return { isDocument: true, format: 'hml' };
  }

  return { isDocument: false, format: null };
}

/**
 * 파일 데이터가 HWP 또는 HWPX 파일인지 매직 넘버로 확인한다.
 * @param {ArrayBuffer|Uint8Array} data
 * @returns {{ isHwp: boolean, format: 'hwp'|'hwpx'|null }}
 */
function verifyHwpSignature(data) {
  const result = verifyDocumentSignature(data);
  const isHwp = result.format === 'hwp' || result.format === 'hwpx';
  return { isHwp, format: isHwp ? result.format : null };
}

// 내보내기
if (typeof module !== 'undefined' && module.exports) {
  module.exports = {
    HWP_SIGNATURE,
    HWPX_SIGNATURE,
    decodeHmlPrefix,
    hasHwpmlRoot,
    verifyDocumentSignature,
    verifyHwpSignature,
  };
}
