/** 문단 길이의 하위 비트는 CHAR 단위 표지다. 절대 길이는 정수 HWPUNIT으로 반올림한 뒤 저장한다. */
export function paragraphLengthFromHwpunit(hu: number): number {
  const magnitude = Math.round(Math.abs(hu)) * 2;
  return hu < 0 && magnitude > 0 ? -magnitude : magnitude;
}

export function paragraphLengthFromPt(pt: number): number {
  // 십진수 1.005pt는 정확히 100.5 HWPUNIT이다. 이진 부동소수 곱으로 경계값을 잃지 않는다.
  const [coefficient, exponent = '0'] = String(pt).split('e');
  return paragraphLengthFromHwpunit(Number(coefficient + 'e' + (Number(exponent) + 2)));
}

export function paragraphLengthFromPx(px: number): number {
  return paragraphLengthFromHwpunit(px * 75);
}
