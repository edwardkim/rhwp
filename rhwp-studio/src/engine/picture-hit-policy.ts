/**
 * Master-page drawings decorate document pages. CanvasKit replays them behind
 * document text, so a serialized foreground wrap must not capture body clicks.
 */
export function isMasterPageDecoration(control: {
  plane?: number;
  headerFooter?: unknown;
}): boolean {
  return control.plane === 1 && !control.headerFooter;
}
