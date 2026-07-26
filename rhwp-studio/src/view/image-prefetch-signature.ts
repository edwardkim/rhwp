/**
 * 페이지 그림 prefetch 의 재사용 판정 (Task #3315).
 *
 * prefetch 는 "브라우저 디코드 캐시 데우기"이고, 그 대상은 페이지가 그리는 그림 집합이다.
 * 집합이 그대로면 다시 데울 것이 없는데, 그 사실을 확인하려고 수 MB 짜리 레이어 트리
 * JSON 을 다시 받아 정규식으로 훑는 것은 낭비다. Rust 가 내주는 그림 신원 키 목록
 * (`getPageSourceImageKeys`, 수백 바이트)을 서명으로 쓴다.
 */

export interface PrefetchSignature {
  /** `getPageSourceImageKeys` 응답 원문. */
  imageKeys: string;
  /**
   * 직전 prefetch 때 이 페이지에 rawSvg(차트/OLE 미리보기)가 있었는지.
   *
   * rawSvg 내용은 그림 신원 키가 덮지 못하므로, 하나라도 있으면 건너뛰지 않는다.
   */
  hadRawSvg: boolean;
}

/**
 * 이미 디코드를 마친 그림 집합과 같으면 prefetch 를 건너뛴다.
 *
 * 판정 재료가 없을 때(`imageKeys === null` — 키 조회를 지원하지 않는 구형 WASM, 기록
 * 없음 — 아직 한 번도 데우지 않은 페이지)는 건너뛰지 않는다. 안전망을 없애는 쪽이 아니라
 * 이미 끝난 일을 되풀이하지 않는 쪽으로만 작동해야 한다.
 */
export function shouldSkipImagePrefetch(
  cached: PrefetchSignature | undefined,
  imageKeys: string | null,
): boolean {
  if (imageKeys === null || !cached) return false;
  if (cached.hadRawSvg) return false;
  return cached.imageKeys === imageKeys;
}
