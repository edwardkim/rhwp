/**
 * HWPUNIT crop 좌표를 **원본 픽셀**로 환산하는 축척 — studio 두 백엔드의 단일 출처.
 *
 * rust `compute_image_crop_src`(`src/renderer/svg.rs`) 와 같은 폴백 사슬을 쓴다.
 *
 *   ① `cropReferenceSize`(paint op 의 `originalSizeHu` = `imgDim`) 가 있으면 그것
 *   ② 없으면 0 에서 시작하는 축의 끝점을 디코딩 범위에 수용하는 공통 등방 축척을 쓴다.
 *      끝점이 원본 전체라는 확정은 아니다. 두 후보의 최댓값을 두 축에 함께 적용한다.
 *   ③ 둘 다 못 쓰면 96dpi 가정(75 HU/px)
 *
 * ②가 빠지면 `imgDim` 을 보존하지 않는 구형 HWP5 의 비-96dpi 스캔 그림에서 곧장 ③으로
 * 떨어져 원본에서 **다른 창**을 잘라 온다 — 좁게 잘린 만큼 같은 자리에 늘어난다(#3239·#6954).
 *
 * ①은 rust 와 같이 **두 축을 함께** 판정한다. 한 축만 유효한 reference 로 다른 축을 섞으면
 * 원본에 없는 사영이 된다. ②의 축별 판정은 그와 다르다 — 거기서 갈리는 것은 "이 축의
 * `right`/`bottom` 이 기준 후보가 될 수 있는가" 이고, 배율 하나를 두 축에 쓰는 것은 HWP5 crop
 * 좌표가 등방이기 때문이다(#7525: studio 가 #7015 를 따라가지 않아 30442 3쪽 로고가 절반만,
 * 14쪽 사진이 엉뚱한 창으로 그려졌다).
 */
export const HWPUNIT_PER_PIXEL = 75;

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

/** 저장된 자르기 선택의 빈 영역은 자르기 없음과 다르다. */
export function imageCropSelectionIsEmpty(crop?: ImageCrop | null): boolean {
  return !!crop && (crop.right <= crop.left || crop.bottom <= crop.top);
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
 * @param crop crop 네 변(HWPUNIT). 폴백 ②는 시작이 0 인 축의 `right`/`bottom` 을 축척 후보로 읽는다.
 * @param imageWidth 디코딩된 원본 픽셀 폭.
 * @param imageHeight 디코딩된 원본 픽셀 높이.
 */
export function imageCropScale(
  cropReferenceSize: readonly [number, number] | null | undefined,
  crop: ImageCrop,
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

  // Same isotropic fallback as Rust: a zero start can also mean right/bottom-only
  // cropping. Separate scales would turn either shortened edge into a full image.
  // The largest candidate is the smallest common scale that fits both extents.
  const axisScale = (start: number, end: number, pixels: number): number | null => {
    if (start !== 0 || !(end > 0)) return null;
    const scale = end / pixels;
    return Number.isFinite(scale) && scale > 0 ? scale : null;
  };
  const adaptiveX = axisScale(crop.left, crop.right, imageWidth);
  const adaptiveY = axisScale(crop.top, crop.bottom, imageHeight);
  if (adaptiveX !== null && adaptiveY !== null) {
    const scale = Math.max(adaptiveX, adaptiveY);
    return { scaleX: scale, scaleY: scale };
  }
  const candidate = adaptiveX ?? adaptiveY;
  if (candidate !== null) return { scaleX: candidate, scaleY: candidate };

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
 * 판정은 원본 픽셀 격자에서 한다 — crop 이 원본 전 범위를 가리키면 잘라 올 창이 곧 원본 전체이므로 `null` 이다.
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
