/** clampRenderScale 페이지 상한과 같은 픽셀 단위. 총 surface 예산의 studio 기본값. */
export const PAGE_SURFACE_PIXEL_CAP = 67_108_864;

/** 고정 행 수가 아니라 픽셀 예산으로 eviction한다. RGBA 바이트 = 픽셀 × 4. */
export const TOTAL_SURFACE_PIXEL_BUDGET = PAGE_SURFACE_PIXEL_CAP;

export interface PageSurfaceCacheKey {
  pageIdx: number;
  revision: number | string;
  backend: string;
  renderScaleTier: number;
}

export interface PageSurfaceLruStats {
  hits: number;
  misses: number;
  evictions: number;
  puts: number;
  size: number;
  pixels: number;
  bytes: number;
}

export function quantizeRenderScaleTier(scale: number): number {
  if (!Number.isFinite(scale) || scale <= 0) return 1;
  return Math.round(scale * 1000) / 1000;
}

export function pageSurfaceCacheKey(parts: PageSurfaceCacheKey): string {
  return `${parts.pageIdx}|${parts.revision}|${parts.backend}|${parts.renderScaleTier}`;
}

export function estimateSurfaceBytes(pixels: number): number {
  return Math.max(0, pixels) * 4;
}

interface SurfaceEntry {
  pageIdx: number;
  key: string;
  pixels: number;
}

/**
 * 페이지·revision·backend·renderScale tier를 키로 하는 표면 LRU.
 * Canvas 소유권은 CanvasPool에 두고, 여기선 키·예산·최근 사용만 추적한다.
 */
export class PageSurfaceLru {
  private readonly budgetPixels: number;
  private readonly order: string[] = [];
  private readonly entries = new Map<string, SurfaceEntry>();
  private readonly byPage = new Map<number, string>();
  private hits = 0;
  private misses = 0;
  private evictions = 0;
  private puts = 0;

  constructor(budgetPixels = TOTAL_SURFACE_PIXEL_BUDGET) {
    this.budgetPixels = Math.max(1, budgetPixels);
  }

  get budget(): number {
    return this.budgetPixels;
  }

  has(pageIdx: number, key: string): boolean {
    return this.byPage.get(pageIdx) === key;
  }

  touch(pageIdx: number, key: string): boolean {
    if (!this.has(pageIdx, key)) {
      this.misses += 1;
      return false;
    }
    this.hits += 1;
    this.moveToRecent(key);
    return true;
  }

  put(
    pageIdx: number,
    key: string,
    pixels: number,
    onEvict: (pageIdx: number) => void,
    protect: ReadonlySet<number> = new Set(),
  ): void {
    const safePixels = Math.max(1, pixels);
    const previousKey = this.byPage.get(pageIdx);
    if (previousKey && previousKey !== key) {
      this.removeKey(previousKey);
    }

    const existing = this.entries.get(key);
    if (existing) {
      existing.pixels = safePixels;
      this.moveToRecent(key);
    } else {
      this.entries.set(key, { pageIdx, key, pixels: safePixels });
      this.byPage.set(pageIdx, key);
      this.order.push(key);
      this.puts += 1;
    }

    const keep = new Set(protect);
    keep.add(pageIdx);
    this.evictToBudget(onEvict, keep);
  }

  remove(pageIdx: number): void {
    const key = this.byPage.get(pageIdx);
    if (key) this.removeKey(key);
  }

  evictToBudget(onEvict: (pageIdx: number) => void, protect: ReadonlySet<number>): void {
    let index = 0;
    while (this.pixelCount() > this.budgetPixels && index < this.order.length) {
      const key = this.order[index];
      const entry = this.entries.get(key);
      if (!entry || protect.has(entry.pageIdx)) {
        index += 1;
        continue;
      }
      this.removeKey(key);
      this.evictions += 1;
      onEvict(entry.pageIdx);
    }
  }

  clear(): void {
    this.order.length = 0;
    this.entries.clear();
    this.byPage.clear();
  }

  stats(): PageSurfaceLruStats {
    const pixels = this.pixelCount();
    return {
      hits: this.hits,
      misses: this.misses,
      evictions: this.evictions,
      puts: this.puts,
      size: this.entries.size,
      pixels,
      bytes: estimateSurfaceBytes(pixels),
    };
  }

  private pixelCount(): number {
    let total = 0;
    for (const entry of this.entries.values()) total += entry.pixels;
    return total;
  }

  private moveToRecent(key: string): void {
    const index = this.order.indexOf(key);
    if (index < 0) {
      this.order.push(key);
      return;
    }
    this.order.splice(index, 1);
    this.order.push(key);
  }

  private removeKey(key: string): void {
    const entry = this.entries.get(key);
    if (!entry) return;
    this.entries.delete(key);
    if (this.byPage.get(entry.pageIdx) === key) this.byPage.delete(entry.pageIdx);
    const index = this.order.indexOf(key);
    if (index >= 0) this.order.splice(index, 1);
  }
}
