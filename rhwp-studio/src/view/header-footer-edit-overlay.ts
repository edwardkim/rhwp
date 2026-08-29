import type { PageInfo } from '@/core/types';

export interface HeaderFooterBandBox {
  x: number;
  y: number;
  width: number;
  height: number;
}

/**
 * 렌더러의 HF hit-test와 같은 영역을 쓴다.
 *
 * 새 WASM은 PageAreas의 결과를 직접 내보내고, 구 WASM에서만 PageDef
 * 여백으로 동일한 공식을 재구성한다.
 */
export function resolveHeaderFooterBandBox(
  page: PageInfo,
  isHeader: boolean,
): HeaderFooterBandBox {
  const exact = isHeader ? page.headerArea : page.footerArea;
  if (exact) return exact;

  const x = page.bodyLeft;
  const width = Math.max(0, page.bodyRight - page.bodyLeft);
  if (isHeader) {
    return {
      x,
      y: page.marginTop,
      width,
      height: Math.max(0, page.marginHeader),
    };
  }
  return {
    x,
    y: Math.max(0, page.height - page.marginFooter - page.marginBottom),
    width,
    height: Math.max(0, page.marginBottom),
  };
}

export function headerFooterClipPath(
  page: PageInfo,
  band: HeaderFooterBandBox,
  zoom: number,
): string {
  const top = Math.max(0, band.y * zoom);
  const right = Math.max(0, (page.width - band.x - band.width) * zoom);
  const bottom = Math.max(0, (page.height - band.y - band.height) * zoom);
  const left = Math.max(0, band.x * zoom);
  return `inset(${top}px ${right}px ${bottom}px ${left}px)`;
}
