import type {
  DiagnosticPageCapture,
  PageReferenceLayer,
  ReferencePageRenderRequest,
} from '@/view/page-reference-layer';
import {
  computeReferencePixelDiff,
  detectHorizontalRuleBands,
} from './pdf-reference-diff.ts';
import {
  buildFidelityScanReport,
  firstDocumentDivergence,
  isFidelityDocumentCurrent,
  isFidelityScanCurrent,
  summarizeFidelityScan,
  type FidelityPageObservation,
  type FidelityScanReport,
  type FidelityScanSummary,
} from './fidelity-scan.ts';
import {
  attachDocumentErrorTrace,
  formatFirstLineBreakError,
  formatDocumentError,
  MAX_LAYOUT_TRACE_ENTRIES,
  parseLayoutTrace,
  sendDocumentErrorLine,
  type LayoutTraceEntry,
  type LineBreakVisibleResult,
} from './document-error-log.ts';
import { DiagnosticPauseGate, yieldToInteractiveWork } from './fidelity-yield.ts';
import { fetchWithBusyRetry } from './pdf-reference-fetch.ts';

const DIFF_SAMPLE_WIDTH = 512;
const WHOLE_SCAN_SAMPLE_WIDTH = 256;
const DIFF_THRESHOLD = 24;
const MAX_PENDING_LAYOUT_TRACE_BATCHES = 4;

export class LayoutTraceMailbox {
  private readonly getRenderCodeRevision: () => string | null;
  private batches: Array<{ serialized: string; renderCodeRevision: string }> = [];

  constructor(getRenderCodeRevision: () => string | null) {
    this.getRenderCodeRevision = getRenderCodeRevision;
  }

  push(serialized: string, renderCodeRevision = this.getRenderCodeRevision()): void {
    if (serialized === '[]' || renderCodeRevision === null) return;
    if (this.batches.length === MAX_PENDING_LAYOUT_TRACE_BATCHES) this.batches.shift();
    this.batches.push({ serialized, renderCodeRevision });
  }

  takeCurrent(): string[] {
    const current = this.getRenderCodeRevision();
    const batches = this.batches
      .filter(batch => batch.renderCodeRevision === current)
      .map(batch => batch.serialized);
    this.batches = [];
    return batches;
  }
}

export interface PdfReferenceHarnessOptions {
  errorLogCapability: string;
  documentDigest: string | null;
  documentGeneration: number;
  referenceGeneration: number;
  getDocumentDigest(): string | null;
  getDocumentGeneration(): number;
  getHwpPageCount(): number;
  traceLayout?<T>(run: () => T): T;
  takeLayoutTrace?(): readonly string[];
  capturePage(
    pageIndex: number,
    sampleWidth: number,
    signal?: AbortSignal,
  ): Promise<DiagnosticPageCapture & { layoutTrace?: string }>;
  getRenderGeneration(): number;
}

interface FidelityHarnessState {
  status: 'idle' | 'scanning' | 'ready' | 'error' | 'destroyed';
  activeScanId: number | null;
  completedPages: number;
  totalPages: number;
  latestReport: FidelityScanReport | null;
  lastError: string | null;
}

export type FidelityHarnessSnapshot = Omit<FidelityHarnessState, 'status' | 'latestReport'> & {
  schemaVersion: 1;
  owner: string;
  status: FidelityHarnessState['status'] | 'stale';
  current: boolean;
  latestReport: FidelityScanSummary | null;
};

export interface FidelityHarnessApi {
  readonly schemaVersion: 1;
  snapshot(): FidelityHarnessSnapshot;
  scan(): Promise<FidelityScanSummary | null>;
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
  private readonly pageImageBaseUrl: string;
  private readonly pixelWidth: number;
  private readonly pageCount: number | null;
  readonly pdfName: string;
  private readonly harness: PdfReferenceHarnessOptions;
  private readonly mounted = new Map<number, MountedReferencePage>();
  private readonly referenceKey: string;
  private readonly rasterSession: string;
  private destroyed = false;
  private scanSerial = 0;
  private scanAbortController: AbortController | null = null;
  private diffLogDeliveryFailed = false;
  private readonly diagnosticsPause = new DiagnosticPauseGate();
  private readonly harnessState: FidelityHarnessState;
  private readonly harnessApi: FidelityHarnessApi;

  constructor(
    pageImageBaseUrl: string,
    pixelWidth: number,
    pageCount: number | null,
    pdfName: string,
    harness: PdfReferenceHarnessOptions,
  ) {
    this.pageImageBaseUrl = pageImageBaseUrl;
    this.pixelWidth = pixelWidth;
    this.pageCount = pageCount;
    this.pdfName = pdfName;
    this.harness = harness;
    this.rasterSession = `${Math.trunc(performance.timeOrigin)}-${harness.referenceGeneration}`;
    const segments = pageImageBaseUrl.split('/').filter(Boolean);
    this.referenceKey = segments.at(-2) ?? pageImageBaseUrl;
    this.harnessState = {
      status: 'idle',
      activeScanId: null,
      completedPages: 0,
      totalPages: 0,
      latestReport: null,
      lastError: null,
    };
    this.harnessApi = {
      schemaVersion: 1,
      snapshot: () => this.harnessSnapshot(),
      scan: async () => {
        const report = await this.scanWholeDocument('manual');
        return report ? summarizeFidelityScan(report) : null;
      },
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
      lastError: this.harnessState.lastError,
    });
  }

  startBaselineScan(): void {
    void this.scanWholeDocument('baseline');
  }

  onRenderCodePatched(): void {
    void this.scanWholeDocument('subsecond-patch');
  }

  private ownerKey(): string {
    return [
      this.referenceKey,
      this.harness.documentGeneration,
      this.harness.referenceGeneration,
    ].join(':');
  }

  private latestReportIsCurrent(): boolean {
    return !this.destroyed
      && this.documentIsCurrent()
      && this.harnessState.activeScanId === null
      && isFidelityScanCurrent(this.harnessState.latestReport, this.harness.getRenderGeneration());
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
  }

  private async inspectLineBreak(
    pageIndex: number,
    stale: () => boolean,
    signal: AbortSignal,
  ): Promise<{ line: string | null; stale: boolean }> {
    const inspect = (window as FidelityHarnessWindow).rhwpDev?.lineBreakVisible;
    if (!inspect) return { line: null, stale: false };
    const visited = new Set<number>();
    let start = 0;
    let total: number | null = null;
    while (!visited.has(start) && (total === null || visited.size <= total)) {
      await this.diagnosticsPause.wait(signal);
      if (stale()) return { line: null, stale: true };
      visited.add(start);
      this.takeLayoutTrace();
      let result: LineBreakVisibleResult | undefined;
      try {
        result = inspect(pageIndex, {
          start,
          limit: 100,
          geometry: false,
          measurement: false,
        });
      } catch {
        this.takeLayoutTrace();
        return { line: null, stale: false };
      }
      this.takeLayoutTrace();
      if (stale()) return { line: null, stale: true };
      const mismatches: Array<{ line: string; offset: number | null }> = [];
      for (let index = 0; index < (result?.items?.length ?? 0); index++) {
        const line = formatFirstLineBreakError(pageIndex + 1, [result!.items![index]]);
        if (!line) continue;
        const offset = result?.itemOffsets?.[index];
        mismatches.push({
          line,
          offset: Number.isSafeInteger(offset) && Number(offset) >= 0 ? Number(offset) : null,
        });
      }
      for (const mismatch of mismatches) {
        const mismatchOffset = mismatch.offset;
        if (mismatchOffset === null) return { line: mismatch.line, stale: false };
        await this.diagnosticsPause.wait(signal);
        if (stale()) return { line: null, stale: true };
        this.takeLayoutTrace();
        let exact: LineBreakVisibleResult | undefined;
        try {
          exact = this.withLayoutTrace(() => inspect(pageIndex, {
            start: mismatchOffset,
            limit: 1,
            geometry: false,
            measurement: false,
          }));
        } catch {
          this.takeLayoutTrace();
          return stale()
            ? { line: null, stale: true }
            : { line: mismatch.line, stale: false };
        }
        const trace = this.takeLayoutTrace();
        if (stale()) return { line: null, stale: true };
        if (!exact || exact.available === false || (exact.errors?.length ?? 0) > 0) {
          return { line: mismatch.line, stale: false };
        }
        const line = formatFirstLineBreakError(pageIndex + 1, exact.items ?? []);
        if (line) return { line: attachDocumentErrorTrace(line, trace), stale: false };
        await yieldToInteractiveWork(signal);
        if (stale()) return { line: null, stale: true };
      }
      const reportedTotal = result?.total;
      total ??= Number.isSafeInteger(reportedTotal) && Number(reportedTotal) >= 0
        ? Number(reportedTotal)
        : 0;
      const next = result?.nextOffset;
      if (!Number.isSafeInteger(next) || Number(next) <= start) {
        return { line: null, stale: false };
      }
      start = Number(next);
      await yieldToInteractiveWork(signal);
      if (stale()) return { line: null, stale: true };
    }
    return { line: null, stale: false };
  }

  private fallbackDocumentErrorLine(
    report: FidelityScanReport,
    tracePageIndex: number | null,
    trace: readonly LayoutTraceEntry[],
  ): string | null {
    const first = firstDocumentDivergence(report);
    if (!first) return null;
    const selectedTrace = first.pageIndex === tracePageIndex ? trace : [];
    if (first.kind === 'page-count') {
      return attachDocumentErrorTrace(formatDocumentError('page-count', [
        ['page', first.pageIndex + 1],
        ['expected', report.pdfPageCount ?? 0],
        ['actual', report.hwpPageCount],
      ]), selectedTrace);
    }
    const page = report.pages.find(candidate => candidate.pageIndex === first.pageIndex);
    if (!page) return null;
    const bounds = page.bounds;
    const ruleDelta = page.horizontalRuleDelta;
    return attachDocumentErrorTrace(formatDocumentError('paint', [
      ['page', page.pageIndex + 1],
      ['ratio', page.mismatchRatio],
      ['pdfOnly', page.pdfOnlyPixels],
      ['rhwpOnly', page.hwpOnlyPixels],
      ['colorOnly', page.colorMismatchPixels],
      ['bounds', bounds ? `${bounds.x},${bounds.y},${bounds.width},${bounds.height}` : 'none'],
      ['ruleDelta', [
        ruleDelta.countDelta,
        ruleDelta.maxCenterDelta ?? '-',
        ruleDelta.hwpEvidenceCenters.join(':') || '-',
        ruleDelta.pdfEvidenceCenters.join(':') || '-',
      ].join(',')],
    ]), selectedTrace);
  }

  private takeLayoutTrace(): LayoutTraceEntry[] {
    return this.mergeLayoutTrace(
      ...(this.harness.takeLayoutTrace?.() ?? []).map(parseLayoutTrace),
    );
  }

  private mergeLayoutTrace(...groups: readonly (readonly LayoutTraceEntry[])[]): LayoutTraceEntry[] {
    const merged = new Map<number, LayoutTraceEntry>();
    for (const entry of groups.flat()) {
      merged.set(entry.id, entry);
    }
    return [...merged.values()].sort((a, b) => a.id - b.id).slice(-MAX_LAYOUT_TRACE_ENTRIES);
  }

  private withLayoutTrace<T>(run: () => T): T {
    return this.harness.traceLayout ? this.harness.traceLayout(run) : run();
  }

  private deliverDocumentError(line: string): void {
    void sendDocumentErrorLine(line, this.harness.errorLogCapability).catch((error) => {
      if (this.diffLogDeliveryFailed) return;
      this.diffLogDeliveryFailed = true;
      console.warn('document error log delivery failed; suppressing repeats:', error);
    });
  }

  private async scanWholeDocument(
    trigger: FidelityScanReport['trigger'],
  ): Promise<FidelityScanReport | null> {
    if (this.destroyed) return null;
    const scanId = ++this.scanSerial;
    this.scanAbortController?.abort();
    const abortController = new AbortController();
    this.scanAbortController = abortController;
    const hwpPageCount = this.harness.getHwpPageCount();
    const sharedPageCount = Math.min(hwpPageCount, this.pageCount ?? hwpPageCount);
    const renderGeneration = this.harness.getRenderGeneration();
    const startedAt = performance.now();
    this.harnessState.status = 'scanning';
    this.harnessState.activeScanId = scanId;
    this.harnessState.completedPages = 0;
    this.harnessState.totalPages = sharedPageCount;
    this.harnessState.lastError = null;

    const targetChanged = (): boolean =>
      hwpPageCount !== this.harness.getHwpPageCount()
      || renderGeneration !== this.harness.getRenderGeneration()
      || !this.documentIsCurrent();
    const abandonIfStale = (): boolean => {
      const stale = abortController.signal.aborted
        || this.destroyed
        || scanId !== this.scanSerial
        || targetChanged();
      if (!stale) return false;
      if (!this.destroyed && scanId === this.scanSerial) {
        this.endScan('idle');
        if (!abortController.signal.aborted && this.documentIsCurrent()) {
          queueMicrotask(() => {
            if (this.destroyed || this.scanSerial !== scanId) return;
            void this.scanWholeDocument(trigger);
          });
        }
      }
      return true;
    };

    const buildReport = (observations: FidelityPageObservation[]): FidelityScanReport =>
      buildFidelityScanReport({
      scanId,
      trigger,
      identity: {
        documentKey: this.referenceKey,
        documentDigest: this.harness.documentDigest,
        documentGeneration: this.harness.documentGeneration,
        referenceGeneration: this.harness.referenceGeneration,
        renderGeneration,
        pdfName: this.pdfName,
      },
      hwpPageCount,
      pdfPageCount: this.pageCount,
      observations,
      startedAt,
      completedAt: performance.now(),
    });

    try {
      let fallbackTracePage: number | null = null;
      let fallbackTrace: LayoutTraceEntry[] = [];
      let semanticErrorDelivered = false;
      if ((window as FidelityHarnessWindow).rhwpDev?.lineBreakVisible) {
        for (let pageIndex = 0; pageIndex < hwpPageCount; pageIndex++) {
          await this.diagnosticsPause.wait(abortController.signal);
          if (abandonIfStale()) return null;
          const semantic = await this.inspectLineBreak(
            pageIndex,
            abandonIfStale,
            abortController.signal,
          );
          if (semantic.stale) return null;
          if (semantic.line) {
            this.deliverDocumentError(semantic.line);
            semanticErrorDelivered = true;
            break;
          }
          await yieldToInteractiveWork(abortController.signal);
        }
      }

      const observations: FidelityPageObservation[] = [];
      for (let pageIndex = 0; pageIndex < sharedPageCount; pageIndex++) {
        await this.diagnosticsPause.wait(abortController.signal);
        if (abandonIfStale()) return null;
        const capture = await this.harness.capturePage(
          pageIndex,
          WHOLE_SCAN_SAMPLE_WIDTH,
          abortController.signal,
        );
        if (abandonIfStale()) return null;
        const reference = await this.loadReferencePixels(
          pageIndex,
          capture.width,
          capture.height,
          abortController.signal,
        );
        if (abandonIfStale()) return null;
        const observation = this.observePage(pageIndex, capture, reference);
        observations.push(observation);
        if (fallbackTracePage === null) {
          const first = firstDocumentDivergence(buildReport([observation]));
          if (first?.kind === 'structural' && first.pageIndex === pageIndex) {
            fallbackTracePage = pageIndex;
            fallbackTrace = parseLayoutTrace(capture.layoutTrace ?? '[]');
          }
        }
        this.harnessState.completedPages = pageIndex + 1;
        await yieldToInteractiveWork(abortController.signal);
      }
      if (abandonIfStale()) return null;
      const report = buildReport(observations);
      this.harnessState.latestReport = report;
      this.endScan('ready');
      if (!semanticErrorDelivered) {
        const line = this.fallbackDocumentErrorLine(report, fallbackTracePage, fallbackTrace);
        if (line) this.deliverDocumentError(line);
      }
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
    const src = this.pageRasterUrl(pageIndex, width);
    const response = await fetchWithBusyRetry(src, { signal });
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

  private pageRasterUrl(pageIndex: number, width: number): string {
    return `${this.pageImageBaseUrl}/${pageIndex}.png?width=${width}&session=${this.rasterSession}`;
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
      hwpSize: { width: capture.width, height: capture.height },
      referenceSize: { width: reference.width, height: reference.height },
      mismatch: computeReferencePixelDiff(hwpPixels, referencePixels, width, height, DIFF_THRESHOLD),
      hwpHorizontalRules: bands(hwpPixels),
      pdfHorizontalRules: bands(referencePixels),
      hwpInkRows: bands(hwpPixels, true),
      pdfInkRows: bands(referencePixels, true),
    };
  }

  syncPage(request: ReferencePageRenderRequest): void {
    if (this.destroyed) return;
    if (this.pageCount !== null && request.pageIndex >= this.pageCount) {
      this.removePage(request.pageIndex);
      return;
    }
    const src = this.pageRasterUrl(request.pageIndex, this.pixelWidth);
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
