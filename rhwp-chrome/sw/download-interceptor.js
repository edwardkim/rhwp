// 다운로드 가로채기
// - .hwp/.hwpx 다운로드 감지 → 뷰어로 열기
// - 사용자 설정(autoOpen)에 따라 동작
//
// #198 (chrome-fd-001): HWP 가 아닌 일반 파일 다운로드에는 suggest() 를 호출하지 않아
//                       Chrome 의 마지막 저장 위치 기억 동작을 보존한다.

import { openViewer } from './viewer-launcher.js';

/** filename 또는 URL 에서 .hwp/.hwpx 확장자를 감지 (쿼리 문자열 허용). */
const HWP_EXTENSION_RE = /\.(hwp|hwpx)(\?|$)/i;

/** 한컴 HWP/HWPX MIME 타입 힌트 (소문자 비교). */
const HWP_MIME_HINTS = ['haansoft', 'x-hwp', 'hwp+zip'];

/**
 * 재요청 불가 다운로드 패턴 (#198).
 *
 * Chrome `chrome.downloads.DownloadItem` 에는 HTTP method 필드가 없어 POST 직접 감지 불가.
 * 대신 url / referrer 의 알려진 핸들러 패턴으로 추정한다.
 *
 * 이런 다운로드는:
 * - rhwp 뷰어가 url 을 GET 으로 재요청해도 빈 응답/에러 반환 (POST 전용)
 * - 토큰/세션 만료 가능
 * → 인터셉트 포기 (Chrome 기본 다운로드만 진행, 빈 뷰어 탭 안 띄움)
 *
 * 블랙리스트 방식으로 운영. 사용자 보고로 새 패턴이 들어오면 본 배열에 추가.
 */
const NON_REFETCHABLE_PATTERNS = [
  /\/dext5handler\.[a-z0-9]+/i,  // DEXT5 (예: dext5handler.ndo, .jsp, .do)
];

/**
 * 다운로드 항목이 HWP/HWPX 인지 판별 (#198).
 *
 * filename / url / finalUrl / mime 어느 하나라도 매치되면 true.
 * Chrome API 와 무관한 순수 함수 — 단위 테스트 가능.
 *
 * @param {{filename?: string, url?: string, finalUrl?: string, mime?: string}} item
 * @returns {boolean}
 */
export function shouldInterceptDownload(item) {
  if (!item) return false;

  // 재요청 불가 패턴 (POST / 세션 의존 핸들러) — 뷰어가 GET 으로 다시 받지 못함
  // → 인터셉트 포기 (Chrome 기본 다운로드만 진행, 빈 뷰어 탭 안 띄움)
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

/**
 * 다운로드 인터셉터를 설정한다.
 *
 * #198 변경:
 * - HWP/HWPX 다운로드: handleHwpDownload + suggest 호출 (자체 뷰어 트리거)
 * - 일반 파일: suggest 호출 안 함 → Chrome 의 마지막 저장 위치 기억 동작 유지
 */
export function setupDownloadInterceptor() {
  chrome.downloads.onDeterminingFilename.addListener((item, suggest) => {
    if (shouldInterceptDownload(item)) {
      handleHwpDownload(item);
      suggest({ filename: item.filename });
    }
    // HWP 가 아니면 suggest 호출하지 않는다 — Chrome 기본 동작 유지 (#198)
  });
}

async function handleHwpDownload(item) {
  try {
    const settings = await chrome.storage.sync.get({ autoOpen: true });

    if (settings.autoOpen) {
      // 대용량 파일 경고 (50MB 초과)
      if (item.fileSize > 50 * 1024 * 1024) {
        console.warn(`[rhwp] 대용량 파일: ${item.filename} (${(item.fileSize / 1024 / 1024).toFixed(1)}MB)`);
      }

      openViewer({
        url: item.url,
        filename: item.filename
      });
    }
  } catch (err) {
    console.error('[rhwp] 다운로드 인터셉터 오류:', err);
  }
}
