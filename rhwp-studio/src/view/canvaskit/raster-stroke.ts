import type { LayerLineOp, LayerRenderProfile } from '../../core/types.ts';

/** Native/Web Canvas의 raster_stroke와 같은 장치 좌표의 가는 실선 계약. */
export function rasterLineGeometry(
  line: LayerLineOp,
  matrix: ArrayLike<number> | undefined,
  profile: LayerRenderProfile,
  paintWidth: number,
): [number, number, number, number, number] {
  const original: [number, number, number, number, number] = [line.x1, line.y1, line.x2, line.y2, paintWidth];
  const style = line.style ?? {};
  const width = style.width ?? 1;
  if (!['print', 'highQuality'].includes(profile) || !matrix || matrix.length !== 9
    || (style.lineType ?? 'single') !== 'single' || (style.dash ?? 'solid') !== 'solid'
    || style.compoundComponent
    || (style.startArrow ?? 'none') !== 'none' || (style.endArrow ?? 'none') !== 'none'
    || line.arrowHeads?.length || style.shadow || !Number.isFinite(paintWidth) || paintWidth <= 0
    || !Number.isFinite(width) || width <= 0
    || !original.every(Number.isFinite) || !Array.from(matrix).every(Number.isFinite)
    || matrix[6] !== 0 || matrix[7] !== 0 || matrix[8] !== 1) return original;

  // CanvasKit의 행 우선3×3을 CSS/Native와 같은 affine 순서로 읽는다.
  const [a, c, e, b, d, f] = Array.from(matrix);
  if (!((b === 0 && c === 0) || (a === 0 && d === 0))) return original;
  const determinant = a * d - b * c;
  if (!Number.isFinite(determinant) || determinant === 0) return original;
  const dx = line.x2 - line.x1;
  const dy = line.y2 - line.y1;
  let nx: number, ny: number;
  if (dx !== 0 && dy === 0) [nx, ny] = [0, 1];
  else if (dx === 0 && dy !== 0) [nx, ny] = [1, 0];
  else return original;
  const normalScale = Math.hypot(a * nx + c * ny, b * nx + d * ny);
  const deviceWidth = paintWidth * normalScale;
  if (!(deviceWidth > 0 && deviceWidth < 1)) return original;
  const rasterWidth = 1 / normalScale;
  if (!Number.isFinite(rasterWidth) || rasterWidth <= 0) return original;
  const [px, py] = [a * line.x1 + c * line.y1 + e, b * line.x1 + d * line.y1 + f];
  const [ex, ey] = [a * dx + c * dy, b * dx + d * dy];
  if (![px, py, ex, ey].every(Number.isFinite)) return original;
  let sx = 0, sy = 0;
  if (ex !== 0 && ey === 0) sy = Math.floor(py) + 0.5 - py;
  else if (ex === 0 && ey !== 0) sx = Math.floor(px) + 0.5 - px;
  else return original;
  const lx = (d * sx - c * sy) / determinant;
  const ly = (-b * sx + a * sy) / determinant;
  const adjusted: [number, number, number, number, number] =
    [line.x1 + lx, line.y1 + ly, line.x2 + lx, line.y2 + ly, rasterWidth];
  return adjusted.every(Number.isFinite) ? adjusted : original;
}

/** 기존 좌표 조회도 새 공통 래스터 계획의 원본 좌표를 사용한다. */
export function rasterLineEndpoints(
  line: LayerLineOp, matrix: ArrayLike<number> | undefined,
  profile: LayerRenderProfile, paintWidth: number,
): [number, number, number, number] {
  const [x1, y1, x2, y2] = rasterLineGeometry(line, matrix, profile, paintWidth);
  return [x1, y1, x2, y2];
}
