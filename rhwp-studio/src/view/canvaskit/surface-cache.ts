import type { Surface } from 'canvaskit-wasm';

export interface CanvasKitSurfaceTarget {
  surface: Surface;
  canvas: HTMLCanvasElement;
  backend: 'default' | 'software' | 'webgpu';
  fallbackReason: string | null;
  releaseContext?: () => void;
}

/** One live surface is sufficient for warm page replay and bounds native GPU ownership. */
export class CanvasKitSurfaceCache {
  private target: CanvasKitSurfaceTarget | null = null;
  private width = 0;
  private height = 0;
  created = 0;
  reused = 0;

  get(canvas: HTMLCanvasElement, create: () => CanvasKitSurfaceTarget): CanvasKitSurfaceTarget {
    // WebGPU swapchain textures expire between frames; only WebGL/SW surfaces are reusable.
    if (this.target?.backend !== 'webgpu' && this.target?.canvas === canvas
      && this.width === canvas.width && this.height === canvas.height) {
      this.reused++;
      return this.target;
    }
    this.clear();
    const target = create();
    this.target = target;
    this.width = target.canvas.width;
    this.height = target.canvas.height;
    this.created++;
    return target;
  }

  clear(): void {
    const target = this.target;
    this.target = null;
    this.width = 0;
    this.height = 0;
    try { target?.surface.delete(); } finally { target?.releaseContext?.(); }
  }
}
