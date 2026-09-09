/**
 * HWPUNIT crop 좌표를 **원본 픽셀**로 환산하는 축척 — studio 두 백엔드의 단일 출처.
 *
 * rust `compute_image_crop_src`(`src/renderer/svg.rs`) 와 같은 폴백 사슬을 쓴다.
 *
 *   ① `cropReferenceSize`(paint op 의 `originalSizeHu` = `imgDim`) 가 있으면 그것
 *   ② 없으면 crop 의 `right`/`bottom` 이 원본 전체 범위를 가리킨다고 본다 (#3239)
 *   ③ 둘 다 못 쓰면 96dpi 가정(75 HU/px)
 *
 * ②가 빠지면 `imgDim` 을 보존하지 않는 구형 HWP5 의 비-96dpi 스캔 그림에서 곧장 ③으로
 * 떨어져 원본에서 **다른 창**을 잘라 온다 — 좁게 잘린 만큼 같은 자리에 늘어난다(#3239·#6954).
 *
 * 판정은 rust 와 같이 **두 축을 함께** 한다. 한 축만 유효한 reference 로 다른 축을 섞으면
 * 원본에 없는 사영이 된다.
 */
export const HWPUNIT_PER_PIXEL = 75;

/** crop 의 전체 범위 축(오른쪽·아래). 폴백 ②가 이 둘을 원본 크기로 읽는다. */
export interface ImageCropExtent {
  right: number;
  bottom: number;
}

export interface ImageCropScale {
  scaleX: number;
  scaleY: number;
}

/** HWPUNIT crop 네 변. */
export interface ImageCrop {
  left: number;
  top: number;
  right: number;
  bottom: number;
}

/** 원본 픽셀 좌표로 환산한, 실제로 잘라 올 창. */
export interface ImageCropSourceRect {
  x: number;
  y: number;
  width: number;
  height: number;
}

function usableScale(scaleX: number, scaleY: number): boolean {
  return (
    Number.isFinite(scaleX)
    && Number.isFinite(scaleY)
    && scaleX > 0
    && scaleY > 0
  );
}

function positive(value: number | undefined): boolean {
  return Number.isFinite(value) && (value ?? 0) > 0;
}

/**
 * @param cropReferenceSize paint op 의 `originalSizeHu`(HWPUNIT). 없으면 `null`/`undefined`.
 * @param crop crop 의 `right`/`bottom`(HWPUNIT).
 * @param imageWidth 디코딩된 원본 픽셀 폭.
 * @param imageHeight 디코딩된 원본 픽셀 높이.
 */
export function imageCropScale(
  cropReferenceSize: readonly [number, number] | null | undefined,
  crop: ImageCropExtent,
  imageWidth: number,
  imageHeight: number,
): ImageCropScale {
  if (!(imageWidth > 0) || !(imageHeight > 0)) {
    return { scaleX: HWPUNIT_PER_PIXEL, scaleY: HWPUNIT_PER_PIXEL };
  }

  const referenceWidth = cropReferenceSize?.[0];
  const referenceHeight = cropReferenceSize?.[1];
  if (positive(referenceWidth) && positive(referenceHeight)) {
    const scaleX = (referenceWidth as number) / imageWidth;
    const scaleY = (referenceHeight as number) / imageHeight;
    if (usableScale(scaleX, scaleY)) return { scaleX, scaleY };
  }

  if (crop.right > 0 && crop.bottom > 0) {
    const scaleX = crop.right / imageWidth;
    const scaleY = crop.bottom / imageHeight;
    if (usableScale(scaleX, scaleY)) return { scaleX, scaleY };
  }

  return { scaleX: HWPUNIT_PER_PIXEL, scaleY: HWPUNIT_PER_PIXEL };
}

function clamp(value: number, min: number, max: number): number {
  return Math.max(min, Math.min(max, value));
}

/**
 * 원본에서 실제로 잘라 올 창. **자를 것이 없으면 `null`** 이다.
 *
 * 두 백엔드가 이 판정을 함께 쓴다. 한쪽만 "자를 것이 없다" 로 보면 같은 그림을 한쪽은
 * 통째로, 한쪽은 소수점 창으로 다시 표본화해 그려 파리티가 벌어진다(#6954).
 *
 * 판정은 원본 픽셀 격자에서 한다 — crop 이 원본 전 범위를 가리키면(축척 폴백 ②가 그런
 * 경우다) 잘라 올 창이 곧 원본 전체이므로 `null` 이다.
 */
export function imageCropSourceRect(
  imageWidth: number,
  imageHeight: number,
  crop?: ImageCrop,
  cropReferenceSize?: readonly [number, number] | null,
): ImageCropSourceRect | null {
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

  const { scaleX, scaleY } = imageCropScale(cropReferenceSize, crop, imageWidth, imageHeight);
  const x = crop.left / scaleX;
  const y = crop.top / scaleY;
  const width = (crop.right - crop.left) / scaleX;
  const height = (crop.bottom - crop.top) / scaleY;
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
