import type {
  HorizontalRuleDiagnostics,
  PixelDiffBounds,
  PixelDiffMetrics,
} from './pdf-reference-diff';

export interface DiagnosticBandDelta {
  hwpCount: number;
  pdfCount: number;
  countDelta: number;
  pairedCount: number;
  maxCenterDelta: number | null;
  meanCenterDelta: number | null;
}

export interface FidelityPageObservation {
  pageIndex: number;
  hwpFingerprint: string;
  hwpSize: { width: number; height: number };
  referenceSize: { width: number; height: number };
  mismatch: PixelDiffMetrics;
  hwpHorizontalRules: HorizontalRuleDiagnostics;
  pdfHorizontalRules: HorizontalRuleDiagnostics;
  hwpInkRows: HorizontalRuleDiagnostics;
  pdfInkRows: HorizontalRuleDiagnostics;
}

export interface FidelityPageReport {
  pageIndex: number;
  hwpFingerprint: string;
  hwpSize: { width: number; height: number };
  referenceSize: { width: number; height: number };
  referenceHeightDelta: number;
  mismatchPixels: number;
  pdfOnlyPixels: number;
  hwpOnlyPixels: number;
  colorMismatchPixels: number;
  comparedPixels: number;
  mismatchRatio: number;
  meanAbsoluteError: number;
  maxAbsoluteError: number;
  mismatchRatioDelta: number | null;
  bounds: PixelDiffBounds | null;
  changedFromPrevious: boolean | null;
  horizontalRuleDelta: DiagnosticBandDelta;
  inkRowDelta: DiagnosticBandDelta;
}

export interface FidelityScanIdentity {
  documentKey: string;
  documentDigest: string | null;
  documentGeneration: number;
  referenceGeneration: number;
  renderGeneration: number;
  pdfName: string;
}

export interface FidelityScanReport {
  schemaVersion: 1;
  scanId: number;
  status: 'ready';
  pixelMismatchPageCount: number;
  trigger: 'baseline' | 'subsecond-patch' | 'manual';
  identity: FidelityScanIdentity;
  renderRevision: string | null;
  patchIdentity: string | null;
  previousRenderRevision: string | null;
  comparisonGap: {
    supersededScanIds: number[];
    renderRevisions: Array<string | null>;
  } | null;
  hwpPageCount: number;
  pdfPageCount: number | null;
  pageCountDelta: number | null;
  firstDivergentPage: number | null;
  firstStructuralDivergencePage: number | null;
  firstRegressionPage: number | null;
  downstreamChangedPageRange: { start: number; end: number } | null;
  pages: FidelityPageReport[];
  startedAt: number;
  completedAt: number;
}

export type FidelityScanSummary = Omit<FidelityScanReport, 'pages'> & {
  reportedPages: number;
};

export type FirstDocumentDivergence = {
  kind: 'structural' | 'page-count';
  pageIndex: number;
};

export function isFidelityDocumentCurrent(
  expected: { documentDigest: string | null; documentGeneration: number },
  live: { documentDigest: string | null; documentGeneration: number },
): boolean {
  return expected.documentGeneration === live.documentGeneration
    && expected.documentDigest === live.documentDigest;
}

export function firstDocumentDivergence(report: Pick<
  FidelityScanReport,
  'firstStructuralDivergencePage' | 'hwpPageCount' | 'pdfPageCount' | 'pageCountDelta'
>): FirstDocumentDivergence | null {
  const structuralPage = report.firstStructuralDivergencePage;
  const pageCountBoundary = report.pageCountDelta !== null
    && report.pageCountDelta !== 0
    && report.pdfPageCount !== null
    ? Math.min(report.hwpPageCount, report.pdfPageCount)
    : null;
  if (pageCountBoundary !== null && (structuralPage === null || pageCountBoundary <= structuralPage)) {
    return { kind: 'page-count', pageIndex: pageCountBoundary };
  }
  return structuralPage === null
    ? null
    : { kind: 'structural', pageIndex: structuralPage };
}

export function isFidelityScanCurrent(
  report: {
    renderRevision: string | null;
    identity: { renderGeneration: number };
  } | null,
  target: { renderRevision: string | null; renderGeneration: number },
): boolean {
  return report !== null
    && report.renderRevision === target.renderRevision
    && report.identity.renderGeneration === target.renderGeneration;
}

export function isComparableFidelityPredecessor(
  report: { identity: { renderGeneration: number } } | null,
  trigger: FidelityScanReport['trigger'],
  target: { renderGeneration: number; previousRenderGeneration: number | null },
): boolean {
  const expectedGeneration = trigger === 'subsecond-patch'
    ? target.previousRenderGeneration
    : target.renderGeneration;
  return report !== null
    && expectedGeneration !== null
    && report.identity.renderGeneration === expectedGeneration;
}

const STRUCTURAL_MISMATCH_RATIO = 0.05;
const STRUCTURAL_BAND_DRIFT_PX = 2;
const STRUCTURAL_INK_ROW_COUNT_DELTA = 1;

/** Ignore glyph raster/color noise and retain page-scale layout evidence. */
export function isStructuralPageDivergence(page: FidelityPageReport): boolean {
  return page.referenceHeightDelta !== 0
    || page.mismatchRatio >= STRUCTURAL_MISMATCH_RATIO
    || page.horizontalRuleDelta.countDelta !== 0
    || (page.horizontalRuleDelta.maxCenterDelta ?? 0) >= STRUCTURAL_BAND_DRIFT_PX
    || Math.abs(page.inkRowDelta.countDelta) >= STRUCTURAL_INK_ROW_COUNT_DELTA
    || (page.inkRowDelta.maxCenterDelta ?? 0) >= STRUCTURAL_BAND_DRIFT_PX;
}

export interface FidelityPageQuery {
  start?: number;
  limit?: number;
  changedOnly?: boolean;
  divergentOnly?: boolean;
  scanId?: number;
}

export interface FidelityPageQueryResult {
  scanId: number | null;
  current: boolean;
  total: number;
  offset: number;
  limit: number;
  items: FidelityPageReport[];
}

export function queryFidelityPages(
  report: FidelityScanReport | null,
  query: FidelityPageQuery = {},
): FidelityPageQueryResult {
  const start = Math.max(0, Math.floor(query.start ?? 0));
  const limit = Math.min(100, Math.max(1, Math.floor(query.limit ?? 20)));
  const current = query.scanId === undefined || query.scanId === report?.scanId;
  const filtered = current ? (report?.pages ?? []).filter(page => {
    if (query.changedOnly && page.changedFromPrevious !== true) return false;
    if (query.divergentOnly && page.mismatchPixels === 0) return false;
    return true;
  }) : [];
  return {
    scanId: report?.scanId ?? null,
    current,
    total: filtered.length,
    offset: start,
    limit,
    items: structuredClone(filtered.slice(start, start + limit)),
  };
}

export function queryFidelityPage(
  report: FidelityScanReport | null,
  pageIndex: number,
  scanId?: number,
): { scanId: number | null; current: boolean; item: FidelityPageReport | null } {
  const current = scanId === undefined || scanId === report?.scanId;
  const page = current && Number.isInteger(pageIndex) && pageIndex >= 0
    ? report?.pages.find(candidate => candidate.pageIndex === pageIndex) ?? null
    : null;
  return {
    scanId: report?.scanId ?? null,
    current,
    item: page ? structuredClone(page) : null,
  };
}

export function summarizeFidelityScan(report: FidelityScanReport): FidelityScanSummary {
  const { pages, ...summary } = report;
  return { ...summary, reportedPages: pages.length };
}

export function fingerprintPagePixels(pixels: Uint8ClampedArray): string {
  let first = 0x811c9dc5;
  let second = 0x9e3779b9;
  for (let index = 0; index < pixels.length; index++) {
    const value = pixels[index];
    first = Math.imul(first ^ value, 0x01000193);
    second = Math.imul(second ^ (value + index), 0x85ebca6b);
  }
  return `${(first >>> 0).toString(16).padStart(8, '0')}${(second >>> 0).toString(16).padStart(8, '0')}`;
}

export function compareDiagnosticBands(
  hwp: HorizontalRuleDiagnostics,
  pdf: HorizontalRuleDiagnostics,
): DiagnosticBandDelta {
  const pairedCount = Math.min(hwp.bands.length, pdf.bands.length);
  const deltas = Array.from({ length: pairedCount }, (_, index) =>
    Math.abs(hwp.bands[index].centerY - pdf.bands[index].centerY));
  return {
    hwpCount: hwp.totalBands,
    pdfCount: pdf.totalBands,
    countDelta: hwp.totalBands - pdf.totalBands,
    pairedCount,
    maxCenterDelta: deltas.length > 0
      ? Number(Math.max(...deltas).toFixed(3))
      : null,
    meanCenterDelta: deltas.length > 0
      ? Number((deltas.reduce((sum, value) => sum + value, 0) / deltas.length).toFixed(3))
      : null,
  };
}

export function buildFidelityScanReport(input: {
  scanId: number;
  trigger: FidelityScanReport['trigger'];
  identity: FidelityScanIdentity;
  renderRevision: string | null;
  patchIdentity: string | null;
  hwpPageCount: number;
  pdfPageCount: number | null;
  observations: FidelityPageObservation[];
  previous: FidelityScanReport | null;
  startedAt: number;
  completedAt: number;
  supersededBetween?: Array<{ scanId: number; renderRevision: string | null }>;
}): FidelityScanReport {
  const previousByPage = new Map(
    input.previous?.pages.map(page => [page.pageIndex, page]) ?? [],
  );
  const pages = input.observations.map((observation): FidelityPageReport => {
    const previous = previousByPage.get(observation.pageIndex);
    const mismatchRatio = Number(observation.mismatch.mismatchRatio.toFixed(6));
    return {
      pageIndex: observation.pageIndex,
      hwpFingerprint: observation.hwpFingerprint,
      hwpSize: observation.hwpSize,
      referenceSize: observation.referenceSize,
      referenceHeightDelta: observation.referenceSize.height - observation.hwpSize.height,
      mismatchPixels: observation.mismatch.mismatchPixels,
      pdfOnlyPixels: observation.mismatch.pdfOnlyPixels,
      hwpOnlyPixels: observation.mismatch.hwpOnlyPixels,
      colorMismatchPixels: observation.mismatch.colorMismatchPixels,
      comparedPixels: observation.mismatch.comparedPixels,
      mismatchRatio,
      meanAbsoluteError: Number(observation.mismatch.meanAbsoluteError.toFixed(3)),
      maxAbsoluteError: Number(observation.mismatch.maxAbsoluteError.toFixed(3)),
      mismatchRatioDelta: previous
        ? Number((mismatchRatio - previous.mismatchRatio).toFixed(6))
        : null,
      bounds: observation.mismatch.bounds,
      changedFromPrevious: previous
        ? previous.hwpFingerprint !== observation.hwpFingerprint
        : null,
      horizontalRuleDelta: compareDiagnosticBands(
        observation.hwpHorizontalRules,
        observation.pdfHorizontalRules,
      ),
      inkRowDelta: compareDiagnosticBands(observation.hwpInkRows, observation.pdfInkRows),
    };
  });
  const changedPages = pages
    .filter(page => page.changedFromPrevious === true)
    .map(page => page.pageIndex);
  const pageCountChanged = input.previous !== null
    && input.previous.hwpPageCount !== input.hwpPageCount;
  if (pageCountChanged) {
    const sharedCount = Math.min(input.previous!.hwpPageCount, input.hwpPageCount);
    for (let pageIndex = sharedCount; pageIndex < Math.max(input.previous!.hwpPageCount, input.hwpPageCount); pageIndex++) {
      changedPages.push(pageIndex);
    }
  }
  const uniqueChangedPages = Array.from(new Set(changedPages)).sort((left, right) => left - right);
  const sharedPageCount = Math.min(input.hwpPageCount, input.pdfPageCount ?? input.hwpPageCount);
  const pageCountDivergence = input.pdfPageCount !== null
    && input.hwpPageCount !== input.pdfPageCount;
  const previousPageCountError = input.previous?.pdfPageCount === null || !input.previous
    ? null
    : Math.abs(input.previous.hwpPageCount - input.previous.pdfPageCount);
  const currentPageCountError = input.pdfPageCount === null
    ? null
    : Math.abs(input.hwpPageCount - input.pdfPageCount);
  const countRegressed = previousPageCountError !== null
    && currentPageCountError !== null
    && currentPageCountError > previousPageCountError;
  const firstRegressionPage = pages.find(page =>
    page.mismatchRatioDelta !== null && page.mismatchRatioDelta > 0)?.pageIndex
    ?? (countRegressed ? sharedPageCount : null);
  const pixelMismatchPageCount = pages.filter(page => page.mismatchPixels > 0).length;
  const firstDivergentPage = pages.find(page => page.mismatchPixels > 0)?.pageIndex
    ?? (pageCountDivergence ? sharedPageCount : null);
  const firstStructuralDivergencePage = pages.find(isStructuralPageDivergence)?.pageIndex
    ?? (pageCountDivergence ? sharedPageCount : null);
  return {
    schemaVersion: 1,
    scanId: input.scanId,
    status: 'ready',
    pixelMismatchPageCount,
    trigger: input.trigger,
    identity: input.identity,
    renderRevision: input.renderRevision,
    patchIdentity: input.patchIdentity,
    previousRenderRevision: input.previous?.renderRevision ?? null,
    comparisonGap: input.supersededBetween?.length
      ? {
          supersededScanIds: input.supersededBetween.map(scan => scan.scanId),
          renderRevisions: input.supersededBetween.map(scan => scan.renderRevision),
        }
      : null,
    hwpPageCount: input.hwpPageCount,
    pdfPageCount: input.pdfPageCount,
    pageCountDelta: input.pdfPageCount === null
      ? null
      : input.hwpPageCount - input.pdfPageCount,
    firstDivergentPage,
    firstStructuralDivergencePage,
    firstRegressionPage,
    downstreamChangedPageRange: uniqueChangedPages.length > 0
      ? {
          start: uniqueChangedPages[0],
          end: uniqueChangedPages[uniqueChangedPages.length - 1],
        }
      : null,
    pages,
    startedAt: input.startedAt,
    completedAt: input.completedAt,
  };
}
