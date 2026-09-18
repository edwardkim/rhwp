export function computeHangingIndentPx(cursorX: number, firstLineStartX: number): number {
  if (!Number.isFinite(cursorX) || !Number.isFinite(firstLineStartX)) return 0;
  return Math.max(0, cursorX - firstLineStartX);
}

/** 내어쓰기가 바꿀 문단 속성 — 단위는 명령과 같은 raw2x */
export type HangingIndentProps = {
  marginLeft: number;
  indent: number;
} & Record<string, unknown>;

/**
 * 커서까지의 거리를 한컴식 내어쓰기 문단 속성으로 옮긴다.
 *
 * 한컴 Shift+Tab 은 첫 줄을 제자리에 두고 **둘째 줄부터** 커서 자리로 들인다.
 * 그러려면 왼쪽 여백과 첫 줄 들여쓰기가 함께 움직여야 한다: 새 여백은 지금 첫 줄이
 * 시작하는 자리(여백+들여쓰기)에 거리를 더한 값이고, 첫 줄 들여쓰기는 그 거리만큼
 * 음수다 — 둘을 더하면 첫 줄 자리는 그대로다.
 *
 * 여백을 두고 들여쓰기만 음수로 주면 첫 줄이 왼쪽으로 밀려 나가고, 여백이 0인
 * 문단에서는 음수 여백이 잘려 눌러도 아무 일도 일어나지 않는다.
 */
export function computeHangingIndentProps(
  currentMarginLeftRaw: number,
  currentIndentRaw: number,
  hangingRaw: number,
): HangingIndentProps {
  const firstLineStartRaw = currentMarginLeftRaw + currentIndentRaw;
  return {
    marginLeft: firstLineStartRaw + hangingRaw,
    indent: -hangingRaw,
  };
}
