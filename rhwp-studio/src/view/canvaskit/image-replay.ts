export interface CanvasKitImageBounds {
  x: number;
  y: number;
  width: number;
  height: number;
}

export interface CanvasKitImageCrop {
  left: number;
  top: number;
  right: number;
  bottom: number;
}

export interface CanvasKitImageSourceRect {
  x: number;
  y: number;
  width: number;
  height: number;
}

/**
 * HWPUNIT image crop coordinates use the same 96 DPI scale as SVG replay:
 * 7200 HWPUNIT = 96 px, so 75 HWPUNIT = 1 px.
 */
export const HWPUNIT_PER_PIXEL = 75;

export interface CanvasKitImageCacheKeyInput {
  imageRef?: number | string;
  mime?: string;
  base64?: string;
}

export function canvasKitImageCacheKey(input: CanvasKitImageCacheKeyInput): string | null {
  const parts: string[] = [];
  if (input.imageRef !== undefined) {
    parts.push(`ref:${String(input.imageRef)}`);
  }
  if (input.base64) {
    parts.push(`${input.mime ?? 'application/octet-stream'}:${input.base64.length}:${fnv1a32(input.base64)}`);
  }
  return parts.length > 0 ? parts.join('|') : null;
}

export function canvasKitImageSourceRect(
  imageWidth: number,
  imageHeight: number,
  crop?: CanvasKitImageCrop,
): CanvasKitImageSourceRect | null {
  if (!crop) return null;
  if (
    !Number.isFinite(imageWidth)
    || !Number.isFinite(imageHeight)
    || imageWidth <= 0
    || imageHeight <= 0
    || !Number.isFinite(crop.left)
    || !Number.isFinite(crop.top)
    || !Number.isFinite(crop.right)
    || !Number.isFinite(crop.bottom)
  ) {
    return null;
  }

  const x = crop.left / HWPUNIT_PER_PIXEL;
  const y = crop.top / HWPUNIT_PER_PIXEL;
  const width = (crop.right - crop.left) / HWPUNIT_PER_PIXEL;
  const height = (crop.bottom - crop.top) / HWPUNIT_PER_PIXEL;
  if (width <= 0 || height <= 0) return null;

  const clampedX = clamp(x, 0, imageWidth);
  const clampedY = clamp(y, 0, imageHeight);
  const clampedWidth = clamp(width, 0, imageWidth - clampedX);
  const clampedHeight = clamp(height, 0, imageHeight - clampedY);
  if (clampedWidth <= 0 || clampedHeight <= 0) return null;

  const isCropped = x > 0.5
    || y > 0.5
    || Math.abs(clampedWidth - imageWidth) > 1
    || Math.abs(clampedHeight - imageHeight) > 1;
  if (!isCropped) return null;

  return {
    x: clampedX,
    y: clampedY,
    width: clampedWidth,
    height: clampedHeight,
  };
}

export function canvasKitImagePlacement(
  fillMode: string | undefined,
  bbox: CanvasKitImageBounds,
  imageWidth: number,
  imageHeight: number,
): { x: number; y: number } {
  switch (fillMode) {
    case 'centerTop':
      return { x: bbox.x + (bbox.width - imageWidth) / 2, y: bbox.y };
    case 'rightTop':
      return { x: bbox.x + bbox.width - imageWidth, y: bbox.y };
    case 'leftCenter':
      return { x: bbox.x, y: bbox.y + (bbox.height - imageHeight) / 2 };
    case 'center':
      return { x: bbox.x + (bbox.width - imageWidth) / 2, y: bbox.y + (bbox.height - imageHeight) / 2 };
    case 'rightCenter':
      return { x: bbox.x + bbox.width - imageWidth, y: bbox.y + (bbox.height - imageHeight) / 2 };
    case 'leftBottom':
      return { x: bbox.x, y: bbox.y + bbox.height - imageHeight };
    case 'centerBottom':
      return { x: bbox.x + (bbox.width - imageWidth) / 2, y: bbox.y + bbox.height - imageHeight };
    case 'rightBottom':
      return { x: bbox.x + bbox.width - imageWidth, y: bbox.y + bbox.height - imageHeight };
    case 'leftTop':
    default:
      return { x: bbox.x, y: bbox.y };
  }
}

export function canvasKitImageFillModeTiles(fillMode: string | undefined): boolean {
  return fillMode === 'tileAll'
    || fillMode === 'tileHorzTop'
    || fillMode === 'tileHorzBottom'
    || fillMode === 'tileVertLeft'
    || fillMode === 'tileVertRight';
}

function fnv1a32(value: string): string {
  let hash = 2166136261;
  for (let i = 0; i < value.length; i += 1) {
    hash ^= value.charCodeAt(i);
    hash = Math.imul(hash, 16777619);
  }
  return (hash >>> 0).toString(16);
}

function clamp(value: number, min: number, max: number): number {
  return Math.max(min, Math.min(max, value));
}
