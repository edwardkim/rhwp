export class CanvasPool {
  private available: HTMLCanvasElement[] = [];
  private inUse = new Map<number, HTMLCanvasElement>();
  private readonly maxAvailable: number;

  constructor(maxAvailable = 4) {
    this.maxAvailable = maxAvailable;
  }

  /** Canvas를 할당한다 (풀에서 꺼내거나 새로 생성) */
  acquire(pageIdx: number): HTMLCanvasElement {
    if (this.inUse.has(pageIdx)) {
      throw new Error(`페이지 ${pageIdx} Canvas가 이미 할당되어 있습니다`);
    }
    let canvas = this.available.pop();
    if (!canvas) {
      canvas = document.createElement('canvas');
    }
    canvas.classList.add('document-page-canvas');
    this.inUse.set(pageIdx, canvas);
    return canvas;
  }

  /** detached LRU가 소유하던 Canvas를 같은 페이지의 active 소유권으로 되돌린다. */
  adopt(pageIdx: number, canvas: HTMLCanvasElement): void {
    if (this.inUse.has(pageIdx)) {
      throw new Error(`페이지 ${pageIdx} Canvas가 이미 할당되어 있습니다`);
    }
    const availableIndex = this.available.indexOf(canvas);
    if (availableIndex >= 0) {
      throw new Error(`페이지 ${pageIdx} Canvas가 익명 pool과 LRU에 동시에 들어 있습니다`);
    }
    canvas.classList.add('document-page-canvas');
    this.inUse.set(pageIdx, canvas);
  }

  /** CanvasKit이 software fallback canvas로 교체한 경우 pool 소유권을 넘긴다. */
  replace(pageIdx: number, current: HTMLCanvasElement, replacement: HTMLCanvasElement): void {
    if (this.inUse.get(pageIdx) !== current) {
      throw new Error(`페이지 ${pageIdx} Canvas 교체 대상이 현재 pool 항목과 다릅니다`);
    }
    this.inUse.set(pageIdx, replacement);
  }

  /** Canvas를 반환한다 (DOM에서 제거 후 풀에 반환) */
  release(pageIdx: number): void {
    const canvas = this.detach(pageIdx);
    if (canvas) this.releaseDetached(canvas);
  }

  /** active 소유권만 해제한다. 반환값의 새 소유권은 호출자가 명시해야 한다. */
  detach(pageIdx: number): HTMLCanvasElement | null {
    const canvas = this.inUse.get(pageIdx);
    if (!canvas) return null;
    canvas.parentElement?.removeChild(canvas);
    this.inUse.delete(pageIdx);
    return canvas;
  }

  /** LRU 퇴거 Canvas의 backing store를 비운 뒤 제한된 익명 pool로 돌린다. */
  releaseDetached(canvas: HTMLCanvasElement): void {
    canvas.parentElement?.removeChild(canvas);
    canvas.width = 0;
    canvas.height = 0;
    canvas.removeAttribute?.('style');
    for (const key of Object.keys(canvas.dataset)) delete canvas.dataset[key];
    if (this.available.length < Math.max(0, this.maxAvailable)) this.available.push(canvas);
  }

  /** 특정 페이지에 할당된 Canvas를 조회한다 */
  getCanvas(pageIdx: number): HTMLCanvasElement | undefined {
    return this.inUse.get(pageIdx);
  }

  /** 특정 페이지가 이미 할당되어 있는지 확인한다 */
  has(pageIdx: number): boolean {
    return this.inUse.has(pageIdx);
  }

  /** 모든 Canvas를 반환한다 */
  releaseAll(): void {
    const pages = Array.from(this.inUse.keys());
    for (const pageIdx of pages) {
      this.release(pageIdx);
    }
  }

  /** 현재 사용 중인 페이지 인덱스 목록 */
  get activePages(): number[] {
    return Array.from(this.inUse.keys());
  }

  /** 사용 중 + 풀 대기 Canvas 총 수 */
  get totalCount(): number {
    return this.inUse.size + this.available.length;
  }
}
