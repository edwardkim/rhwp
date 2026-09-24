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

/**
 * 포인터가 확정한 쪽에서 객체 선택 overlay를 다시 찾는 순서.
 *
 * `pageIndex`가 있으면 click hit가 이미 해당 layout 항목의 쪽 소유를 확정했다. 이 뒤에
 * 같은 document address를 다른 쪽에서 찾아 쓰면 맞쪽 보기에서 클릭과 핸들이 서로 다른
 * 쪽에 나타날 수 있다. keyboard/명령처럼 page hint가 없는 selection만 전체 검색한다.
 */
export function exactSelectedControlLayoutPages(pageCount: number, selectedPage?: number): number[] {
  if (selectedPage !== undefined && selectedPage >= 0 && selectedPage < pageCount) {
    return [selectedPage];
  }
  return Array.from({ length: Math.max(0, pageCount) }, (_, page) => page);
}

/**
 * 연결선도 도형과 같은 객체 선택 주소를 쓴다. 특히 표 셀 안의 선은 `cellPath`가
 * 없으면 같은 control 번호를 가진 본문 개체로 잘못 다시 찾아 선택 표시가 다른 쪽에
 * 나타날 수 있으므로, hit-test가 얻은 주소를 빠짐없이 보존한다.
 */
export function lineControlReference(control: any, pageIndex?: number) {
  return {
    sec: control.secIdx,
    ppi: control.paraIdx,
    ci: control.controlIdx,
    type: 'line' as const,
    x1: control.x1,
    y1: control.y1,
    x2: control.x2,
    y2: control.y2,
    cellIdx: control.cellIdx,
    cellParaIdx: control.cellParaIdx,
    outerTableControlIdx: control.outerTableControlIdx,
    cellPath: control.cellPath,
    noteRef: control.noteRef,
    headerFooter: control.headerFooter,
    missing: control.missing,
    pageIndex,
  };
}

function pointToSegmentDistance(px: number, py: number, x1: number, y1: number, x2: number, y2: number): number {
  const dx = x2 - x1;
  const dy = y2 - y1;
  const lengthSquared = dx * dx + dy * dy;
  if (lengthSquared === 0) return Math.hypot(px - x1, py - y1);
  const t = Math.max(0, Math.min(1, ((px - x1) * dx + (py - y1) * dy) / lengthSquared));
  return Math.hypot(px - (x1 + t * dx), py - (y1 + t * dy));
}

/** 직선·연결선의 실제 경로만 적중으로 처리한다. */
export function isLineControlHit(control: any, pageX: number, pageY: number): boolean {
  const threshold = 6;
  const { x1, y1, x2, y2 } = control;
  if (![x1, y1, x2, y2].every(Number.isFinite)) return false;
  if (pointToSegmentDistance(pageX, pageY, x1, y1, x2, y2) <= threshold) return true;
  if (!(control.w > 2 && control.h > 2)) return false;

  const mx = control.x + control.w / 2;
  const my = control.y + control.h / 2;
  const segments: [number, number, number, number][] = [
    [x1, y1, mx, y1], [mx, y1, mx, y2], [mx, y2, x2, y2],
    [x1, y1, x1, my], [x1, my, x2, my], [x2, my, x2, y2],
    [x1, y1, x2, y1], [x2, y1, x2, y2],
    [x1, y1, x1, y2], [x1, y2, x2, y2],
  ];
  if (segments.some(([ax, ay, bx, by]) => pointToSegmentDistance(pageX, pageY, ax, ay, bx, by) <= threshold)) {
    return true;
  }

  let previousX = x1;
  let previousY = y1;
  for (let k = 1; k <= 8; k++) {
    const t = k / 8;
    const u = 1 - t;
    const bx = u * u * u * x1 + 3 * u * u * t * mx + 3 * u * t * t * mx + t * t * t * x2;
    const by = u * u * u * y1 + 3 * u * u * t * y1 + 3 * u * t * t * y2 + t * t * t * y2;
    if (pointToSegmentDistance(pageX, pageY, previousX, previousY, bx, by) <= threshold) return true;
    previousX = bx;
    previousY = by;
  }
  return false;
}
