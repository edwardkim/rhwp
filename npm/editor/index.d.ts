/**
 * @rhwp/editor — HWP 에디터 웹 컴포넌트
 */

export interface EditorOptions {
  /** rhwp-studio HTTP(S) URL. file:, data:, browser extension 등 opaque origin은 지원하지 않음 */
  studioUrl?: string;
  /** iframe 너비 (기본: '100%') */
  width?: string;
  /** iframe 높이 (기본: '100%') */
  height?: string;
  /** 모든 method 요청 제한 시간 override(ms, 기본: 일반 10000, load/export 60000) */
  requestTimeoutMs?: number;
  /** v1 협상 제한 시간(ms, 기본: 1000) */
  handshakeTimeoutMs?: number;
}

export interface LoadResult {
  pageCount: number;
}

export interface HwpVerifyResult {
  /** 직렬화된 HWP 바이트 수 */
  bytesLen: number;
  /** 직렬화 직전 페이지 수 */
  pageCountBefore: number;
  /** 자기 재로드 후 페이지 수 (recovered === true 일 때 의미 있음) */
  pageCountAfter: number;
  /** 자기 재로드 성공 여부 */
  recovered: boolean;
}

export interface HmlSaveBlocker {
  code: string;
  xmlPath: string;
  message: string;
  preserved: false;
}

export interface HmlSaveState {
  sourceFormat: string;
  hmlSavable: boolean;
  blockers: HmlSaveBlocker[];
}

export interface CanvasKitRendererDiagnostics {
  mode: 'default' | 'compat';
  surfacePreference: 'auto' | 'webgpu' | 'webgl' | 'software';
  surfaceBackend: 'default' | 'software' | null;
  surfaceFallbackReason: string | null;
  lastRenderCompleted: boolean;
  lastUnsupportedOps: string[];
  lastExpectedUnsupportedOps: string[];
  lastUnexpectedUnsupportedOps: string[];
  lastRenderError: string | null;
  passesRuntimeReadinessGate: boolean;
  readinessBlockers: Array<'renderNotCompleted' | 'renderError' | 'unexpectedUnsupportedOps' | 'localFontsPending'>;
  hiddenCanvas2dOverlayUsed: false;
  lastRenderDurationMs: number | null;
  renderCount: number;
  imageCacheEntries: number;
  imageCacheLimit: number;
  imageCachePixels: number;
  imageCachePixelLimit: number;
  imageCacheHits: number;
  imageCacheMisses: number;
  imageCacheEvictions: number;
  localTypefaceCount: number;
  localTypefaceLoadFailureCount: number;
  localTypefacePendingCount: number;
}

export interface RendererDiagnosticsV1 {
  schemaVersion: 1;
  request: {
    backend: { backend: 'canvas2d' | 'canvaskit'; source: 'default' | 'url'; requested?: string; unsupportedReason?: string };
    canvaskitMode: { mode: 'default' | 'compat'; source: 'default' | 'storage' | 'url'; requested?: string; unsupportedReason?: string };
    canvaskitSurface: { preference: 'auto' | 'webgpu' | 'webgl' | 'software'; requested: string; unsupportedReason?: string };
    renderProfile: 'fastPreview' | 'screen' | 'print' | 'highQuality';
  } | null;
  initialized: boolean;
  initializationError: string | null;
  effectiveBackend: 'canvas2d' | 'canvaskit' | null;
  backendFallbackReason: string | null;
  page: { index: number; canvaskit: CanvasKitRendererDiagnostics | null };
}

export declare class RhwpEditor {
  private constructor();
  /** HWP 파일을 로드합니다 */
  loadFile(data: ArrayBuffer | Uint8Array, fileName?: string): Promise<LoadResult>;
  /** 현재 문서의 페이지 수를 반환합니다 */
  pageCount(): Promise<number>;
  /** 특정 페이지를 SVG 문자열로 렌더링합니다 */
  getPageSvg(page?: number): Promise<string>;
  /** 선택된 renderer와 페이지별 readiness 진단을 반환합니다 */
  getRendererDiagnostics(page?: number): Promise<RendererDiagnosticsV1>;
  /** 현재 문서를 HWP 바이너리로 내보냅니다 */
  exportHwp(): Promise<Uint8Array>;
  /** 현재 문서를 HWPX(ZIP+XML) 바이너리로 내보냅니다 */
  exportHwpx(): Promise<Uint8Array>;
  /** 현재 문서를 HML(XML) 바이너리로 내보냅니다 */
  exportHml(): Promise<Uint8Array>;
  /** 현재 문서의 HML 저장 가능 여부와 blocker를 반환합니다 */
  getHmlSaveState(): Promise<HmlSaveState>;
  /** HWP 직렬화 + 자기 재로드 검증 메타데이터 (#178) */
  exportHwpVerify(): Promise<HwpVerifyResult>;
  /** iframe 엘리먼트를 반환합니다 */
  readonly element: HTMLIFrameElement;
  /** 에디터를 제거합니다 */
  destroy(): void;
}

/**
 * HWP 에디터를 생성하여 지정된 컨테이너에 마운트합니다.
 *
 * @example
 * ```javascript
 * import { createEditor } from '@rhwp/editor';
 *
 * const editor = await createEditor('#container');
 * const resp = await fetch('document.hwp');
 * await editor.loadFile(await resp.arrayBuffer());
 * ```
 */
export declare function createEditor(
  container: string | HTMLElement,
  options?: EditorOptions,
): Promise<RhwpEditor>;
