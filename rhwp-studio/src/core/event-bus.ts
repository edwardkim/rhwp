type Handler = (...args: unknown[]) => void;

/**
 * 스튜디오 전역 이벤트 버스. 구독 48곳, 발행 213곳이 이 인스턴스 하나를 쓴다.
 *
 * ## 배달 계약 (#4591)
 *
 * **한 구독자의 실패가 다른 구독자의 실행을 막지 않는다.** 종전에는
 * `handlers.forEach((h) => h(...args))` 라서 앞선 구독자가 던지면 뒤 구독자는 아예 불리지
 * 않았다. 실제 결함을 만들었다 — #4579 에서 재도색 예외가 subsecond 감시 루프를 영구히 멎게
 * 했다. 구독자가 여럿인 이벤트일수록 나빠진다: `document-changed` 5곳, `document-view-changed`
 * 3곳이고, 배달 순서는 `on()` 을 부른 순서라는 우연이 정한다. `compare-dialog.ts` 처럼
 * 바깥 Promise 를 `resolve()` 하는 구독자는 앞이 던지면 오류가 아니라 **90초 타임아웃**으로
 * 나타난다(지금은 그 이벤트의 구독자가 하나뿐이라 잠재적이다).
 *
 * **예외는 계속 발행자에게 전달된다.** 배달을 끝낸 뒤 첫 실패를 다시 던진다. 삼키지 않는
 * 이유는 이 저장소가 이번 리뷰에서 지운 결함이 바로 "조용한 실패"이기 때문이다 — `emit` 이
 * 절대 던지지 않게 만들면 213개 발행 자리에서 구독자 고장이 보이지 않게 된다.
 *
 * 그래서 이 변경이 바꾸는 것은 **오직** "남은 구독자가 도는가" 하나다. 전달은 그대로다.
 * 실측으로 되풀림에 기대는 발행 자리가 없음을 확인했다 — `await`된 `emit` 0건, `emit` 을 위해
 * 쓰인 `catch` 0건, 되풀림을 단언하는 테스트 0건. 반면 `try` 안의 `emit` 74곳 중 22곳은
 * 구독자 예외를 `catch {}` 로 이미 조용히 삼키고 있었고, 13곳은 성공한 명령을 `return false`
 * 로 뒤집고 있었다. 아래 `console.error` 는 그 22곳에서 유일한 신호다.
 */
export class EventBus {
  private handlers = new Map<string, Set<Handler>>();

  on(event: string, handler: Handler): () => void {
    if (!this.handlers.has(event)) {
      this.handlers.set(event, new Set());
    }
    this.handlers.get(event)!.add(handler);
    return () => {
      this.handlers.get(event)?.delete(handler);
    };
  }

  emit(event: string, ...args: unknown[]): void {
    const handlers = this.handlers.get(event);
    if (!handlers) return;
    // `forEach` 가 아니라 `for...of` 인 것은 배달 실패를 루프 밖으로 들고 나오기 위해서다.
    // `Set` 반복 자체는 둘이 같다 — 배달 중 해지한 구독자는 양쪽 모두 건너뛴다.
    let firstFailure: { error: unknown } | null = null;
    for (const handler of handlers) {
      try {
        handler(...args);
      } catch (error) {
        // 어느 이벤트에서 났는지는 여기서만 알 수 있다. 되풀리는 예외는 그 사실을 잃는다.
        console.error(`[EventBus] '${event}' 구독자가 던졌다 — 남은 구독자는 계속 배달한다:`, error);
        firstFailure ??= { error };
      }
    }
    if (firstFailure) throw firstFailure.error;
  }

  removeAll(): void {
    this.handlers.clear();
  }
}
