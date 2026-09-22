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

/**
 * `cellPath` 첫 항목은 표 또는 글상자처럼 안쪽 개체를 소유한 바깥 control을 가리킨다.
 * 같은 문단의 독립 도형은 이 경로의 조상이 아니므로, 그 도형과 셀 그림이 겹쳐도
 * 도형 클릭을 셀 그림으로 바꾸면 안 된다.
 */
export function isNestedCellDescendantOfControl(
  candidate: { secIdx?: number; paraIdx?: number; controlIdx?: number },
  nested: { secIdx?: number; paraIdx?: number; cellPath?: unknown },
): boolean {
  if (candidate.secIdx !== nested.secIdx || candidate.paraIdx !== nested.paraIdx) return false;
  if (!Array.isArray(nested.cellPath) || candidate.controlIdx === undefined) return false;
  return nested.cellPath.some((entry: any) =>
    (entry?.controlIndex ?? entry?.controlIdx) === candidate.controlIdx);
}

/**
 * A control address can occur on more than one page (for example, a split
 * layout item).  A mouse hit already knows the concrete page, so preserve it
 * as the first lookup target; retain the complete scan as a safe fallback for
 * keyboard-created selections and reflowed layouts.
 */
export function orderedControlLayoutPages(pageCount: number, selectedPage?: number): number[] {
  const pages = Array.from({ length: Math.max(0, pageCount) }, (_, page) => page);
  if (selectedPage === undefined || selectedPage < 0 || selectedPage >= pageCount) return pages;
  return [selectedPage, ...pages.filter((page) => page !== selectedPage)];
}
