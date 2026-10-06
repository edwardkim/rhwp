import { blake3 } from '@noble/hashes/blake3.js';
import { bytesToHex } from '@noble/hashes/utils.js';

import { HWPUNIT_PER_PIXEL, imageCropSourceRect } from '../image-crop-scale.ts';

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
 * HWPUNIT crop 좌표의 마지막 폴백(96dpi 가정: 7200 HWPUNIT = 96 px → 75 HU/px).
 *
 * 실제 축척 판정은 canvas2d(DOM) 백엔드와 공유하는 `image-crop-scale.ts` 가 한다.
 */
export { HWPUNIT_PER_PIXEL };

export interface CanvasKitImageCacheKeyInput {
  imageRef?: number | string;
  sourceImageKey?: string;
  mime?: string;
  base64?: string;
}

export function boundedCanvasKitSourceImageKey(value: string | undefined): string | null {
  return value !== undefined
    && value.length > 0
    && value.length <= 256
    && !/[\u0000-\u001f\u007f]/.test(value)
    ? value
    : null;
}

export function canvasKitImageCacheKey(
  input: CanvasKitImageCacheKeyInput,
  documentGeneration?: number,
): string | null {
  const parts: string[] = [];
  const sourceImageKey = boundedCanvasKitSourceImageKey(input.sourceImageKey);
  if (sourceImageKey !== null) {
    parts.push(`source:${sourceImageKey}`);
  } else {
    if (
      (typeof input.imageRef === 'number' && Number.isSafeInteger(input.imageRef))
      || (
        typeof input.imageRef === 'string'
        && input.imageRef.length > 0
        && input.imageRef.length <= 256
        && !/[\u0000-\u001f\u007f]/.test(input.imageRef)
      )
    ) {
      parts.push(`ref:${String(input.imageRef)}`);
    }
    if (input.base64) {
      const mime = input.mime
        && input.mime.length <= 128
        && !/[\u0000-\u001f\u007f]/.test(input.mime)
        ? input.mime
        : 'application/octet-stream';
      const digest = bytesToHex(blake3(new TextEncoder().encode(input.base64)));
      parts.push(`${mime}:${input.base64.length}:blake3:${digest}`);
    }
  }
  if (parts.length === 0) return null;
  return Number.isSafeInteger(documentGeneration)
    ? `document:${documentGeneration}|${parts.join('|')}`
    : parts.join('|');
}

/**
 * [#6954] 잘라 올 창 판정은 canvas2d(DOM) 백엔드와 **같은 함수**가 한다.
 *
 * 두 백엔드가 각자 폴백과 각자 "자를 것이 있나" 판정을 갖고 있어 갈렸다. 사슬과 근거는
 * [`imageCropSourceRect`] 주석 참조.
 */
export function canvasKitImageSourceRect(
  imageWidth: number,
  imageHeight: number,
  crop?: CanvasKitImageCrop,
  cropReferenceSize?: [number, number],
): CanvasKitImageSourceRect | null {
  return imageCropSourceRect(imageWidth, imageHeight, crop, cropReferenceSize);
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

export function canvasKitImageFillModeStretches(fillMode: string | undefined): boolean {
  return fillMode === undefined || fillMode === 'fitToSize' || fillMode === 'total';
}

/**
 * [#7235] 영역에 맞춰 종횡비를 지키며 축소해 가운데 놓는 채우기 유형.
 *
 * `none` 은 이진 채우기 유형 15 로, 배치(원본 크기) 모드가 아니다 — 한/글은 칸에 맞춰
 * 축소한다(156467175 머리 표 칸 253.37x57.11 에서 원본 1628x563 로고가 가로 511~676px).
 * `zoom` 은 HWPX `imgBrush mode="ZOOM"`(#6310) 으로 같은 의미다. 둘 다 종전에는 이
 * 판정이 없어 배치 모드로 떨어져 원본 픽셀 크기를 칸 왼쪽 위에 놓고 잘렸다.
 * SVG backend 의 `ImageFillMode::Zoom | ImageFillMode::None` 팔과 같은 결과다.
 */
export function canvasKitImageFillModeContains(fillMode: string | undefined): boolean {
  return fillMode === 'zoom' || fillMode === 'none';
}

/**
 * [#7235] contain 배치 사각형 — 종횡비를 지켜 영역 안에 넣고 가운데 맞춘다.
 *
 * 크기를 못 읽은 경우(0 이하·비유한)는 영역 전체를 돌려준다. SVG 의
 * `preserveAspectRatio="xMidYMid meet"` 와 같은 기하다.
 */
export function canvasKitImageContainRect(
  bbox: CanvasKitImageBounds,
  imageWidth: number,
  imageHeight: number,
): { x: number; y: number; width: number; height: number } {
  const usable = Number.isFinite(imageWidth) && Number.isFinite(imageHeight)
    && imageWidth > 0 && imageHeight > 0;
  if (!usable) {
    return { x: bbox.x, y: bbox.y, width: bbox.width, height: bbox.height };
  }
  const scale = Math.min(bbox.width / imageWidth, bbox.height / imageHeight);
  const width = imageWidth * scale;
  const height = imageHeight * scale;
  return {
    x: bbox.x + (bbox.width - width) / 2,
    y: bbox.y + (bbox.height - height) / 2,
    width,
    height,
  };
}
