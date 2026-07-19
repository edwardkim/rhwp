export type BorderHitBox = {
  x: number;
  y: number;
  width: number;
  height: number;
};

/** 페이지 좌표의 점이 bbox 외곽선 tolerance 안인지 판정한다. */
export function isPointNearBoxBorder(
  pageX: number,
  pageY: number,
  bbox: BorderHitBox,
  tolerance = 5,
): boolean {
  const nearLeft = Math.abs(pageX - bbox.x) <= tolerance;
  const nearRight = Math.abs(pageX - (bbox.x + bbox.width)) <= tolerance;
  const nearTop = Math.abs(pageY - bbox.y) <= tolerance;
  const nearBottom = Math.abs(pageY - (bbox.y + bbox.height)) <= tolerance;
  const inVertRange = pageY >= bbox.y - tolerance && pageY <= bbox.y + bbox.height + tolerance;
  const inHorzRange = pageX >= bbox.x - tolerance && pageX <= bbox.x + bbox.width + tolerance;
  return (nearLeft && inVertRange) || (nearRight && inVertRange) ||
         (nearTop && inHorzRange) || (nearBottom && inHorzRange);
}
