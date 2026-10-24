import * as wasm from '@wasm/rhwp.js';
import type { Canvas, CanvasKit, Image, Paint, Rect } from 'canvaskit-wasm';

type Resample = (rgba: Uint8Array, width: number, height: number,
  destination: Float64Array) => Uint8Array | undefined;

type FrameResample = (rgba: Uint8Array, width: number, height: number,
  frame: Float64Array, scale: Float64Array) => Uint8Array | undefined;

/** 원본 좌표를 보존한 공통 Print 프레임을 실제 장치 배율로 재생한다. */
export function drawPrintFrameImage(kit: CanvasKit, canvas: Canvas, image: Image,
  source: Rect, frame: number[] | undefined, paint: Paint): boolean {
  const resample = (wasm as unknown as { resamplePrintFrameRgba?: FrameResample }).resamplePrintFrameRgba;
  if (!resample || frame?.length !== 8) return false;
  const m = canvas.getTotalMatrix(), width = image.width(), height = image.height();
  if (source[0] !== 0 || source[1] !== 0 || source[2] !== width || source[3] !== height
    || m[1] !== 0 || m[3] !== 0 || m[6] !== 0 || m[7] !== 0 || m[8] !== 1
    || m[0] <= 0 || m[4] <= 0 || !Number.isInteger(m[2]) || !Number.isInteger(m[5])) return false;
  const info = { width, height, colorType: kit.ColorType.RGBA_8888,
    alphaType: kit.AlphaType.Unpremul, colorSpace: kit.ColorSpace.SRGB };
  let sampled: Image | null = null;
  try {
    const rgba = image.readPixels(0, 0, info);
    if (!(rgba instanceof Uint8Array)) return false;
    const output = resample(rgba, width, height, new Float64Array(frame), new Float64Array([m[0], m[4]]));
    if (!output || output.length < 16) return false;
    const header = new DataView(output.buffer, output.byteOffset, 16);
    const x = header.getInt32(0, true), y = header.getInt32(4, true);
    const w = header.getInt32(8, true), h = header.getInt32(12, true);
    if (w <= 0 || h <= 0 || w * h > 4_194_304 || output.length !== 16 + w * h * 4) return false;
    sampled = kit.MakeImage({ ...info, width: w, height: h }, output.subarray(16), w * 4);
    if (!sampled) return false;
    canvas.save();
    try {
      canvas.concat([1 / m[0], 0, -m[2] / m[0], 0, 1 / m[4], -m[5] / m[4], 0, 0, 1]);
      canvas.drawImage(sampled, x + m[2], y + m[5], paint);
    } finally { canvas.restore(); }
    return true;
  } catch { return false; }
  finally { sampled?.delete(); }
}

/** 장치의 실제 위상과 축별 크기로 Rust 공통 필터를 호출한다. */
export function drawAnisotropicPrintImage(kit: CanvasKit, canvas: Canvas, image: Image,
  source: Rect, destination: Rect, paint: Paint): boolean {
  const resample = (wasm as unknown as { resamplePrintBitmapRgba?: Resample }).resamplePrintBitmapRgba;
  if (!resample) return false;
  const m = canvas.getTotalMatrix();
  const width = image.width(), height = image.height();
  if (source[0] !== 0 || source[1] !== 0 || source[2] !== width || source[3] !== height
    || m[1] !== 0 || m[3] !== 0 || m[6] !== 0 || m[7] !== 0 || m[8] !== 1
    || m[0] <= 0 || m[4] <= 0) return false;
  const x = destination[0] * m[0] + m[2], y = destination[1] * m[4] + m[5];
  const w = (destination[2] - destination[0]) * m[0];
  const h = (destination[3] - destination[1]) * m[4];
  const ow = Math.ceil(x + w) - Math.floor(x), oh = Math.ceil(y + h) - Math.floor(y);
  if (![x, y, w, h, ow, oh].every(Number.isFinite) || ow <= 0 || oh <= 0 || ow * oh > 4_194_304
    || !((w >= width && h < height) || (h >= height && w < width))) return false;
  const info = { width, height, colorType: kit.ColorType.RGBA_8888,
    alphaType: kit.AlphaType.Unpremul, colorSpace: kit.ColorSpace.SRGB };
  let sampled: Image | null = null;
  try {
    const rgba = image.readPixels(0, 0, info);
    if (!(rgba instanceof Uint8Array)) return false;
    const pixels = resample(rgba, width, height, new Float64Array([x, y, w, h]));
    if (!pixels || pixels.length !== ow * oh * 4) return false;
    sampled = kit.MakeImage({ ...info, width: ow, height: oh }, pixels, ow * 4);
    if (!sampled) return false;
    canvas.save();
    try {
      canvas.concat([1 / m[0], 0, -m[2] / m[0], 0, 1 / m[4], -m[5] / m[4], 0, 0, 1]);
      canvas.drawImage(sampled, Math.floor(x), Math.floor(y), paint);
    } finally { canvas.restore(); }
    return true;
  } catch {
    return false;
  } finally { sampled?.delete(); }
}
