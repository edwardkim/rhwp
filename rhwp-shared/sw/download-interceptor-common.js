// 다운로드 인터셉터 공통 판정 로직 (Chrome / Firefox 공용)
//
// Chrome(onDeterminingFilename)·Firefox(onCreated/onChanged) 모두 동일한
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
 * 다운로드 항목이 HWP/HWPX 인지 판별 (#198 / #207).
 *
 * filename / url / finalUrl / mime / referrer 어느 조합이든 수용.
 * Chrome `chrome.downloads.DownloadItem` 과 Firefox `browser.downloads.DownloadItem`
 * 모두 대응 (필드 부재 시 안전하게 false 반환).
 *
 * @param {{filename?: string, url?: string, finalUrl?: string, mime?: string, referrer?: string}} item
 * @returns {boolean}
 */
export function shouldInterceptDownload(item) {
  if (!item) return false;

  // 재요청 불가 패턴 (POST / 세션 의존 핸들러)
  const url = item.url || '';
  const referrer = item.referrer || '';
  if (NON_REFETCHABLE_PATTERNS.some(re => re.test(url) || re.test(referrer))) {
    return false;
  }

  const filename = item.filename || '';
  if (HWP_EXTENSION_RE.test(filename)) return true;

  if (HWP_EXTENSION_RE.test(url)) return true;

  const finalUrl = item.finalUrl || '';
  if (finalUrl !== url && HWP_EXTENSION_RE.test(finalUrl)) return true;

  const mime = (item.mime || '').toLowerCase();
  if (HWP_MIME_HINTS.some(hint => mime.includes(hint))) return true;

  return false;
}
