import type { LayerRenderProfile, PageInfo } from '@/core/types';

export type RenderBackend = 'canvas2d' | 'canvaskit';
export type CanvasKitRenderMode = 'default' | 'compat';
export type CanvasKitSurfacePreference = 'auto' | 'webgpu' | 'webgl' | 'software';
export type CanvasKitSurfaceUnsupportedReason = 'unsupportedSurfaceBackend';
export type RenderBackendUnsupportedReason = 'unsupportedRenderBackend';

export interface CanvasKitSurfaceRequest {
  preference: CanvasKitSurfacePreference;
  requested: string;
  unsupportedReason?: CanvasKitSurfaceUnsupportedReason;
}

export interface RenderBackendRequest {
  backend: RenderBackend;
  requested?: string;
  unsupportedReason?: RenderBackendUnsupportedReason;
}

export type { LayerRenderProfile } from '@/core/types';

export const DEFAULT_CANVASKIT_SURFACE_REQUEST: CanvasKitSurfaceRequest = {
  preference: 'auto',
  requested: 'auto',
};

const BACKEND_STORAGE_KEY = 'rhwp.renderBackend';
const CANVASKIT_MODE_STORAGE_KEY = 'rhwp.canvaskitMode';
const RENDER_PROFILE_STORAGE_KEY = 'rhwp.renderProfile';

function readStorage(key: string): string | null {
  try {
    return globalThis.localStorage?.getItem(key) ?? null;
  } catch {
    return null;
  }
}

function writeStorage(key: string, value: string): void {
  try {
    globalThis.localStorage?.setItem(key, value);
  } catch {
    /* storage can be unavailable in private contexts */
  }
}

function searchParam(search: string, ...keys: string[]): string | null {
  const params = new URLSearchParams(search);
  for (const key of keys) {
    const value = params.get(key);
    if (value !== null) return value;
  }
  return null;
}

export function resolveRenderBackend(search = ''): RenderBackend {
  return resolveRenderBackendRequest(search).backend;
}

export function resolveRenderBackendRequest(search = ''): RenderBackendRequest {
  const explicit = searchParam(search, 'renderer', 'renderBackend', 'backend');
  const normalized = explicit?.trim().toLowerCase();
  if (!normalized) return { backend: 'canvas2d' };
  if (normalized === 'canvaskit' || normalized === 'skia') {
    return { backend: 'canvaskit', requested: normalized };
  }
  if (normalized === 'canvas' || normalized === 'canvas2d' || normalized === 'legacy') {
    return { backend: 'canvas2d', requested: normalized };
  }
  return {
    backend: 'canvas2d',
    requested: explicit ?? normalized,
    unsupportedReason: 'unsupportedRenderBackend',
  };
}

export function persistRenderBackend(value: RenderBackend): void {
  writeStorage(BACKEND_STORAGE_KEY, value);
}

export function resolveCanvasKitRenderMode(search = ''): CanvasKitRenderMode {
  const explicit = searchParam(search, 'canvaskitMode', 'skiaMode') ?? readStorage(CANVASKIT_MODE_STORAGE_KEY);
  const normalized = explicit?.trim().toLowerCase();
  if (normalized === 'compat' || normalized === 'compatibility') return 'compat';
  return 'default';
}

export function persistCanvasKitRenderMode(value: CanvasKitRenderMode): void {
  writeStorage(CANVASKIT_MODE_STORAGE_KEY, value);
}

export function resolveCanvasKitSurfaceRequest(search = ''): CanvasKitSurfaceRequest {
  const requested = searchParam(search, 'canvaskitSurface', 'skiaSurface')?.trim().toLowerCase() ?? 'auto';
  if (requested === 'auto' || requested === 'webgpu' || requested === 'webgl' || requested === 'software') {
    return { preference: requested, requested };
  }
  if (requested === 'gpu') return { preference: 'webgl', requested };
  if (requested === 'sw' || requested === 'cpu') return { preference: 'software', requested };
  return {
    preference: 'auto',
    requested,
    unsupportedReason: 'unsupportedSurfaceBackend',
  };
}

export function resolveRenderProfile(search = ''): LayerRenderProfile {
  const explicit = searchParam(search, 'renderProfile', 'profile') ?? readStorage(RENDER_PROFILE_STORAGE_KEY);
  const normalized = explicit?.trim().toLowerCase();
  if (normalized === 'fast' || normalized === 'fast-preview' || normalized === 'fastpreview') return 'fastPreview';
  if (normalized === 'print') return 'print';
  if (normalized === 'high' || normalized === 'high-quality' || normalized === 'highquality') return 'highQuality';
  return 'screen';
}

export function persistRenderProfile(value: LayerRenderProfile): void {
  writeStorage(RENDER_PROFILE_STORAGE_KEY, value);
}

export function clampRenderScale(pageInfo: PageInfo, requestedScale: number): number {
  const scale = Number.isFinite(requestedScale) && requestedScale > 0 ? requestedScale : 1;
  const maxPixels = 67_108_864;
  const pixels = pageInfo.width * scale * pageInfo.height * scale;
  if (pixels <= maxPixels) return scale;
  return Math.max(1, Math.sqrt(maxPixels / (pageInfo.width * pageInfo.height)));
}
