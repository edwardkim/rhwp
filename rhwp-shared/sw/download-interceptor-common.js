// 다운로드 인터셉터 공통 판정 로직 (Chrome / Firefox 공용)
//
// Chrome·Firefox의 onCreated/onChanged adapter가 동일한
// 판정 기준을 사용하도록 추출한 순수 함수. 브라우저 API에 의존하지 않아
// 단위 테스트 가능.
//
// 원본: rhwp-shared/sw/download-interceptor-common.js
// 참조: rhwp-chrome/sw/download-interceptor-common.js (symlink)
//       rhwp-firefox/sw/download-interceptor-common.js (symlink)
//
// 관련 이슈:
// - #198: Chrome 마지막 저장 위치 보존 + DEXT5 블랙리스트 + MIME 힌트
// - #207: 동일 판정 로직을 Firefox 측에도 적용

/** filename 또는 URL 에서 .hwp/.hwpx/.hml 확장자를 감지 (쿼리 문자열 허용). */
export const HWP_EXTENSION_RE = /\.(hwp|hwpx|hml)(\?|$)/i;

/** 한컴 HWP/HWPX MIME 타입 힌트 (소문자 비교). */
export const HWP_MIME_HINTS = ['haansoft', 'x-hwp', 'hwp+zip'];

/** 확정 파일명에서 HWP가 아님을 강하게 나타내는 문서 확장자. */
export const NON_HWP_EXTENSION_RE = /\.(?:xls|xlsx|xlsm|xlsb|xlt|xltx|doc|docx|docm|dot|dotx|ppt|pptx|pptm|pot|potx|pdf|zip|odt|ods|odp)(?:[?#]|$)/i;

/** generic/octet-stream과 달리 HWP가 아님을 강하게 나타내는 MIME prefix. */
export const NON_HWP_MIME_PREFIXES = [
  'application/vnd.openxmlformats-officedocument.',
  'application/vnd.ms-excel',
  'application/vnd.ms-word',
  'application/vnd.ms-powerpoint',
  'application/vnd.oasis.opendocument.',
  'application/pdf',
  'application/zip',
];

/**
 * 재요청 불가 다운로드 패턴 (#198).
 *
 * POST 요청 / 세션 토큰 의존 핸들러는 rhwp 뷰어가 url 을 GET 으로 다시 받지 못해
 * 빈 응답/에러 발생 → 인터셉트 포기 (브라우저 기본 다운로드만 진행).
 *
 * 블랙리스트 방식. 사용자 보고로 새 패턴이 들어오면 본 배열에 추가.
 */
export const NON_REFETCHABLE_PATTERNS = [
  /\/dext5handler\.[a-z0-9]+/i,  // DEXT5 (예: dext5handler.ndo, .jsp, .do)
];

/**
 * 다운로드 항목의 HWP 자동 열기 여부를 근거 우선순위와 이벤트 단계로 판별한다.
 *
 * filename / url / finalUrl / mime / referrer의 충돌을 명시적으로 해소한다.
 * Chrome `chrome.downloads.DownloadItem` 과 Firefox `browser.downloads.DownloadItem`
 * 모두 대응한다.
 *
 * @param {{filename?: string, url?: string, finalUrl?: string, mime?: string, referrer?: string}} item
 * @param {{metadataFinalized?: boolean}} [context]
 * @returns {{action: 'intercept'|'defer'|'ignore', reason: string}}
 */
export function classifyDownload(item, { metadataFinalized = false } = {}) {
  if (!item) return { action: 'ignore', reason: 'no-hwp-evidence' };

  // 재요청 불가 패턴 (POST / 세션 의존 핸들러)
  const url = item.url || '';
  const referrer = item.referrer || '';
  if (NON_REFETCHABLE_PATTERNS.some(re => re.test(url) || re.test(referrer))) {
    return { action: 'ignore', reason: 'non-refetchable' };
  }

  const filename = item.filename || '';
  if (HWP_EXTENSION_RE.test(filename)) {
    return { action: 'intercept', reason: 'hwp-filename' };
  }
  if (NON_HWP_EXTENSION_RE.test(filename)) {
    return { action: 'ignore', reason: 'non-hwp-filename' };
  }

  const mime = (item.mime || '').trim().toLowerCase();
  if (NON_HWP_MIME_PREFIXES.some(prefix => mime.startsWith(prefix))) {
    return { action: 'ignore', reason: 'non-hwp-mime' };
  }

  const finalUrl = item.finalUrl || '';
  const hasHwpEvidence = HWP_EXTENSION_RE.test(url)
    || (finalUrl !== url && HWP_EXTENSION_RE.test(finalUrl))
    || HWP_MIME_HINTS.some(hint => mime.includes(hint));

  if (!hasHwpEvidence) {
    return { action: 'ignore', reason: 'no-hwp-evidence' };
  }

  if (!metadataFinalized) {
    return { action: 'defer', reason: 'provisional-hwp-evidence' };
  }

  return { action: 'intercept', reason: 'final-hwp-evidence' };
}

/**
 * 기존 boolean 소비자를 위한 호환 wrapper.
 *
 * 단계 정보가 없는 호출은 종전처럼 현재 item을 최종 메타데이터로 보고 판정한다.
 * Chrome/Firefox observer는 반드시 `classifyDownload()`에 실제 단계 context를 전달한다.
 *
 * @param {{filename?: string, url?: string, finalUrl?: string, mime?: string, referrer?: string}} item
 * @returns {boolean}
 */
export function shouldInterceptDownload(item) {
  return classifyDownload(item, { metadataFinalized: true }).action === 'intercept';
}
