/** DEV only: observed ownership-boundary high-water mark, not allocator/GPU/RSS peak. */
export interface SurfaceMemorySample {
  at: number;
  boundary: string;
  zoom: number;
  activePixels: number;
  idlePixels: number;
  cachedPixels: number;
  active: { page: number; visible: boolean; focused: boolean; pixels: number }[];
}

export class SurfaceMemoryObservation {
  private count = 0;
  private peak: (SurfaceMemorySample & { totalPixels: number }) | null = null;
  private last: (SurfaceMemorySample & { totalPixels: number }) | null = null;

  observe(sample: SurfaceMemorySample): void {
    const totalPixels = sample.activePixels + sample.idlePixels + sample.cachedPixels;
    if (![sample.activePixels, sample.idlePixels, sample.cachedPixels].every(n => Number.isFinite(n) && n >= 0)) return;
    this.count++;
    this.last = { ...sample, active: sample.active.map(p => ({ ...p })), totalPixels };
    if (!this.peak || totalPixels > this.peak.totalPixels) this.peak = this.last;
  }

  clear(): void { this.count = 0; this.peak = null; this.last = null; }

  snapshot() {
    return structuredClone({
      kind: 'ownership-boundary-samples-v1', count: this.count, peak: this.peak, last: this.last,
      note: 'Observed lower bound on peak owned surface pixels, not GPU/RSS. Misses temporary native/WASM surfaces and transitions inside a boundary. DOM scans affect timings; do not use memory-on runs for performance comparison.',
    });
  }
}
