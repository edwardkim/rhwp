/**
 * 문서 비교 디버그 (밀림 / ID / 폴백 원인 추적).
 *
 * 켜는 방법(택일):
 * - `localStorage.setItem('rhwp:compareDebug', '1')` 후 페이지 새로고침
 * - URL에 `?compareDebug=1`
 * - 콘솔에서 `globalThis.__RHWP_COMPARE_DEBUG__ = true`
 */

export function isCompareDebugEnabled(): boolean {
  try {
    if (typeof globalThis !== 'undefined' && (globalThis as unknown as { __RHWP_COMPARE_DEBUG__?: boolean }).__RHWP_COMPARE_DEBUG__) {
      return true;
    }
    if (typeof window !== 'undefined') {
      if (new URLSearchParams(window.location.search).get('compareDebug') === '1') return true;
      if (window.localStorage?.getItem('rhwp:compareDebug') === '1') return true;
    }
  } catch {
    /* localStorage 접근 불가 */
  }
  return false;
}

export function compareDbg(...args: unknown[]): void {
  if (!isCompareDebugEnabled()) return;
  console.log('[rhwp:compare]', ...args);
}
