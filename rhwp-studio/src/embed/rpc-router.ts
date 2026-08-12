import type { HmlSaveState } from '../core/hml-save-capability.ts';
import type {
  CanvasKitRenderModeRequest,
  CanvasKitSurfaceRequest,
  LayerRenderProfile,
  RenderBackend,
  RenderBackendRequest,
} from '../view/render-backend.ts';

export interface EmbedNotifySavedResult {
  ok: true;
  wasDirty: boolean;
}

export interface EmbedRpcHandlers {
  ready(): Promise<boolean>;
  loadFile(
    data: Uint8Array,
    fileName: string,
    skipUnsavedGuard: boolean,
    suppressDialogs: boolean,
  ): Promise<{ pageCount: number }>;
  pageCount(): Promise<number>;
  getRendererDiagnostics(page: number): Promise<EmbedRendererDiagnosticsV1>;
  getPageSvg(page: number): Promise<string>;
  exportHwp(): Promise<Uint8Array>;
  exportHwpx(): Promise<Uint8Array>;
  exportHml(): Promise<Uint8Array>;
  getHmlSaveState(): Promise<HmlSaveState>;
  exportHwpVerify(): Promise<unknown>;
  notifySaved(fileName?: string): Promise<EmbedNotifySavedResult>;

  // ── 브리지 확장 (P4) ────────────────────────────────────
  // 자동화·플러그인·창 제어. 부모는 **구조화 복제 가능한 값만** 보낼 수 있으므로 함수를 받는
  // 표면(예: 확장 커맨드 등록)은 여기 없다 — 그건 iframe 안의 플러그인만 할 수 있다.
  automationList(): Promise<unknown>;
  automationMenuModel(): Promise<unknown>;
  automationIsEnabled(id: string): Promise<boolean>;
  automationExecute(
    id: string,
    params?: Record<string, unknown>,
    options?: { allowDialog?: boolean },
  ): Promise<unknown>;
  automationContext(): Promise<unknown>;
  pluginList(): Promise<unknown>;
  pluginLoad(id: string): Promise<unknown>;
  pluginUnload(id: string): Promise<unknown>;
  pluginInvoke(id: string, method: string, args: unknown[]): Promise<unknown>;
  chromeGet(): Promise<unknown>;
  chromeSet(next: Record<string, unknown>): Promise<unknown>;
}

export interface EmbedRendererDiagnosticsV1 {
  schemaVersion: 1;
  request: EmbedRendererRuntimeRequestV1 | null;
  initialized: boolean;
  initializationError: string | null;
  effectiveBackend: 'canvas2d' | 'canvaskit' | null;
  backendFallbackReason: string | null;
  selection: unknown;
  page: { index: number; canvaskit: unknown };
}

export interface EmbedRendererRuntimeRequestV1 {
  backend: Omit<RenderBackendRequest, 'backend'> & { backend: RenderBackend };
  canvaskitMode: CanvasKitRenderModeRequest;
  canvaskitSurface: CanvasKitSurfaceRequest;
  renderProfile: LayerRenderProfile;
}

function asParams(value: unknown): Record<string, unknown> {
  return typeof value === 'object' && value !== null ? value as Record<string, unknown> : {};
}

/** 비어 있지 않은 문자열 식별자. 라우터에서 거르지 않으면 핸들러마다 같은 검사를 반복한다. */
function asId(value: unknown): string {
  if (typeof value !== 'string' || value.length === 0) {
    throw new Error('id must be a non-empty string');
  }
  return value;
}

function asBytes(value: unknown, allowLegacyArray: boolean): Uint8Array {
  if (value instanceof Uint8Array) return value;
  if (value instanceof ArrayBuffer) return new Uint8Array(value);
  if (allowLegacyArray && Array.isArray(value)) return new Uint8Array(value);
  throw new Error('loadFile requires binary data');
}

export async function routeEmbedRequest(
  method: string,
  rawParams: unknown,
  handlers: EmbedRpcHandlers,
  allowLegacyArray = false,
): Promise<unknown> {
  const params = asParams(rawParams);
  switch (method) {
    case 'ready': return handlers.ready();
    case 'loadFile':
      return handlers.loadFile(
        asBytes(params.data, allowLegacyArray),
        typeof params.fileName === 'string' ? params.fileName : 'document.hwp',
        params.skipUnsavedGuard === true,
        params.suppressDialogs === true,
      );
    case 'pageCount': return handlers.pageCount();
    case 'getRendererDiagnostics': {
      const page = params.page ?? 0;
      if (!Number.isSafeInteger(page) || (page as number) < 0) {
        throw new Error('page must be a non-negative safe integer');
      }
      return handlers.getRendererDiagnostics(page as number);
    }
    case 'getPageSvg': return handlers.getPageSvg(
      typeof params.page === 'number' ? params.page : 0,
    );
    case 'exportHwp': return handlers.exportHwp();
    case 'exportHwpx': return handlers.exportHwpx();
    case 'exportHml': return handlers.exportHml();
    case 'getHmlSaveState': return handlers.getHmlSaveState();
    case 'exportHwpVerify': return handlers.exportHwpVerify();
    case 'notifySaved': return handlers.notifySaved(
      typeof params.fileName === 'string' && params.fileName.length > 0
        ? params.fileName
        : undefined,
    );
    // ── 브리지 확장 (P4) ──────────────────────────────────
    case 'automation.list': return handlers.automationList();
    case 'automation.menuModel': return handlers.automationMenuModel();
    case 'automation.isEnabled': return handlers.automationIsEnabled(asId(params.id));
    case 'automation.execute': return handlers.automationExecute(
      asId(params.id),
      typeof params.params === 'object' && params.params !== null
        ? params.params as Record<string, unknown>
        : undefined,
      { allowDialog: params.allowDialog === true },
    );
    case 'automation.context': return handlers.automationContext();
    case 'plugin.list': return handlers.pluginList();
    case 'plugin.load': return handlers.pluginLoad(asId(params.id));
    case 'plugin.unload': return handlers.pluginUnload(asId(params.id));
    case 'plugin.invoke': return handlers.pluginInvoke(
      asId(params.id), asId(params.method), Array.isArray(params.args) ? params.args : [],
    );
    case 'chrome.get': return handlers.chromeGet();
    case 'chrome.set': return handlers.chromeSet(asParams(params.visibility));
    default: throw new Error(`Unknown method: ${method}`);
  }
}
