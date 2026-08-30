import type { LayerRenderProfile } from '@/core/types';

export const RENDER_DPR_BUCKETS = [1, 1.5, 2] as const;
export type RenderDprBucket = (typeof RENDER_DPR_BUCKETS)[number];
export type AdaptiveRenderTier = 'overview' | 'preview' | 'screen' | 'export';
export type AdaptiveRenderInteraction = 'idle' | 'pinch' | 'fast-scroll';

/** 한 행에 이 쪽 수 이상이면 overview tier. */
export const OVERVIEW_PAGES_PER_ROW = 3;
export const DEFAULT_PAGE_LAYER_COUNT = 4;
/** 동시에 보이는 페이지 Canvas(레이어 포함) 물리 픽셀 상한. */
export const MAX_VISIBLE_CANVAS_PIXELS = 32_000_000;
/** 프리페치·보존 중인 페이지까지 포함한 물리 픽셀 상한. */
export const MAX_RETAINED_CANVAS_PIXELS = 48_000_000;
export const DPR_BUCKET_HYSTERESIS = 0.12;

const MID_LOW = 1.25;
const MID_HIGH = 1.75;

export function isExportRenderProfile(profile: LayerRenderProfile | string): boolean {
  return profile === 'print' || profile === 'highQuality';
}

export function isOverviewLayout(pagesPerRow: number): boolean {
  return Number.isFinite(pagesPerRow) && pagesPerRow >= OVERVIEW_PAGES_PER_ROW;
}

export function pageCanvasPixels(
  pageWidth: number,
  pageHeight: number,
  zoom: number,
  dpr: number,
): number {
  const cssW = Math.max(0, pageWidth) * Math.max(0, zoom);
  const cssH = Math.max(0, pageHeight) * Math.max(0, zoom);
  const scale = Math.max(0, dpr);
  return cssW * cssH * scale * scale;
}

export function estimateSurfacePixels(args: {
  pageWidth: number;
  pageHeight: number;
  zoom: number;
  dpr: number;
  pageCount: number;
  layerCount: number;
}): number {
  const pages = Math.max(0, args.pageCount);
  const layers = Math.max(1, args.layerCount);
  return pageCanvasPixels(args.pageWidth, args.pageHeight, args.zoom, args.dpr) * pages * layers;
}

export function quantizeDprBucket(
  requestedDpr: number,
  previous?: RenderDprBucket | null,
): RenderDprBucket {
  const requested = Number.isFinite(requestedDpr) && requestedDpr > 0 ? requestedDpr : 1;
  if (previous === 1 && requested < MID_LOW + DPR_BUCKET_HYSTERESIS) return 1;
  if (
    previous === 1.5
    && requested > MID_LOW - DPR_BUCKET_HYSTERESIS
    && requested < MID_HIGH + DPR_BUCKET_HYSTERESIS
  ) {
    return 1.5;
  }
  if (previous === 2 && requested > MID_HIGH - DPR_BUCKET_HYSTERESIS) return 2;
  if (requested < MID_LOW) return 1;
  if (requested < MID_HIGH) return 1.5;
  return 2;
}

/** overview는 raw DPR≥2에서 full-DPR 대비 Canvas 픽셀을 50% 이상 줄인다. */
export function overviewDprBucket(_rawDpr: number): RenderDprBucket {
  return 1;
}

export function screenEffectiveDpr(
  rawDpr: number,
  previous?: RenderDprBucket | null,
): number {
  const raw = Number.isFinite(rawDpr) && rawDpr > 0 ? rawDpr : 1;
  if (raw > 2) return raw;
  return quantizeDprBucket(raw, previous);
}

export interface AdaptiveRenderScaleInput {
  pageWidth: number;
  pageHeight: number;
  zoom: number;
  rawDpr: number;
  pagesPerRow: number;
  visiblePageCount: number;
  retainedPageCount: number;
  layerCount?: number;
  isFocused?: boolean;
  isEditing?: boolean;
  interaction?: AdaptiveRenderInteraction;
  renderProfile?: LayerRenderProfile | string;
  previousBucket?: RenderDprBucket | null;
}

export interface AdaptiveRenderScaleResult {
  displayZoom: number;
  renderScale: number;
  rawDpr: number;
  effectiveDpr: number;
  bucket: RenderDprBucket;
  tier: AdaptiveRenderTier;
  cssWidth: number;
  cssHeight: number;
  canvasWidth: number;
  canvasHeight: number;
  canvasPixels: number;
  layerCount: number;
  estimatedVisibleSurfacePixels: number;
  estimatedRetainedSurfacePixels: number;
}

function safeZoom(zoom: number): number {
  return Number.isFinite(zoom) && zoom > 0 ? zoom : 1;
}

function safePositive(value: number, fallback: number): number {
  return Number.isFinite(value) && value > 0 ? value : fallback;
}

function asBucket(dpr: number): RenderDprBucket {
  if (dpr >= 2) return 2;
  if (dpr >= 1.5) return 1.5;
  return 1;
}

function demoteBucket(bucket: RenderDprBucket): RenderDprBucket {
  if (bucket === 2) return 1.5;
  return 1;
}

function fitToPixelBudget(args: {
  pageWidth: number;
  pageHeight: number;
  zoom: number;
  dpr: number;
  visiblePageCount: number;
  retainedPageCount: number;
  layerCount: number;
}): number {
  let dpr = args.dpr;
  let bucket: RenderDprBucket = dpr > 2 ? 2 : asBucket(dpr);
  for (let i = 0; i < 3; i++) {
    const visible = estimateSurfacePixels({
      ...args,
      dpr: bucket,
      pageCount: args.visiblePageCount,
    });
    const retained = estimateSurfacePixels({
      ...args,
      dpr: bucket,
      pageCount: args.retainedPageCount,
    });
    if (visible <= MAX_VISIBLE_CANVAS_PIXELS && retained <= MAX_RETAINED_CANVAS_PIXELS) {
      return dpr > 2 && bucket === 2 ? dpr : bucket;
    }
    const next = demoteBucket(bucket);
    if (next === bucket) return bucket;
    bucket = next;
    dpr = next;
  }
  return bucket;
}

export function canvasCssSize(
  pageWidth: number,
  pageHeight: number,
  zoom: number,
  renderScale: number,
): { width: number; height: number } {
  const safe = safeZoom(zoom);
  const dpr = renderScale / safe;
  return {
    width: Math.max(1, Math.ceil(pageWidth * renderScale)) / dpr,
    height: Math.max(1, Math.ceil(pageHeight * renderScale)) / dpr,
  };
}

export function resolveAdaptiveRenderScale(
  input: AdaptiveRenderScaleInput,
): AdaptiveRenderScaleResult {
  const zoom = safeZoom(input.zoom);
  const rawDpr = safePositive(input.rawDpr, 1);
  const pageWidth = safePositive(input.pageWidth, 1);
  const pageHeight = safePositive(input.pageHeight, 1);
  const layerCount = Math.max(1, Math.round(input.layerCount ?? DEFAULT_PAGE_LAYER_COUNT));
  const visiblePageCount = Math.max(1, Math.round(input.visiblePageCount || 1));
  const retainedPageCount = Math.max(
    visiblePageCount,
    Math.round(input.retainedPageCount || visiblePageCount),
  );
  const profile = input.renderProfile ?? 'screen';
  const promoted = input.isFocused === true || input.isEditing === true;
  const overview = isOverviewLayout(input.pagesPerRow);
  const interaction = input.interaction ?? 'idle';
  const previous = input.previousBucket ?? null;

  let tier: AdaptiveRenderTier;
  let effectiveDpr: number;

  if (isExportRenderProfile(profile)) {
    tier = 'export';
    effectiveDpr = rawDpr;
  } else if (interaction !== 'idle') {
    tier = 'preview';
    if (previous) {
      effectiveDpr = previous;
    } else if (overview && !promoted) {
      effectiveDpr = overviewDprBucket(rawDpr);
    } else {
      effectiveDpr = screenEffectiveDpr(rawDpr, previous);
    }
  } else if (overview && !promoted) {
    tier = 'overview';
    effectiveDpr = overviewDprBucket(rawDpr);
  } else {
    tier = 'screen';
    effectiveDpr = screenEffectiveDpr(rawDpr, previous);
  }

  if (tier !== 'export' && overview && !promoted) {
    effectiveDpr = fitToPixelBudget({
      pageWidth,
      pageHeight,
      zoom,
      dpr: effectiveDpr,
      visiblePageCount,
      retainedPageCount,
      layerCount,
    });
  }

  const bucket: RenderDprBucket = effectiveDpr > 2 ? 2 : asBucket(effectiveDpr);
  const renderScale = zoom * effectiveDpr;
  const cssWidth = pageWidth * zoom;
  const cssHeight = pageHeight * zoom;
  const canvasWidth = Math.max(1, Math.ceil(pageWidth * renderScale));
  const canvasHeight = Math.max(1, Math.ceil(pageHeight * renderScale));
  const estimatedVisibleSurfacePixels = estimateSurfacePixels({
    pageWidth,
    pageHeight,
    zoom,
    dpr: effectiveDpr,
    pageCount: visiblePageCount,
    layerCount,
  });
  const estimatedRetainedSurfacePixels = estimateSurfacePixels({
    pageWidth,
    pageHeight,
    zoom,
    dpr: effectiveDpr,
    pageCount: retainedPageCount,
    layerCount,
  });

  return {
    displayZoom: zoom,
    renderScale,
    rawDpr,
    effectiveDpr,
    bucket,
    tier,
    cssWidth,
    cssHeight,
    canvasWidth,
    canvasHeight,
    canvasPixels: canvasWidth * canvasHeight,
    layerCount,
    estimatedVisibleSurfacePixels,
    estimatedRetainedSurfacePixels,
  };
}
