import type { CanvasKit, Image } from 'canvaskit-wasm';
import type { LayerImageOp } from '../../core/types.ts';
import { applyLayerImageEffectPixels } from '../image-effect-pixels.ts';
import { canvasKitImageSourceRect } from './image-replay.ts';

export function canvasKitEffectCrop(original: Image, op: LayerImageOp) {
  return ['fitToSize', 'total', 'none'].includes(op.fillMode ?? 'fitToSize')
    ? canvasKitImageSourceRect(original.width(), original.height(), op.crop, op.originalSizeHu) : null;
}

export function canvasKitImageEffectKey(op: LayerImageOp): string | null {
  if (op.bakedWatermark) return '';
  const effect = op.effect ?? 'realPic';
  const brightness = op.brightness ?? 0;
  const contrast = op.contrast ?? 0;
  if (!['realPic', 'grayScale', 'blackWhite', 'pattern8x8'].includes(effect)
    || ![brightness, contrast].every(value => Number.isFinite(value) && value >= -100 && value <= 100)) return null;
  return effect === 'realPic' && brightness === 0 && contrast === 0
    ? '' : JSON.stringify([effect, brightness, contrast]);
}

/** 원본 alpha를 유지한 unpremultiplied 픽셀 변환. 실패하면 원본을 성공으로 반환하지 않는다. */
export function makeCanvasKitEffectImage(kit: CanvasKit, original: Image, op: LayerImageOp,
  onReadback?: (route: 'direct' | 'surface') => void): Image | null {
  const crop = canvasKitEffectCrop(original, op);
  const x = crop?.x ?? 0, y = crop?.y ?? 0;
  const sourceWidth = crop?.width ?? original.width(), sourceHeight = crop?.height ?? original.height();
  const width = Math.max(1, Math.round(sourceWidth)), height = Math.max(1, Math.round(sourceHeight));
  const info = {
    width, height,
    colorType: kit.ColorType.RGBA_8888,
    alphaType: kit.AlphaType.Unpremul,
    colorSpace: kit.ColorSpace.SRGB,
  };
  try {
    let pixels: Uint8Array | Float32Array | null = null;
    const exactRead = [x, y, sourceWidth, sourceHeight].every(Number.isInteger);
    // Integer crops need no temporary surface. Fractional crops use the same bounded
    // native sampling as committed Skia before applying the tone/dither operation.
    if (exactRead) pixels = original.readPixels(x, y, info);
    let route: 'direct' | 'surface' = 'direct';
    if (!pixels) {
      route = 'surface';
      const surface = kit.MakeSurface?.(width, height);
      if (!surface) return null;
      try {
        const paint = new kit.Paint();
        try {
          surface.getCanvas().drawImageRect(original, kit.XYWHRect(x, y, sourceWidth, sourceHeight),
            kit.XYWHRect(0, 0, width, height), paint, false);
        } finally { paint.delete(); }
        surface.flush();
        const snapshot = surface.makeImageSnapshot();
        try { pixels = snapshot.readPixels(0, 0, info); } finally { snapshot.delete(); }
      } finally { surface.delete(); }
    }
    if (!(pixels instanceof Uint8Array) || pixels.length !== width * height * 4) return null;
    onReadback?.(route);
    applyLayerImageEffectPixels(pixels, width, op.effect, Math.floor(x), Math.floor(y), op.brightness ?? 0, op.contrast ?? 0);
    return kit.MakeImage(info, pixels, width * 4);
  } catch {
    return null;
  }
}
