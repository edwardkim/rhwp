import type { PageInfo } from '@/core/types';

export interface ReferencePageRenderRequest {
  pageIndex: number;
  pageInfo: PageInfo;
  sourceCanvas: HTMLCanvasElement;
  zoom: number;
  dpr: number;
}

export interface DiagnosticPageCapture {
  width: number;
  height: number;
  pixels: Uint8ClampedArray;
}

export async function waitForFontsOrAbort(
  ready: PromiseLike<unknown>,
  signal?: AbortSignal,
): Promise<void> {
  if (!signal) {
    await ready;
    return;
  }
  if (signal.aborted) return;
  let onAbort!: () => void;
  const aborted = new Promise<void>(resolve => {
    onAbort = resolve;
    signal.addEventListener('abort', onAbort, { once: true });
  });
  try {
    await Promise.race([Promise.resolve(ready).then(() => {}), aborted]);
  } finally {
    signal.removeEventListener('abort', onAbort);
  }
}

export function nextDiagnosticRenderGeneration(
  current: number,
  decisionChanged: boolean,
  effectiveRendererChanged: boolean,
): number {
  return current + (decisionChanged || effectiveRendererChanged ? 1 : 0);
}

/** 개발용 정답지처럼 페이지와 같은 좌표계에 놓이는 외부 기준층. */
export interface PageReferenceLayer {
  syncPage(request: ReferencePageRenderRequest): void;
  setDiagnosticsPaused(paused: boolean): void;
  removePage(pageIndex: number): void;
  retainPages(pageIndices: readonly number[]): void;
  clearMountedPages(): void;
}
