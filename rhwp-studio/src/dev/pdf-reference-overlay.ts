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
  getRenderGeneration(): number;
}

interface FidelityScanTarget {
  renderRevision: string | null;
  renderGeneration: number;
  previousRenderGeneration: number | null;
}

interface SupersededFidelityScan {
  scanId: number;
  completedPages: number;
  target: FidelityScanTarget;
}

interface FidelityHarnessState {
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

export type FidelityHarnessSnapshot = Omit<FidelityHarnessState, 'status' | 'latestReport'> & {
  schemaVersion: 1;
  owner: string;
  status: FidelityHarnessState['status'] | 'stale';
  current: boolean;
  latestReport: FidelityScanSummary | null;
};

type FidelityPageResult = ReturnType<typeof queryFidelityPage>;
type FidelityNavigationResult = {
  scanId: number | null;
  current: boolean;
  pageIndex: number | null;
  navigatedPageIndex: number | null;
  navigated: boolean;
};

export interface FidelityHarnessApi {
  readonly schemaVersion: 1;
  snapshot(): FidelityHarnessSnapshot;
  pages(query?: FidelityPageQuery): FidelityPageQueryResult;
  page(pageIndex: number, scanId?: number): FidelityPageResult;
  scan(): Promise<FidelityScanSummary | null>;
  gotoFirstRegression(scanId?: number): FidelityNavigationResult;
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
  expectedCssHeight: number;
  sourceCanvas: HTMLCanvasElement;
  lastZoom: number | null;
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

  constructor(
    private readonly pageImageBaseUrl: string,
    private readonly pixelWidth: number,
    private readonly pageCount: number | null,
    readonly pdfName: string,
    private readonly harness: PdfReferenceHarnessOptions,
  ) {
    const segments = pageImageBaseUrl.split('/').filter(Boolean);
    this.referenceKey = segments.at(-2) ?? pageImageBaseUrl;
    this.harnessState = {
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
    return structuredClone({
      schemaVersion: 1,
      owner: this.ownerKey(),
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
      history: this.harnessState.history,
      activeTarget: this.harnessState.activeTarget,
      supersededScans: this.harnessState.supersededScans,
      lastError: this.harnessState.lastError,
    });
  }

  private queryPages(query: FidelityPageQuery = {}): FidelityPageQueryResult {
    if (this.harnessState.activeScanId !== null || !this.latestReportIsCurrent()) {
      const stale = queryFidelityPages(this.harnessState.latestReport, query);
      return { ...stale, current: false, items: [] };
    }
    return queryFidelityPages(this.harnessState.latestReport, query);
  }

  private pageReport(pageIndex: number, scanId?: number): FidelityPageResult {
    if (this.harnessState.activeScanId !== null || !this.latestReportIsCurrent()) {
      return {
        scanId: this.harnessState.latestReport?.scanId ?? null,
        current: false,
        item: null,
      };
    }
    return queryFidelityPage(this.harnessState.latestReport, pageIndex, scanId);
  }

  startBaselineScan(): void {
    void this.scanWholeDocument('baseline');
  }

  onRenderCodePatched(
    renderRevision: string,
    previousRenderGeneration: number,
  ): void {
    void this.scanWholeDocument('subsecond-patch', {
      renderRevision,
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

  private endScan(status: FidelityHarnessState['status']): void {
    this.harnessState.status = status;
    this.harnessState.activeScanId = null;
    this.harnessState.activeTarget = null;
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
    const overflow = this.harnessState.supersededScans.length - 16;
    if (overflow > 0) this.harnessState.supersededScans.splice(0, overflow);
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
        this.endScan('idle');
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
      this.harnessState.history.splice(0, Math.max(0, this.harnessState.history.length - MAX_SCAN_HISTORY));
      this.endScan('ready');
      const documentError = this.documentErrorLine(report);
      if (documentError) this.deliverDocumentError(documentError);
      return report;
    } catch (error) {
      if (abortController.signal.aborted || this.destroyed || scanId !== this.scanSerial) return null;
      this.endScan('error');
      this.harnessState.lastError = error instanceof Error ? error.message : String(error);
      console.warn('[pdf-fidelity] whole-document scan failed:', error);
      return null;
    } finally {
      if (this.scanAbortController === abortController) {
        this.scanAbortController = null;
        if (this.harnessState.activeScanId === scanId) this.endScan(this.harnessState.status);
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
    const { width } = capture;
    const height = reference.surfaceHeight;
    const pad = (pixels: Uint8ClampedArray): Uint8ClampedArray => {
      const surface = new Uint8ClampedArray(width * height * 4).fill(255);
      surface.set(pixels.subarray(0, surface.length));
      return surface;
    };
    const hwpPixels = pad(capture.pixels);
    const referencePixels = pad(reference.pixels);
    const bands = (pixels: Uint8ClampedArray, rows = false) => detectHorizontalRuleBands(
      pixels,
      width,
      height,
      rows
        ? { inkThreshold: 96, minSpanRatio: 1.1, minCoverageRatio: 0.012, maxBands: 96 }
        : { inkThreshold: 96, minSpanRatio: 0.35, minCoverageRatio: 0.55, maxBands: 96 },
    );
    return {
      pageIndex,
      hwpFingerprint: `${capture.width}x${capture.height}:${fingerprintPagePixels(capture.pixels)}`,
      hwpSize: { width: capture.width, height: capture.height },
      referenceSize: { width: reference.width, height: reference.height },
      mismatch: computeReferencePixelDiff(hwpPixels, referencePixels, width, height, DIFF_THRESHOLD),
      hwpHorizontalRules: bands(hwpPixels),
      pdfHorizontalRules: bands(referencePixels),
      hwpInkRows: bands(hwpPixels, true),
      pdfInkRows: bands(referencePixels, true),
    };
  }

  private gotoFirstRegression(scanId?: number): FidelityNavigationResult {
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
      Object.assign(host.dataset, {
        rhwpReferencePage: String(request.pageIndex),
        rhwpReferenceDocument: this.referenceKey,
        rhwpReferenceRenderer: 'ghostscript-media-png',
        rhwpReferenceReady: 'false',
      });
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
        expectedCssHeight: 0,
        sourceCanvas: request.sourceCanvas,
        lastZoom: null,
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
    this.endScan('destroyed');
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
    if (!mounted.image) mounted.host.dataset.rhwpReferenceGeometry = 'loading';
    const active = (): boolean =>
      !this.destroyed && this.mounted.get(pageIndex) === mounted && mounted.desiredSrc === src;
    const loading = (): boolean => active() && mounted.pendingImage === image;

    image.onload = () => {
      void image.decode().catch(() => {}).then(() => {
        if (!loading()) return;
        this.recordGeometry(pageIndex, mounted, image);
        if (mounted.image) mounted.image.replaceWith(image);
        else mounted.host.appendChild(image);
        mounted.image = image;
        mounted.pendingImage = null;
        if (mounted.imageRetryTimer !== null) window.clearTimeout(mounted.imageRetryTimer);
        mounted.imageRetryTimer = null;
        mounted.imageRetryCount = 0;
        mounted.host.dataset.rhwpReferenceReady = 'true';
        this.scheduleDiff(pageIndex, mounted);
      });
    };
    image.onerror = () => {
      if (!loading()) return;
      mounted.pendingImage = null;
      if (mounted.imageRetryCount >= 3) {
        if (!mounted.image) mounted.host.dataset.rhwpReferenceGeometry = 'error';
        return;
      }
      const retry = ++mounted.imageRetryCount;
      mounted.imageRetryTimer = window.setTimeout(() => {
        mounted.imageRetryTimer = null;
        if (
          !active()
          || mounted.imageRetryCount !== retry
        ) return;
        this.loadReplacement(pageIndex, mounted, src);
      }, Math.min(2_000, 250 * 2 ** (retry - 1)));
    };
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
    mounted.diffTimer = window.setTimeout(() => {
      mounted.diffTimer = null;
      if (this.destroyed || this.mounted.get(pageIndex) !== mounted) return;
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
    const { top, left, transform, transformOrigin, width, height } = sourceCanvas.style;
    Object.assign(host.style, { top, left, transform, transformOrigin, width, height });
  }
}
