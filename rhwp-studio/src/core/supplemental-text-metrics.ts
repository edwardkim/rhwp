/** Canvas2D preparation only. A result does not identify an exact fallback face,
 * nor authorize another backend to consume the advance. No document writes. */
export interface SupplementalMetricRequest {
  /** Owner-issued key including style, size and cluster identity. */
  key: string;
  cluster: string;
  /** Final paint font input; the normal Studio substitution setter still runs. */
  font: string;
}

export interface CanvasSupplementalMetric extends SupplementalMetricRequest {
  backend: 'canvas2d';
  evidence: 'backendMeasured';
  /** Read back from the context after the normal substitution setter. */
  resolvedFont: string;
  /** Unscaled measureText width at `font`'s actual CSS size, not the Rust
   * pre-document-ratio advance. Rust's shared paint setup performs conversion. */
  measuredAdvancePx: number;
}

export interface SupplementalMetricGeneration {
  document: number;
  fonts: number;
}

type MeasurementContext = Pick<CanvasRenderingContext2D, 'font' | 'measureText'>;

export type SupplementalMetricPreparation =
  | { status: 'ready'; generation: SupplementalMetricGeneration;
      metrics: CanvasSupplementalMetric[]; measured: number; cacheHits: number }
  | { status: 'stale'; generation: SupplementalMetricGeneration };

const MAX_ENTRIES = 4096;
const MAX_KEY_BYTES = 4 * 1024 * 1024;
const utf8 = new TextEncoder();

function generationKey(value: SupplementalMetricGeneration): string {
  if (![value.document, value.fonts].every(v => Number.isSafeInteger(v) && v >= 0)) {
    throw new Error('Invalid supplemental metric generation');
  }
  return `${value.document}:${value.fonts}`;
}

function requestKey(request: SupplementalMetricRequest): string {
  return JSON.stringify([request.key, request.cluster, request.font]);
}

function requestBytes(request: SupplementalMetricRequest): number {
  if (request.key.length + request.cluster.length + request.font.length > MAX_KEY_BYTES) {
    throw new Error('Supplemental metric key limit exceeded');
  }
  return utf8.encode(request.key).length + utf8.encode(request.cluster).length
    + utf8.encode(request.font).length;
}

function validateRequests(requests: readonly SupplementalMetricRequest[]): number {
  if (requests.length > MAX_ENTRIES) throw new Error('Supplemental metric entry limit exceeded');
  const keys = new Set<string>();
  let bytes = 0;
  for (const request of requests) {
    if (!request.key || !request.font || /[\u0000-\u001f\u007f]/u.test(request.font)) {
      throw new Error('Invalid supplemental metric request');
    }
    // This matches the A publication boundary: do not measure part of a ZWJ/VS
    // sequence and advertise it as a complete cluster.
    const scalars = [...request.cluster];
    const codepoint = request.cluster.codePointAt(0);
    if (scalars.length !== 1 || codepoint === undefined || codepoint < 32
      || (codepoint >= 127 && codepoint <= 159)
      || (codepoint >= 0xd800 && codepoint <= 0xdfff)) {
      throw new Error('Unsupported supplemental metric cluster');
    }
    if (keys.has(request.key)) throw new Error('Duplicate supplemental metric key');
    keys.add(request.key);
    bytes += requestBytes(request);
    if (bytes > MAX_KEY_BYTES) throw new Error('Supplemental metric key limit exceeded');
  }
  return bytes;
}

/** Per-document owner. Call reset when document OR font resources change, even
 * while fonts.ready is pending. Reject a late result instead of publishing it. */
export class CanvasSupplementalMetricProvider {
  private epoch = 0;
  private current = '';
  private cache = new Map<string, CanvasSupplementalMetric>();

  reset(generation: SupplementalMetricGeneration): void {
    const key = generationKey(generation);
    if (key === this.current) return;
    this.epoch += 1;
    this.current = key;
    this.cache.clear();
  }

  dispose(): void {
    this.epoch += 1;
    this.current = '';
    this.cache.clear();
  }

  async prepare(
    generation: SupplementalMetricGeneration,
    requests: readonly SupplementalMetricRequest[],
    fontsReady: () => Promise<unknown>,
    createContext: () => MeasurementContext | null,
  ): Promise<SupplementalMetricPreparation> {
    // Copy before await; an owner mutating its request array cannot change a batch.
    if (requests.length > MAX_ENTRIES) throw new Error('Supplemental metric entry limit exceeded');
    const batch = requests.map(request => ({ ...request }));
    validateRequests(batch);
    const captured = { ...generation };
    const key = generationKey(captured);
    const epoch = this.epoch;
    const stale = (): boolean => this.epoch !== epoch || this.current !== key;
    if (stale()) return { status: 'stale', generation: captured };
    await fontsReady();
    if (stale()) return { status: 'stale', generation: captured };

    const next = new Map(this.cache);
    const metrics: CanvasSupplementalMetric[] = [];
    let measured = 0;
    let cacheHits = 0;
    let context: MeasurementContext | null = null;
    let bytes = [...next.values()].reduce((sum, entry) =>
      sum + requestBytes(entry) + utf8.encode(entry.resolvedFont).length, 0);
    for (const request of batch) {
      const cacheKey = requestKey(request);
      const cached = next.get(cacheKey);
      if (cached) {
        metrics.push({ ...cached });
        cacheHits += 1;
        continue;
      }
      if (next.size >= MAX_ENTRIES) throw new Error('Supplemental metric cache limit exceeded');
      context ??= createContext();
      if (!context) throw new Error('Canvas2D measurement context unavailable');
      // Canvas ignores invalid font assignments. Two distinct reset values
      // distinguish a valid canonical descriptor from retained previous state.
      context.font = '10px sans-serif';
      context.font = request.font;
      const resolvedFont = context.font;
      if (resolvedFont.length > MAX_KEY_BYTES) {
        throw new Error('Supplemental metric resolved font limit exceeded');
      }
      context.font = '20px monospace';
      context.font = request.font;
      if (!resolvedFont || context.font !== resolvedFont) {
        throw new Error('Canvas rejected supplemental metric font descriptor');
      }
      const measuredAdvancePx = context.measureText(request.cluster).width;
      if (!Number.isFinite(measuredAdvancePx) || measuredAdvancePx < 0) {
        throw new Error('Invalid Canvas2D supplemental advance');
      }
      const entry: CanvasSupplementalMetric = {
        ...request, resolvedFont, measuredAdvancePx,
        backend: 'canvas2d', evidence: 'backendMeasured',
      };
      bytes += requestBytes(entry) + utf8.encode(resolvedFont).length;
      if (bytes > MAX_KEY_BYTES) throw new Error('Supplemental metric cache key limit exceeded');
      next.set(cacheKey, entry);
      metrics.push({ ...entry });
      measured += 1;
    }
    if (stale()) return { status: 'stale', generation: captured };
    this.cache = next; // failure above never partially publishes a batch
    return { status: 'ready', generation: captured, metrics, measured, cacheHits };
  }
}
