import type { CanvasKitDocumentPreflight, LayerRenderProfile } from '@/core/types';
import type { CanvasKitLayerRenderer } from './canvaskit-renderer';
import type {
  CanvasKitRenderModeRequest,
  CanvasKitSurfaceRequest,
  RenderBackend,
  RenderBackendFallbackReason,
  RenderBackendRequest,
} from './render-backend';

export type RendererSelectionReason =
  | 'defaultCanvas2d'
  | 'explicitCanvas2d'
  | 'explicitCanvasKit'
  | 'autoEligible'
  | 'autoIneligible'
  | 'autoPreflightIncomplete'
  | 'autoRevisionPending'
  | 'superseded'
  | 'canvaskitInitializationFailed'
  | 'canvaskitResourcePreparationFailed'
  | 'canvaskitRuntimeFailed';

export interface RendererSessionDiagnostics {
  schemaVersion: 1;
  request: RenderBackendRequest;
  requestedBackend: RenderBackendRequest['backend'];
  effectiveBackend: RenderBackend;
  selectionReason: RendererSelectionReason;
  fallbackReason: RenderBackendFallbackReason | null;
  documentRevision: number;
  resourceGeneration: number;
  renderProfile: LayerRenderProfile;
  documentDigest: string | null;
  decisionKey: string;
  preflight: CanvasKitDocumentPreflight | null;
  initializationError: string | null;
  selectionError: string | null;
}

export interface RendererSessionSelection {
  backend: RenderBackend;
  canvaskitRenderer: CanvasKitLayerRenderer | null;
  diagnostics: RendererSessionDiagnostics;
}

export interface RendererPreflightSource {
  getCanvasKitDocumentPreflight(
    mode: CanvasKitRenderModeRequest['mode'],
    profile: LayerRenderProfile,
  ): CanvasKitDocumentPreflight;
}

type CanvasKitRendererFactory = (
  mode: CanvasKitRenderModeRequest['mode'],
  surface: CanvasKitSurfaceRequest,
) => Promise<CanvasKitLayerRenderer>;

interface RendererDecisionSnapshot {
  key: string;
  documentRevision: number;
  resourceGeneration: number;
  documentDigest: string | null;
}

export interface RendererSessionOptions {
  transformCanvasKitPreflight?: (
    preflight: CanvasKitDocumentPreflight,
  ) => CanvasKitDocumentPreflight;
  prepareCanvasKitDocument?: (
    renderer: CanvasKitLayerRenderer,
    preflight: CanvasKitDocumentPreflight,
  ) => Promise<void>;
}

function errorText(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

export class RendererSession {
  private readonly request: RenderBackendRequest;
  private readonly canvaskitMode: CanvasKitRenderModeRequest;
  private readonly canvaskitSurface: CanvasKitSurfaceRequest;
  private readonly renderProfile: LayerRenderProfile;
  private readonly createCanvasKitRenderer: CanvasKitRendererFactory;
  private readonly options: RendererSessionOptions;
  private documentRevision = 0;
  private resourceGeneration = 0;
  private documentDigest: string | null = null;
  private current: RendererSessionSelection | null = null;
  private pending: { key: string; promise: Promise<RendererSessionSelection> } | null = null;
  private canvaskitRenderer: CanvasKitLayerRenderer | null = null;
  private canvaskitInitialization: Promise<CanvasKitLayerRenderer> | null = null;
  private canvaskitInitializationError: string | null = null;
  private disposed = false;

  constructor(
    request: RenderBackendRequest,
    canvaskitMode: CanvasKitRenderModeRequest,
    canvaskitSurface: CanvasKitSurfaceRequest,
    renderProfile: LayerRenderProfile,
    createCanvasKitRenderer: CanvasKitRendererFactory,
    options: RendererSessionOptions = {},
  ) {
    this.request = request;
    this.canvaskitMode = canvaskitMode;
    this.canvaskitSurface = canvaskitSurface;
    this.renderProfile = renderProfile;
    this.createCanvasKitRenderer = createCanvasKitRenderer;
    this.options = options;
  }

  beginDocument(documentDigest: string | null): void {
    if (this.disposed) return;
    this.documentRevision += 1;
    this.resourceGeneration += 1;
    this.documentDigest = documentDigest;
    this.current = null;
    this.pending = null;
    this.canvaskitInitializationError = null;
    this.canvaskitRenderer?.resetDocumentResources();
  }

  invalidateDocument(options: { resetResources?: boolean } = {}): void {
    if (this.disposed) return;
    this.documentRevision += 1;
    this.resourceGeneration += 1;
    this.current = null;
    this.pending = null;
    if (options.resetResources !== false) {
      this.canvaskitRenderer?.resetDocumentResources();
    } else {
      this.canvaskitRenderer?.cancelDocumentPreparation?.();
    }
  }

  resolve(source: RendererPreflightSource): Promise<RendererSessionSelection> {
    if (this.disposed) return Promise.reject(new Error('RendererSession is disposed'));
    const decision = this.decisionSnapshot();
    const key = decision.key;
    if (this.current?.diagnostics.decisionKey === key) return Promise.resolve(this.current);
    if (this.pending?.key === key) return this.pending.promise;

    const promise = this.resolveUncached(source, decision).then((selection) => {
      if (this.decisionKey() === key) this.current = selection;
      if (this.pending?.key === key) this.pending = null;
      return selection;
    }, (error) => {
      if (this.pending?.key === key) this.pending = null;
      throw error;
    });
    this.pending = { key, promise };
    return promise;
  }

  isCurrent(selection: RendererSessionSelection): boolean {
    return selection.diagnostics.decisionKey === this.decisionKey();
  }

  isAutoRequest(): boolean {
    return this.request.backend === 'auto';
  }

  /**
   * A mutation invalidates the document-wide capability report. Auto mode pins
   * the new revision to Canvas2D immediately; a caller may perform one bounded
   * auto re-evaluation after a quiet period.
   */
  pinAutoMutationRevision(): RendererSessionSelection | null {
    if (!this.isAutoRequest() || this.disposed) return null;
    this.invalidateDocument();
    const decision = this.decisionSnapshot();
    const selection = this.selection(
      decision,
      'canvas2d',
      'autoRevisionPending',
      'canvaskitRevisionInvalidated',
      null,
    );
    this.current = selection;
    return selection;
  }

  fallbackFromResourceFailure(
    error: unknown,
    expectedDecisionKey: string,
  ): RendererSessionSelection | null {
    if (!this.isAutoRequest()) return null;
    return this.fallbackForCurrentDecision(
      error,
      expectedDecisionKey,
      'canvaskitResourcePreparationFailed',
      'canvaskitResourcePreparationFailed',
    );
  }

  fallbackFromRuntimeFailure(
    error: unknown,
    expectedDecisionKey: string,
  ): RendererSessionSelection | null {
    if (!this.isAutoRequest()) return null;
    return this.fallbackForCurrentDecision(
      error,
      expectedDecisionKey,
      'canvaskitRuntimeFailed',
      'canvaskitRuntimeFailed',
    );
  }

  private fallbackForCurrentDecision(
    error: unknown,
    expectedDecisionKey: string,
    selectionReason: RendererSelectionReason,
    fallbackReason: RenderBackendFallbackReason,
  ): RendererSessionSelection | null {
    const decision = this.decisionSnapshot();
    if (expectedDecisionKey !== decision.key) return null;
    const selection = this.selection(
      decision,
      'canvas2d',
      selectionReason,
      fallbackReason,
      this.current?.diagnostics.preflight ?? null,
      errorText(error),
    );
    this.current = selection;
    this.pending = null;
    return selection;
  }

  diagnostics(): RendererSessionDiagnostics | null {
    const diagnostics = this.current?.diagnostics;
    if (!diagnostics) return null;
    return {
      ...diagnostics,
      request: { ...diagnostics.request },
      preflight: diagnostics.preflight ? {
        ...diagnostics.preflight,
        limits: { ...diagnostics.preflight.limits },
        summary: { ...diagnostics.preflight.summary },
        blockers: diagnostics.preflight.blockers.map(blocker => ({ ...blocker })),
        requiredFontFamilies: [...diagnostics.preflight.requiredFontFamilies],
      } : null,
    };
  }

  getCanvasKitRenderer(): CanvasKitLayerRenderer | null {
    return this.canvaskitRenderer;
  }

  dispose(): void {
    if (this.disposed) return;
    this.disposed = true;
    this.documentRevision += 1;
    this.current = null;
    this.pending = null;
    this.canvaskitInitialization = null;
    const renderer = this.canvaskitRenderer;
    this.canvaskitRenderer = null;
    renderer?.dispose();
  }

  private async resolveUncached(
    source: RendererPreflightSource,
    decision: RendererDecisionSnapshot,
  ): Promise<RendererSessionSelection> {
    const key = decision.key;
    if (this.request.backend === 'canvas2d') {
      return this.selection(
        decision,
        'canvas2d',
        this.request.source === 'default' ? 'defaultCanvas2d' : 'explicitCanvas2d',
        this.request.unsupportedReason ?? null,
        null,
      );
    }

    let preflight: CanvasKitDocumentPreflight | null = null;
    if (this.request.backend === 'auto') {
      try {
        preflight = this.readPreflight(source);
      } catch (error) {
        return this.selection(
          decision,
          'canvas2d',
          'autoPreflightIncomplete',
          'canvaskitDocumentPreflightIncomplete',
          null,
          errorText(error),
        );
      }
      if (!preflight.complete || preflight.status === 'incomplete') {
        return this.selection(
          decision,
          'canvas2d',
          'autoPreflightIncomplete',
          'canvaskitDocumentPreflightIncomplete',
          preflight,
        );
      }
      if (!preflight.eligible || preflight.status !== 'eligible') {
        return this.selection(
          decision,
          'canvas2d',
          'autoIneligible',
          'canvaskitDocumentIneligible',
          preflight,
        );
      }
    }

    let renderer: CanvasKitLayerRenderer;
    try {
      renderer = await this.ensureCanvasKitRenderer();
    } catch (error) {
      return this.selection(
        decision,
        'canvas2d',
        'canvaskitInitializationFailed',
        'canvaskitInitializationFailed',
        preflight,
        errorText(error),
        errorText(error),
      );
    }
    if (this.decisionKey() !== key) {
      return this.selection(
        decision,
        'canvas2d',
        'superseded',
        'canvaskitRevisionInvalidated',
        preflight,
      );
    }
    if (preflight && this.options.prepareCanvasKitDocument) {
      try {
        await this.options.prepareCanvasKitDocument(renderer, preflight);
      } catch (error) {
        if (this.decisionKey() === key) {
          return this.selection(
            decision,
            'canvas2d',
            'canvaskitResourcePreparationFailed',
            'canvaskitResourcePreparationFailed',
            preflight,
            errorText(error),
          );
        }
      }
      if (this.decisionKey() !== key) {
        return this.selection(
          decision,
          'canvas2d',
          'superseded',
          'canvaskitRevisionInvalidated',
          preflight,
        );
      }
    }
    return this.selection(
      decision,
      'canvaskit',
      this.request.backend === 'auto' ? 'autoEligible' : 'explicitCanvasKit',
      null,
      preflight,
      null,
      null,
      renderer,
    );
  }

  private readPreflight(source: RendererPreflightSource): CanvasKitDocumentPreflight {
    const preflight = source.getCanvasKitDocumentPreflight(
      this.canvaskitMode.mode,
      this.renderProfile,
    );
    if (preflight.schemaVersion !== 1) {
      throw new Error(`지원하지 않는 CanvasKit document preflight schema: ${preflight.schemaVersion}`);
    }
    if (preflight.mode !== this.canvaskitMode.mode || preflight.profile !== this.renderProfile) {
      throw new Error(
        `CanvasKit document preflight 요청 불일치: requested=${this.canvaskitMode.mode}/${this.renderProfile};received=${preflight.mode}/${preflight.profile}`,
      );
    }
    return this.options.transformCanvasKitPreflight?.(preflight) ?? preflight;
  }

  private ensureCanvasKitRenderer(): Promise<CanvasKitLayerRenderer> {
    if (this.canvaskitRenderer) return Promise.resolve(this.canvaskitRenderer);
    if (this.canvaskitInitializationError) {
      return Promise.reject(new Error(this.canvaskitInitializationError));
    }
    if (this.canvaskitInitialization) return this.canvaskitInitialization;

    const initialization = this.createCanvasKitRenderer(
      this.canvaskitMode.mode,
      this.canvaskitSurface,
    ).then((renderer) => {
      if (this.disposed) {
        renderer.dispose();
        throw new Error('RendererSession was disposed during CanvasKit initialization');
      }
      if (this.canvaskitInitialization === initialization) {
        this.canvaskitRenderer = renderer;
        this.canvaskitInitialization = null;
        this.canvaskitInitializationError = null;
      }
      return renderer;
    }, (error) => {
      if (this.canvaskitInitialization === initialization) {
        this.canvaskitInitialization = null;
        this.canvaskitInitializationError = errorText(error);
      }
      throw error;
    });
    this.canvaskitInitialization = initialization;
    return initialization;
  }

  private selection(
    decision: RendererDecisionSnapshot,
    backend: RenderBackend,
    selectionReason: RendererSelectionReason,
    fallbackReason: RenderBackendFallbackReason | null,
    preflight: CanvasKitDocumentPreflight | null,
    selectionError: string | null = null,
    initializationError: string | null = null,
    canvaskitRenderer: CanvasKitLayerRenderer | null = null,
  ): RendererSessionSelection {
    return {
      backend,
      canvaskitRenderer,
      diagnostics: {
        schemaVersion: 1,
        request: { ...this.request },
        requestedBackend: this.request.backend,
        effectiveBackend: backend,
        selectionReason,
        fallbackReason,
        documentRevision: decision.documentRevision,
        resourceGeneration: decision.resourceGeneration,
        renderProfile: this.renderProfile,
        documentDigest: decision.documentDigest,
        decisionKey: decision.key,
        preflight,
        initializationError,
        selectionError,
      },
    };
  }

  private decisionKey(): string {
    return this.decisionSnapshot().key;
  }

  private decisionSnapshot(): RendererDecisionSnapshot {
    const key = [
      this.documentDigest ?? 'document:none',
      `revision:${this.documentRevision}`,
      `profile:${this.renderProfile}`,
      `resources:${this.resourceGeneration}`,
      `request:${this.request.backend}`,
      `mode:${this.canvaskitMode.mode}`,
    ].join('|');
    return {
      key,
      documentRevision: this.documentRevision,
      resourceGeneration: this.resourceGeneration,
      documentDigest: this.documentDigest,
    };
  }
}
