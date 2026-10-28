import { CanvasSupplementalMetricProvider } from './supplemental-text-metrics.ts';
import type { SupplementalMetricGeneration, SupplementalMetricRequest } from './supplemental-text-metrics.ts';

export interface CanvasMetricDocument {
  collectCanvasMetricRequests(document: number, fonts: number): string;
  registerCanvasMetricReplies(document: number, fonts: number, revision: number, json: string): boolean;
  selectCanvasMetrics(enabled: boolean): boolean;
  canvasMetricsActive(): boolean;
  getCanvasPageLayerTree(page: number, profile: string, omitFontBytes: boolean): string;
}

/** Async preparation belongs to a view revision, not an individual paint call. */
export class CanvasMetricSession {
  private provider = new CanvasSupplementalMetricProvider();
  private epoch = 0;
  lastError: string | null = null;

  invalidate(): void { this.epoch += 1; this.provider.dispose(); }

  async prepare(
    doc: CanvasMetricDocument,
    generation: SupplementalMetricGeneration,
    backend: string,
    isCurrent: () => boolean,
    fontsReady: () => Promise<unknown>,
    createContext: () => Pick<CanvasRenderingContext2D, 'font' | 'measureText'> | null,
  ): Promise<boolean> {
    const epoch = ++this.epoch;
    const stale = (): boolean => epoch !== this.epoch || !isCurrent();
    if (backend !== 'canvas2d') return doc.selectCanvasMetrics(false);
    this.provider.reset(generation);
    try {
      await fontsReady();
      if (stale()) return false;
      const batch = JSON.parse(doc.collectCanvasMetricRequests(generation.document, generation.fonts)) as {
        revision: number; requests: SupplementalMetricRequest[];
      };
      const result = await this.provider.prepare(generation, batch.requests, fontsReady, createContext);
      if (stale() || result.status === 'stale') return false;
      const replies = result.metrics.map(({ key, cluster, font, resolvedFont, measuredAdvancePx }) =>
        ({ key, cluster, font, resolvedFont, measuredAdvancePx }));
      const registered = doc.registerCanvasMetricReplies(generation.document, generation.fonts, batch.revision, JSON.stringify(replies));
      const selected = doc.selectCanvasMetrics(replies.length > 0);
      this.lastError = null;
      return registered || selected;
    } catch (error) {
      if (stale()) return false;
      this.lastError = String(error);
      const changed = doc.selectCanvasMetrics(false);
      console.warn('[Canvas metrics] portable fallback:', error);
      return changed;
    }
  }
}

/** A synchronous export transaction restores the view even if export fails.
 * Do not yield while portable pagination is selected. */
export function withPortableMetrics<T>(doc: CanvasMetricDocument | null, operation: () => T): T {
  const restore = doc?.canvasMetricsActive() ?? false;
  if (restore) doc!.selectCanvasMetrics(false);
  try { return operation(); }
  finally { if (restore) doc!.selectCanvasMetrics(true); }
}
