import type {
  DiagnosticPageCapture,
  PageReferenceLayer,
  ReferencePageRenderRequest,
} from '@/view/page-reference-layer';
import {
  computeReferencePixelDiff,
  detectHorizontalRuleBands,
} from './pdf-reference-diff';
import {
  buildFidelityScanReport,
  compareDiagnosticBands,
  fingerprintPagePixels,
  firstDocumentDivergence,
  isFidelityDocumentCurrent,
  isComparableFidelityPredecessor,
  isFidelityScanCurrent,
  queryFidelityPage,
  queryFidelityPages,
  summarizeFidelityScan,
  type FidelityPageReport,
  type FidelityPageQuery,
  type FidelityPageQueryResult,
  type FidelityPageObservation,
  type FidelityScanReport,
  type FidelityScanSummary,
} from './fidelity-scan';
import {
  formatDocumentError,
  findFirstLineBreakError,
  sendDocumentErrorLine,
  type LineBreakVisibleResult,
} from './document-error-log';
import { DiagnosticPauseGate, yieldToInteractiveWork } from './fidelity-yield';
import { fetchReferenceWithRetry } from './pdf-reference-fetch';

const DIFF_SAMPLE_WIDTH = 512;
const WHOLE_SCAN_SAMPLE_WIDTH = 256;
const DIFF_THRESHOLD = 24;
const MAX_SCAN_HISTORY = 8;

export interface PdfReferenceHarnessOptions {
  documentDigest: string | null;
  documentGeneration: number;
  referenceGeneration: number;
  errorLogCapability: string;
  getDocumentDigest(): string | null;
  getDocumentGeneration(): number;
  getHwpPageCount(): number;
  capturePage(
    pageIndex: number,
    sampleWidth: number,
    signal?: AbortSignal,
  ): Promise<DiagnosticPageCapture>;
  gotoPage(pageIndex: number): boolean;
  getRenderRevision(): string | null;
  getCommittedPatchIdentity(): string | null;
  getRenderGeneration(): number;
}

interface FidelityScanTarget {
  renderRevision: string | null;
  patchIdentity: string | null;
  renderGeneration: number;
  previousRenderGeneration: number | null;
}

interface SupersededFidelityScan {
  scanId: number;
  completedPages: number;
  target: FidelityScanTarget;
}

interface FidelityHarnessState {
  schemaVersion: 1;
  owner: string;
  status: 'idle' | 'scanning' | 'ready' | 'error' | 'destroyed';
  activeScanId: number | null;
  completedPages: number;
  totalPages: number;
  latestReport: FidelityScanReport | null;
  history: FidelityScanSummary[];
  activeTarget: FidelityScanTarget | null;
  supersededScans: SupersededFidelityScan[];
  lastError: string | null;
}

export interface FidelityHarnessSnapshot {
  schemaVersion: 1;
  owner: string;
  status: FidelityHarnessState['status'] | 'stale';
  current: boolean;
  activeScanId: number | null;
  completedPages: number;
  totalPages: number;
  latestReport: FidelityScanSummary | null;
  history: FidelityScanSummary[];
  activeTarget: FidelityScanTarget | null;
  supersededScans: SupersededFidelityScan[];
  lastError: string | null;
}

export interface FidelityHarnessApi {
  readonly schemaVersion: 1;
  snapshot(): FidelityHarnessSnapshot;
  pages(query?: FidelityPageQuery): FidelityPageQueryResult;
  page(pageIndex: number, scanId?: number): {
    scanId: number | null;
    current: boolean;
    item: FidelityPageReport | null;
  };
  scan(): Promise<FidelityScanSummary | null>;
  gotoFirstRegression(scanId?: number): {
    scanId: number | null;
    current: boolean;
    pageIndex: number | null;
    navigatedPageIndex: number | null;
    navigated: boolean;
  };
}

type FidelityHarnessWindow = Window & {
  __rhwpFidelityHarness?: FidelityHarnessApi;
  rhwpDev?: {
    lineBreakVisible?: (
      pageIndex: number,
      options: { start: number; limit: number; geometry: boolean; measurement: boolean },
    ) => LineBreakVisibleResult;
  };
};

interface MountedReferencePage {
  host: HTMLDivElement;
  image: HTMLImageElement | null;
  diffCanvas: HTMLCanvasElement;
  pendingImage: HTMLImageElement | null;
  desiredSrc: string;
  loadingSrc: string;
  expectedCssHeight: number;
  sourceCanvas: HTMLCanvasElement;
  lastZoom: number | null;
  diffSequence: number;
  diffTimer: number | null;
  imageRetryTimer: number | null;
  imageRetryCount: number;
}

interface ReferencePixelCapture {
  width: number;
  height: number;
  surfaceHeight: number;
  pixels: Uint8ClampedArray;
}

/** Ghostscript가 PDF MediaBox 기준으로 만든 페이지 PNG를 Studio 페이지 좌표계에 겹친다. */
export class PdfReferenceOverlay implements PageReferenceLayer {
  private readonly mounted = new Map<number, MountedReferencePage>();
  private readonly referenceKey: string;
  private destroyed = false;
  private scanSerial = 0;
  private scanAbortController: AbortController | null = null;
  private diffLogDeliveryFailed = false;
  private readonly diagnosticsPause = new DiagnosticPauseGate();
  private readonly harnessState: FidelityHarnessState;
  private readonly harnessApi: FidelityHarnessApi;

  private constructor(
    private readonly pageImageBaseUrl: string,
    private readonly pixelWidth: number,
    private readonly pageCount: number | null,
    readonly pdfName: string,
    private readonly harness: PdfReferenceHarnessOptions,
  ) {
    const segments = pageImageBaseUrl.split('/').filter(Boolean);
    this.referenceKey = segments.at(-2) ?? pageImageBaseUrl;
    this.harnessState = {
      schemaVersion: 1,
      owner: this.ownerKey(),
      status: 'idle',
      activeScanId: null,
      completedPages: 0,
      totalPages: 0,
      latestReport: null,
      history: [],
      activeTarget: null,
      supersededScans: [],
      lastError: null,
    };
    this.harnessApi = {
      schemaVersion: 1,
      snapshot: () => this.harnessSnapshot(),
      pages: query => this.queryPages(query),
      page: (pageIndex, scanId) => this.pageReport(pageIndex, scanId),
      scan: async () => {
        const report = await this.scanWholeDocument('manual');
        return report ? summarizeFidelityScan(report) : null;
      },
      gotoFirstRegression: scanId => this.gotoFirstRegression(scanId),
    };
    const harnessWindow = window as FidelityHarnessWindow;
    harnessWindow.__rhwpFidelityHarness = this.harnessApi;
  }

  private harnessSnapshot(): FidelityHarnessSnapshot {
    const current = this.latestReportIsCurrent();
    return {
      schemaVersion: 1,
      owner: this.harnessState.owner,
      status: this.harnessState.status === 'ready' && !current
        ? 'stale'
        : this.harnessState.status,
      current,
      activeScanId: this.harnessState.activeScanId,
      completedPages: this.harnessState.completedPages,
      totalPages: this.harnessState.totalPages,
      latestReport: this.harnessState.latestReport
        ? summarizeFidelityScan(this.harnessState.latestReport)
        : null,
      history: structuredClone(this.harnessState.history),
      activeTarget: structuredClone(this.harnessState.activeTarget),
      supersededScans: structuredClone(this.harnessState.supersededScans),
      lastError: this.harnessState.lastError,
    };
  }

  private queryPages(query: FidelityPageQuery = {}): FidelityPageQueryResult {
    if (this.harnessState.activeScanId !== null || !this.latestReportIsCurrent()) {
      const stale = queryFidelityPages(this.harnessState.latestReport, query);
      return { ...stale, current: false, items: [] };
    }
    return queryFidelityPages(this.harnessState.latestReport, query);
  }

  private pageReport(pageIndex: number, scanId?: number): {
    scanId: number | null;
    current: boolean;
    item: FidelityPageReport | null;
  } {
    if (this.harnessState.activeScanId !== null || !this.latestReportIsCurrent()) {
      return {
        scanId: this.harnessState.latestReport?.scanId ?? null,
        current: false,
        item: null,
      };
    }
    return queryFidelityPage(this.harnessState.latestReport, pageIndex, scanId);
  }

  static async open(
    pageImageBaseUrl: string,
    pixelWidth: number,
    pageCount: number | null,
    pdfName: string,
    harness: PdfReferenceHarnessOptions,
  ): Promise<PdfReferenceOverlay> {
    return new PdfReferenceOverlay(pageImageBaseUrl, pixelWidth, pageCount, pdfName, harness);
  }

  startBaselineScan(): void {
    void this.scanWholeDocument('baseline');
  }

  onRenderCodePatched(
    renderRevision: string,
    patchIdentity: string | null,
    previousRenderGeneration: number,
  ): void {
    void this.scanWholeDocument('subsecond-patch', {
      renderRevision,
      patchIdentity,
      renderGeneration: this.harness.getRenderGeneration(),
      previousRenderGeneration,
    });
  }

  private ownerKey(): string {
    return [
      this.referenceKey,
      this.harness.documentGeneration,
      this.harness.referenceGeneration,
    ].join(':');
  }

  private currentScanTarget(): FidelityScanTarget {
    return {
      renderRevision: this.harness.getRenderRevision(),
      patchIdentity: this.harness.getCommittedPatchIdentity(),
      renderGeneration: this.harness.getRenderGeneration(),
      previousRenderGeneration: null,
    };
  }

  private latestReportIsCurrent(): boolean {
    return !this.destroyed
      && this.documentIsCurrent()
      && this.harnessState.activeScanId === null
      && isFidelityScanCurrent(this.harnessState.latestReport, this.currentScanTarget());
  }

  private documentIsCurrent(): boolean {
    return isFidelityDocumentCurrent(this.harness, {
      documentDigest: this.harness.getDocumentDigest(),
      documentGeneration: this.harness.getDocumentGeneration(),
    });
  }

  private documentErrorLine(
    report: FidelityScanReport,
  ): string | null {
    const first = firstDocumentDivergence(report);
    if (!first) return null;
    if (first.kind === 'page-count') {
      return formatDocumentError('page-count', [
        ['page', first.pageIndex + 1],
        ['expected', report.pdfPageCount ?? 0],
        ['actual', report.hwpPageCount],
      ]);
    }
    if (first.kind === 'structural') {
      try {
        const inspect = (window as FidelityHarnessWindow).rhwpDev?.lineBreakVisible;
        const lineBreak = inspect ? findFirstLineBreakError(
          first.pageIndex + 1,
          start => inspect(first.pageIndex, {
            start,
            limit: 100,
            geometry: false,
            measurement: false,
          }),
        ) : null;
        if (lineBreak) return lineBreak;
      } catch {
        // A semantic inspector failure must not suppress the already-proven paint error.
      }
    }
    const firstDivergentPage = report.pages.find(page => page.pageIndex === first.pageIndex) ?? null;
    if (firstDivergentPage) {
      const bounds = firstDivergentPage.bounds;
      return formatDocumentError('paint', [
        ['page', firstDivergentPage.pageIndex + 1],
        ['ratio', firstDivergentPage.mismatchRatio],
        ['pdfOnly', firstDivergentPage.pdfOnlyPixels],
        ['rhwpOnly', firstDivergentPage.hwpOnlyPixels],
        ['colorOnly', firstDivergentPage.colorMismatchPixels],
        ['bounds', bounds ? `${bounds.x},${bounds.y},${bounds.width},${bounds.height}` : 'none'],
      ]);
    }
    return null;
  }

  private deliverDocumentError(line: string): void {
    void sendDocumentErrorLine(line, this.harness.errorLogCapability).catch((error) => {
      if (this.diffLogDeliveryFailed) return;
      this.diffLogDeliveryFailed = true;
      console.warn('document error log delivery failed; suppressing repeats:', error);
    });
  }

  private rememberSupersededScan(
    scanId: number,
    completedPages: number,
    target: FidelityScanTarget,
  ): void {
    if (this.harnessState.supersededScans.some(scan => scan.scanId === scanId)) return;
    this.harnessState.supersededScans.push({ scanId, completedPages, target });
    if (this.harnessState.supersededScans.length > 16) {
      this.harnessState.supersededScans.splice(
        0,
        this.harnessState.supersededScans.length - 16,
      );
    }
  }

  private async scanWholeDocument(
    trigger: FidelityScanReport['trigger'],
    target = this.currentScanTarget(),
  ): Promise<FidelityScanReport | null> {
    if (this.destroyed) return null;
    const scanId = ++this.scanSerial;
    if (this.harnessState.activeScanId !== null && this.harnessState.activeTarget) {
      this.rememberSupersededScan(
        this.harnessState.activeScanId,
        this.harnessState.completedPages,
        this.harnessState.activeTarget,
      );
    }
    this.scanAbortController?.abort();
    const abortController = new AbortController();
    this.scanAbortController = abortController;
    const hwpPageCount = this.harness.getHwpPageCount();
    const sharedPageCount = Math.min(hwpPageCount, this.pageCount ?? hwpPageCount);
    const startedAt = performance.now();
    this.harnessState.status = 'scanning';
    this.harnessState.activeScanId = scanId;
    this.harnessState.activeTarget = target;
    this.harnessState.completedPages = 0;
    this.harnessState.totalPages = sharedPageCount;
    this.harnessState.lastError = null;

    const renderTargetChanged = (): boolean =>
      hwpPageCount !== this.harness.getHwpPageCount()
      || target.renderRevision !== this.harness.getRenderRevision()
      || target.renderGeneration !== this.harness.getRenderGeneration();
    const targetChanged = (): boolean => !this.documentIsCurrent() || renderTargetChanged();
    const abandonIfStale = (completedPages: number): boolean => {
      const stale = abortController.signal.aborted
        || this.destroyed
        || scanId !== this.scanSerial
        || targetChanged();
      if (!stale) return false;
      if (!this.destroyed && scanId === this.scanSerial) {
        this.rememberSupersededScan(scanId, completedPages, target);
        this.harnessState.status = 'idle';
        this.harnessState.activeScanId = null;
        this.harnessState.activeTarget = null;
        if (!abortController.signal.aborted && this.documentIsCurrent() && renderTargetChanged()) {
          queueMicrotask(() => {
            if (this.destroyed || this.scanSerial !== scanId) return;
            void this.scanWholeDocument(trigger);
          });
        }
      }
      return true;
    };

    try {
      const observations: FidelityPageObservation[] = [];
      for (let pageIndex = 0; pageIndex < sharedPageCount; pageIndex++) {
        await this.diagnosticsPause.wait(abortController.signal);
        if (abandonIfStale(pageIndex)) return null;
        const capture = await this.harness.capturePage(
          pageIndex,
          WHOLE_SCAN_SAMPLE_WIDTH,
          abortController.signal,
        );
        if (abandonIfStale(pageIndex)) return null;
        const reference = await this.loadReferencePixels(
          pageIndex,
          capture.width,
          capture.height,
          abortController.signal,
        );
        observations.push(this.observePage(pageIndex, capture, reference));
        this.harnessState.completedPages = pageIndex + 1;
        await yieldToInteractiveWork(abortController.signal);
      }
      if (abandonIfStale(observations.length)) return null;
      const previousCandidate = this.harnessState.latestReport?.identity.documentKey === this.referenceKey
        && this.harnessState.latestReport.identity.documentGeneration === this.harness.documentGeneration
        ? this.harnessState.latestReport
        : null;
      const previous = isComparableFidelityPredecessor(previousCandidate, trigger, target)
        ? previousCandidate
        : null;
      const report = buildFidelityScanReport({
        scanId,
        trigger,
        identity: {
          documentKey: this.referenceKey,
          documentDigest: this.harness.documentDigest,
          documentGeneration: this.harness.documentGeneration,
          referenceGeneration: this.harness.referenceGeneration,
          renderGeneration: target.renderGeneration,
          pdfName: this.pdfName,
        },
        renderRevision: target.renderRevision,
        patchIdentity: target.patchIdentity,
        hwpPageCount,
        pdfPageCount: this.pageCount,
        observations,
        previous,
        startedAt,
        completedAt: performance.now(),
        supersededBetween: this.harnessState.supersededScans
          .filter(scan => scan.scanId > (previous?.scanId ?? 0) && scan.scanId < scanId)
          .map(scan => ({
            scanId: scan.scanId,
            renderRevision: scan.target.renderRevision,
          })),
      });
      this.harnessState.latestReport = report;
      this.harnessState.history.push(summarizeFidelityScan(report));
      if (this.harnessState.history.length > MAX_SCAN_HISTORY) {
        this.harnessState.history.splice(0, this.harnessState.history.length - MAX_SCAN_HISTORY);
      }
      this.harnessState.status = 'ready';
      this.harnessState.activeScanId = null;
      this.harnessState.activeTarget = null;
      const documentError = this.documentErrorLine(report);
      if (documentError) this.deliverDocumentError(documentError);
      return report;
    } catch (error) {
      if (abortController.signal.aborted || this.destroyed || scanId !== this.scanSerial) return null;
      this.harnessState.status = 'error';
      this.harnessState.activeScanId = null;
      this.harnessState.activeTarget = null;
      this.harnessState.lastError = error instanceof Error ? error.message : String(error);
      console.warn('[pdf-fidelity] whole-document scan failed:', error);
      return null;
    } finally {
      if (this.scanAbortController === abortController) {
        this.scanAbortController = null;
        if (this.harnessState.activeScanId === scanId) {
          this.harnessState.activeScanId = null;
          this.harnessState.activeTarget = null;
        }
      }
    }
  }

  private async loadReferencePixels(
    pageIndex: number,
    width: number,
    height: number,
    signal: AbortSignal,
  ): Promise<ReferencePixelCapture> {
    if (signal.aborted) throw new DOMException('reference image load aborted', 'AbortError');
    const src = `${this.pageImageBaseUrl}/${pageIndex}.png?width=${width}`;
    const response = await fetchReferenceWithRetry(src, signal);
    if (!response.ok) throw new Error(`reference page ${pageIndex} failed to load (${response.status})`);
    const image = await createImageBitmap(await response.blob());
    try {
      if (signal.aborted) throw new DOMException('reference image load aborted', 'AbortError');
      const canvas = document.createElement('canvas');
      const renderedHeight = Math.round(image.height * width / image.width);
      canvas.width = width;
      canvas.height = Math.max(height, renderedHeight);
      const context = canvas.getContext('2d', { willReadFrequently: true });
      if (!context) throw new Error('reference scan canvas is unavailable');
      context.fillStyle = '#fff';
      context.fillRect(0, 0, canvas.width, canvas.height);
      context.drawImage(image, 0, 0, width, renderedHeight);
      return {
        width: canvas.width,
        height: renderedHeight,
        surfaceHeight: canvas.height,
        pixels: context.getImageData(0, 0, canvas.width, canvas.height).data,
      };
    } finally {
      image.close();
    }
  }

  private observePage(
    pageIndex: number,
    capture: DiagnosticPageCapture,
    reference: ReferencePixelCapture,
  ): FidelityPageObservation {
    const comparisonHeight = reference.surfaceHeight;
    const hwpPixels = new Uint8ClampedArray(capture.width * comparisonHeight * 4).fill(255);
    hwpPixels.set(capture.pixels.subarray(0, capture.width * capture.height * 4));
    const referencePixels = new Uint8ClampedArray(capture.width * comparisonHeight * 4).fill(255);
    referencePixels.set(reference.pixels.subarray(0, capture.width * comparisonHeight * 4));
    const horizontalRuleOptions = {
      inkThreshold: 96,
      minSpanRatio: 0.35,
      minCoverageRatio: 0.55,
      maxBands: 96,
    } as const;
    const inkRowOptions = {
      inkThreshold: 96,
      minSpanRatio: 1.1,
      minCoverageRatio: 0.012,
      maxBands: 96,
    } as const;
    return {
      pageIndex,
      hwpFingerprint: `${capture.width}x${capture.height}:${fingerprintPagePixels(capture.pixels)}`,
      hwpSize: { width: capture.width, height: capture.height },
      referenceSize: { width: reference.width, height: reference.height },
      mismatch: computeReferencePixelDiff(
        hwpPixels,
        referencePixels,
        capture.width,
        comparisonHeight,
        DIFF_THRESHOLD,
      ),
      hwpHorizontalRules: detectHorizontalRuleBands(
        hwpPixels,
        capture.width,
        comparisonHeight,
        horizontalRuleOptions,
      ),
      pdfHorizontalRules: detectHorizontalRuleBands(
        referencePixels,
        capture.width,
        comparisonHeight,
        horizontalRuleOptions,
      ),
      hwpInkRows: detectHorizontalRuleBands(
        hwpPixels,
        capture.width,
        comparisonHeight,
        inkRowOptions,
      ),
      pdfInkRows: detectHorizontalRuleBands(
        referencePixels,
        capture.width,
        comparisonHeight,
        inkRowOptions,
      ),
    };
  }

  private gotoFirstRegression(scanId?: number): {
    scanId: number | null;
    current: boolean;
    pageIndex: number | null;
    navigatedPageIndex: number | null;
    navigated: boolean;
  } {
    const report = this.harnessState.latestReport;
    const current = this.latestReportIsCurrent()
      && (scanId === undefined || scanId === report?.scanId);
    if (!report || !current) {
      return {
        scanId: report?.scanId ?? null,
        current,
        pageIndex: null,
        navigatedPageIndex: null,
        navigated: false,
      };
    }
    const pageIndex = report.firstRegressionPage ?? report.firstDivergentPage;
    const navigablePage = pageIndex === null
      ? null
      : Math.min(pageIndex, Math.max(0, report.hwpPageCount - 1));
    return {
      scanId: report.scanId,
      current,
      pageIndex,
      navigatedPageIndex: navigablePage,
      navigated: navigablePage !== null && this.harness.gotoPage(navigablePage),
    };
  }

  syncPage(request: ReferencePageRenderRequest): void {
    if (this.destroyed) return;
    if (this.pageCount !== null && request.pageIndex >= this.pageCount) {
      this.removePage(request.pageIndex);
      return;
    }
    const src = `${this.pageImageBaseUrl}/${request.pageIndex}.png?width=${this.pixelWidth}`;
    let mounted = this.mounted.get(request.pageIndex);
    if (!mounted) {
      const host = document.createElement('div');
      host.className = 'pdf-reference-overlay';
      host.dataset.rhwpReferencePage = String(request.pageIndex);
      host.dataset.rhwpReferenceDocument = this.referenceKey;
      host.dataset.rhwpReferenceRenderer = 'ghostscript-media-png';
      host.dataset.rhwpReferenceReady = 'false';
      host.setAttribute('aria-hidden', 'true');
      const diffCanvas = document.createElement('canvas');
      diffCanvas.className = 'pdf-reference-diff';
      host.appendChild(diffCanvas);
      mounted = {
        host,
        image: null,
        diffCanvas,
        pendingImage: null,
        desiredSrc: '',
        loadingSrc: '',
        expectedCssHeight: 0,
        sourceCanvas: request.sourceCanvas,
        lastZoom: null,
        diffSequence: 0,
        diffTimer: null,
        imageRetryTimer: null,
        imageRetryCount: 0,
      };
      this.mounted.set(request.pageIndex, mounted);
      request.sourceCanvas.parentElement?.appendChild(host);
    }

    this.applyPageBox(mounted.host, request.sourceCanvas);
    mounted.expectedCssHeight = Number.parseFloat(request.sourceCanvas.style.height);
    mounted.sourceCanvas = request.sourceCanvas;
    const zoomOnly = mounted.lastZoom !== null
      && Math.abs(mounted.lastZoom - request.zoom) > 1e-6;
    mounted.lastZoom = request.zoom;
    if (!zoomOnly) {
      mounted.diffSequence += 1;
      if (mounted.image) this.scheduleDiff(request.pageIndex, mounted);
    }
    if (mounted.desiredSrc === src) return;
    if (mounted.imageRetryTimer !== null) {
      window.clearTimeout(mounted.imageRetryTimer);
      mounted.imageRetryTimer = null;
    }
    mounted.imageRetryCount = 0;
    mounted.desiredSrc = src;
    this.loadReplacement(request.pageIndex, mounted, src);
  }

  setDiagnosticsPaused(paused: boolean): void {
    if (this.diagnosticsPause.paused === paused) return;
    this.diagnosticsPause.set(paused);
    for (const [pageIndex, mounted] of this.mounted) {
      if (mounted.diffTimer !== null) {
        window.clearTimeout(mounted.diffTimer);
        mounted.diffTimer = null;
      }
      if (!paused && mounted.image) this.scheduleDiff(pageIndex, mounted);
    }
  }

  removePage(pageIndex: number): void {
    const mounted = this.mounted.get(pageIndex);
    if (!mounted) return;
    mounted.pendingImage?.removeAttribute('src');
    mounted.image?.removeAttribute('src');
    if (mounted.diffTimer !== null) window.clearTimeout(mounted.diffTimer);
    if (mounted.imageRetryTimer !== null) window.clearTimeout(mounted.imageRetryTimer);
    mounted.host.remove();
    this.mounted.delete(pageIndex);
  }

  retainPages(pageIndices: readonly number[]): void {
    const retained = new Set(pageIndices);
    for (const pageIndex of Array.from(this.mounted.keys())) {
      if (!retained.has(pageIndex)) this.removePage(pageIndex);
    }
  }

  clearMountedPages(): void {
    for (const pageIndex of Array.from(this.mounted.keys())) this.removePage(pageIndex);
  }

  async destroy(): Promise<void> {
    if (this.destroyed) return;
    this.destroyed = true;
    this.scanSerial += 1;
    this.scanAbortController?.abort();
    this.scanAbortController = null;
    this.clearMountedPages();
    this.harnessState.status = 'destroyed';
    this.harnessState.activeScanId = null;
    const harnessWindow = window as FidelityHarnessWindow;
    if (harnessWindow.__rhwpFidelityHarness === this.harnessApi) {
      delete harnessWindow.__rhwpFidelityHarness;
    }
  }

  private loadReplacement(
    pageIndex: number,
    mounted: MountedReferencePage,
    src: string,
  ): void {
    mounted.pendingImage?.removeAttribute('src');
    const image = document.createElement('img');
    image.className = 'pdf-reference-image';
    image.alt = '';
    image.decoding = 'async';
    image.draggable = false;
    mounted.pendingImage = image;
    mounted.loadingSrc = src;
    if (!mounted.image) mounted.host.dataset.rhwpReferenceGeometry = 'loading';

    image.addEventListener('load', () => {
      void image.decode().catch(() => {}).then(() => {
        const current = this.mounted.get(pageIndex);
        if (
          this.destroyed
          || current !== mounted
          || current.pendingImage !== image
          || current.desiredSrc !== src
          || current.loadingSrc !== src
        ) return;

        this.recordGeometry(pageIndex, current, image);
        if (current.image) current.image.replaceWith(image);
        else current.host.appendChild(image);
        current.image = image;
        current.pendingImage = null;
        current.loadingSrc = '';
        if (current.imageRetryTimer !== null) window.clearTimeout(current.imageRetryTimer);
        current.imageRetryTimer = null;
        current.imageRetryCount = 0;
        current.host.dataset.rhwpReferenceReady = 'true';
        this.scheduleDiff(pageIndex, current);
      });
    }, { once: true });
    image.addEventListener('error', () => {
      const current = this.mounted.get(pageIndex);
      if (current !== mounted || current.pendingImage !== image) return;
      current.pendingImage = null;
      current.loadingSrc = '';
      if (current.imageRetryCount >= 3) {
        if (!current.image) current.host.dataset.rhwpReferenceGeometry = 'error';
        return;
      }
      current.imageRetryCount += 1;
      const retryCount = current.imageRetryCount;
      current.imageRetryTimer = window.setTimeout(() => {
        current.imageRetryTimer = null;
        if (
          this.destroyed
          || this.mounted.get(pageIndex) !== current
          || current.desiredSrc !== src
          || current.loadingSrc !== ''
          || current.imageRetryCount !== retryCount
        ) return;
        this.loadReplacement(pageIndex, current, src);
      }, Math.min(2_000, 250 * 2 ** (retryCount - 1)));
    }, { once: true });
    image.src = src;
  }

  private recordGeometry(
    pageIndex: number,
    mounted: MountedReferencePage,
    image: HTMLImageElement,
  ): void {
    if (image.naturalWidth <= 0) return;
    const cssWidth = Number.parseFloat(mounted.host.style.width);
    const renderedHeight = image.naturalHeight * cssWidth / image.naturalWidth;
    const delta = Math.abs(renderedHeight - mounted.expectedCssHeight);
    mounted.host.dataset.rhwpReferenceGeometry = delta <= 1.5 ? 'matched' : 'mismatch';
    if (delta > 1.5) {
      console.warn(
        `[pdf-reference] page ${pageIndex + 1} size mismatch: ` +
        `hwpHeight=${mounted.expectedCssHeight.toFixed(2)} ` +
        `pdfHeight=${renderedHeight.toFixed(2)}`,
      );
    }
  }

  private scheduleDiff(pageIndex: number, mounted: MountedReferencePage): void {
    if (mounted.diffTimer !== null) window.clearTimeout(mounted.diffTimer);
    if (this.diagnosticsPause.paused) {
      mounted.diffTimer = null;
      return;
    }
    const sequence = mounted.diffSequence;
    mounted.diffTimer = window.setTimeout(() => {
      mounted.diffTimer = null;
      if (this.destroyed || this.mounted.get(pageIndex) !== mounted || sequence !== mounted.diffSequence) return;
      this.reportDiff(pageIndex, mounted);
    }, 120);
  }

  private reportDiff(
    pageIndex: number,
    mounted: MountedReferencePage,
  ): void {
    const image = mounted.image;
    const sourceCanvas = mounted.sourceCanvas;
    if (!image || !image.complete || image.naturalWidth <= 0 || sourceCanvas.width <= 0) return;
    const sampleWidth = Math.min(DIFF_SAMPLE_WIDTH, sourceCanvas.width);
    const sampleHeight = Math.max(1, Math.round(sourceCanvas.height * sampleWidth / sourceCanvas.width));
    const hwpCanvas = document.createElement('canvas');
    const referenceCanvas = document.createElement('canvas');
    hwpCanvas.width = referenceCanvas.width = sampleWidth;
    hwpCanvas.height = referenceCanvas.height = sampleHeight;
    const hwpContext = hwpCanvas.getContext('2d', { willReadFrequently: true });
    const referenceContext = referenceCanvas.getContext('2d', { willReadFrequently: true });
    if (!hwpContext || !referenceContext) return;

    try {
      hwpContext.fillStyle = '#fff';
      hwpContext.fillRect(0, 0, sampleWidth, sampleHeight);
      const layerCanvases = [
        sourceCanvas,
        ...Array.from(
          sourceCanvas.parentElement?.querySelectorAll<HTMLCanvasElement>(
            `canvas[data-rhwp-overlay-page="${pageIndex}"]`,
          ) ?? [],
        ),
      ].sort((left, right) => {
        const leftZ = Number.parseInt(getComputedStyle(left).zIndex, 10) || 0;
        const rightZ = Number.parseInt(getComputedStyle(right).zIndex, 10) || 0;
        if (leftZ !== rightZ) return leftZ - rightZ;
        return Array.prototype.indexOf.call(left.parentElement?.children ?? [], left)
          - Array.prototype.indexOf.call(right.parentElement?.children ?? [], right);
      });
      for (const layer of layerCanvases) {
        hwpContext.drawImage(layer, 0, 0, sampleWidth, sampleHeight);
      }

      const pageBox = mounted.host.getBoundingClientRect();
      const domImages = Array.from(
        sourceCanvas.parentElement?.querySelectorAll<HTMLImageElement>(
          `[data-rhwp-overlay-page="${pageIndex}"] img`,
        ) ?? [],
      ).filter(domImage => domImage.complete && domImage.naturalWidth > 0);
      for (const domImage of domImages) {
        const imageBox = domImage.getBoundingClientRect();
        hwpContext.drawImage(
          domImage,
          (imageBox.left - pageBox.left) * sampleWidth / pageBox.width,
          (imageBox.top - pageBox.top) * sampleHeight / pageBox.height,
          imageBox.width * sampleWidth / pageBox.width,
          imageBox.height * sampleHeight / pageBox.height,
        );
      }

      const referenceHeight = Math.round(image.naturalHeight * sampleWidth / image.naturalWidth);
      referenceContext.drawImage(image, 0, 0, sampleWidth, referenceHeight);
      const hwpPixels = hwpContext.getImageData(0, 0, sampleWidth, sampleHeight).data;
      const referencePixels = referenceContext.getImageData(0, 0, sampleWidth, sampleHeight).data;
      const mismatchMask = new Uint8ClampedArray(sampleWidth * sampleHeight * 4);
      computeReferencePixelDiff(
        hwpPixels,
        referencePixels,
        sampleWidth,
        sampleHeight,
        DIFF_THRESHOLD,
        mismatchMask,
      );
      mounted.diffCanvas.width = sampleWidth;
      mounted.diffCanvas.height = sampleHeight;
      mounted.diffCanvas.getContext('2d')?.putImageData(
        new ImageData(mismatchMask, sampleWidth, sampleHeight),
        0,
        0,
      );
    } catch (error) {
      console.warn(formatDocumentError('paint', [
        ['page', pageIndex + 1],
        ['capture', 'failed'],
      ]), error);
    }
  }

  private applyPageBox(host: HTMLDivElement, sourceCanvas: HTMLCanvasElement): void {
    host.style.top = sourceCanvas.style.top;
    host.style.left = sourceCanvas.style.left;
    host.style.transform = sourceCanvas.style.transform;
    host.style.transformOrigin = sourceCanvas.style.transformOrigin;
    host.style.width = sourceCanvas.style.width;
    host.style.height = sourceCanvas.style.height;
  }
}
