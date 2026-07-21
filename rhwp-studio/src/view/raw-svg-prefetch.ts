/** UTF-8 바이트 기준 base64 (Rust base64::STANDARD 과 동일 산출). */
function utf8ToBase64(text: string): string {
  const bytes = new TextEncoder().encode(text);
  let binary = '';
  const chunkSize = 0x8000;
  for (let i = 0; i < bytes.length; i += chunkSize) {
    binary += String.fromCharCode(...bytes.subarray(i, i + chunkSize));
  }
  return btoa(binary);
}

/**
 * 벡터 rawSvg 조각을 `src/renderer/svg_fragment.rs::wrap_svg_fragment`와
 * 바이트 동일하게 감싼 SVG data URL로 변환한다.
 */
function rawSvgFragmentToDataUrl(
  fragment: string,
  x: number,
  y: number,
  width: number,
  height: number,
): string {
  const f = (value: number): string => value.toFixed(3);
  const svg =
    '<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" '
    + `width="${f(width)}" height="${f(height)}" viewBox="${f(x)} ${f(y)} ${f(width)} ${f(height)}">\n`
    + `${fragment}\n</svg>`;
  return `data:image/svg+xml;base64,${utf8ToBase64(svg)}`;
}

/**
 * 레이어 트리에서 벡터 rawSvg(내부 raster data URL이 없는) op을 수집한다.
 *
 * `getPageLayerTree`의 bbox 계약은 `x/y/width/height`다. 구형
 * `getPageRenderTree` JSON의 `w/h`와 혼동하지 않는다.
 */
export function collectVectorRawSvgDataUrls(node: unknown, out: string[]): void {
  if (!node || typeof node !== 'object') return;
  const record = node as Record<string, unknown>;
  if (
    record.type === 'rawSvg'
    && typeof record.svg === 'string'
    && !record.svg.includes('data:image/')
  ) {
    const bbox = record.bbox as
      | { x?: unknown; y?: unknown; width?: unknown; height?: unknown }
      | undefined;
    if (
      bbox
      && typeof bbox.x === 'number'
      && typeof bbox.y === 'number'
      && typeof bbox.width === 'number'
      && typeof bbox.height === 'number'
    ) {
      out.push(
        rawSvgFragmentToDataUrl(record.svg, bbox.x, bbox.y, bbox.width, bbox.height),
      );
    }
  }
  for (const value of Object.values(record)) {
    if (Array.isArray(value)) {
      for (const child of value) collectVectorRawSvgDataUrls(child, out);
    } else if (value && typeof value === 'object') {
      collectVectorRawSvgDataUrls(value, out);
    }
  }
}
