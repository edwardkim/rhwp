import type { LayerLineOp, LayerRenderProfile } from '../../core/types.ts';

/** Native/Web Canvas의 raster_stroke와 같은 장치 좌표의 가는 실선 계약. */
export function rasterLineEndpoints(
  line: LayerLineOp,
  matrix: ArrayLike<number> | undefined,
  profile: LayerRenderProfile,
): [number, number, number, number] {
  const original: [number, number, number, number] = [line.x1, line.y1, line.x2, line.y2];
  const style = line.style ?? {};
  const width = style.width ?? 1;
  if (!['print', 'highQuality'].includes(profile) || !matrix || matrix.length !== 9
    || (style.lineType ?? 'single') !== 'single' || (style.dash ?? 'solid') !== 'solid'
    || (style.startArrow ?? 'none') !== 'none' || (style.endArrow ?? 'none') !== 'none'
    || line.arrowHeads?.length || style.shadow || !Number.isFinite(width) || width <= 0
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
  const deviceWidth = width * Math.hypot(a * nx + c * ny, b * nx + d * ny);
  if (!(deviceWidth > 0 && deviceWidth < 1)) return original;
  const px = a * line.x1 + c * line.y1 + e;
  const py = b * line.x1 + d * line.y1 + f;
  const ex = a * dx + c * dy;
  const ey = b * dx + d * dy;
  if (![px, py, ex, ey].every(Number.isFinite)) return original;
  let sx: number, sy: number;
  if (ex !== 0 && ey === 0) [sx, sy] = [0, Math.floor(py) + 0.5 - py];
  else if (ex === 0 && ey !== 0) [sx, sy] = [Math.floor(px) + 0.5 - px, 0];
  else return original;
  const lx = (d * sx - c * sy) / determinant;
  const ly = (-b * sx + a * sy) / determinant;
  const adjusted: [number, number, number, number] = [
    line.x1 + lx, line.y1 + ly, line.x2 + lx, line.y2 + ly,
  ];
  return adjusted.every(Number.isFinite) ? adjusted : original;
}
