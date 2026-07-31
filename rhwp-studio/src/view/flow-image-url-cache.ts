/**
 * 그림 신원 키별 object URL 캐시 (Task #3315).
 *
 * 종전에는 그림마다 `data:{mime};base64,{...}` 를 만들어 `<img>.src` 에 넣었다. 그 문자열은
 * 레이어 트리 JSON 에서 온 것이고, JSON 은 편집마다 다시 받으므로 **같은 그림의 같은 바이트를
 * 키 입력마다 다시 옮기고 다시 문자열로 만들었다**.
 *
 * ## 문서 신원을 항목이 직접 든다
 *
 * 그림 키는 **문서 안에서만** 신원이다 — `bin_data_id` 는 문서마다 1 부터 다시 매겨지고 세대
 * 번호도 문서마다 0 에서 시작한다. 그래서 두 문서의 0쪽 첫 그림이 똑같이 `bin:0:1:src` 다.
 * 반면 이 캐시는 `PageRenderer` 에 있고 `PageRenderer` 는 문서보다 오래 산다.
 *
 * 처음에는 "문서가 갈릴 때 호출부가 `releaseAll()` 을 불러 준다"에 기댔는데, 그 계약은 두 쪽에서
 * 깨졌다. 호출부(`invalidateDocumentRevision`)는 renderer decision key 에 묶여 있어 ①같은 문서를
 * 편집할 때마다 불려 캐시를 매번 비우고(=캐시가 없는 것과 같다) ②backend 가 함께 바뀌는 문서
 * 교체에서는 아예 불리지 않아 옛 문서의 URL 이 새 문서의 같은 키에 히트했다.
 *
 * 그래서 `image-prefetch-signature.ts` 의 `PrefetchSignature` 가 같은 문제에 대해 이미 택한 답을
 * 따른다 — **비우기에 기대지 않고 항목 자체가 어느 문서의 것인지 말하게 한다.** 신원이 달라지면
 * 조회 시점에 스스로 회수하고 다시 채운다. 수명 관리가 호출 시점의 정확성에 의존하지 않는다.
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
  private identity: FlowImageDocumentIdentity | null = null;

  /**
   * 키에 해당하는 object URL. 캐시에 없으면 `loadBytes` 로 바이트를 받아 만든다.
   *
   * `null` 을 돌려주는 경우는 둘이다 — 바이트를 받을 수 없거나(세대가 바뀐 낡은 키·구형 WASM),
   * 문서 신원을 모르는 상태다. 호출부는 종전의 base64 경로로 되돌아가야 한다.
   */
  urlFor(
    key: string,
    mime: string,
    document: FlowImageDocumentIdentity,
    loadBytes: (key: string) => Uint8Array | null,
  ): string | null {
    // 신원을 모르면 항목을 어느 문서 것이라고 표시할 수 없다 — 캐시하지 않고 되돌린다.
    if (document.digest === null) return null;

    if (
      this.identity === null
      || this.identity.digest !== document.digest
      || this.identity.generation !== document.generation
    ) {
      this.releaseAll();
      this.identity = { digest: document.digest, generation: document.generation };
    }

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
   * 들고 있는 URL 을 전부 회수한다.
   *
   * 정상 경로에서는 `urlFor` 가 신원 변화를 보고 스스로 부른다. 바깥에서 부를 자리는 renderer
   * 를 버릴 때(`dispose`)뿐이다 — 그때는 다시 조회될 일이 없어 스스로 회수할 기회도 없으므로,
   * 브라우저가 URL 을 문서 수명 내내 붙들지 않게 명시적으로 거둔다.
   *
   * 편집(문서 revision 변화)으로는 부르지 않는다. 키가 내용에서 유도되므로 바이트가 바뀌면 키가
   * 바뀌고, 그러면 옛 항목은 다시 조회되지 않는다 — 편집 때 비우면 캐시를 두는 의미가 없다.
   */
  releaseAll(): void {
    for (const url of this.urls.values()) {
      URL.revokeObjectURL(url);
    }
    this.urls.clear();
    this.identity = null;
  }
}
