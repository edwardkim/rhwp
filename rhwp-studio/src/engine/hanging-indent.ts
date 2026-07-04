export function computeHangingIndentPx(cursorX: number, firstLineStartX: number): number {
  if (!Number.isFinite(cursorX) || !Number.isFinite(firstLineStartX)) return 0;
  return Math.max(0, cursorX - firstLineStartX);
}
