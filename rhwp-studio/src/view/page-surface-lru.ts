import { DEFAULT_RETAINED_SURFACE_PIXEL_BUDGET } from './render-surface-budget.ts';

export interface PageSurfaceCacheEntry {
  key: string;
  lookupKey: string;
  pageIndex: number;
  pixelCount: number;
}

export interface PageSurfaceLruSnapshot {
  pixelBudget: number;
  reservedPixels: number;
  cachedPixels: number;
  totalAccountedPixels: number;
  overBudgetMandatory: boolean;
  entryCount: number;
  hits: number;
  misses: number;
  evictions: number;
  rejected: number;
  invalidations: number;
}

function safePixels(value: number): number {
  if (!Number.isFinite(value) || value <= 0) return 0;
  return Math.ceil(value);
}

/**
 * DOM과 무관한 page surface LRU 원장.
 *
 * `reservedPixels`는 현재 active surface와 아직 만들지 않은 승인 작업의 합이다. 캐시는 전체 retained
 * 예산에서 그 예약을 뺀 headroom만 사용한다. mandatory 예약만으로 예산을 넘으면 캐시를 0으로 만들되
 * 화질 정책은 바꾸지 않는다.
 */
export class PageSurfaceLru<T extends PageSurfaceCacheEntry> {
  private readonly entries = new Map<string, T>();
  private readonly lookupKeys = new Map<string, string>();
  private readonly disposeEntry: (entry: T) => void;
  private pixelBudget: number;
  private reservedPixels = 0;
  private cachedPixels = 0;
  private hits = 0;
  private misses = 0;
  private evictions = 0;
  private rejected = 0;
  private invalidations = 0;

  constructor(
    disposeEntry: (entry: T) => void,
    pixelBudget = DEFAULT_RETAINED_SURFACE_PIXEL_BUDGET,
  ) {
    this.disposeEntry = disposeEntry;
    this.pixelBudget = safePixels(pixelBudget) || DEFAULT_RETAINED_SURFACE_PIXEL_BUDGET;
  }

  has(key: string): boolean {
    return this.entries.has(key);
  }

  hasLookup(lookupKey: string): boolean {
    return this.lookupKeys.has(lookupKey);
  }

  take(key: string): T | null {
    const entry = this.entries.get(key);
    if (!entry) {
      this.misses += 1;
      return null;
    }
    this.entries.delete(key);
    this.lookupKeys.delete(entry.lookupKey);
    this.cachedPixels -= safePixels(entry.pixelCount);
    this.hits += 1;
    return entry;
  }

  takeLookup(lookupKey: string): T | null {
    const key = this.lookupKeys.get(lookupKey);
    if (!key) {
      this.misses += 1;
      return null;
    }
    return this.takeKnown(key);
  }

  put(entry: T): boolean {
    const pixels = safePixels(entry.pixelCount);
    if (pixels === 0) {
      this.rejected += 1;
      this.disposeEntry(entry);
      return false;
    }

    const previous = this.entries.get(entry.key);
    if (previous) {
      this.entries.delete(entry.key);
      this.lookupKeys.delete(previous.lookupKey);
      this.cachedPixels -= safePixels(previous.pixelCount);
      this.disposeEntry(previous);
    }
    const previousLookupKey = this.lookupKeys.get(entry.lookupKey);
    if (previousLookupKey && previousLookupKey !== entry.key) {
      const previousLookup = this.entries.get(previousLookupKey);
      if (previousLookup) {
        this.entries.delete(previousLookupKey);
        this.cachedPixels -= safePixels(previousLookup.pixelCount);
        this.disposeEntry(previousLookup);
      }
    }
    this.entries.set(entry.key, entry);
    this.lookupKeys.set(entry.lookupKey, entry.key);
    this.cachedPixels += pixels;
    this.trim();
    return this.entries.get(entry.key) === entry;
  }

  deletePage(pageIndex: number): void {
    for (const [key, entry] of this.entries) {
      if (entry.pageIndex !== pageIndex) continue;
      this.entries.delete(key);
      this.lookupKeys.delete(entry.lookupKey);
      this.cachedPixels -= safePixels(entry.pixelCount);
      this.invalidations += 1;
      this.disposeEntry(entry);
    }
  }

  reconcile(pixelBudget: number, reservedPixels: number): void {
    this.pixelBudget = safePixels(pixelBudget) || DEFAULT_RETAINED_SURFACE_PIXEL_BUDGET;
    this.reservedPixels = safePixels(reservedPixels);
    this.trim();
  }

  clear(): void {
    for (const entry of this.entries.values()) this.disposeEntry(entry);
    this.invalidations += this.entries.size;
    this.entries.clear();
    this.lookupKeys.clear();
    this.cachedPixels = 0;
    this.reservedPixels = 0;
  }

  snapshot(): PageSurfaceLruSnapshot {
    return {
      pixelBudget: this.pixelBudget,
      reservedPixels: this.reservedPixels,
      cachedPixels: this.cachedPixels,
      totalAccountedPixels: this.reservedPixels + this.cachedPixels,
      overBudgetMandatory: this.reservedPixels > this.pixelBudget,
      entryCount: this.entries.size,
      hits: this.hits,
      misses: this.misses,
      evictions: this.evictions,
      rejected: this.rejected,
      invalidations: this.invalidations,
    };
  }

  private trim(): void {
    const headroom = Math.max(0, this.pixelBudget - this.reservedPixels);
    while (this.cachedPixels > headroom) {
      const oldest = this.entries.entries().next().value as [string, T] | undefined;
      if (!oldest) break;
      const [key, entry] = oldest;
      this.entries.delete(key);
      this.lookupKeys.delete(entry.lookupKey);
      this.cachedPixels -= safePixels(entry.pixelCount);
      this.evictions += 1;
      this.disposeEntry(entry);
    }
  }

  private takeKnown(key: string): T | null {
    const entry = this.entries.get(key);
    if (!entry) {
      this.misses += 1;
      return null;
    }
    this.entries.delete(key);
    this.lookupKeys.delete(entry.lookupKey);
    this.cachedPixels -= safePixels(entry.pixelCount);
    this.hits += 1;
    return entry;
  }
}
