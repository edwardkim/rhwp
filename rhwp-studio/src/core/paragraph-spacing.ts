/** HWP 문단 간격의 최하위 비트는 CHAR 단위 표지다. pt 입력은 항상 짝수로 저장한다. */
export function paragraphSpacingFromPt(pt: number): number {
  return Math.round(pt * 100) * 2;
}
