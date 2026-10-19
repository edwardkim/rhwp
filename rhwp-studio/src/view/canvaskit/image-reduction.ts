import type { Canvas, CanvasKit, Image, Paint, Rect } from 'canvaskit-wasm';

/** 원본 전체를 축소할 때만 장치 배율에 맞는 저역 통과 표본을 사용한다. */
export function drawImageRectWithReduction(
  kit: CanvasKit,
  canvas: Canvas,
  image: Image,
  source: Rect,
  destination: Rect,
  paint: Paint,
): void {
  const reduced = imageRectIsReduced(canvas, image, source, destination);
  if (!reduced || typeof image.makeCopyWithDefaultMipmaps !== 'function'
    || typeof canvas.drawImageRectOptions !== 'function') {
    canvas.drawImageRect(image, source, destination, paint);
    return;
  }
  const mipmapped = image.makeCopyWithDefaultMipmaps();
  if (!mipmapped) {
    canvas.drawImageRect(image, source, destination, paint);
    return;
  }
  try {
    canvas.drawImageRectOptions(
      mipmapped, source, destination, kit.FilterMode.Linear, kit.MipmapMode.Linear, paint,
    );
  } finally {
    // 원본 cache는 계속 소유하고, 이번 그리기의 추가 표본만 해제한다.
    mipmapped.delete();
  }
}

/** Native·Rust Canvas2D와 같은 한 레벨 이내의 축소 재구성 범위. */
export function imageRectUsesCubic(
  canvas: Canvas, image: Image, source: Rect, destination: Rect,
): boolean {
  const width = typeof image.width === 'function' ? image.width() : Number(image.width);
  const height = typeof image.height === 'function' ? image.height() : Number(image.height);
  const matrix = typeof canvas.getTotalMatrix === 'function' ? canvas.getTotalMatrix() : null;
  if (!matrix || source[0] !== 0 || source[1] !== 0 || source[2] !== width || source[3] !== height
    || matrix[1] !== 0 || matrix[3] !== 0 || matrix[6] !== 0 || matrix[7] !== 0 || matrix[8] !== 1
    || matrix[0] <= 0 || matrix[4] <= 0) return false;
  const deviceWidth = (destination[2] - destination[0]) * matrix[0];
  const deviceHeight = (destination[3] - destination[1]) * matrix[4];
  return [width, height, deviceWidth, deviceHeight].every(value => Number.isFinite(value) && value > 0)
    && deviceWidth <= width && deviceHeight <= height
    && (deviceWidth < width || deviceHeight < height)
    && width <= 2 * deviceWidth && height <= 2 * deviceHeight;
}

/** 실제 장치 좌표에서 원본 전체가 축소되는지 공통 표본·cache 경로가 조회한다. */
export function imageRectIsReduced(
  canvas: Canvas, image: Image, source: Rect, destination: Rect,
): boolean {
  // 잘린 그림의 strict source 경계와 1:1·확대의 기존 표본은 보존한다.
  const width = typeof image.width === 'function' ? image.width() : Number(image.width);
  const height = typeof image.height === 'function' ? image.height() : Number(image.height);
  const fullSource = source[0] === 0 && source[1] === 0
    && source[2] === width && source[3] === height;
  const matrix = typeof canvas.getTotalMatrix === 'function' ? canvas.getTotalMatrix() : null;
  return fullSource && matrix !== null
    && matrix[6] === 0 && matrix[7] === 0 && matrix[8] === 1
    && (source[2] > (destination[2] - destination[0]) * Math.hypot(matrix[0], matrix[3])
      || source[3] > (destination[3] - destination[1]) * Math.hypot(matrix[1], matrix[4]));
}
