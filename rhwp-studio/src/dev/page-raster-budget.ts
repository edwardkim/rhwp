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
  const aspectRatio = pageSize.height / pageSize.width;
  const pixelHeight = Math.round(pixelWidth * aspectRatio);
  if (
    !Number.isSafeInteger(pixelWidth)
    || pixelWidth < PAGE_RASTER_BUDGET.minWidth
    || pixelWidth > PAGE_RASTER_BUDGET.maxWidth
    || !Number.isFinite(pageSize.width)
    || !Number.isFinite(pageSize.height)
    || !(pageSize.width > 0)
    || !(pageSize.height > 0)
    || !Number.isFinite(aspectRatio)
    || aspectRatio > PAGE_RASTER_BUDGET.maxAspectRatio
    || aspectRatio < 1 / PAGE_RASTER_BUDGET.maxAspectRatio
    || !Number.isSafeInteger(pixelHeight)
    || pixelHeight <= 0
    || pixelHeight > PAGE_RASTER_BUDGET.maxHeight
    || pixelWidth * pixelHeight > PAGE_RASTER_BUDGET.maxPixels
  ) {
    throw new Error(`${label} raster dimensions exceed the harness budget`);
  }
  return { width: pixelWidth, height: pixelHeight };
}
