const MIN_REQUESTED_ZOOM = 0.1;
const MAX_REQUESTED_ZOOM = 4;
const HORIZONTAL_FRAME_PADDING = 40;
const VERTICAL_FRAME_PADDING = 20;

function clampRequestedZoom(zoom: number): number {
  return Math.max(MIN_REQUESTED_ZOOM, Math.min(MAX_REQUESTED_ZOOM, zoom));
}

export function calculateFitWidthZoom(
  containerWidth: number,
  pageWidth: number,
): number {
  if (pageWidth <= 0) return 1;
  return clampRequestedZoom(
    (containerWidth - HORIZONTAL_FRAME_PADDING) / pageWidth,
  );
}

export function calculateFitPageZoom(
  containerWidth: number,
  containerHeight: number,
  pageWidth: number,
  pageHeight: number,
): number {
  if (pageWidth <= 0 || pageHeight <= 0) return 1;
  return clampRequestedZoom(Math.min(
    (containerWidth - HORIZONTAL_FRAME_PADDING) / pageWidth,
    (containerHeight - VERTICAL_FRAME_PADDING) / pageHeight,
  ));
}
