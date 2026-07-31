/**
 * 그림 신원 키별 object URL 캐시 (Task #3315).
 *
 * 종전에는 그림마다 `data:{mime};base64,{...}` 를 만들어 `<img>.src` 에 넣었다. 그 문자열은
 * 레이어 트리 JSON 에서 온 것이고, JSON 은 편집마다 다시 받으므로 **같은 그림의 같은 바이트를
 * 키 입력마다 다시 옮기고 다시 문자열로 만들었다**.
 *
 * 신원 키(`bin:{epoch}:{bin_data_id}:{variant}`)는 바이트가 바뀌면 함께 바뀌므로, 키를 캐시
 * 키로 쓰면 스스로 무효화된다 — 편집 때 비울 필요가 없고, 비우면 캐시를 두는 의미가 없다.
 *
 * ## 수명
 *
 * `URL.createObjectURL` 은 명시적으로 revoke 하지 않으면 문서가 살아 있는 동안 남는다. 문서를
 * 갈아끼우면 epoch 가 올라가 옛 키는 다시 조회되지 않으므로, 그 시점에 전부 revoke 한다.
 * 개별 항목을 참조 카운트로 회수하지는 않는다 — `<img>` 가 아직 디코드 중인 URL 을 거두면 그림이
 * 빈 채로 남고, 페이지의 그림 수는 revoke 를 정교하게 할 만큼 크지 않다.
 */
export class FlowImageUrlCache {
  private urls = new Map<string, string>();

  /**
   * 키에 해당하는 object URL. 캐시에 없으면 `loadBytes` 로 바이트를 받아 만든다.
   *
   * 바이트를 받을 수 없으면 `null` — 세대가 바뀐 낡은 키이거나 구형 WASM 이다. 호출부는
   * 종전의 base64 경로로 되돌아가야 한다.
   */
  urlFor(
    key: string,
    mime: string,
    loadBytes: (key: string) => Uint8Array | null,
  ): string | null {
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

  /** 문서를 갈아끼울 때·정리할 때 전부 회수한다. */
  releaseAll(): void {
    for (const url of this.urls.values()) {
      URL.revokeObjectURL(url);
    }
    this.urls.clear();
  }
}
