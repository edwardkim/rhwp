import type { PageInfo } from '@/core/types';

export interface HeaderFooterBandBox {
  x: number;
  y: number;
  width: number;
  height: number;
}

export interface HeaderFooterBadgeMetrics {
  fontSizePx: number;
  gapPx: number;
}

const HEADER_FOOTER_BADGE_BASE_FONT_SIZE_PX = 10;
const HEADER_FOOTER_BADGE_BASE_GAP_PX = 4;
const HEADER_FOOTER_BADGE_MAX_SCALE = 2;

/**
 * HF 안내 라벨은 화면 UI이므로 문서와 똑같이 확대하지 않는다.
 * 100% 이하는 읽을 수 있는 최소 크기를 유지하고, 고배율에서는 제곱근만큼
 * 완만하게 키우되 2배에서 멈춰 문서 내용을 가리지 않게 한다.
 */
export function resolveHeaderFooterBadgeMetrics(zoom: number): HeaderFooterBadgeMetrics {
  const safeZoom = Number.isFinite(zoom) && zoom > 0 ? zoom : 1;
  const scale = Math.min(
    HEADER_FOOTER_BADGE_MAX_SCALE,
    Math.max(1, Math.sqrt(safeZoom)),
  );
  return {
    fontSizePx: HEADER_FOOTER_BADGE_BASE_FONT_SIZE_PX * scale,
    gapPx: HEADER_FOOTER_BADGE_BASE_GAP_PX * scale,
  };
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
