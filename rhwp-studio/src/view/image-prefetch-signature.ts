/**
 * 페이지 그림 prefetch 의 재사용 판정 (Task #3315).
 *
 * prefetch 는 "브라우저 디코드 캐시 데우기"이고, 그 대상은 페이지가 그리는 그림 집합이다.
 * 집합이 그대로면 다시 데울 것이 없는데, 그 사실을 확인하려고 수 MB 짜리 레이어 트리
 * JSON 을 다시 받아 정규식으로 훑는 것은 낭비다. Rust 가 내주는 그림 신원 키 목록
 * (`getPageSourceImageKeys`, 수백 바이트)을 서명으로 쓴다.
 */

export interface PrefetchSignature {
  /**
   * 이 서명이 설명하는 문서 (`WasmBridge.documentDigest`).
   *
   * 그림 키는 **문서 안에서만** 신원이다 — `bin_data_id` 는 문서마다 1 부터 다시 매겨지고
   * 세대 번호도 문서마다 0 에서 시작한다. 반면 이 서명을 담는 맵은 `PageRenderer` 에 있고
   * `PageRenderer` 는 문서보다 오래 산다. 그래서 서명은 자기가 어느 문서의 것인지 함께
   * 들고 다녀야 한다 — 그러지 않으면 두 문서의 0쪽 첫 그림이 똑같이 `bin:0:1:src` 라서
   * 서로의 서명이 맞아떨어진다.
   */
  documentDigest: string;
  /** 같은 원본 파일을 다시 연 경우까지 구분하는 문서 인스턴스 세대. */
  documentGeneration: number;
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
 * PageLayerTree 안의 raster image op를 찾아 브라우저 prefetch용 data URL로 바꾼다.
 *
 * image op에는 `bbox`처럼 중첩 객체가 들어가므로 직렬화 JSON을 `[^}]*` 정규식으로
 * 훑으면 첫 닫는 중괄호에서 멈춘다. 구조를 직접 순회해 필드 순서와 중첩 깊이에
 * 의존하지 않게 한다.
 */
export function collectImagePrefetchDataUrls(node: unknown, out: string[]): void {
  if (!node || typeof node !== 'object') return;
  if (Array.isArray(node)) {
    for (const child of node) collectImagePrefetchDataUrls(child, out);
    return;
  }

  const record = node as Record<string, unknown>;
  if (
    record.type === 'image'
    && typeof record.mime === 'string'
    && record.mime.startsWith('image/')
    && typeof record.base64 === 'string'
    && record.base64.length > 0
  ) {
    out.push(`data:${record.mime};base64,${record.base64}`);
  }

  for (const value of Object.values(record)) {
    collectImagePrefetchDataUrls(value, out);
  }
}

/** 안정된 키가 없는 합성 이미지가 하나라도 있으면 페이지 전체를 캐시하지 않는다. */
export function cacheableImageKeySignature(raw: string | null): string | null {
  if (raw === null) return null;
  try {
    const parsed = JSON.parse(raw) as { cacheable?: unknown };
    return parsed.cacheable === false ? null : raw;
  } catch {
    return null;
  }
}

/**
 * 이미 디코드를 마친 그림 집합과 같으면 prefetch 를 건너뛴다.
 *
 * 판정 재료가 없으면 건너뛰지 않는다 — 키 조회를 지원하지 않는 구형 WASM(`imageKeys`
 * 없음), 아직 한 번도 데우지 않은 페이지(기록 없음), 문서 신원을 모르는 상태
 * (`documentDigest` 없음). 안전망을 없애는 쪽이 아니라 이미 끝난 일을 되풀이하지 않는
 * 쪽으로만 작동해야 한다.
 */
export function shouldSkipImagePrefetch(
  cached: PrefetchSignature | undefined,
  imageKeys: string | null,
  documentDigest: string | null,
  documentGeneration: number,
  currentRawSvgCount: number,
): boolean {
  if (imageKeys === null || documentDigest === null || !cached) return false;
  if (cached.documentDigest !== documentDigest) return false;
  if (cached.documentGeneration !== documentGeneration) return false;
  // 그림 키가 덮지 못하는 rawSvg가 현재 새로 생긴 경우도 전체 JSON을 다시 읽어야 한다.
  if (currentRawSvgCount > 0) return false;
  if (cached.hadRawSvg) return false;
  return cached.imageKeys === imageKeys;
}

/**
 * 모든 이미지 decode가 실제로 끝난 뒤에만 prefetch 서명을 기록한다.
 * 실패나 빈 작업을 완료로 캐시하면 다음 편집이 재시도를 건너뛰어 빈 그림이 고착된다.
 */
export async function completeImagePrefetch(
  tasks: readonly Promise<boolean>[],
  isCurrent: () => boolean,
  recordSignature: () => void,
): Promise<boolean> {
  if (tasks.length === 0) return false;
  let results: boolean[];
  try {
    results = await Promise.all(tasks);
  } catch {
    return false;
  }
  if (!results.every(Boolean)) return false;
  if (!isCurrent()) return false;
  recordSignature();
  return true;
}
