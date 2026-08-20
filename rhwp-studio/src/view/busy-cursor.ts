/**
 * 문서 로딩처럼 오래 걸리는 처리 동안 대기 커서를 보여준다.
 *
 * 큰 문서는 파싱·쪽 계산 동안 빈 화면만 오래 보이고 커서는 평소 그대로여서, 앱이 멈춘 건지
 * 일하는 중인지 구분되지 않았다. 루트에 클래스 하나를 걸고 CSS(`src/style.css`)가
 * `cursor: wait` 를 덮는다 — 편집 영역이 인라인 style 로 커서를 바꾸므로 규칙은 `!important` 다.
 *
 * 로딩 경로는 겹쳐 불릴 수 있어(loadFile → loadBytes) 깊이를 센다. 바깥 것이 끝날 때까지
 * 커서를 유지하고, 예외가 나도 `withBusyCursor` 의 finally 가 반드시 되돌린다.
 * 루트는 인자로 받아 전역 `document` 없이 검증할 수 있다.
 */

/** 대기 커서를 켜는 루트 클래스 */
export const BUSY_CLASS = 'rhwp-busy';

/** 이 모듈이 쓰는 루트 요소 표면만 좁혀 받는다. */
export interface BusyRoot {
  classList: { add(token: string): void; remove(token: string): void };
}

let depth = 0;

/** 대기 커서 시작 (겹침 허용) */
export function beginBusy(root: BusyRoot): void {
  depth += 1;
  if (depth === 1) root.classList.add(BUSY_CLASS);
}

/** 대기 커서 종료 — 가장 바깥 처리가 끝날 때만 되돌린다. */
export function endBusy(root: BusyRoot): void {
  if (depth === 0) return;
  depth -= 1;
  if (depth === 0) root.classList.remove(BUSY_CLASS);
}

/** 현재 겹침 깊이 (0 이면 대기 커서 없음) */
export function busyDepth(): number {
  return depth;
}

/** 처리 하나를 대기 커서로 감싼다. 실패해도 커서를 반드시 되돌린다. */
export async function withBusyCursor<T>(root: BusyRoot, task: () => Promise<T>): Promise<T> {
  beginBusy(root);
  try {
    return await task();
  } finally {
    endBusy(root);
  }
}
