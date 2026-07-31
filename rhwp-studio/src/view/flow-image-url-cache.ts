/**
 * 그림 신원 키별 object URL 캐시 (Task #3315).
 *
 * 종전에는 그림마다 `data:{mime};base64,{...}` 를 만들어 `<img>.src` 에 넣었다. 그 문자열은
 * 레이어 트리 JSON 에서 온 것이고, JSON 은 편집마다 다시 받으므로 **같은 그림의 같은 바이트를
 * 키 입력마다 다시 옮기고 다시 문자열로 만들었다**.
 *
 * ## 캐시는 한 문서의 것이다
 *
 * 그림 키는 **문서 안에서만** 신원이다 — `bin_data_id` 는 문서마다 1 부터 다시 매겨지고 세대
 * 번호도 문서마다 0 에서 시작한다. 그래서 두 문서의 0쪽 첫 그림이 똑같이 `bin:0:1:src` 다.
 * 반면 이 캐시는 `PageRenderer` 에 있고 `PageRenderer` 는 문서보다 오래 산다.
 *
 * ## 수명은 문서 (재)로드 경계가 가른다
 *
 * `URL.createObjectURL` 은 명시적으로 revoke 하지 않으면 브라우저가 문서 수명 내내 붙든다. 그래서
 * 회수 시점은 **조회 시점이 아니라 문서가 갈리는 시점**이어야 한다. 조회 시점에 미루면 새 문서가
 * flow 그림을 한 장도 조회하지 않는 경우(그림 없는 문서·CanvasKit 경로) 회수가 영원히 오지 않아
 * 옛 문서의 수 MB 가 그대로 남는다.
 *
 * 그 경계는 이미 있다 — `CanvasView.prepareDocumentLoad` 가 문서 (재)로드마다
 * `RendererSession.beginDocument` 를 불러 renderer 의 문서 범위 자원(CanvasKit 자원 포함)을 거둔다.
 * 이 캐시도 같은 자리에 붙는다(`PageRenderer.beginDocument`).
 *
 * 편집(문서 revision 변화)으로는 비우지 않는다 — 키가 내용에서 유도되므로 바이트가 바뀌면 키가
 * 함께 바뀌어 옛 항목은 다시 조회되지 않는다. 편집마다 비우면 캐시를 두는 의미가 없다. 그리고 이
 * 경계는 같은 문서를 다시 로드할 때도 불리므로(외부 그림 주입 후 뷰 갱신), 신원이 그대로면
 * 그대로 둔다.
 */

/** 캐시가 어느 문서의 것인지 — `WasmBridge` 의 문서 신원과 같은 재료를 쓴다. */
export interface FlowImageDocumentIdentity {
  /** `WasmBridge.documentDigest`. 문서를 모르는 상태(`null`)에서는 캐시하지 않는다. */
  digest: string | null;
  /** 같은 원본 파일을 다시 연 경우까지 구분하는 `WasmBridge.documentGeneration`. */
  generation: number;
}

export class FlowImageUrlCache {
  private urls = new Map<string, string>();
  private identity: { digest: string; generation: number } | null = null;

  /**
   * 문서 (재)로드 경계에서 이 캐시가 어느 문서의 것인지 정한다.
   *
   * 신원이 달라졌으면 이전 문서의 URL 을 그 자리에서 회수한다 — 새 문서가 조회해 줄 때까지
   * 기다리면 조회가 없는 문서에서는 영원히 남는다. 같은 문서를 다시 로드한 경우에는 그대로
   * 둔다. 신원을 모르면(`digest === null`) 항목을 어느 문서 것이라고 표시할 수 없으므로 이후
   * 조회는 캐시하지 않고 되돌린다.
   */
  beginDocument(document: FlowImageDocumentIdentity): void {
    if (
      this.identity !== null
      && this.identity.digest === document.digest
      && this.identity.generation === document.generation
    ) return;

    this.releaseAll();
    if (document.digest === null) return;
    this.identity = { digest: document.digest, generation: document.generation };
  }

  /**
   * 키에 해당하는 object URL. 캐시에 없으면 `loadBytes` 로 바이트를 받아 만든다.
   *
   * `null` 을 돌려주는 경우는 둘이다 — 어느 문서의 캐시인지 정해지지 않았거나(`beginDocument`
   * 전·문서 신원을 모르는 상태), 바이트를 받을 수 없는 경우(세대가 바뀐 낡은 키·구형 WASM)다.
   * 호출부는 종전의 base64 경로로 되돌아가야 한다.
   */
  urlFor(
    key: string,
    mime: string,
    loadBytes: (key: string) => Uint8Array | null,
  ): string | null {
    // 어느 문서의 것인지 모르면 항목을 표시할 수 없다 — 캐시하지 않고 되돌린다.
    if (this.identity === null) return null;

    const cached = this.urls.get(key);
    if (cached !== undefined) return cached;

    const bytes = loadBytes(key);
    if (bytes === null || bytes.length === 0) return null;

    // Blob 은 전달한 바이트를 복사해 소유하므로, WASM 메모리가 이후 재배치돼도 안전하다.
    // `as BlobPart` 는 저장소 관례 — lib.dom 의 BlobPart 가 SharedArrayBuffer 로 뒷받침될
    // 수 있는 뷰를 배제하는데, 런타임은 모든 ArrayBufferView 를 받는다
    // (`src/hwpctl/index.ts`, `src/command/commands/file.ts` 와 같은 형태).
    const url = URL.createObjectURL(new Blob([bytes as BlobPart], { type: mime }));
    this.urls.set(key, url);
    return url;
  }

  /** 캐시에 들고 있는 키 수 (진단·테스트용). */
  get size(): number {
    return this.urls.size;
  }

  has(key: string): boolean {
    return this.urls.has(key);
  }

  /**
   * 들고 있는 URL 을 전부 회수하고 문서 신원을 지운다.
   *
   * 부를 자리는 둘이다 — 문서가 갈릴 때(`beginDocument`)와 renderer 를 버릴 때(`dispose`)다.
   * 편집(문서 revision 변화)으로는 부르지 않는다.
   */
  releaseAll(): void {
    for (const url of this.urls.values()) {
      URL.revokeObjectURL(url);
    }
    this.urls.clear();
    this.identity = null;
  }
}
