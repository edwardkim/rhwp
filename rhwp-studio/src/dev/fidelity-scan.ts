import type { HorizontalRuleDiagnostics, PixelDiffMetrics } from './pdf-reference-diff';

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
  hwpSize: { width: number; height: number };
  referenceSize: { width: number; height: number };
  mismatch: PixelDiffMetrics;
  hwpHorizontalRules: HorizontalRuleDiagnostics;
  pdfHorizontalRules: HorizontalRuleDiagnostics;
  hwpInkRows: HorizontalRuleDiagnostics;
  pdfInkRows: HorizontalRuleDiagnostics;
}

export interface FidelityPageReport extends PixelDiffMetrics {
  pageIndex: number;
  hwpSize: { width: number; height: number };
  referenceSize: { width: number; height: number };
  referenceHeightDelta: number;
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
  trigger: 'baseline' | 'subsecond-patch' | 'manual';
  identity: FidelityScanIdentity;
  hwpPageCount: number;
  pdfPageCount: number | null;
  pageCountDelta: number | null;
  firstDivergentPage: number | null;
  firstStructuralDivergencePage: number | null;
  pages: FidelityPageReport[];
  startedAt: number;
  completedAt: number;
}

export type FidelityScanSummary = Omit<FidelityScanReport, 'pages'> & { reportedPages: number };
export type FirstDocumentDivergence = { kind: 'structural' | 'page-count'; pageIndex: number };

const round = (value: number, digits: number): number => Number(value.toFixed(digits));

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
  const structural = report.firstStructuralDivergencePage;
  const countBoundary = report.pageCountDelta && report.pdfPageCount !== null
    ? Math.min(report.hwpPageCount, report.pdfPageCount)
    : null;
  if (countBoundary !== null && (structural === null || countBoundary <= structural)) {
    return { kind: 'page-count', pageIndex: countBoundary };
  }
  return structural === null ? null : { kind: 'structural', pageIndex: structural };
}

export function isFidelityScanCurrent(
  report: { identity: { renderGeneration: number } } | null,
  renderGeneration: number,
): boolean {
  return report?.identity.renderGeneration === renderGeneration;
}

const STRUCTURAL_MISMATCH_RATIO = 0.05;
const STRUCTURAL_BAND_DRIFT_PX = 2;

/** Ignore glyph raster/color noise and retain page-scale layout evidence. */
export function isStructuralPageDivergence(page: FidelityPageReport): boolean {
  return page.referenceHeightDelta !== 0
    || page.mismatchRatio >= STRUCTURAL_MISMATCH_RATIO
    || page.horizontalRuleDelta.countDelta !== 0
    || (page.horizontalRuleDelta.maxCenterDelta ?? 0) >= STRUCTURAL_BAND_DRIFT_PX
    || Math.abs(page.inkRowDelta.countDelta) >= 1
    || (page.inkRowDelta.maxCenterDelta ?? 0) >= STRUCTURAL_BAND_DRIFT_PX;
}

export function summarizeFidelityScan(report: FidelityScanReport): FidelityScanSummary {
  const { pages, ...summary } = report;
  return { ...summary, reportedPages: pages.length };
}

export function compareDiagnosticBands(
  hwp: HorizontalRuleDiagnostics,
  pdf: HorizontalRuleDiagnostics,
): DiagnosticBandDelta {
  const pairedCount = Math.min(hwp.bands.length, pdf.bands.length);
  const deltas = Array.from({ length: pairedCount }, (_, index) =>
    Math.abs(hwp.bands[index].centerY - pdf.bands[index].centerY));
  const mean = deltas.reduce((sum, value) => sum + value, 0) / deltas.length;
  return {
    hwpCount: hwp.totalBands,
    pdfCount: pdf.totalBands,
    countDelta: hwp.totalBands - pdf.totalBands,
    pairedCount,
    maxCenterDelta: deltas.length ? round(Math.max(...deltas), 3) : null,
    meanCenterDelta: deltas.length ? round(mean, 3) : null,
  };
}

export function buildFidelityScanReport(input: {
  scanId: number;
  trigger: FidelityScanReport['trigger'];
  identity: FidelityScanIdentity;
  hwpPageCount: number;
  pdfPageCount: number | null;
  observations: FidelityPageObservation[];
  startedAt: number;
  completedAt: number;
}): FidelityScanReport {
  const pages = input.observations.map((page): FidelityPageReport => ({
    pageIndex: page.pageIndex,
    hwpSize: page.hwpSize,
    referenceSize: page.referenceSize,
    referenceHeightDelta: page.referenceSize.height - page.hwpSize.height,
    ...page.mismatch,
    mismatchRatio: round(page.mismatch.mismatchRatio, 6),
    meanAbsoluteError: round(page.mismatch.meanAbsoluteError, 3),
    maxAbsoluteError: round(page.mismatch.maxAbsoluteError, 3),
    horizontalRuleDelta: compareDiagnosticBands(page.hwpHorizontalRules, page.pdfHorizontalRules),
    inkRowDelta: compareDiagnosticBands(page.hwpInkRows, page.pdfInkRows),
  }));
  const sharedCount = Math.min(input.hwpPageCount, input.pdfPageCount ?? input.hwpPageCount);
  const countMismatch = input.pdfPageCount !== null && input.hwpPageCount !== input.pdfPageCount;
  return {
    schemaVersion: 1,
    scanId: input.scanId,
    status: 'ready',
    trigger: input.trigger,
    identity: input.identity,
    hwpPageCount: input.hwpPageCount,
    pdfPageCount: input.pdfPageCount,
    pageCountDelta: input.pdfPageCount === null ? null : input.hwpPageCount - input.pdfPageCount,
    firstDivergentPage: pages.find(page => page.mismatchPixels > 0)?.pageIndex
      ?? (countMismatch ? sharedCount : null),
    firstStructuralDivergencePage: pages.find(isStructuralPageDivergence)?.pageIndex
      ?? (countMismatch ? sharedCount : null),
    pages,
    startedAt: input.startedAt,
    completedAt: input.completedAt,
  };
}
