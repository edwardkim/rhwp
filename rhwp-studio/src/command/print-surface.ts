const DEFAULT_PRINT_SURFACE_PATH = 'print.html';
const DEFAULT_PRINT_SURFACE_TIMEOUT_MS = 10_000;
const PRINT_FRAME_ID = 'rhwp-print-surface';

export interface PrintDocumentSurface {
  readonly window: Window;
  readonly document: Document;
}

export interface PrintSurface extends PrintDocumentSurface {
  readonly frame: HTMLIFrameElement;
  dispose(): void;
}

export interface PrintPreviewSurface extends PrintDocumentSurface {
  close(): void;
}

export interface PrintSurfaceOptions {
  hostDocument?: Document;
  surfacePath?: string;
  timeoutMs?: number;
}

export interface PrintPreviewSurfaceOptions {
  hostWindow?: Window;
  surfacePath?: string;
  timeoutMs?: number;
}

export class PrintPreviewBlockedError extends Error {
  constructor() {
    super('인쇄 미리보기 팝업이 차단되었습니다.');
    this.name = 'PrintPreviewBlockedError';
  }
}

export function resolvePrintSurfaceUrl(
  baseUrl: string,
  surfacePath = DEFAULT_PRINT_SURFACE_PATH,
): string {
  return new URL(surfacePath, baseUrl).href;
}

export async function createPrintSurface(
  options: PrintSurfaceOptions = {},
): Promise<PrintSurface> {
  const hostDocument = options.hostDocument ?? document;
  const hostWindow = hostDocument.defaultView;
  if (!hostWindow || !hostDocument.body) {
    throw new Error('인쇄 surface를 만들 수 없습니다.');
  }

  hostDocument.getElementById(PRINT_FRAME_ID)?.remove();

  const frame = hostDocument.createElement('iframe');
  frame.id = PRINT_FRAME_ID;
  frame.title = '인쇄 문서';
  frame.setAttribute('aria-hidden', 'true');
  frame.style.position = 'fixed';
  frame.style.inset = '0';
  frame.style.width = '100vw';
  frame.style.height = '100vh';
  frame.style.border = '0';
  frame.style.opacity = '0';
  frame.style.pointerEvents = 'none';
  frame.style.zIndex = '-1';

  const surfaceUrl = resolvePrintSurfaceUrl(
    hostDocument.baseURI,
    options.surfacePath ?? DEFAULT_PRINT_SURFACE_PATH,
  );
  const timeoutMs = options.timeoutMs ?? DEFAULT_PRINT_SURFACE_TIMEOUT_MS;

  await new Promise<void>((resolve, reject) => {
    let settled = false;
    const finish = (error?: Error) => {
      if (settled) return;
      settled = true;
      hostWindow.clearTimeout(timeoutId);
      frame.removeEventListener('load', onLoad);
      frame.removeEventListener('error', onError);
      if (error) reject(error);
      else resolve();
    };
    const onLoad = () => finish();
    const onError = () => finish(new Error('인쇄 surface를 불러오지 못했습니다.'));
    const timeoutId = hostWindow.setTimeout(
      () => finish(new Error('인쇄 surface 준비 시간이 초과되었습니다.')),
      timeoutMs,
    );

    frame.addEventListener('load', onLoad);
    frame.addEventListener('error', onError);
    frame.src = surfaceUrl;
    hostDocument.body.appendChild(frame);
  }).catch((error) => {
    frame.remove();
    throw error;
  });

  const printWindow = frame.contentWindow;
  const printDocument = frame.contentDocument;
  if (!printWindow || !printDocument) {
    frame.remove();
    throw new Error('same-origin 인쇄 surface에 접근할 수 없습니다.');
  }
  if (printWindow.location.origin !== hostWindow.location.origin) {
    frame.remove();
    throw new Error('인쇄 surface의 origin이 Studio와 다릅니다.');
  }

  let disposed = false;
  return {
    frame,
    window: printWindow,
    document: printDocument,
    dispose() {
      if (disposed) return;
      disposed = true;
      frame.remove();
    },
  };
}

/**
 * 사용자 클릭의 동기 구간에서 same-origin 인쇄 미리보기 창을 먼저 확보한다.
 * 함수가 Promise를 반환하지만 window.open 자체는 첫 await 전에 즉시 실행된다.
 */
export function createPrintPreviewSurface(
  options: PrintPreviewSurfaceOptions = {},
): Promise<PrintPreviewSurface> {
  const hostWindow = options.hostWindow ?? window;
  const surfaceUrl = resolvePrintSurfaceUrl(
    hostWindow.document.baseURI,
    options.surfacePath ?? DEFAULT_PRINT_SURFACE_PATH,
  );
  const timeoutMs = options.timeoutMs ?? DEFAULT_PRINT_SURFACE_TIMEOUT_MS;
  const previewWindow = hostWindow.open(surfaceUrl, '_blank');
  if (!previewWindow) {
    return Promise.reject(new PrintPreviewBlockedError());
  }

  return new Promise<PrintPreviewSurface>((resolve, reject) => {
    let settled = false;
    const finish = (error?: Error) => {
      if (settled) return;
      settled = true;
      hostWindow.clearTimeout(timeoutId);
      previewWindow.removeEventListener('load', onLoad);

      if (error) {
        previewWindow.close();
        reject(error);
        return;
      }

      try {
        const previewDocument = previewWindow.document;
        if (previewWindow.location.origin !== hostWindow.location.origin) {
          throw new Error('인쇄 미리보기 창의 origin이 Studio와 다릅니다.');
        }
        resolve({
          window: previewWindow,
          document: previewDocument,
          close() {
            if (!previewWindow.closed) previewWindow.close();
          },
        });
      } catch (error) {
        previewWindow.close();
        reject(error);
      }
    };
    const onLoad = () => finish();
    const timeoutId = hostWindow.setTimeout(
      () => finish(new Error('인쇄 미리보기 준비 시간이 초과되었습니다.')),
      timeoutMs,
    );

    previewWindow.addEventListener('load', onLoad);
    if (
      previewWindow.location.href === surfaceUrl
      && previewWindow.document.readyState === 'complete'
    ) {
      hostWindow.queueMicrotask(onLoad);
    }
  });
}

function waitForAnimationFrame(windowLike: Window): Promise<void> {
  return new Promise((resolve) => {
    windowLike.requestAnimationFrame(() => resolve());
  });
}

export async function waitForPrintSurfaceReady(surface: PrintDocumentSurface): Promise<void> {
  const fontSet = surface.document.fonts;
  if (fontSet) {
    await fontSet.ready;
  }

  await waitForAnimationFrame(surface.window);
  await waitForAnimationFrame(surface.window);

  // 인쇄 호출 직전에 style/layout 계산을 완료시킨다.
  void surface.document.documentElement.getBoundingClientRect();
}
