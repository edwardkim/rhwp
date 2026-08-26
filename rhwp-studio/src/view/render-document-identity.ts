/**
 * 렌더러가 들고 있는 파생 상태가 **어느 문서의 것인지** (Task #3315).
 *
 * `PageRenderer` 는 문서보다 오래 산다. 그래서 페이지 단위로 캐시한 항목은 자기가 어느 문서의
 * 것인지 함께 들고 다녀야 한다 — `bin_data_id` 는 문서마다 1 부터, 세대 번호는 0 부터 다시
 * 매겨져서 두 문서의 0쪽 첫 그림이 똑같이 `bin:0:1:src` 이기 때문이다.
 *
 * 같은 판정이 세 곳에서 필요하다 — flow 그림 object URL 캐시(`FlowImageUrlCache`), 그림 prefetch
 * 서명(`shouldSkipImagePrefetch`), 그리고 문서 경계에서 파생 상태를 거두는
 * `PageRenderer.beginDocument`. 규칙이 갈라지면 한쪽만 옛 문서의 항목을 살려 두게 되므로 한 곳에
 * 둔다.
 */

/** 문서 인스턴스의 신원 — `WasmBridge` 가 내주는 재료를 그대로 쓴다. */
export interface RenderDocumentIdentity {
  /** `WasmBridge.documentDigest`. 문서를 모르는 상태는 `null` 이다. */
  digest: string | null;
  /** 같은 원본 파일을 다시 연 경우까지 가르는 `WasmBridge.documentGeneration`. */
  generation: number;
}

/**
 * 두 신원이 같은 문서 인스턴스를 가리키는지.
 *
 * 문서를 모르는 상태(`digest === null`)는 **어느 것과도 같지 않다.** 신원을 모르면 들고 있던
 * 항목이 그 문서의 것이라고 말할 수 없으므로, 재사용하지 않는 쪽(=거두는 쪽)으로 판정한다.
 */
export function isSameRenderDocument(
  a: RenderDocumentIdentity | null,
  b: RenderDocumentIdentity | null,
): boolean {
  if (a === null || b === null) return false;
  if (a.digest === null || b.digest === null) return false;
  return a.digest === b.digest && a.generation === b.generation;
}
