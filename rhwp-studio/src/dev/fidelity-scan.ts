import type { PixelDiffMetrics } from './pdf-reference-diff.ts';

export interface FidelityPageObservation {
  pageIndex: number;
  hwpSize: { width: number; height: number };
  referenceSize: { width: number; height: number };
  mismatch: PixelDiffMetrics;
}

export interface FidelityPageReport extends PixelDiffMetrics {
  pageIndex: number;
  hwpSize: { width: number; height: number };
  referenceSize: { width: number; height: number };
  referenceHeightDelta: number;
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
  schemaVersion: 2;
  scanId: number;
  status: 'ready';
  trigger: 'baseline' | 'subsecond-patch' | 'manual';
  identity: FidelityScanIdentity;
  hwpPageCount: number;
  pdfPageCount: number | null;
  pageCountDelta: number | null;
  firstDivergentPage: number | null;
  firstPaintErrorPage: number | null;
  pages: FidelityPageReport[];
  startedAt: number;
  completedAt: number;
}

export type FidelityScanSummary = Omit<FidelityScanReport, 'pages'> & { reportedPages: number };
export type FirstDocumentDivergence = { kind: 'paint' | 'page-count'; pageIndex: number };

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
  'firstPaintErrorPage' | 'hwpPageCount' | 'pdfPageCount' | 'pageCountDelta'
>): FirstDocumentDivergence | null {
  const paint = report.firstPaintErrorPage;
  const countBoundary = report.pageCountDelta && report.pdfPageCount !== null
    ? Math.min(report.hwpPageCount, report.pdfPageCount)
    : null;
  if (countBoundary !== null && (paint === null || countBoundary <= paint)) {
    return { kind: 'page-count', pageIndex: countBoundary };
  }
  return paint === null ? null : { kind: 'paint', pageIndex: paint };
}

export function isFidelityScanCurrent(
  report: { identity: { renderGeneration: number } } | null,
  renderGeneration: number,
): boolean {
  return report?.identity.renderGeneration === renderGeneration;
}

const STRUCTURAL_MISMATCH_RATIO = 0.05;

/** PDF paint is an automatic error only when at least 5% of the sampled page differs. */
export function isPdfPaintError(page: Pick<PixelDiffMetrics, 'mismatchRatio'>): boolean {
  return page.mismatchRatio >= STRUCTURAL_MISMATCH_RATIO;
}

export function summarizeFidelityScan(report: FidelityScanReport): FidelityScanSummary {
  const { pages, ...summary } = report;
  return { ...summary, reportedPages: pages.length };
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
  }));
  const sharedCount = Math.min(input.hwpPageCount, input.pdfPageCount ?? input.hwpPageCount);
  const countMismatch = input.pdfPageCount !== null && input.hwpPageCount !== input.pdfPageCount;
  const firstPaintPage = input.observations
    .find(page => isPdfPaintError(page.mismatch))
    ?.pageIndex ?? null;
  return {
    schemaVersion: 2,
    scanId: input.scanId,
    status: 'ready',
    trigger: input.trigger,
    identity: input.identity,
    hwpPageCount: input.hwpPageCount,
    pdfPageCount: input.pdfPageCount,
    pageCountDelta: input.pdfPageCount === null ? null : input.hwpPageCount - input.pdfPageCount,
    firstDivergentPage: pages.find(page => page.mismatchPixels > 0)?.pageIndex
      ?? (countMismatch ? sharedCount : null),
    firstPaintErrorPage: firstPaintPage,
    pages,
    startedAt: input.startedAt,
    completedAt: input.completedAt,
  };
}
