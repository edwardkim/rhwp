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
