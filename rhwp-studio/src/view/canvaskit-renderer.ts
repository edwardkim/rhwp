import CanvasKitInit from 'canvaskit-wasm';
import type {
  Canvas,
  CanvasKit,
  Color,
  Font,
  FontMgr,
  Image as SkImage,
  Paint,
  Path,
  PathBuilder,
  Rect,
  Surface,
  Typeface,
} from 'canvaskit-wasm';
import canvaskitWasmUrl from 'canvaskit-wasm/bin/canvaskit.wasm?url';

import type {
  LayerBounds,
  LayerClipNode,
  LayerEllipseOp,
  LayerFormObjectOp,
  LayerAffineTransform,
  LayerGlyphOutlineOp,
  LayerImageOp,
  LayerInfo,
  LayerLeafNode,
  LayerLineOp,
  LayerNode,
  LayerPageBackgroundOp,
  LayerPaintOp,
  LayerPathCommand,
  LayerPathOp,
  LayerPlaceholderOp,
  LayerRectangleOp,
  LayerRenderProfile,
  LayerShapeStyle,
  LayerTextRunOp,
  PageInfo,
  PageLayerTree,
} from '@/core/types';
import {
  DEFAULT_CANVASKIT_SURFACE_REQUEST,
  type CanvasKitRenderMode,
  type CanvasKitSurfacePreference,
  type CanvasKitSurfaceRequest,
} from './render-backend';
import {
  canvasKitImageCacheKey,
  canvasKitImageFillModeTiles,
  canvasKitImagePlacement,
  canvasKitImageSourceRect,
} from './canvaskit/image-replay';
import { canvaskitClipRightPad } from './canvaskit/policy';
import {
  CANVASKIT_REPLAY_PLANES,
  type CanvasKitReplayPlane,
  layerPaintOpReplayPlane,
} from './canvaskit/replay-plane';
import { isExpectedCanvasKitUnsupportedOp } from './canvaskit/diagnostics';
import { glyphOutlinePayloadStatus } from './glyph-outline-payload-status';
import { loadLocalFontBytesFor, localFontFaceKey, resolveLocalFont, type LocalFontRecord } from '@/core/local-fonts';

type CanvasKitApi = CanvasKit;
type SkCanvas = Canvas;
type SkPaint = Paint;
type SkSurface = Surface;
type MutablePath = Path & Pick<PathBuilder, 'arcToRotated' | 'close' | 'cubicTo' | 'lineTo' | 'moveTo'>;
type LayerColorGraph = NonNullable<NonNullable<LayerGlyphOutlineOp['colorLayers']>['paintGraph']>;
type LayerColorGraphNode = NonNullable<LayerColorGraph['nodes']>[number];
interface CanvasKitSurfaceTarget {
  surface: SkSurface;
  canvas: HTMLCanvasElement;
}

interface CanvasKitLocalTypeface {
  typeface: Typeface | null;
  fontManager: FontMgr | null;
  fontFamily: string | null;
}

export interface CanvasKitRenderDiagnostics {
  mode: CanvasKitRenderMode;
  surfacePreference: CanvasKitSurfacePreference;
  surfaceBackend: 'default' | 'software' | null;
  surfaceFallbackReason: string | null;
  lastRenderCompleted: boolean;
  lastUnsupportedOps: string[];
  lastExpectedUnsupportedOps: string[];
  lastUnexpectedUnsupportedOps: string[];
  lastRenderError: string | null;
  passesRuntimeReadinessGate: boolean;
  readinessBlockers: CanvasKitReadinessBlocker[];
  hiddenCanvas2dOverlayUsed: false;
}

export type CanvasKitReadinessBlocker =
  | 'renderNotCompleted'
  | 'renderError'
  | 'unexpectedUnsupportedOps';

export class CanvasKitLayerRenderer {
  // Prevent pathological tiled fills from monopolizing the render loop.
  private static readonly MAX_IMAGE_TILE_DRAWS = 4096;
  // 단일 text run은 줄바꿈 없이 문서가 지정한 위치에 재생한다.
  private static readonly MAX_SHAPED_TEXT_WIDTH = 1_000_000;

  private readonly imageCache = new Map<string, SkImage>();
  private readonly localTypefaces = new Map<string, CanvasKitLocalTypeface>();
  private readonly localTypefaceLoadFailures = new Set<string>();
  private readonly unsupportedOps = new Set<string>();
  private surfaceBackend: 'default' | 'software' | null = null;
  private surfaceFallbackReason: string | null = null;
  private lastRenderError: string | null = null;
  private lastRenderCompleted = false;
  private disposed = false;

  private constructor(
    private readonly canvasKit: CanvasKitApi,
    private readonly renderMode: CanvasKitRenderMode,
    private readonly surfaceRequest: CanvasKitSurfaceRequest,
    private readonly defaultTypeface: Typeface | null,
    private readonly defaultFontManager: FontMgr | null = null,
    private readonly defaultFontFamily: string | null = null,
  ) {}

  static async create(
    renderMode: CanvasKitRenderMode = 'default',
    surfaceRequest: CanvasKitSurfaceRequest | CanvasKitSurfacePreference = DEFAULT_CANVASKIT_SURFACE_REQUEST,
  ): Promise<CanvasKitLayerRenderer> {
    const canvasKit = await CanvasKitInit({
      locateFile: (file) => file === 'canvaskit.wasm' ? canvaskitWasmUrl : file,
    });
    const resolvedSurfaceRequest = typeof surfaceRequest === 'string'
      ? { ...DEFAULT_CANVASKIT_SURFACE_REQUEST, preference: surfaceRequest, requested: surfaceRequest }
      : surfaceRequest;
    // 기본 Noto는 local face가 없거나 등록에 실패한 text run의 안정적인 CJK fallback이다.
    let defaultTypeface: Typeface | null = null;
    let defaultFontManager: FontMgr | null = null;
    let defaultFontFamily: string | null = null;
    try {
      const response = await fetch('fonts/NotoSansKR-Regular.woff2');
      if (response.ok) {
        const bytes = await response.arrayBuffer();
        defaultTypeface = canvasKit.Typeface.MakeFreeTypeFaceFromData(bytes)
          ?? canvasKit.Typeface.MakeTypefaceFromData(bytes);
        defaultFontManager = canvasKit.FontMgr.FromData(bytes);
        if (defaultFontManager && defaultFontManager.countFamilies() > 0) {
          defaultFontFamily = defaultFontManager.getFamilyName(0);
        }
      }
    } catch (error) {
      console.warn('[CanvasKitLayerRenderer] 기본 CJK 폰트 로딩 실패:', error);
    }
    return new CanvasKitLayerRenderer(
      canvasKit,
      renderMode,
      resolvedSurfaceRequest,
      defaultTypeface,
      defaultFontManager,
      defaultFontFamily,
    );
  }

  /** 현재 문서가 실제로 사용하는 설치 글꼴만 CanvasKit native 객체로 등록한다. */
  async prepareLocalFonts(fontNames: readonly string[] | undefined): Promise<number> {
    if (this.disposed || !fontNames?.length) return 0;
    const pendingRecords = new Map<string, LocalFontRecord>();
    for (const fontName of fontNames) {
      const record = resolveLocalFont(fontName);
      const faceKey = record ? localFontFaceKey(record) : '';
      if (!record || !faceKey || this.localTypefaces.has(faceKey) || this.localTypefaceLoadFailures.has(faceKey)) continue;
      pendingRecords.set(faceKey, record);
    }

    const bytesByFace = await loadLocalFontBytesFor([...pendingRecords.values()].map(record => record.fullName));
    let registered = 0;
    for (const [faceKey, record] of pendingRecords) {
      const bytes = bytesByFace.get(faceKey);
      if (this.disposed) return registered;
      if (!bytes) {
        this.localTypefaceLoadFailures.add(faceKey);
        continue;
      }
      let typeface: Typeface | null = null;
      let fontManager: FontMgr | null = null;
      try {
        typeface = this.canvasKit.Typeface.MakeFreeTypeFaceFromData(bytes)
          ?? this.canvasKit.Typeface.MakeTypefaceFromData(bytes);
        fontManager = this.canvasKit.FontMgr.FromData(bytes.slice(0));
        if (!typeface && !fontManager) {
          this.localTypefaceLoadFailures.add(faceKey);
          continue;
        }
        const fontFamily = fontManager && fontManager.countFamilies() > 0
          ? fontManager.getFamilyName(0)
          : record.family;
        this.localTypefaces.set(faceKey, { typeface, fontManager, fontFamily });
        registered += 1;
      } catch (error) {
        typeface?.delete?.();
        fontManager?.delete?.();
        this.localTypefaceLoadFailures.add(faceKey);
        console.warn(`[CanvasKitLayerRenderer] ${record.displayName} local Typeface 등록 실패:`, error);
      }
      // native font parsing은 동기 작업이므로 face 사이에서 paint/event loop에 양보한다.
      await new Promise<void>(resolve => window.setTimeout(resolve, 0));
    }
    return registered;
  }

  renderPage(
    tree: PageLayerTree,
    targetCanvas: HTMLCanvasElement,
    scale: number,
    pageInfo?: PageInfo,
  ): HTMLCanvasElement {
    if (this.disposed) {
      throw new Error('CanvasKit renderer가 이미 dispose되었습니다');
    }
    this.unsupportedOps.clear();
    this.lastRenderError = null;
    this.lastRenderCompleted = false;
    let surface: SkSurface | null = null;
    let renderedCanvas = targetCanvas;
    try {
      const surfaceTarget = this.makeSurface(targetCanvas);
      surface = surfaceTarget.surface;
      renderedCanvas = surfaceTarget.canvas;
      const canvas = surface.getCanvas();
      let hasPageBackground = false;
      const stack: LayerNode[] = [tree.root];
      while (stack.length > 0 && !hasPageBackground) {
        const node = stack.pop()!;
        if (node.kind === 'group') {
          stack.push(...node.children);
        } else if (node.kind === 'clipRect') {
          stack.push(node.child);
        } else {
          hasPageBackground = node.ops.some((op) => op.type === 'pageBackground');
        }
      }
      canvas.save();
      canvas.clear(this.color(hasPageBackground ? 'rgba(0,0,0,0)' : '#ffffff'));
      canvas.scale(scale, scale);
      const rightOverflowSlop =
        tree.outputOptions?.showParagraphMarks || tree.outputOptions?.showControlCodes ? 48 : undefined;
      for (const replayPlane of CANVASKIT_REPLAY_PLANES) {
        this.renderNode(canvas, tree.root, tree.profile ?? 'screen', replayPlane, null, rightOverflowSlop);
      }
      if (pageInfo) {
        const paint = this.makeStrokePaint('#c0c0c0', 0.3);
        const left = pageInfo.marginLeft;
        const top = pageInfo.marginHeader + pageInfo.marginTop;
        const right = pageInfo.width - pageInfo.marginRight;
        const bottom = pageInfo.height - pageInfo.marginFooter - pageInfo.marginBottom;
        const length = 15;
        canvas.drawLine(left, top - length, left, top, paint);
        canvas.drawLine(left, top, left - length, top, paint);
        canvas.drawLine(right + length, top, right, top, paint);
        canvas.drawLine(right, top, right, top - length, paint);
        canvas.drawLine(left - length, bottom, left, bottom, paint);
        canvas.drawLine(left, bottom, left, bottom + length, paint);
        canvas.drawLine(right, bottom + length, right, bottom, paint);
        canvas.drawLine(right, bottom, right + length, bottom, paint);
        paint.delete();
      }
      canvas.restore();
      surface.flush();
      this.lastRenderCompleted = true;
    } catch (error) {
      this.recordRenderFailure(error);
      throw error;
    } finally {
      surface?.delete();
    }
    return renderedCanvas;
  }

  releaseLayerTree(_tree: PageLayerTree): void {
    /* Per-tree native picture interning is not implemented yet. */
  }

  diagnostics(): CanvasKitRenderDiagnostics {
    const lastUnsupportedOps = [...this.unsupportedOps].sort();
    const lastExpectedUnsupportedOps = lastUnsupportedOps.filter(isExpectedCanvasKitUnsupportedOp);
    const lastUnexpectedUnsupportedOps = lastUnsupportedOps.filter(
      (op) => !isExpectedCanvasKitUnsupportedOp(op),
    );
    const surfaceFallbackReason = this.surfaceFallbackReason ?? this.surfaceRequest.unsupportedReason ?? null;
    const readinessBlockers: CanvasKitReadinessBlocker[] = [];
    if (!this.lastRenderCompleted) readinessBlockers.push('renderNotCompleted');
    if (this.lastRenderError !== null) readinessBlockers.push('renderError');
    if (lastUnexpectedUnsupportedOps.length > 0) readinessBlockers.push('unexpectedUnsupportedOps');
    return {
      mode: this.renderMode,
      surfacePreference: this.surfaceRequest.preference,
      surfaceBackend: this.surfaceBackend,
      surfaceFallbackReason,
      lastRenderCompleted: this.lastRenderCompleted,
      lastUnsupportedOps,
      lastExpectedUnsupportedOps,
      lastUnexpectedUnsupportedOps,
      lastRenderError: this.lastRenderError,
      passesRuntimeReadinessGate: readinessBlockers.length === 0,
      readinessBlockers,
      hiddenCanvas2dOverlayUsed: false,
    };
  }

  recordRenderFailure(error: unknown, resetReplayState = false): void {
    if (resetReplayState) {
      this.unsupportedOps.clear();
      this.surfaceBackend = null;
      this.surfaceFallbackReason = null;
    }
    this.lastRenderCompleted = false;
    this.lastRenderError = error instanceof Error ? error.message : String(error);
    this.unsupportedOps.add('renderPage');
  }

  dispose(): void {
    this.disposed = true;
    for (const image of this.imageCache.values()) {
      image?.delete?.();
    }
    this.imageCache.clear();
    for (const { typeface, fontManager } of this.localTypefaces.values()) {
      typeface?.delete?.();
      fontManager?.delete?.();
    }
    this.localTypefaces.clear();
    this.localTypefaceLoadFailures.clear();
    this.defaultTypeface?.delete();
    this.defaultFontManager?.delete();
  }

  private makeSurface(
    targetCanvas: HTMLCanvasElement,
  ): CanvasKitSurfaceTarget {
    this.surfaceBackend = null;
    this.surfaceFallbackReason = this.surfaceRequest.unsupportedReason ?? null;
    if (this.surfaceRequest.preference === 'webgpu' && this.surfaceFallbackReason === null) {
      this.surfaceFallbackReason = 'webgpuSurfaceUnsupported';
    }
    const reuseSoftwareFallbackCanvas = targetCanvas.classList.contains('ck-replaced');
    if (this.surfaceRequest.preference === 'software' || reuseSoftwareFallbackCanvas) {
      const swSurface = this.canvasKit.MakeSWCanvasSurface(targetCanvas);
      if (swSurface) {
        this.surfaceBackend = 'software';
        if (reuseSoftwareFallbackCanvas && this.surfaceFallbackReason === null) {
          this.surfaceFallbackReason = 'defaultSurfaceUnavailableUsingSoftware';
        }
        return { surface: swSurface, canvas: targetCanvas };
      }
      this.surfaceFallbackReason = 'softwareSurfaceUnavailable';
    }
    const originalParent = targetCanvas.parentElement;
    const originalChildIndex = originalParent
      ? Array.prototype.indexOf.call(originalParent.children, targetCanvas)
      : -1;
    try {
      const surface = this.canvasKit.MakeCanvasSurface(targetCanvas);
      if (surface) {
        const replacement = originalParent && originalChildIndex >= 0
          ? originalParent.children.item(originalChildIndex)
          : null;
        if (targetCanvas.parentElement !== originalParent && replacement instanceof HTMLCanvasElement) {
          this.surfaceBackend = 'software';
          if (this.surfaceFallbackReason === null) {
            this.surfaceFallbackReason = 'defaultSurfaceUnavailableUsingSoftware';
          }
          return { surface, canvas: replacement };
        }
        this.surfaceBackend = 'default';
        return { surface, canvas: targetCanvas };
      }
    } catch {
      if (this.surfaceFallbackReason === null) {
        this.surfaceFallbackReason = 'defaultSurfaceCreationFailed';
      }
    }
    const internalReplacement = originalParent && originalChildIndex >= 0
      ? originalParent.children.item(originalChildIndex)
      : null;
    let softwareCanvas = targetCanvas.parentElement !== originalParent
      && internalReplacement instanceof HTMLCanvasElement
      ? internalReplacement
      : targetCanvas;
    if (softwareCanvas === targetCanvas && targetCanvas.parentElement) {
      const parent = targetCanvas.parentElement;
      const replacement = targetCanvas.cloneNode(true) as HTMLCanvasElement;
      replacement.classList.add('ck-replaced');
      parent.replaceChild(replacement, targetCanvas);
      softwareCanvas = replacement;
    }
    const softwareSurface = this.canvasKit.MakeSWCanvasSurface(softwareCanvas);
    if (softwareSurface) {
      this.surfaceBackend = 'software';
      if (this.surfaceFallbackReason === null) {
        this.surfaceFallbackReason = 'defaultSurfaceUnavailableUsingSoftware';
      }
      return { surface: softwareSurface, canvas: softwareCanvas };
    }
    throw new Error('CanvasKit surface를 만들 수 없습니다');
  }

  private renderNode(
    canvas: SkCanvas,
    node: LayerNode,
    profile: LayerRenderProfile,
    replayPlane: CanvasKitReplayPlane,
    inheritedLayer: LayerInfo | null = null,
    rightOverflowSlop?: number,
  ): void {
    const activeLayer = node.layer ?? inheritedLayer;
    if (node.kind === 'group') {
      for (const child of node.children) {
        this.renderNode(canvas, child, profile, replayPlane, activeLayer, rightOverflowSlop);
      }
      return;
    }
    if (node.kind === 'clipRect') {
      this.renderClipNode(canvas, node, profile, replayPlane, activeLayer, rightOverflowSlop);
      return;
    }
    this.renderLeaf(canvas, node, replayPlane, activeLayer);
  }

  private renderClipNode(
    canvas: SkCanvas,
    node: LayerClipNode,
    profile: LayerRenderProfile,
    replayPlane: CanvasKitReplayPlane,
    inheritedLayer: LayerInfo | null,
    rightOverflowSlop?: number,
  ): void {
    const pad = canvaskitClipRightPad(this.renderMode, profile, node.clipKind, rightOverflowSlop);
    const clip = {
      ...node.clip,
      width: node.clip.width + pad,
    };
    canvas.save();
    canvas.clipRect(this.rect(clip), this.canvasKit.ClipOp?.Intersect ?? 0, true);
    this.renderNode(canvas, node.child, profile, replayPlane, inheritedLayer, rightOverflowSlop);
    canvas.restore();
  }

  private renderLeaf(
    canvas: SkCanvas,
    node: LayerLeafNode,
    replayPlane: CanvasKitReplayPlane,
    inheritedLayer: LayerInfo | null,
  ): void {
    const activeLayer = node.layer ?? inheritedLayer;
    for (const op of node.ops) {
      if (layerPaintOpReplayPlane(op, activeLayer) !== replayPlane) {
        continue;
      }
      this.renderOp(canvas, op);
    }
  }

  private renderOp(canvas: SkCanvas, op: LayerPaintOp): void {
    switch (op.type) {
      case 'pageBackground':
        this.renderPageBackground(canvas, op);
        return;
      case 'rectangle':
        this.renderRectangle(canvas, op);
        return;
      case 'ellipse':
        this.renderEllipse(canvas, op);
        return;
      case 'line':
        this.renderLine(canvas, op);
        return;
      case 'path':
        this.renderPath(canvas, op);
        return;
      case 'image':
        this.renderImage(canvas, op);
        return;
      case 'textRun':
        this.renderTextRun(canvas, op);
        return;
      case 'footnoteMarker':
        this.renderTextRun(canvas, {
          type: 'textRun',
          bbox: op.bbox,
          text: op.text,
          baseline: op.fontSize ?? 7,
          style: { fontFamily: op.fontFamily, fontSize: op.fontSize, color: op.color },
        });
        return;
      case 'formObject':
        this.renderFormObject(canvas, op);
        return;
      case 'placeholder':
        this.renderPlaceholder(canvas, op);
        return;
      case 'equation':
        this.unsupportedOps.add('equation:unsupportedDirectReplay');
        return;
      case 'rawSvg':
        this.unsupportedOps.add('rawSvg:unsupportedDirectReplay');
        return;
      case 'charOverlap':
      case 'glyphRun':
      case 'tabLeader':
      case 'textControlMark':
      case 'textDecoration':
        this.unsupportedOps.add(op.type);
        return;
      case 'glyphOutline': {
        const status = glyphOutlinePayloadStatus(op, { allowColrv1Stage1ColorGraph: true });
        if (status.supported && op.payloadKind === 'colorLayers') {
          this.renderGlyphOutline(canvas, op);
          return;
        }
        this.unsupportedOps.add(status.reason ? `glyphOutline:${status.reason}` : 'glyphOutline');
        return;
      }
      default:
        this.unsupportedOps.add((op as { type?: string }).type ?? 'unknown');
    }
  }

  private renderPageBackground(canvas: SkCanvas, op: LayerPageBackgroundOp): void {
    if (op.backgroundColor) {
      const paint = this.makeFillPaint(op.backgroundColor);
      canvas.drawRect(this.rect(op.bbox), paint);
      paint.delete?.();
    }
    if (op.borderColor && (op.borderWidth ?? 0) > 0) {
      const paint = this.makeStrokePaint(op.borderColor, op.borderWidth ?? 1);
      canvas.drawRect(this.rect(op.bbox), paint);
      paint.delete?.();
    }
  }

  private renderRectangle(canvas: SkCanvas, op: LayerRectangleOp): void {
    this.drawStyledShape(canvas, op.bbox, op.style, (paint) => {
      const cornerRadius = op.cornerRadius ?? 0;
      if (cornerRadius > 0) {
        canvas.drawRRect(this.canvasKit.RRectXY(this.rect(op.bbox), cornerRadius, cornerRadius), paint);
      } else {
        canvas.drawRect(this.rect(op.bbox), paint);
      }
    });
  }

  private renderEllipse(canvas: SkCanvas, op: LayerEllipseOp): void {
    this.drawStyledShape(canvas, op.bbox, op.style, (paint) => {
      canvas.drawOval(this.rect(op.bbox), paint);
    });
  }

  private renderLine(canvas: SkCanvas, op: LayerLineOp): void {
    const paint = this.makeStrokePaint(op.style?.color ?? '#000000', op.style?.width ?? 1);
    canvas.drawLine(op.x1, op.y1, op.x2, op.y2, paint);
    paint.delete?.();
  }

  private renderPath(canvas: SkCanvas, op: LayerPathOp): void {
    const path = new this.canvasKit.Path() as MutablePath;
    let currentX = op.bbox.x;
    let currentY = op.bbox.y;
    for (const command of op.commands ?? []) {
      [currentX, currentY] = this.applyPathCommand(path, command, currentX, currentY);
    }
    const style = op.style ?? {
      strokeColor: op.lineStyle?.color ?? '#000000',
      strokeWidth: op.lineStyle?.width ?? 1,
      fillColor: null,
    };

    // [Task #1067] HWPX/HWP 도형의 회전 + flip 변환 적용.
    // Rust paint pipeline (src/paint/json.rs::write_transform) 이 emit 하는
    // {"rotation": <degrees>, "horzFlip": <bool>, "vertFlip": <bool>} 매핑.
    // renderTextRun (line 410-416) 패턴 정합.
    const tr = op.transform;
    const rotation = tr?.rotation ?? 0;
    const horzFlip = tr?.horzFlip ?? false;
    const vertFlip = tr?.vertFlip ?? false;
    const needsTransform = rotation !== 0 || horzFlip || vertFlip;
    if (needsTransform) {
      const cx = op.bbox.x + (op.bbox.width ?? 0) / 2;
      const cy = op.bbox.y + (op.bbox.height ?? 0) / 2;
      canvas.save();
      if (horzFlip || vertFlip) {
        canvas.translate(cx, cy);
        canvas.scale(horzFlip ? -1 : 1, vertFlip ? -1 : 1);
        canvas.translate(-cx, -cy);
      }
      if (rotation !== 0) {
        canvas.rotate(rotation, cx, cy);
      }
    }
    this.drawStyledPath(canvas, path, style);
    if (needsTransform) {
      canvas.restore();
    }
    path.delete?.();
  }

  private applyPathCommand(path: MutablePath, command: LayerPathCommand, currentX: number, currentY: number): [number, number] {
    switch (command.type) {
      case 'moveTo':
        path.moveTo(command.x, command.y);
        return [command.x, command.y];
      case 'lineTo':
        path.lineTo(command.x, command.y);
        return [command.x, command.y];
      case 'curveTo':
        path.cubicTo(command.x1, command.y1, command.x2, command.y2, command.x3, command.y3);
        return [command.x3, command.y3];
      case 'arcTo':
        if (typeof path.arcToRotated === 'function') {
          path.arcToRotated(command.rx, command.ry, command.rotation, command.largeArc, command.sweep, command.x, command.y);
        } else {
          path.lineTo(command.x, command.y);
        }
        return [command.x, command.y];
      case 'closePath':
        path.close();
        return [currentX, currentY];
    }
  }

  private renderImage(canvas: SkCanvas, op: LayerImageOp): void {
    const image = this.imageForOp(op);
    if (!image) {
      this.unsupportedOps.add('image:dataMissing');
      return;
    }
    this.recordImageCoverageGaps(op);
    this.withImageTransform(canvas, op.bbox, op.transform, () => this.drawImageOp(canvas, image, op));
  }

  private renderGlyphOutline(canvas: SkCanvas, op: LayerGlyphOutlineOp): void {
    const graph = op.colorLayers?.paintGraph;
    const nodes = graph?.nodes ?? [];
    if (!graph || nodes.length === 0 || graph.rootNodeId === undefined) {
      this.unsupportedOps.add('glyphOutline:replayInvariant');
      return;
    }
    const nodesById = new Map<number, LayerColorGraphNode>();
    for (const node of nodes) {
      if (node.nodeId !== undefined) {
        nodesById.set(node.nodeId, node);
      }
    }
    canvas.save();
    const matrix = this.affineToCanvasKitMatrix(op.placement?.runToPage);
    if (matrix) {
      (canvas as unknown as { concat?: (matrix: number[]) => void }).concat?.(matrix);
    }
    try {
      this.renderColorPaintGraphNode(canvas, nodesById, graph.rootNodeId, new Set());
    } finally {
      canvas.restore();
    }
  }

  private renderColorPaintGraphNode(
    canvas: SkCanvas,
    nodesById: Map<number, LayerColorGraphNode>,
    nodeId: number,
    visited: Set<number>,
  ): void {
    if (visited.has(nodeId)) {
      this.unsupportedOps.add('glyphOutline:replayInvariant');
      return;
    }
    visited.add(nodeId);
    const node = nodesById.get(nodeId);
    if (!node) {
      this.unsupportedOps.add('glyphOutline:replayInvariant');
      return;
    }
    if (node.kind === 'transform') {
      const transformNode = node.transform;
      const matrix = this.affineToCanvasKitMatrix(transformNode?.transform);
      if (!matrix || transformNode?.childNodeId === undefined) {
        this.unsupportedOps.add('glyphOutline:replayInvariant');
        return;
      }
      canvas.save();
      (canvas as unknown as { concat?: (matrix: number[]) => void }).concat?.(matrix);
      try {
        this.renderColorPaintGraphNode(canvas, nodesById, transformNode.childNodeId, visited);
      } finally {
        canvas.restore();
      }
      return;
    }
    const pathNode = node.solidPath ?? node.linearGradientPath ?? node.radialGradientPath ?? node.sweepGradientPath;
    if (!pathNode?.commands) {
      this.unsupportedOps.add('glyphOutline:replayInvariant');
      return;
    }
    const path = new this.canvasKit.Path() as MutablePath;
    let currentX = 0;
    let currentY = 0;
    for (const command of pathNode.commands) {
      [currentX, currentY] = this.applyPathCommand(path, command, currentX, currentY);
    }
    this.applyFillRule(path, pathNode.fillRule);
    const paint = new this.canvasKit.Paint();
    let shader: unknown | undefined;
    try {
      paint.setAntiAlias?.(true);
      paint.setStyle(this.canvasKit.PaintStyle.Fill);
      if (node.kind === 'solidPath' && node.solidPath?.fill) {
        paint.setColor(this.resolvedColor(node.solidPath.fill));
      } else if (node.kind === 'linearGradientPath' && node.linearGradientPath?.gradient) {
        shader = this.makeLinearGradientShader(node.linearGradientPath.gradient);
        if (!shader) {
          return;
        }
        (paint as unknown as { setShader: (shader: unknown) => void }).setShader(shader);
      } else if (node.kind === 'radialGradientPath' && node.radialGradientPath?.gradient) {
        shader = this.makeRadialGradientShader(node.radialGradientPath.gradient);
        if (!shader) {
          return;
        }
        (paint as unknown as { setShader: (shader: unknown) => void }).setShader(shader);
      } else if (node.kind === 'sweepGradientPath' && node.sweepGradientPath?.gradient) {
        shader = this.makeSweepGradientShader(node.sweepGradientPath.gradient);
        if (!shader) {
          return;
        }
        (paint as unknown as { setShader: (shader: unknown) => void }).setShader(shader);
      } else {
        return;
      }
      canvas.drawPath(path, paint);
    } finally {
      (shader as { delete?: () => void } | undefined)?.delete?.();
      paint.delete?.();
      path.delete?.();
    }
  }

  private affineToCanvasKitMatrix(transform: LayerAffineTransform | undefined): number[] | null {
    if (!transform) return null;
    return [
      transform.a,
      transform.c,
      transform.e,
      transform.b,
      transform.d,
      transform.f,
      0,
      0,
      1,
    ];
  }

  private applyFillRule(path: MutablePath, fillRule: string | undefined): void {
    if (fillRule === 'evenodd') {
      (path as unknown as { setFillType?: (fillType: unknown) => void }).setFillType?.(this.canvasKit.FillType.EvenOdd);
    }
  }

  private resolvedColor(color: { rgba?: number[] }): Color {
    const rgba = color.rgba ?? [0, 0, 0, 1];
    return this.canvasKit.Color(
      clampUnit(rgba[0]),
      clampUnit(rgba[1]),
      clampUnit(rgba[2]),
      clampUnit(rgba[3]),
    );
  }

  private makeLinearGradientShader(gradient: NonNullable<LayerColorGraphNode['linearGradientPath']>['gradient']): unknown {
    const shaderApi = this.canvasKit.Shader as unknown as { MakeLinearGradient?: (...args: unknown[]) => unknown };
    return shaderApi.MakeLinearGradient?.(
      [gradient?.x0 ?? 0, gradient?.y0 ?? 0],
      [gradient?.x1 ?? 0, gradient?.y1 ?? 0],
      gradientColors(gradient?.stops),
      gradientPositions(gradient?.stops),
      this.canvasKit.TileMode.Clamp,
    );
  }

  private makeRadialGradientShader(gradient: NonNullable<LayerColorGraphNode['radialGradientPath']>['gradient']): unknown {
    const shaderApi = this.canvasKit.Shader as unknown as { MakeRadialGradient?: (...args: unknown[]) => unknown };
    return shaderApi.MakeRadialGradient?.(
      [gradient?.cx ?? 0, gradient?.cy ?? 0],
      gradient?.radius ?? 1,
      gradientColors(gradient?.stops),
      gradientPositions(gradient?.stops),
      this.canvasKit.TileMode.Clamp,
    );
  }

  private makeSweepGradientShader(gradient: NonNullable<LayerColorGraphNode['sweepGradientPath']>['gradient']): unknown {
    const shaderApi = this.canvasKit.Shader as unknown as { MakeSweepGradient?: (...args: unknown[]) => unknown };
    return shaderApi.MakeSweepGradient?.(
      gradient?.cx ?? 0,
      gradient?.cy ?? 0,
      gradientColors(gradient?.stops),
      gradientPositions(gradient?.stops),
      this.canvasKit.TileMode.Clamp,
      null,
      0,
      gradient?.startAngleDegrees ?? 0,
      gradient?.endAngleDegrees ?? 360,
    );
  }

  private drawImageOp(canvas: SkCanvas, image: SkImage, op: LayerImageOp): void {
    const imageWithDimensions = image as SkImage & { width?: unknown; height?: unknown };
    const widthMember = imageWithDimensions.width;
    const heightMember = imageWithDimensions.height;
    const imageWidth = typeof widthMember === 'function'
      ? (widthMember as () => number).call(image)
      : typeof widthMember === 'number'
        ? widthMember
        : null;
    const imageHeight = typeof heightMember === 'function'
      ? (heightMember as () => number).call(image)
      : typeof heightMember === 'number'
        ? heightMember
        : null;
    if (!this.boundsAreDrawable(op.bbox)) {
      this.unsupportedOps.add('image:invalidBounds');
      return;
    }
    if (
      imageWidth === null
      || imageHeight === null
      || !Number.isFinite(imageWidth)
      || !Number.isFinite(imageHeight)
      || imageWidth <= 0
      || imageHeight <= 0
    ) {
      const paint = new this.canvasKit.Paint();
      paint.setAntiAlias?.(true);
      try {
        canvas.drawImage(image, op.bbox.x, op.bbox.y, paint);
        this.unsupportedOps.add('image:dimensionUnavailable');
      } finally {
        paint.delete?.();
      }
      return;
    }

    const crop = canvasKitImageSourceRect(imageWidth, imageHeight, op.crop);
    const opacity = Number.isFinite(op.opacity) ? Math.max(0, Math.min(1, op.opacity ?? 1)) : 1;
    const drawImage = (dstX: number, dstY: number, dstW: number, dstH: number) => {
      const src = crop
        ? this.canvasKit.XYWHRect(crop.x, crop.y, crop.width, crop.height)
        : this.canvasKit.XYWHRect(0, 0, imageWidth, imageHeight);
      this.drawImageRect(canvas, image, src, this.canvasKit.XYWHRect(dstX, dstY, dstW, dstH), opacity);
    };

    const fillMode = op.fillMode ?? 'fitToSize';
    if (fillMode === 'fitToSize') {
      drawImage(op.bbox.x, op.bbox.y, op.bbox.width, op.bbox.height);
      return;
    }

    let tileWidth = op.originalSize?.width ?? imageWidth;
    let tileHeight = op.originalSize?.height ?? imageHeight;
    if (!Number.isFinite(tileWidth) || tileWidth <= 0) tileWidth = imageWidth;
    if (!Number.isFinite(tileHeight) || tileHeight <= 0) tileHeight = imageHeight;

    canvas.save();
    try {
      canvas.clipRect(this.rect(op.bbox), this.canvasKit.ClipOp?.Intersect ?? 0, true);
      if (canvasKitImageFillModeTiles(fillMode)) {
        this.drawTiledImage(canvas, op.bbox, fillMode, tileWidth, tileHeight, drawImage);
      } else {
        const placed = canvasKitImagePlacement(fillMode, op.bbox, tileWidth, tileHeight);
        drawImage(placed.x, placed.y, tileWidth, tileHeight);
      }
    } finally {
      canvas.restore();
    }
  }

  private drawImageRect(canvas: SkCanvas, image: SkImage, source: Rect, dest: Rect, opacity = 1): void {
    const paint = new this.canvasKit.Paint();
    paint.setAntiAlias?.(true);
    if (opacity < 1) {
      paint.setAlphaf(opacity);
    }
    try {
      canvas.drawImageRect(image, source, dest, paint);
    } finally {
      paint.delete?.();
    }
  }

  private drawTiledImage(
    canvas: SkCanvas,
    bbox: LayerBounds,
    fillMode: string,
    tileWidth: number,
    tileHeight: number,
    drawImage: (dstX: number, dstY: number, dstW: number, dstH: number) => void,
  ): void {
    const maxTileDraws = CanvasKitLayerRenderer.MAX_IMAGE_TILE_DRAWS;
    let tileDraws = 0;
    const drawTile = (x: number, y: number) => {
      if (tileDraws >= maxTileDraws) return;
      drawImage(x, y, tileWidth, tileHeight);
      tileDraws += 1;
    };

    if (fillMode === 'tileAll') {
      for (let y = bbox.y; y < bbox.y + bbox.height && tileDraws < maxTileDraws; y += tileHeight) {
        for (let x = bbox.x; x < bbox.x + bbox.width && tileDraws < maxTileDraws; x += tileWidth) {
          drawTile(x, y);
        }
      }
    } else if (fillMode === 'tileHorzTop' || fillMode === 'tileHorzBottom') {
      const y = fillMode === 'tileHorzTop' ? bbox.y : bbox.y + bbox.height - tileHeight;
      for (let x = bbox.x; x < bbox.x + bbox.width && tileDraws < maxTileDraws; x += tileWidth) {
        drawTile(x, y);
      }
    } else {
      const x = fillMode === 'tileVertLeft' ? bbox.x : bbox.x + bbox.width - tileWidth;
      for (let y = bbox.y; y < bbox.y + bbox.height && tileDraws < maxTileDraws; y += tileHeight) {
        drawTile(x, y);
      }
    }

    if (tileDraws >= maxTileDraws) {
      this.unsupportedOps.add('image:tileLimit');
    }
  }

  private withImageTransform(
    canvas: SkCanvas,
    bounds: LayerBounds,
    transform: LayerImageOp['transform'],
    draw: () => void,
  ): void {
    const rotation = transform?.rotation ?? 0;
    const horzFlip = transform?.horzFlip ?? false;
    const vertFlip = transform?.vertFlip ?? false;
    if (rotation === 0 && !horzFlip && !vertFlip) {
      draw();
      return;
    }

    const cx = bounds.x + bounds.width / 2;
    const cy = bounds.y + bounds.height / 2;
    canvas.save();
    try {
      if (horzFlip || vertFlip) {
        canvas.translate(cx, cy);
        canvas.scale(horzFlip ? -1 : 1, vertFlip ? -1 : 1);
        canvas.translate(-cx, -cy);
      }
      if (rotation !== 0) {
        canvas.rotate(rotation, cx, cy);
      }
      draw();
    } finally {
      canvas.restore();
    }
  }

  private recordImageCoverageGaps(op: LayerImageOp): void {
    if (op.bakedWatermark) return;
    if (op.effect && op.effect !== 'realPic') {
      this.unsupportedOps.add(`imageEffect:${op.effect}`);
    }
    if ((op.brightness ?? 0) !== 0 || (op.contrast ?? 0) !== 0) {
      this.unsupportedOps.add('imageEffect:brightnessContrast');
    }
  }

  private recordTextRunCoverageGaps(op: LayerTextRunOp): void {
    const style = op.style ?? {};
    if (op.isVertical) {
      this.unsupportedOps.add('textRun:verticalText');
    }
    if (style.underline && style.underline !== 'none') {
      this.unsupportedOps.add('textRun:textDecoration');
    }
    if (style.strikethrough) {
      this.unsupportedOps.add('textRun:textDecoration');
    }
    if (style.emphasisDot && style.emphasisDot !== 0) {
      this.unsupportedOps.add('textRun:emphasisDot');
    }
    if (style.outlineType && style.outlineType !== 0) {
      this.unsupportedOps.add('textRun:outlineTextEffect');
    }
    if (style.shadowType && style.shadowType !== 0) {
      this.unsupportedOps.add('textRun:shadowTextEffect');
    }
    if (style.emboss) {
      this.unsupportedOps.add('textRun:embossTextEffect');
    }
    if (style.engrave) {
      this.unsupportedOps.add('textRun:engraveTextEffect');
    }
    if (style.shadeColor && style.shadeColor.toLowerCase() !== '#ffffff') {
      this.unsupportedOps.add('textRun:shadeTextEffect');
    }
    if (style.ratio !== undefined && Math.abs(style.ratio - 1) > Number.EPSILON) {
      this.unsupportedOps.add('textRun:ratioTextEffect');
    }
  }

  private boundsAreDrawable(bounds: LayerBounds): boolean {
    return Number.isFinite(bounds.x)
      && Number.isFinite(bounds.y)
      && Number.isFinite(bounds.width)
      && Number.isFinite(bounds.height)
      && bounds.width > 0
      && bounds.height > 0;
  }

  private renderTextRun(canvas: SkCanvas, op: LayerTextRunOp): void {
    const replayText = op.displayText ?? op.text;
    const replayPositions = op.displayText !== undefined ? op.displayPositions : op.positions;
    if (!replayText) return;
    const style = op.style ?? {};
    this.recordTextRunCoverageGaps(op);
    const paint = this.makeFillPaint(style.color ?? '#000000');
    const baseFontSize = style.fontSize ?? Math.max(1, op.bbox.height || 12);
    let fontSize = baseFontSize;
    let baselineShift = 0;
    if (style.superscript) {
      fontSize = baseFontSize * 0.7;
      baselineShift -= baseFontSize * 0.3;
    } else if (style.subscript) {
      fontSize = baseFontSize * 0.7;
      baselineShift += baseFontSize * 0.15;
    }
    const placementMatrix = this.affineToCanvasKitMatrix(op.placement?.runToPage);
    const originX = placementMatrix ? 0 : op.bbox.x;
    const originY = placementMatrix
      ? (op.placement?.baselineY ?? 0)
      : op.bbox.y + (op.baseline ?? baseFontSize);
    const rotation = op.rotation ?? 0;
    const codePoints = Array.from(replayText);
    const needsPreservedAdvances = style.superscript || style.subscript;
    const hasSimpleScriptText = codePoints.every((codePoint) => {
      const code = codePoint.charCodeAt(0);
      return codePoint.length === 1 && code >= 0x20 && code <= 0x7e;
    });
    const hasLayoutPositions = replayPositions?.length === codePoints.length + 1
      && replayPositions.every(Number.isFinite);
    const localTypeface = this.findLocalTypeface(style.fontFamily);
    const typeface = localTypeface?.typeface ?? this.defaultTypeface;
    const fontManager = localTypeface?.fontManager ?? this.defaultFontManager;
    const fontFamily = localTypeface?.fontFamily ?? this.defaultFontFamily;
    let font: Font | null = null;
    let canvasSaved = false;
    try {
      paint.setAntiAlias?.(true);
      if (!typeface && !fontManager && /[^\u0000-\u00ff]/.test(replayText)) {
        this.unsupportedOps.add('textRunFont');
        return;
      }
      canvas.save();
      canvasSaved = true;
      if (placementMatrix) {
        canvas.concat(placementMatrix);
      } else if (rotation !== 0) {
        canvas.rotate(rotation, originX, originY);
      }

      if (needsPreservedAdvances && !hasSimpleScriptText) {
        if (!this.renderShapedScriptText(
          canvas,
          replayText,
          style.color ?? '#000000',
          fontSize,
          originX,
          originY,
          baselineShift,
          fontManager,
          fontFamily,
        )) {
          this.unsupportedOps.add('textRun:scriptTextRequiresShaping');
        }
      } else {
        font = new this.canvasKit.Font(typeface, fontSize);
        if (needsPreservedAdvances && hasLayoutPositions) {
          const glyphIds = font.getGlyphIDs(replayText, codePoints.length);
          const hasGlyphMapping = glyphIds.length === codePoints.length
            && glyphIds.every((glyphId) => glyphId !== 0);
          if (hasGlyphMapping) {
            const glyphPositions = new Float32Array(codePoints.length * 2);
            for (let index = 0; index < codePoints.length; index += 1) {
              glyphPositions[index * 2] = replayPositions![index];
              glyphPositions[index * 2 + 1] = baselineShift;
            }
            canvas.drawGlyphs(glyphIds, glyphPositions, originX, originY, font, paint);
          } else {
            this.unsupportedOps.add('textRun:glyphMapping');
            canvas.drawText(replayText, originX, originY + baselineShift, paint, font);
          }
        } else if (needsPreservedAdvances) {
          this.unsupportedOps.add('textRun:layoutPositions');
          canvas.drawText(replayText, originX, originY + baselineShift, paint, font);
        } else {
          canvas.drawText(replayText, originX, originY, paint, font);
        }
      }
    } finally {
      try {
        if (canvasSaved) canvas.restore();
      } finally {
        font?.delete?.();
        paint.delete?.();
      }
    }
  }

  private renderShapedScriptText(
    canvas: SkCanvas,
    text: string,
    color: string,
    fontSize: number,
    originX: number,
    originY: number,
    baselineShift: number,
    fontManager: FontMgr | null,
    fontFamily: string | null,
  ): boolean {
    if (!fontManager) return false;
    const textStyle = {
      color: this.color(color),
      fontSize,
      ...(fontFamily ? { fontFamilies: [fontFamily] } : {}),
    };
    const paragraphStyle = new this.canvasKit.ParagraphStyle({
      maxLines: 1,
      textStyle,
    });
    const builder = this.canvasKit.ParagraphBuilder.Make(paragraphStyle, fontManager);
    try {
      builder.addText(text);
      const paragraph = builder.build();
      try {
        paragraph.layout(CanvasKitLayerRenderer.MAX_SHAPED_TEXT_WIDTH);
        canvas.drawParagraph(paragraph, originX, originY - fontSize + baselineShift);
        return true;
      } finally {
        paragraph.delete?.();
      }
    } finally {
      builder.delete?.();
    }
  }

  private findLocalTypeface(fontFamily: string | undefined): CanvasKitLocalTypeface | null {
    if (!fontFamily) return null;
    const record = resolveLocalFont(fontFamily);
    return record ? this.localTypefaces.get(localFontFaceKey(record)) ?? null : null;
  }

  private renderFormObject(canvas: SkCanvas, op: LayerFormObjectOp): void {
    const fill = op.backColor && op.backColor !== '#000000' ? op.backColor : '#f7f7f7';
    this.drawStyledShape(canvas, op.bbox, {
      fillColor: fill,
      strokeColor: op.foreColor ?? '#555555',
      strokeWidth: 1,
      opacity: op.enabled === false ? 0.55 : 1,
    }, (paint) => canvas.drawRect(this.rect(op.bbox), paint));
    if (op.value && (op.formType === 'checkbox' || op.formType === 'radio')) {
      const paint = this.makeStrokePaint(op.foreColor ?? '#111111', 1.5);
      const b = op.bbox;
      canvas.drawLine(b.x + b.width * 0.25, b.y + b.height * 0.55, b.x + b.width * 0.45, b.y + b.height * 0.75, paint);
      canvas.drawLine(b.x + b.width * 0.45, b.y + b.height * 0.75, b.x + b.width * 0.78, b.y + b.height * 0.28, paint);
      paint.delete?.();
    }
    const label = op.caption || op.text;
    if (label) {
      this.renderTextRun(canvas, {
        type: 'textRun',
        bbox: { ...op.bbox, x: op.bbox.x + 4, width: Math.max(0, op.bbox.width - 8) },
        text: label,
        baseline: Math.max(10, op.bbox.height * 0.68),
        style: { fontSize: Math.max(9, Math.min(14, op.bbox.height * 0.55)), color: op.foreColor ?? '#111111' },
      });
    }
  }

  private renderPlaceholder(canvas: SkCanvas, op: LayerPlaceholderOp): void {
    this.drawStyledShape(canvas, op.bbox, {
      fillColor: op.fillColor ?? '#f2f2f2',
      strokeColor: op.strokeColor ?? '#999999',
      strokeWidth: 1,
    }, (paint) => canvas.drawRect(this.rect(op.bbox), paint));
    if (op.label) {
      this.renderTextRun(canvas, {
        type: 'textRun',
        bbox: { ...op.bbox, x: op.bbox.x + 4 },
        text: op.label,
        baseline: Math.max(10, op.bbox.height * 0.65),
        style: { fontSize: Math.max(9, Math.min(14, op.bbox.height * 0.45)), color: '#555555' },
      });
    }
  }

  private drawStyledShape(
    canvas: SkCanvas,
    bounds: LayerBounds,
    style: LayerShapeStyle | undefined,
    draw: (paint: SkPaint) => void,
  ): void {
    if (style?.fillColor) {
      const paint = this.makeFillPaint(style.fillColor, style.opacity);
      draw(paint);
      paint.delete?.();
    }
    if (style?.strokeColor && (style.strokeWidth ?? 0) > 0) {
      const paint = this.makeStrokePaint(style.strokeColor, style.strokeWidth ?? 1, style.opacity);
      draw(paint);
      paint.delete?.();
    }
    if (!style?.fillColor && !style?.strokeColor) {
      const paint = this.makeStrokePaint('#000000', 1);
      draw(paint);
      paint.delete?.();
    }
  }

  private drawStyledPath(canvas: SkCanvas, path: Path, style: LayerShapeStyle): void {
    let drawn = false;
    if (style.fillColor) {
      const paint = this.makeFillPaint(style.fillColor, style.opacity);
      canvas.drawPath(path, paint);
      paint.delete?.();
      drawn = true;
    }
    if (style.strokeColor && (style.strokeWidth ?? 0) > 0) {
      const paint = this.makeStrokePaint(style.strokeColor, style.strokeWidth ?? 1, style.opacity);
      canvas.drawPath(path, paint);
      paint.delete?.();
      drawn = true;
    }
    if (!drawn) {
      const paint = this.makeStrokePaint('#000000', 1);
      canvas.drawPath(path, paint);
      paint.delete?.();
    }
  }

  private imageForOp(op: LayerImageOp): SkImage | null {
    const key = canvasKitImageCacheKey(op);
    if (!key) return null;
    const cached = this.imageCache.get(key);
    if (cached) return cached;
    const bytes = base64ToBytes(op.base64 ?? '');
    const image = this.canvasKit.MakeImageFromEncoded(bytes);
    if (!image) return null;
    this.imageCache.set(key, image);
    return image;
  }

  private makeFillPaint(color: string, opacity = 1): SkPaint {
    const paint = new this.canvasKit.Paint();
    paint.setAntiAlias?.(true);
    paint.setStyle(this.canvasKit.PaintStyle.Fill);
    paint.setColor(this.color(color, opacity));
    return paint;
  }

  private makeStrokePaint(color: string, width: number, opacity = 1): SkPaint {
    const paint = new this.canvasKit.Paint();
    paint.setAntiAlias?.(true);
    paint.setStyle(this.canvasKit.PaintStyle.Stroke);
    paint.setStrokeWidth(Math.max(0.1, width));
    paint.setColor(this.color(color, opacity));
    return paint;
  }

  private rect(bounds: LayerBounds): Rect {
    return this.canvasKit.XYWHRect(bounds.x, bounds.y, bounds.width, bounds.height);
  }

  private color(cssColor: string, opacity = 1): Color {
    const { r, g, b, a } = parseCssColor(cssColor);
    const alpha = Math.max(0, Math.min(1, a * opacity));
    return this.canvasKit.Color(r, g, b, alpha);
  }
}

function parseCssColor(value: string): { r: number; g: number; b: number; a: number } {
  const trimmed = value.trim();
  if (trimmed === 'transparent') {
    return { r: 0, g: 0, b: 0, a: 0 };
  }
  if (trimmed === 'black') {
    return { r: 0, g: 0, b: 0, a: 1 };
  }
  if (trimmed === 'white') {
    return { r: 255, g: 255, b: 255, a: 1 };
  }
  const shortHex = /^#?([0-9a-f]{3,4})$/i.exec(trimmed);
  if (shortHex) {
    const value = shortHex[1];
    return {
      r: Number.parseInt(value[0] + value[0], 16),
      g: Number.parseInt(value[1] + value[1], 16),
      b: Number.parseInt(value[2] + value[2], 16),
      a: value.length === 4 ? Number.parseInt(value[3] + value[3], 16) / 255 : 1,
    };
  }
  const hexWithAlpha = /^#?([0-9a-f]{8})$/i.exec(trimmed);
  if (hexWithAlpha) {
    const n = Number.parseInt(hexWithAlpha[1], 16);
    return {
      r: (n >> 24) & 0xff,
      g: (n >> 16) & 0xff,
      b: (n >> 8) & 0xff,
      a: (n & 0xff) / 255,
    };
  }
  const hex = /^#?([0-9a-f]{6})$/i.exec(trimmed);
  if (hex) {
    const n = Number.parseInt(hex[1], 16);
    return {
      r: (n >> 16) & 0xff,
      g: (n >> 8) & 0xff,
      b: n & 0xff,
      a: 1,
    };
  }
  const rgb = /^rgba?\((\d+),\s*(\d+),\s*(\d+)(?:,\s*([0-9.]+))?\)$/i.exec(trimmed);
  if (rgb) {
    return {
      r: Number(rgb[1]),
      g: Number(rgb[2]),
      b: Number(rgb[3]),
      a: rgb[4] === undefined ? 1 : Number(rgb[4]),
    };
  }
  return { r: 0, g: 0, b: 0, a: 1 };
}

function clampUnit(value: number | undefined): number {
  return Math.max(0, Math.min(1, Number.isFinite(value) ? value ?? 0 : 0));
}

function gradientColors(stops: Array<{ color?: { rgba?: number[] } }> | undefined): number[][] {
  return (stops ?? []).map((stop) => {
    const rgba = stop.color?.rgba ?? [0, 0, 0, 1];
    return [
      clampUnit(rgba[0]),
      clampUnit(rgba[1]),
      clampUnit(rgba[2]),
      clampUnit(rgba[3]),
    ];
  });
}

function gradientPositions(stops: Array<{ offset?: number }> | undefined): number[] {
  return (stops ?? []).map((stop) => Math.max(0, Math.min(1, stop.offset ?? 0)));
}

function base64ToBytes(base64: string): Uint8Array {
  const binary = atob(base64);
  const bytes = new Uint8Array(binary.length);
  for (let i = 0; i < binary.length; i += 1) {
    bytes[i] = binary.charCodeAt(i);
  }
  return bytes;
}
