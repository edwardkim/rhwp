/**
 * 동시 진행 방지 래퍼 — 앞선 실행이 끝나기 전의 재요청은 건너뛴다.
 *
 * "한 번에 하나만" 성립해야 하는 사용자 작업에 쓴다. 특히 브라우저의 File System
 * Access picker 는 창 단위 "picker active" 플래그를 두고 있어, 앞선 요청이 어떤
 * 이유로든 정리되지 않은 채 남아 있으면 다음 요청을
 * `NotAllowedError: File picker already active.` 로 거부한다. 이때 사용자에게는
 * 아무 대화상자도 보이지 않는 상태에서 오류 알림만 뜨므로, 앱 쪽에서 중복 요청을
 * 걸러 내 조용히 무시한다.
 *
 * 진행 중이라 건너뛴 호출은 `undefined` 를 돌려준다. 작업이 예외를 던져도 가드는
 * 반드시 풀린다 — 한 번 실패했다고 이후 요청이 영구히 막히면 안 된다.
 */
export function singleFlight<A extends unknown[], R>(
  task: (...args: A) => Promise<R>,
): (...args: A) => Promise<R | undefined> {
  let inFlight = false;
  return async (...args: A): Promise<R | undefined> => {
    if (inFlight) return undefined;
    inFlight = true;
    try {
      return await task(...args);
    } finally {
      inFlight = false;
    }
  };
}
