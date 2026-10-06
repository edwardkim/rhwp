import type { LayerBounds } from '@/core/types';

/** Translates the producer-published clip without adding backend policy. */
export function canvasKitCanonicalClipRect<T>(
  bounds: LayerBounds,
  xywhRect: (x: number, y: number, width: number, height: number) => T,
): T {
  return xywhRect(bounds.x, bounds.y, bounds.width, bounds.height);
}

export function canvasKitCanonicalClipEnabled(
  buildOption: boolean | undefined,
  compatibilityMirror: boolean | undefined,
): boolean {
  return buildOption ?? compatibilityMirror ?? true;
}
