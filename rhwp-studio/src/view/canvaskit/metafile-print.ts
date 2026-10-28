import * as wasm from '@wasm/rhwp.js';
import type { Canvas, CanvasKit, Image } from 'canvaskit-wasm';

type ReplayApi = {
  isPrintMetafile?: (source: Uint8Array) => boolean;
  rasterizePrintMetafile?: (source: Uint8Array, matrix: Float64Array,
    decode: (jpeg: Uint8Array) => Uint8Array | undefined) => Uint8Array | undefined;
};

export function isPrintMetafile(source: Uint8Array): boolean {
  return (wasm as unknown as ReplayApi).isPrintMetafile?.(source) === true;
}

/** device CTM과 Skia JPEG codec만 제공한다. 글꼴·간격·사진 표본은 Rust가 소유한다. */
export function drawPrintMetafile(kit: CanvasKit, canvas: Canvas, source: Uint8Array,
  opacity: number): boolean {
  const api = wasm as unknown as ReplayApi;
  if (!api.isPrintMetafile?.(source) || !api.rasterizePrintMetafile) return false;
  const m = canvas.getTotalMatrix();
  if (m[1] !== 0 || m[3] !== 0 || m[6] !== 0 || m[7] !== 0 || m[8] !== 1
    || m[0] <= 0 || m[4] <= 0 || !Number.isInteger(m[2]) || !Number.isInteger(m[5])) return false;
  const info = { colorType: kit.ColorType.RGBA_8888, alphaType: kit.AlphaType.Unpremul,
    colorSpace: kit.ColorSpace.SRGB };
  let sampled: Image | null = null;
  const paint = new kit.Paint();
  paint.setAlphaf(Math.max(0, Math.min(1, opacity)));
  try {
    const output = api.rasterizePrintMetafile(source, new Float64Array([m[0], m[4], m[2], m[5]]), jpeg => {
      let image: Image | null = null;
      try {
        image = kit.MakeImageFromEncoded(jpeg);
        if (!image) return undefined;
        const pixels = image.readPixels(0, 0, { ...info, width: image.width(), height: image.height() });
        return pixels instanceof Uint8Array ? pixels : undefined;
      } finally { image?.delete(); }
    });
    if (!output || output.length < 16) return false;
    const header = new DataView(output.buffer, output.byteOffset, 16);
    const x = header.getInt32(0, true), y = header.getInt32(4, true);
    const width = header.getInt32(8, true), height = header.getInt32(12, true);
    if (width <= 0 || height <= 0 || width * height > 67_108_864
      || output.length !== 16 + width * height * 4) return false;
    sampled = kit.MakeImage({ ...info, width, height }, output.subarray(16), width * 4);
    if (!sampled) return false;
    canvas.save();
    try {
      canvas.concat([1 / m[0], 0, -m[2] / m[0], 0, 1 / m[4], -m[5] / m[4], 0, 0, 1]);
      canvas.drawImage(sampled, x, y, paint);
    } finally { canvas.restore(); }
    return true;
  } catch { return false; }
  finally { sampled?.delete(); paint.delete(); }
}
