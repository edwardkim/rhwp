import type { PageInfo } from '@/core/types';
import type { GridViewSettings } from './grid-settings';

const MM_TO_PX = 96 / 25.4;
const CLIP_CORNER_LENGTH_PX = 10;

export function createGridOverlay(
  pageIdx: number,
  pageInfo: PageInfo,
  zoom: number,
  settings: GridViewSettings,
): HTMLDivElement {
  const overlay = document.createElement('div');
  overlay.className = 'page-grid-overlay';
  overlay.dataset.rhwpGridPage = String(pageIdx);
  overlay.style.backgroundImage = buildBackgroundImage(settings, zoom);
  overlay.style.backgroundSize = `${settings.horizontalMm * MM_TO_PX * zoom}px ${settings.verticalMm * MM_TO_PX * zoom}px`;
  overlay.style.backgroundPosition = buildBackgroundPosition(pageInfo, zoom, settings);
  overlay.style.clipPath = buildClipPath(pageInfo, zoom, settings);
  overlay.style.zIndex = settings.layer === 'inFrontOfText' ? '4' : '1';
  overlay.style.opacity = settings.layer === 'inFrontOfText' ? '0.85' : '1';
  return overlay;
}

export function applyGridOverlayBox(
  overlay: HTMLElement,
  canvas: HTMLCanvasElement,
): void {
  overlay.style.position = 'absolute';
  overlay.style.top = canvas.style.top;
  overlay.style.left = canvas.style.left;
  overlay.style.transform = canvas.style.transform;
  overlay.style.width = canvas.style.width;
  overlay.style.height = canvas.style.height;
  overlay.style.pointerEvents = 'none';
  overlay.style.overflow = 'hidden';
}

export function createGridClipCornerOverlay(
  pageIdx: number,
  pageInfo: PageInfo,
  zoom: number,
  settings: GridViewSettings,
): HTMLDivElement | null {
  if (settings.origin !== 'page') return null;

  const pageArea = getPageGridAreaPx(pageInfo);
  const overlay = document.createElement('div');
  overlay.className = 'page-grid-clip-corners';
  overlay.dataset.rhwpGridPage = String(pageIdx);
  overlay.style.zIndex = settings.layer === 'inFrontOfText' ? '5' : '2';

  const left = pageArea.left * zoom;
  const top = pageArea.top * zoom;
  const right = (pageInfo.width - pageArea.right) * zoom;
  const bottom = (pageInfo.height - pageArea.bottom) * zoom;
  const length = CLIP_CORNER_LENGTH_PX * zoom;
  const color = 'rgba(0, 0, 0, 0.5)';

  appendClipCorner(overlay, left, top, length, 'top-left', color);
  appendClipCorner(overlay, right, top, length, 'top-right', color);
  appendClipCorner(overlay, left, bottom, length, 'bottom-left', color);
  appendClipCorner(overlay, right, bottom, length, 'bottom-right', color);
  return overlay;
}

function buildBackgroundImage(settings: GridViewSettings, zoom: number): string {
  const color = 'rgba(0, 32, 150, 0.9)';
  switch (settings.pattern) {
    case 'horizontal':
      return `linear-gradient(to bottom, ${color} 0, ${color} 1px, transparent 1px)`;
    case 'vertical':
      return `linear-gradient(to right, ${color} 0, ${color} 1px, transparent 1px)`;
    case 'both':
      return [
        `linear-gradient(to right, ${color} 0, ${color} 1px, transparent 1px)`,
        `linear-gradient(to bottom, ${color} 0, ${color} 1px, transparent 1px)`,
      ].join(', ');
    case 'dots':
    default:
      return buildDotTileImage(settings, zoom);
  }
}

function buildDotTileImage(settings: GridViewSettings, zoom: number): string {
  const width = settings.horizontalMm * MM_TO_PX * zoom;
  const height = settings.verticalMm * MM_TO_PX * zoom;
  const svg = [
    `<svg xmlns="http://www.w3.org/2000/svg" width="${width}" height="${height}" viewBox="0 0 ${width} ${height}">`,
    '<rect x="0" y="0" width="1" height="1" fill="#002096" fill-opacity="0.9"/>',
    '</svg>',
  ].join('');
  return `url("data:image/svg+xml,${encodeURIComponent(svg)}")`;
}

function buildBackgroundPosition(
  pageInfo: PageInfo,
  zoom: number,
  settings: GridViewSettings,
): string {
  const origin = getGridOriginPx(pageInfo, settings);
  const x = (origin.x + settings.offsetXmm * MM_TO_PX) * zoom;
  const y = (origin.y + settings.offsetYmm * MM_TO_PX) * zoom;
  if (settings.pattern === 'dots') {
    return `${Math.round(x)}px ${Math.round(y)}px`;
  }
  return `${x}px ${y}px`;
}

function buildClipPath(
  pageInfo: PageInfo,
  zoom: number,
  settings: GridViewSettings,
): string {
  if (settings.origin === 'paper') return 'none';

  const pageArea = getPageGridAreaPx(pageInfo);
  const left = pageArea.left * zoom;
  const top = pageArea.top * zoom;
  const right = pageArea.right * zoom;
  const bottom = pageArea.bottom * zoom;
  return `inset(${top}px ${right}px ${bottom}px ${left}px)`;
}

function getGridOriginPx(
  pageInfo: PageInfo,
  settings: GridViewSettings,
): { x: number; y: number } {
  if (settings.origin === 'paper') {
    return { x: 0, y: 0 };
  }
  const pageArea = getPageGridAreaPx(pageInfo);
  return {
    x: pageArea.left,
    y: pageArea.top,
  };
}

function getPageGridAreaPx(pageInfo: PageInfo): { left: number; right: number; top: number; bottom: number } {
  // 좌우는 해석된 본문 상자에서 온다 — 원본 여백을 쓰면 제본 여백만큼 어긋나고 맞쪽 제본
  // 짝수 쪽에서는 좌우가 뒤바뀐다 (#4971).
  return {
    left: pageInfo.bodyLeft,
    right: pageInfo.width - pageInfo.bodyRight,
    top: pageInfo.marginTop + pageInfo.marginHeader,
    bottom: pageInfo.marginBottom + pageInfo.marginFooter,
  };
}

function appendClipCorner(
  parent: HTMLElement,
  x: number,
  y: number,
  length: number,
  corner: 'top-left' | 'top-right' | 'bottom-left' | 'bottom-right',
  color: string,
): void {
  const horizontal = document.createElement('div');
  const vertical = document.createElement('div');
  const isLeft = corner.endsWith('left');
  const isTop = corner.startsWith('top');

  horizontal.style.position = 'absolute';
  horizontal.style.left = `${isLeft ? x : x - length}px`;
  horizontal.style.top = `${y}px`;
  horizontal.style.width = `${length}px`;
  horizontal.style.height = '1px';
  horizontal.style.background = color;

  vertical.style.position = 'absolute';
  vertical.style.left = `${x}px`;
  vertical.style.top = `${isTop ? y : y - length}px`;
  vertical.style.width = '1px';
  vertical.style.height = `${length}px`;
  vertical.style.background = color;

  parent.append(horizontal, vertical);
}
