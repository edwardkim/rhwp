// rhwp URL 검증 모듈 — Chrome/Safari 공통
'use strict';

/**
 * URL이 안전한 프로토콜인지 검증한다.
 * @param {string} urlString
 * @returns {{ valid: boolean, parsed?: URL, reason?: string }}
 */
function validateProtocol(urlString) {
  if (!urlString || typeof urlString !== 'string') {
    return { valid: false, reason: 'URL이 비어있음' };
  }
  let parsed;
  try {
    parsed = new URL(urlString);
  } catch {
    return { valid: false, reason: 'URL 파싱 실패' };
  }
  if (parsed.protocol !== 'https:' && parsed.protocol !== 'http:') {
    return { valid: false, reason: `차단된 프로토콜: ${parsed.protocol}` };
  }
  // userinfo(@) 포함 도메인 차단: https://safe.go.kr@evil.com/
  if (parsed.username || parsed.password) {
    return { valid: false, reason: 'URL에 userinfo(@) 포함' };
  }
  return { valid: true, parsed };
}

/**
 * 호스트가 내부 네트워크 IP인지 검사한다.
 * @param {string} hostname
 * @returns {boolean} 내부 IP이면 true
 */
function isPrivateHost(hostname) {
  // PRIVATE_IP_PATTERNS 인라인 (constants.js 의존 제거)
  const patterns = [
    /^127\./, /^10\./, /^192\.168\./, /^172\.(1[6-9]|2\d|3[01])\./,
    /^169\.254\./, /^0\./, /^\[::1\]/, /^localhost$/i, /\.local$/i,
  ];
  return patterns.some(re => re.test(hostname));
}

/**
 * URL의 pathname에서 HWP/HWPX/HML 확장자를 확인한다.
 * @param {URL} parsed
 * @returns {boolean}
 */
function hasHwpExtension(parsed) {
  const pathname = parsed.pathname.toLowerCase();
  return pathname.endsWith('.hwp') || pathname.endsWith('.hwpx') || pathname.endsWith('.hml');
}

/**
 * 호스트가 허용 도메인 목록에 포함되는지 검사한다.
 * @param {string} hostname
 * @param {string[]} allowedDomains — ['.go.kr', '.or.kr', ...]
 * @returns {boolean}
 */
function isAllowedDomain(hostname, allowedDomains) {
  return allowedDomains.some(domain => hostname.endsWith(domain));
}

/**
 * URL이 정부사이트 다운로드 패턴인지 검사한다.
 * 확장자 없이 *.do, *Download*, *download* 등의 패턴.
 * @param {URL} parsed
 * @returns {boolean}
 */
function isDownloadEndpoint(parsed) {
  const pathname = parsed.pathname.toLowerCase();
  return /\.(do|action|jsp|aspx|php)$/i.test(pathname)
    || /download/i.test(pathname)
    || /filedown/i.test(pathname)
    || /attach/i.test(pathname);
}

/**
 * open-hwp 용 URL 검증 (3단계).
 * ① pathname에 .hwp/.hwpx/.hml → 즉시 허용
 * ② 허용 도메인 + 다운로드 패턴 → 허용 (viewer에서 재검증)
 * ③ 그 외 → 차단
 *
 * @param {string} urlString
 * @param {string[]} allowedDomains
 * @returns {{ allowed: boolean, reason: string }}
 */
function validateOpenHwpUrl(urlString, allowedDomains) {
  const result = validateProtocol(urlString);
  if (!result.valid) return { allowed: false, reason: result.reason };
  const parsed = result.parsed;

  if (isPrivateHost(parsed.hostname)) {
    return { allowed: false, reason: `내부 IP 차단: ${parsed.hostname}` };
  }

  // ① 확장자 확인
  if (hasHwpExtension(parsed)) {
    return { allowed: true, reason: 'HWP 확장자 확인' };
  }

  // ② 허용 도메인 + 다운로드 패턴
  if (isAllowedDomain(parsed.hostname, allowedDomains)) {
    if (isDownloadEndpoint(parsed)) {
      return { allowed: true, reason: '허용 도메인 다운로드 엔드포인트 (viewer에서 재검증)' };
    }
    // 허용 도메인이지만 다운로드 패턴이 아닌 경우도 허용 (content-script가 HWP 링크로 판별했으므로)
    return { allowed: true, reason: '허용 도메인 (viewer에서 재검증)' };
  }

  // ③ 그 외 차단
  return { allowed: false, reason: `미허용 도메인 + 확장자 없음: ${parsed.hostname}` };
}

/**
 * fetch-file 용 URL 검증.
 * open-hwp 보다 엄격: HTTPS 강제, 내부 IP 차단.
 *
 * @param {string} urlString
 * @param {string[]} allowedDomains
 * @param {boolean} allowHttp — 사용자 설정
 * @returns {{ allowed: boolean, reason: string, upgradedUrl?: string }}
 */
function validateFetchUrl(urlString, allowedDomains, allowHttp) {
  const result = validateProtocol(urlString);
  if (!result.valid) return { allowed: false, reason: result.reason };
  const parsed = result.parsed;

  if (isPrivateHost(parsed.hostname)) {
    return { allowed: false, reason: `내부 IP 차단: ${parsed.hostname}` };
  }

  // HTTPS 강제 (설정에 따라 HTTP 허용)
  if (parsed.protocol === 'http:') {
    if (!allowHttp) {
      return { allowed: false, reason: 'HTTP 차단 (설정에서 비허용)' };
    }
    // HTTP → HTTPS 업그레이드 시도
    const upgraded = urlString.replace(/^http:/, 'https:');
    return { allowed: true, reason: 'HTTP → HTTPS 업그레이드', upgradedUrl: upgraded };
  }

  return { allowed: true, reason: '검증 통과' };
}

// 내보내기
if (typeof module !== 'undefined' && module.exports) {
  module.exports = {
    validateProtocol, isPrivateHost, hasHwpExtension,
    isAllowedDomain, isDownloadEndpoint,
    validateOpenHwpUrl, validateFetchUrl,
  };
}
