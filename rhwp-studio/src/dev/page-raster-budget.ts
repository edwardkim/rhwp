export const PAGE_RASTER_BUDGET = {
  minWidth: 128,
  maxWidth: 4_096,
  maxHeight: 8_192,
  maxPixels: 24_000_000,
  maxAspectRatio: 16,
} as const;

export function boundedPageRasterSize(
  pageSize: { width: number; height: number },
  pixelWidth: number,
  label = 'page',
): { width: number; height: number } {
  const { minWidth, maxWidth, maxHeight, maxPixels, maxAspectRatio } = PAGE_RASTER_BUDGET;
  const aspect = pageSize.height / pageSize.width;
  const height = Math.round(pixelWidth * aspect);
  if (
    !Number.isSafeInteger(pixelWidth)
    || pixelWidth < minWidth
    || pixelWidth > maxWidth
    || !Number.isFinite(pageSize.width)
    || !Number.isFinite(pageSize.height)
    || pageSize.width <= 0
    || pageSize.height <= 0
    || !Number.isFinite(aspect)
    || aspect > maxAspectRatio
    || aspect < 1 / maxAspectRatio
    || !Number.isSafeInteger(height)
    || height <= 0
    || height > maxHeight
    || pixelWidth * height > maxPixels
  ) {
    throw new Error(`${label} raster dimensions exceed the harness budget`);
  }
  return { width: pixelWidth, height };
}
