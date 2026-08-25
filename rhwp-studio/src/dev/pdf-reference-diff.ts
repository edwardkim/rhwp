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

export interface HorizontalRuleBand {
  startY: number;
  endY: number;
  centerY: number;
  thickness: number;
  peakInkCoverage: number;
  peakSpanRatio: number;
}

export interface HorizontalRuleDiagnostics {
  totalBands: number;
  truncated: boolean;
  bands: HorizontalRuleBand[];
}

export interface HorizontalRuleDetectionOptions {
  inkThreshold?: number;
  minSpanRatio?: number;
  minCoverageRatio?: number;
  maxBands?: number;
}

const DEFAULT_HORIZONTAL_RULE_OPTIONS = {
  inkThreshold: 96,
  minSpanRatio: 0.35,
  minCoverageRatio: 0.55,
  maxBands: 96,
} as const;
const DIFF_COLORS = [
  [220, 53, 69],
  [15, 118, 110],
  [217, 119, 6],
] as const;

function roundedRatio(value: number): number {
  return Number(value.toFixed(4));
}

/**
 * 합성된 페이지 RGBA에서 긴 가로 괘선을 찾는다.
 *
 * 외부 이미지 도구로 사후 crop/평균을 만들지 않고 diff와 정확히 같은 샘플 좌표를 쓴다.
 * 텍스트가 빽빽한 행은 ink 총량이 커질 수 있으므로, 연속 run과 전체 coverage 중 하나가
 * 충분히 긴 경우만 후보로 삼는다. 연속된 후보 Y행은 선 두께 하나로 묶는다.
 */
export function detectHorizontalRuleBands(
  pixels: Uint8ClampedArray,
  width: number,
  height: number,
  options: HorizontalRuleDetectionOptions = {},
): HorizontalRuleDiagnostics {
  const required = width * height * 4;
  if (!Number.isSafeInteger(width) || !Number.isSafeInteger(height) || width <= 0 || height <= 0) {
    throw new Error('horizontal rule surface dimensions must be positive integers');
  }
  if (pixels.length < required) {
    throw new Error('pixel buffer is smaller than the requested horizontal rule surface');
  }

  const inkThreshold = options.inkThreshold ?? DEFAULT_HORIZONTAL_RULE_OPTIONS.inkThreshold;
  const minSpanRatio = options.minSpanRatio ?? DEFAULT_HORIZONTAL_RULE_OPTIONS.minSpanRatio;
  const minCoverageRatio = options.minCoverageRatio
    ?? DEFAULT_HORIZONTAL_RULE_OPTIONS.minCoverageRatio;
  const maxBands = Math.max(
    1,
    Math.floor(options.maxBands ?? DEFAULT_HORIZONTAL_RULE_OPTIONS.maxBands),
  );
  const detected: HorizontalRuleBand[] = [];
  let open: HorizontalRuleBand | null = null;

  for (let y = 0; y < height; y++) {
    let inkPixels = 0;
    let run = 0;
    let longestRun = 0;
    for (let x = 0; x < width; x++) {
      const offset = (y * width + x) * 4;
      const alpha = pixels[offset + 3] / 255;
      const red = pixels[offset] * alpha + 255 * (1 - alpha);
      const green = pixels[offset + 1] * alpha + 255 * (1 - alpha);
      const blue = pixels[offset + 2] * alpha + 255 * (1 - alpha);
      const ink = 255 - Math.min(red, green, blue);
      if (ink >= inkThreshold) {
        inkPixels += 1;
        run += 1;
        longestRun = Math.max(longestRun, run);
      } else {
        run = 0;
      }
    }

    const coverage = inkPixels / width;
    const spanRatio = longestRun / width;
    const qualifies = spanRatio >= minSpanRatio || coverage >= minCoverageRatio;
    if (qualifies) {
      if (!open) {
        open = {
          startY: y,
          endY: y,
          centerY: y,
          thickness: 1,
          peakInkCoverage: roundedRatio(coverage),
          peakSpanRatio: roundedRatio(spanRatio),
        };
      } else {
        open.endY = y;
        open.centerY = Number(((open.startY + y) / 2).toFixed(2));
        open.thickness = y - open.startY + 1;
        open.peakInkCoverage = Math.max(open.peakInkCoverage, roundedRatio(coverage));
        open.peakSpanRatio = Math.max(open.peakSpanRatio, roundedRatio(spanRatio));
      }
    } else if (open) {
      detected.push(open);
      open = null;
    }
  }
  if (open) detected.push(open);

  if (detected.length <= maxBands) {
    return { totalBands: detected.length, truncated: false, bands: detected };
  }
  const leading = Math.ceil(maxBands / 2);
  const trailing = maxBands - leading;
  return {
    totalBands: detected.length,
    truncated: true,
    bands: [
      ...detected.slice(0, leading),
      ...(trailing > 0 ? detected.slice(-trailing) : []),
    ],
  };
}

/** HWP와 Ghostscript PDF RGBA를 각각 흰 종이 위에 합성한 뒤 RGB 최대 오차로 비교한다. */
export function computeReferencePixelDiff(
  hwp: Uint8ClampedArray,
  reference: Uint8ClampedArray,
  width: number,
  height: number,
  threshold = 24,
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
