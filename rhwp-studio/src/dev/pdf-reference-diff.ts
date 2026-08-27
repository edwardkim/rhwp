export interface PixelDiffBounds {
  x: number;
  y: number;
  width: number;
  height: number;
}

export interface PixelDiffMetrics {
  comparedPixels: number;
  mismatchPixels: number;
  pdfOnlyPixels: number;
  hwpOnlyPixels: number;
  colorMismatchPixels: number;
  mismatchRatio: number;
  meanAbsoluteError: number;
  maxAbsoluteError: number;
  bounds: PixelDiffBounds | null;
}

export const REFERENCE_PIXEL_DIFF_THRESHOLD = 24;
const DIFF_COLORS = [
  [220, 53, 69],
  [15, 118, 110],
  [217, 119, 6],
] as const;

/** HWP와 Ghostscript PDF RGBA를 각각 흰 종이 위에 합성한 뒤 RGB 최대 오차로 비교한다. */
export function computeReferencePixelDiff(
  hwp: Uint8ClampedArray,
  reference: Uint8ClampedArray,
  width: number,
  height: number,
  threshold = REFERENCE_PIXEL_DIFF_THRESHOLD,
  mismatchMask?: Uint8ClampedArray,
): PixelDiffMetrics {
  const comparedPixels = width * height;
  if (hwp.length < comparedPixels * 4 || reference.length < comparedPixels * 4) {
    throw new Error('pixel buffers are smaller than the requested diff surface');
  }
  if (mismatchMask && mismatchMask.length < comparedPixels * 4) {
    throw new Error('mismatch mask is smaller than the requested diff surface');
  }

  let mismatchPixels = 0;
  let pdfOnlyPixels = 0;
  let hwpOnlyPixels = 0;
  let colorMismatchPixels = 0;
  let errorSum = 0;
  let maxAbsoluteError = 0;
  let minX = width;
  let minY = height;
  let maxX = -1;
  let maxY = -1;

  for (let pixel = 0; pixel < comparedPixels; pixel++) {
    const offset = pixel * 4;
    const sourceAlpha = hwp[offset + 3] / 255;
    const redOnWhite = hwp[offset] * sourceAlpha + 255 * (1 - sourceAlpha);
    const greenOnWhite = hwp[offset + 1] * sourceAlpha + 255 * (1 - sourceAlpha);
    const blueOnWhite = hwp[offset + 2] * sourceAlpha + 255 * (1 - sourceAlpha);
    const referenceAlpha = reference[offset + 3] / 255;
    const referenceRed = reference[offset] * referenceAlpha + 255 * (1 - referenceAlpha);
    const referenceGreen = reference[offset + 1] * referenceAlpha + 255 * (1 - referenceAlpha);
    const referenceBlue = reference[offset + 2] * referenceAlpha + 255 * (1 - referenceAlpha);
    const hwpInk = 255 - Math.min(redOnWhite, greenOnWhite, blueOnWhite);
    const referenceInk = 255 - Math.min(referenceRed, referenceGreen, referenceBlue);
    const rawError = Math.max(
      Math.abs(redOnWhite - referenceRed),
      Math.abs(greenOnWhite - referenceGreen),
      Math.abs(blueOnWhite - referenceBlue),
    );
    const absoluteError = rawError < 1e-9 ? 0 : rawError;
    errorSum += absoluteError;
    maxAbsoluteError = Math.max(maxAbsoluteError, absoluteError);
    if (absoluteError < threshold) continue;

    mismatchPixels += 1;
    const inkDelta = referenceInk - hwpInk;
    const kind = inkDelta > threshold / 2 ? 0 : inkDelta < -threshold / 2 ? 1 : 2;
    if (kind === 0) pdfOnlyPixels += 1;
    else if (kind === 1) hwpOnlyPixels += 1;
    else colorMismatchPixels += 1;
    if (mismatchMask) {
      mismatchMask.set(DIFF_COLORS[kind], offset);
      mismatchMask[offset + 3] = Math.min(210, Math.max(56, Math.round(absoluteError * 0.82)));
    }
    const x = pixel % width;
    const y = Math.floor(pixel / width);
    minX = Math.min(minX, x);
    minY = Math.min(minY, y);
    maxX = Math.max(maxX, x);
    maxY = Math.max(maxY, y);
  }

  return {
    comparedPixels,
    mismatchPixels,
    pdfOnlyPixels,
    hwpOnlyPixels,
    colorMismatchPixels,
    mismatchRatio: comparedPixels > 0 ? mismatchPixels / comparedPixels : 0,
    meanAbsoluteError: comparedPixels > 0 ? errorSum / comparedPixels : 0,
    maxAbsoluteError,
    bounds: mismatchPixels === 0 ? null : {
      x: minX,
      y: minY,
      width: maxX - minX + 1,
      height: maxY - minY + 1,
    },
  };
}
