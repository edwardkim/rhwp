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

/** Body editing APIs cannot address a header/footer subList. */
export function isBodyControl(control: { headerFooter?: unknown }): boolean {
  return !control.headerFooter;
}

/** Only direct HF images have a dedicated property/move/delete dispatch. */
export function isSupportedPictureControl(control: {
  type?: string;
  headerFooter?: unknown;
  missing?: boolean;
  cellPath?: unknown;
}): boolean {
  return isBodyControl(control) ||
    (control.type === 'image' && !control.missing && !control.cellPath);
}
