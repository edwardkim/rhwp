export type RenderBackend = 'canvas2d' | 'canvaskit';

const STORAGE_KEY = 'rhwp-render-backend';

export function resolveRenderBackend(search: string): RenderBackend {
  const params = new URLSearchParams(search);
  const requested = params.get('renderer');

  if (requested === 'canvaskit') return 'canvaskit';
  if (requested === 'canvas' || requested === 'canvas2d') return 'canvas2d';

  try {
    return window.localStorage.getItem(STORAGE_KEY) === 'canvaskit' ? 'canvaskit' : 'canvas2d';
  } catch {
    return 'canvas2d';
  }
}

export function persistRenderBackend(backend: RenderBackend): void {
  try {
    window.localStorage.setItem(STORAGE_KEY, backend);
  } catch {
    // private mode / disabled storage: 무시하고 query-param 선택만 사용한다.
  }
}
