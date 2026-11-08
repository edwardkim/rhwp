import type { Canvas, CanvasKit, Image } from 'canvaskit-wasm';
import * as wasmExports from '@wasm/rhwp.js';

interface DeviceRaster {
  left: number;
  top: number;
  width: number;
  height: number;
  data: Uint8Array;
}
type Rasterizer = (rgba: Uint8Array, width: number, height: number,
  x: number, y: number, w: number, h: number,
  pageWidth: number, pageHeight: number, scale: number) => DeviceRaster | null;
type MaskedRasterizer = (encoded: Uint8Array, x: number, y: number, w: number, h: number,
  pageWidth: number, pageHeight: number, scale: number) => DeviceRaster | null;

/** 좌표 변환은 backend의 실제 행렬을 읽고 표본 계산은 Rust 공통 함수에 맡긴다. */
export function drawPrintAreaImage(kit: CanvasKit, canvas: Canvas, source: Image,
  rectangle: { x: number; y: number; width: number; height: number },
  page: { width: number; height: number }, scale: number, encodedMask?: Uint8Array): boolean {
  const m = canvas.getTotalMatrix?.();
  if (!m || m[1] !== 0 || m[3] !== 0 || m[6] !== 0 || m[7] !== 0 || m[8] !== 1
    || m[0] <= 0 || m[4] <= 0) return false;
  const width = source.width(), height = source.height();
  if (width <= 0 || height <= 0 || width > 8192 || height > 8192 || width * height > 32 * 1024 * 1024) return false;
  const info = { width, height, colorType: kit.ColorType.RGBA_8888,
    alphaType: kit.AlphaType.Unpremul, colorSpace: kit.ColorSpace.SRGB };
  const target: [number, number, number, number, number, number, number] = [
    m[0] * rectangle.x + m[2], m[4] * rectangle.y + m[5], m[0] * rectangle.width,
    m[4] * rectangle.height, page.width, page.height, scale,
  ];
  let raster: DeviceRaster | null;
  if (encodedMask) {
    const rasterize = (wasmExports as unknown as { rasterizePrintMaskedImage?: MaskedRasterizer }).rasterizePrintMaskedImage;
    if (!rasterize) throw new Error('Print 색·마스크의 분리 표본을 지원하는 fresh WASM이 필요합니다.');
    // Skia가 먼저 premultiply한 픽셀을 읽으면 투명 픽셀 아래 JPEG 색을 잃는다.
    raster = rasterize(encodedMask, ...target);
  } else {
    const pixels = source.readPixels(0, 0, info);
    if (!(pixels instanceof Uint8Array)) return false;
    const rasterize = (wasmExports as unknown as { rasterizePrintImageRGBA?: Rasterizer }).rasterizePrintImageRGBA;
    if (!rasterize) throw new Error('Print 그림의 공통 장치 표본을 지원하는 fresh WASM이 필요합니다.');
    raster = rasterize(pixels, width, height, ...target);
  }
  if (!raster) return false;
  const image = kit.MakeImage({ ...info, width: raster.width, height: raster.height },
    raster.data, raster.width * 4);
  if (!image) return false;
  canvas.save();
  try {
    canvas.concat([1 / m[0], 0, -m[2] / m[0], 0, 1 / m[4], -m[5] / m[4], 0, 0, 1]);
    canvas.drawImage(image, raster.left, raster.top);
  } finally {
    canvas.restore();
    image.delete();
  }
  return true;
}
