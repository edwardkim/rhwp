import { CanvasKitBundledFontFamily } from './canvaskit/font-style-selection.ts';
import { textRequiresComplexShaping } from './canvaskit/nominal-text-replay.ts';
import CanvasKitInit from 'canvaskit-wasm';
import { blake3 } from '@noble/hashes/blake3.js';
import type {
  Canvas,
  CanvasKit,
  Color,
  Font,
  FontMgr,
  Image as SkImage,
  Paint,
  Paragraph,
  Path,
  PathBuilder,
  Rect,
  Surface,
  SkPicture,
  TextBlob,
  Typeface,
  WebGPUDeviceContext,
} from 'canvaskit-wasm';
import canvaskitWasmUrl from '@/view/canvaskit-wasm-url';

import type {
  LayerBounds,
  LayerCharOverlapOp,
  LayerClipNode,
  LayerEllipseOp,
  LayerEquationLayoutBox,
  LayerEquationOp,
  LayerFormObjectOp,
  LayerGradientFill,
  LayerAffineTransform,
  LayerGlyphOutlineOp,
  LayerGlyphRunOp,
  LayerFontResources,
  LayerImageOp,
  LayerInfo,
  LayerLeafNode,
  LayerLineOp,
  LayerLineStyle,
  LayerNode,
  LayerPageBackgroundOp,
  LayerPaintOp,
  LayerPathCommand,
  LayerPathOp,
  LayerPlaceholderOp,
  LayerRectangleOp,
  LayerRenderProfile,
  LayerResources,
  LayerShapeStyle,
  LayerStrokeDash,
  LayerTabLeaderOp,
  LayerTextControlMarkOp,
  LayerTextDecorationOp,
  LayerTextRunOp,
  LayerTextStyle,
  PageInfo,
  PageLayerTree,
  PageLayerTreeV2,
} from '@/core/types';
import { normalizeTextV2Tree } from '@/core/text-v2-normalizer';
import {
  DEFAULT_CANVASKIT_SURFACE_REQUEST,
  type CanvasKitRenderMode,
  type CanvasKitSurfacePreference,
  type CanvasKitSurfaceRequest,
} from './render-backend';
import { imageCropSelectionIsEmpty } from './image-crop-scale.ts';
import {
  boundedCanvasKitSourceImageKey,
  canvasKitImageCacheKey,
  canvasKitImageContainRect,
  canvasKitImageFillModeContains,
  canvasKitImageFillModeTiles,
  canvasKitImageFillModeStretches,
  canvasKitImagePlacement,
  canvasKitImageSourceRect,
} from './canvaskit/image-replay';
import {
  CANVASKIT_MAX_ENCODED_IMAGE_BASE64_LENGTH,
  decodedImageMatchesEncodedHeader,
  encodedImageHasStableFrame,
  replayableEncodedImageHeader,
} from './canvaskit/image-header';
import {
  canvasKitCanonicalClipEnabled,
  canvasKitCanonicalClipRect,
} from './canvaskit/clip-replay';
import {
  canvasKitCanvasSupportsGlyphRunReplay,
  CanvasKitGlyphRunFontCache,
  drawCanvasKitGlyphRun,
  type FontBytesResolver,
} from './canvaskit/glyph-run-fonts';
import {
  selectLayerTextVariantsForLeaf,
  staticSvgPathLayersAreReplayable,
} from './canvaskit/text-variant-selection';
import {
  CANVASKIT_REPLAY_PLANES,
  type CanvasKitReplayPlane,
  layerPaintOpReplayPlane,
} from './canvaskit/replay-plane';
import { isExpectedCanvasKitUnsupportedOp } from './canvaskit/diagnostics';
import { layerResourceKeyMatches } from './canvaskit/resource-key';
import {
  glyphOutlinePaintIsSupported,
  glyphOutlinePayloadResourceKey,
  glyphOutlinePayloadStatus,
} from './glyph-outline-payload-status';
import {
  parseStaticSvgPathLayers,
  parseStaticSvgTextLayers,
  type StaticSvgPathLayer,
  type StaticSvgTextLayer,
} from './static-svg-path-layers';
import { getLocalFontRecords, loadLocalFontBytesFor, localFontFaceKey, resolveLocalFont, resolveRendererLocalFont,
  hasHostFontProvider, loadRendererLocalFont, getHostFontState, type LocalFontRecord } from '@/core/local-fonts';
import { hostFontStyle } from '@/core/host-font-requests';
import { canvasKitFontFaceData } from './canvaskit/sfnt-face';
import { projectedSubstituteTargets } from '@/core/font-rule-runtime';
import type { CanvasKitBundledFontSource } from '@/core/font-loader';
import type { FontDecisionTraceRecordV1 } from '@/core/font-decision-trace';
import { readBoundedResponseArrayBuffer } from './canvaskit/bounded-response';
import { localFontFaceStyle } from './canvaskit/local-font-style';
import { canvasKitEffectCrop, canvasKitImageEffectKey, makeCanvasKitEffectImage } from './canvaskit/image-effects';
import { BoundedReplayLog, emptyImageEffectDiagnostics,
  type CanvasKitEquationReplayDiagnostic, type CanvasKitImageRecoveryDiagnostic,
  type CanvasKitTextReplayDiagnostic, type LayerTextVariantGroupReport,
  type LayerTextVariantReplayStatus } from './canvaskit/replay-diagnostics';
import { decodeBrowserImage } from './canvaskit/browser-image-decoder';
import { BoundedNativeCache } from './canvaskit/bounded-native-cache';
import { replayFingerprint } from './canvaskit/replay-fingerprint';
import { CanvasKitSurfaceCache } from './canvaskit/surface-cache';
import { formDrawingReplayRejection } from './canvaskit/form-replay';
import { CanvasKitParagraphFonts } from './canvaskit/paragraph-fonts';

type CanvasKitApi = CanvasKit;
type SkCanvas = Canvas;
type SkPaint = Paint;
type SkSurface = Surface;

export interface CanvasKitLayerRendererOptions {
  defaultFontUrl?: string;
  symbolFallbackFontUrl?: string;
  oldHangulFontUrl?: string;
  requirePreparedFontFamilies?: boolean;
}

const OLD_HANGUL_FONT_FAMILY = 'RHWP Old Hangul';
const VERTICAL_PRESENTATION_BASE_TEXT = new Map<string, string>([
  ['\uFE19', '\u2026'],
  ['\uFE31', '\u2014'],
  ['\uFE32', '\u2013'],
  ['\uFE33', '_'],
  ['\uFE34', '~'],
  ['\uFE35', '('],
  ['\uFE36', ')'],
  ['\uFE37', '{'],
  ['\uFE38', '}'],
  ['\uFE39', '['],
  ['\uFE3A', ']'],
  ['\uFE3B', '\u3010'],
  ['\uFE3C', '\u3011'],
  ['\uFE3D', '\u300A'],
  ['\uFE3E', '\u300B'],
  ['\uFE3F', '\u3008'],
  ['\uFE40', '\u3009'],
  ['\uFE41', '\u300C'],
  ['\uFE42', '\u300D'],
  ['\uFE43', '\u300E'],
  ['\uFE44', '\u300F'],
]);
type LayerColorGraph = NonNullable<NonNullable<LayerGlyphOutlineOp['colorLayers']>['paintGraph']>;
type LayerColorGraphNode = NonNullable<LayerColorGraph['nodes']>[number];
interface CanvasKitSurfaceTarget {
  surface: SkSurface;
  canvas: HTMLCanvasElement;
  releaseContext?: () => void;
}

interface CanvasKitLocalTypeface {
  bytes?: ArrayBuffer;
  paragraphTypefaces?: readonly (Typeface | null)[];
  syntheticStyle?: { bold: boolean; italic: boolean };
  hostFace?: { weight?: number; slant?: string };
  typeface: Typeface | null;
  fontManager: FontMgr | null;
  fontFamily: string | null;
  record?: LocalFontRecord;
  bytesDigest?: string;
}

function primaryFontFamily(value: string | null | undefined): string {
  return (value ?? '')
    .split(',')[0]
    .trim()
    .replace(/^(["'])|(["'])$/g, '');
}

function normalizedFontFamily(value: string | null | undefined): string {
  return primaryFontFamily(value)
    .replace(/\u0000/g, '')
    .normalize('NFC')
    .replace(/\s+/g, ' ')
    .trim()
    .toLocaleLowerCase('en-US');
}


function textRunHasPaintEffects(style: NonNullable<LayerTextRunOp['style']>): boolean {
  const shadeColor = (style.shadeColor ?? '#ffffff').toLowerCase();
  return (style.outlineType ?? 0) !== 0
    || (style.shadowType ?? 0) !== 0
    || style.emboss === true
    || style.engrave === true
    || (shadeColor !== '#ffffff' && shadeColor !== '#000000')
    || Math.abs((style.ratio ?? 1) - 1) > Number.EPSILON;
}

interface EquationRenderBudget {
  remainingNodes: number;
}

export interface CanvasKitReplayFeatureCounts {
  dashedStrokes: number;
  glyphRuns: number;
  tabLeaders: number;
  charOverlaps: number;
  verticalPresentationPunctuation: number;
  verticalTextRuns: number;
}

export interface CanvasKitRenderDiagnostics {
  pendingImageLoads: number;
  mode: CanvasKitRenderMode;
  surfacePreference: CanvasKitSurfacePreference;
  surfaceBackend: 'default' | 'software' | 'webgpu' | null;
  surfaceCreated: number;
  surfaceReused: number;
  staticPictureCacheEntries: number;
  staticPictureCacheBytes: number;
  staticPictureCacheHits: number;
  staticPictureCacheMisses: number;
  staticPictureCacheEvictions: number;
  textBlobCacheEntries: number;
  textBlobCacheBytes: number;
  textBlobCacheHits: number;
  textBlobCacheMisses: number;
  fallbackFamilyCacheEntries: number;
  paragraphFontEntries: number;
  paragraphFontBytes: number;
  paragraphProviderBuilds: number;
  paragraphFontRejections: number;
  surfaceFallbackReason: string | null;
  lastRenderCompleted: boolean;
  lastUnsupportedOps: string[];
  lastExpectedUnsupportedOps: string[];
  lastUnexpectedUnsupportedOps: string[];
  lastRenderError: string | null;
  passesRuntimeReadinessGate: boolean;
  readinessBlockers: CanvasKitReadinessBlocker[];
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
  mipmapCacheEntries: number;
  mipmapCacheBytes: number;
  mipmapCacheHits: number;
  mipmapCacheMisses: number;
  imageFailureCacheHits: number;
  imageFailures: CanvasKitImageFailureDiagnostic[];
  localTypefaceCount: number;
  localTypefaceLoadFailureCount: number;
  localTypefacePendingCount: number;
  bundledTypefaceCount: number;
  bundledTypefaceLoadFailureCount: number;
  glyphRunFontBlobCount: number;
  glyphRunFontBlobBytes: number;
  glyphRunTypefaceCount: number;
  glyphRunFontCount: number;
  fontSubstitutionLimit: number;
  unregisteredFontFallbacks: number;
  fontSubstitutions: CanvasKitFontSubstitutionDiagnostic[];
  replayFeatureCounts: CanvasKitReplayFeatureCounts;
  textVariantDiagnosticGroups: number;
  textVariantDiagnosticParts: number;
  replayDiagnosticsDropped: number;
}

export interface CanvasKitFontSubstitutionDiagnostic {
  opId?: string;
  requestedFamily: string;
  resolvedFamily: string;
  source: 'unregisteredDefault' | 'missingGlyphDefault' | 'missingGlyphSymbol' | 'oldHangul';
  kind: 'unregisteredFallback' | 'glyphCoverageFallback';
}

export interface CanvasKitFontDecisionEvidence {
  status: 'complete' | 'notObserved' | 'unsupported' | 'failed';
  certainty: 'observed' | 'resolved' | 'planned' | 'notObserved';
  requested: string;
  candidates: string[];
  resolved: string | null;
  source: 'local' | 'bundled' | 'default' | 'symbol' | 'glyphResource' | null;
  capabilities: string[];
  failures: string[];
}

export type CanvasKitImageFailureReason =
  | 'dataMissing'
  | 'cacheKeyMissing'
  | 'base64DecodeFailed'
  | 'encodedImageRejected'
  | 'imageDecodeFailed'
  | 'decodedDimensionsMismatch'
  | 'imageEffectFailed';

export interface CanvasKitImageFailureDiagnostic {
  source: 'sourceKey' | 'resource' | 'inline' | 'missing';
  sourceImageKey: string | null;
  imageRef: number | string | null;
  reason: CanvasKitImageFailureReason;
}

interface StaticPictureDiagnostics {
  unsupported: string[];
  images: Array<[string, CanvasKitImageFailureDiagnostic]>;
  substitutions: CanvasKitFontSubstitutionDiagnostic[];
  features: CanvasKitReplayFeatureCounts;
  equations: CanvasKitEquationReplayDiagnostic[];
  textRecoveries: CanvasKitTextReplayDiagnostic[];
  textFailures: CanvasKitTextReplayDiagnostic[];
  imageRecoveries: CanvasKitImageRecoveryDiagnostic[];
  patternOps: number;
}

interface StaticPictureEntry {
  picture: SkPicture;
  treeFingerprint: string;
  diagnostics: StaticPictureDiagnostics;
}

export type CanvasKitReadinessBlocker =
  | 'renderNotCompleted'
  | 'renderError'
  | 'unexpectedUnsupportedOps'
  | 'imageReplayFailure'
  | 'localFontsPending';

export class CanvasKitLayerRenderer {
  // Prevent pathological tiled fills from monopolizing the render loop.
  private static readonly MAX_IMAGE_TILE_DRAWS = 4096;
  private static readonly MAX_IMAGE_CACHE_ENTRIES = 128;
  private static readonly MAX_IMAGE_FAILURE_CACHE_ENTRIES = 128;
  private static readonly MAX_SVG_GLYPH_CACHE_ENTRIES = 128;
  private static readonly MAX_IMAGE_CACHE_PIXELS = 64 * 1024 * 1024;
  private static readonly MAX_BITMAP_GLYPH_BASE64_LENGTH = Math.ceil(4 * 1024 * 1024 / 3) * 4;
  private static readonly MAX_STATIC_SVG_GLYPH_BYTES = 1024 * 1024;
  private static readonly MAX_PLACEHOLDER_DASH_SEGMENTS_PER_AXIS = 2048;
  private static readonly MAX_EQUATION_LAYOUT_DEPTH = 64;
  private static readonly MAX_EQUATION_LAYOUT_NODES = 4096;
  private static readonly MAX_EQUATION_TEXT_LENGTH = 4096;
  private static readonly MAX_TEXT_VISUAL_WAVE_SEGMENTS = 4096;
  private static readonly MAX_TEXT_SPECIAL_VISUAL_ITEMS = 4096;
  private static readonly MAX_TEXT_RUN_CODE_POINTS = 4096;
  private static readonly MAX_TEXT_RUN_FALLBACK_SPANS = 4096;
  private static readonly MAX_FONT_SUBSTITUTION_DIAGNOSTICS = 4096;
  // 단일 text run은 줄바꿈 없이 문서가 지정한 위치에 재생한다.
  private static readonly MAX_SHAPED_TEXT_WIDTH = 1_000_000;
  private static readonly MAX_BUNDLED_FONT_BYTES = 32 * 1024 * 1024;

  private readonly imageCache = new Map<string, { image: SkImage; pixels: number }>();
  private readonly mipmapImages = new BoundedNativeCache<SkImage>(32, 64 * 1024 * 1024, image => image.delete());
  private readonly imageNativeKeys = new WeakMap<SkImage, string>();
  private readonly croppedEffectImages = new WeakMap<SkImage, { width: number; height: number }>();
  private nextImageKey = 1;
  private currentProfile: LayerRenderProfile = 'screen';
  private preferRasterDepth = 0;
  private readonly surfaceCache = new CanvasKitSurfaceCache();
  private readonly staticPictures = new BoundedNativeCache<StaticPictureEntry>(32, 64 * 1024 * 1024, entry => entry.picture.delete());
  private readonly textBlobs = new BoundedNativeCache<TextBlob>(2048, 8 * 1024 * 1024, blob => blob.delete());
  private readonly failedTextBlobKeys = new Set<string>();
  private readonly typefaceIds = new WeakMap<Typeface, number>();
  private readonly fontTypefaceIds = new WeakMap<Font, number>();
  private nextTypefaceId = 1;
  private readonly fallbackFamilyCache = new Map<string, CanvasKitLocalTypeface | null>();
  private readonly treeFingerprints = new WeakMap<PageLayerTree, Set<string>>();
  private currentTreeFingerprint: string | null = null;
  private currentPictureContext: string | null = null;
  private currentPageExtent: LayerBounds | null = null;
  private recordingStaticPicture = false;
  private imageResourceRevision = 0;
  private readonly imageDecodeFailures = new Map<string, CanvasKitImageFailureReason>();
  private readonly pendingImageLoads = new Map<string, { promise: Promise<void>; cancel(): void }>();
  private readonly currentImageFailures = new Map<string, CanvasKitImageFailureDiagnostic>();
  private readonly svgGlyphPathCache = new Map<string, StaticSvgPathLayer[]>();
  private readonly svgGlyphParseFailures = new Set<string>();
  private readonly localTypefaces = new Map<string, CanvasKitLocalTypeface>();
  private readonly localTypefaceLoadFailures = new Set<string>();
  private readonly localTypefacePending = new Map<string, number>();
  private localFontPreparationEpoch = 0;
  private localFontRevision = 0;
  private readonly bundledTypefaces = new Map<string, CanvasKitLocalTypeface>();
  private readonly bundledTypefaceAliases = new Map<string, CanvasKitLocalTypeface[]>();
  private readonly bundledFontFamilies = new Map<string, CanvasKitBundledFontFamily>();
  private readonly bundledTypefaceLoadFailures = new Set<string>();
  private readonly currentFontSubstitutions = new Map<string, CanvasKitFontSubstitutionDiagnostic>();
  private readonly equationRoutes = new BoundedReplayLog<CanvasKitEquationReplayDiagnostic>();
  private readonly textRecoveries = new BoundedReplayLog<CanvasKitTextReplayDiagnostic>();
  private readonly textFailures = new BoundedReplayLog<CanvasKitTextReplayDiagnostic>();
  private readonly imageRecoveries = new BoundedReplayLog<CanvasKitImageRecoveryDiagnostic>();
  private readonly imageRecoveryByKey = new Map<string, CanvasKitImageRecoveryDiagnostic>();
  private readonly textVariantReports = new BoundedReplayLog<LayerTextVariantGroupReport>();
  private textVariantReportParts = 0;
  private textV2ValidationIssues: Array<{ code: string; severity: 'error'; message: string }> = [];
  private imageEffectDiagnostics = emptyImageEffectDiagnostics();
  private textBlobConstructionFailures = 0;
  private textBlobFailureCacheHits = 0;
  private textBlobFallbackDraws = 0;
  private patternReplayOps = 0;
  private currentOp: LayerPaintOp | null = null;
  private readonly bundledFontRequests = new Set<AbortController>();
  private readonly glyphRunFonts: CanvasKitGlyphRunFontCache;
  private readonly paragraphFonts: CanvasKitParagraphFonts;
  private readonly unsupportedOps = new Set<string>();
  private surfaceBackend: 'default' | 'software' | 'webgpu' | null = null;
  private surfaceFallbackReason: string | null = null;
  private lastRenderError: string | null = null;
  private lastRenderCompleted = false;
  private lastRenderDurationMs: number | null = null;
  private renderCount = 0;
  private imageCacheHits = 0;
  private imageCacheMisses = 0;
  private imageCacheEvictions = 0;
  private imageFailureCacheHits = 0;
  private imageCachePixels = 0;
  private imagesDecoded = 0;
  private imageFailureAttempts = 0;
  private pendingImageAccesses = 0;
  private equationSvgFailureReason: CanvasKitEquationReplayDiagnostic['reason'] = 'layoutRequested';
  private currentResources: LayerResources | undefined;
  private currentImageBytesResolver: ((key: string) => Uint8Array | null) | undefined;
  private currentFontResources: LayerFontResources | undefined;
  private currentShowParagraphMarks = false;
  private currentShowControlCodes = false;
  private currentClipEnabled = true;
  private currentReplayFeatureCounts: CanvasKitReplayFeatureCounts = {
    dashedStrokes: 0,
    glyphRuns: 0,
    tabLeaders: 0,
    charOverlaps: 0,
    verticalPresentationPunctuation: 0,
    verticalTextRuns: 0,
  };
  private selectedTextVariantOps = new WeakSet<LayerPaintOp>();
  private documentGeneration = 0;
  private sourceDocumentGeneration: number | undefined;
  private disposed = false;

  private constructor(
    private readonly canvasKit: CanvasKitApi,
    private readonly renderMode: CanvasKitRenderMode,
    private readonly surfaceRequest: CanvasKitSurfaceRequest,
    private readonly defaultTypeface: Typeface | null,
    private readonly symbolFallbackTypeface: Typeface | null,
    private readonly defaultFontManager: FontMgr | null = null,
    private readonly defaultFontFamily: string | null = null,
    private readonly defaultFontUrl: string = 'fonts/NotoSansKR-Regular.woff2',
    private readonly requirePreparedFontFamilies: boolean = false,
    private readonly oldHangulTypeface: CanvasKitLocalTypeface | null = null,
    private readonly oldHangulFontUrl: string = 'fonts/RHWPOldHangul-Regular.woff2',
    private readonly defaultFontBytes?: ArrayBuffer,
    private readonly webgpuDeviceContext: WebGPUDeviceContext | null = null,
    private readonly webgpuDevice: GPUDevice | null = null,
    private readonly webgpuInitFailure: string | null = null,
  ) {
    this.glyphRunFonts = new CanvasKitGlyphRunFontCache(canvasKit);
    this.paragraphFonts = new CanvasKitParagraphFonts(canvasKit);
    void this.webgpuDevice?.lost.then(() => {
      if (this.disposed) return;
      this.surfaceCache.clear();
      this.staticPictures.clear();
      this.surfaceFallbackReason = 'webgpuDeviceLost';
      this.lastRenderCompleted = false;
    });
  }

  static async create(
    renderMode: CanvasKitRenderMode = 'default',
    surfaceRequest: CanvasKitSurfaceRequest | CanvasKitSurfacePreference = DEFAULT_CANVASKIT_SURFACE_REQUEST,
    options: CanvasKitLayerRendererOptions = {},
  ): Promise<CanvasKitLayerRenderer> {
    const canvasKit = await CanvasKitInit({
      locateFile: (file) => file === 'canvaskit.wasm' ? canvaskitWasmUrl : file,
    });
    const resolvedSurfaceRequest = typeof surfaceRequest === 'string'
      ? { ...DEFAULT_CANVASKIT_SURFACE_REQUEST, preference: surfaceRequest, requested: surfaceRequest }
      : surfaceRequest;
    let webgpuDeviceContext: WebGPUDeviceContext | null = null;
    let webgpuDevice: GPUDevice | null = null;
    let webgpuInitFailure: string | null = null;
    if (resolvedSurfaceRequest.preference === 'webgpu') {
      const gpu = typeof navigator === 'undefined' ? undefined : navigator.gpu;
      if (!(canvasKit as CanvasKit & { webgpu?: boolean }).webgpu || typeof canvasKit.MakeGPUDeviceContext !== 'function') {
        webgpuInitFailure = 'webgpuBuildUnavailable';
      } else if (!gpu) webgpuInitFailure = 'webgpuNavigatorUnavailable';
      else {
        try {
          const adapter = await gpu.requestAdapter();
          if (!adapter) throw new Error('webgpuAdapterUnavailable');
          webgpuDevice = await adapter.requestDevice();
          webgpuDeviceContext = canvasKit.MakeGPUDeviceContext(webgpuDevice);
          if (!webgpuDeviceContext) throw new Error('webgpuDeviceContextUnavailable');
        } catch (error) {
          webgpuDevice?.destroy();
          webgpuDevice = null;
          webgpuInitFailure = error instanceof Error ? error.message : String(error);
        }
      }
    }
    // 기본 Noto는 local face가 없거나 등록에 실패한 text run의 안정적인 CJK fallback이다.
    let defaultTypeface: Typeface | null = null;
    let defaultFontBytes: ArrayBuffer | undefined;
    let defaultFontManager: FontMgr | null = null;
    let defaultFontFamily: string | null = null;
    const paragraphFontSources: Array<{ face: Typeface; bytes: ArrayBuffer }> = [];
    const defaultFontUrl = options.defaultFontUrl ?? 'fonts/NotoSansKR-Regular.woff2';
    try {
      const response = await fetch(defaultFontUrl);
      if (response.ok) {
        const bytes = await readBoundedResponseArrayBuffer(response, {
          maxBytes: CanvasKitLayerRenderer.MAX_BUNDLED_FONT_BYTES,
        });
        defaultFontBytes = bytes;
        defaultTypeface = canvasKit.Typeface.MakeFreeTypeFaceFromData(bytes)
          ?? canvasKit.Typeface.MakeTypefaceFromData(bytes);
        defaultFontManager = canvasKit.FontMgr.FromData(bytes);
        if (defaultFontManager && defaultFontManager.countFamilies() > 0) {
          defaultFontFamily = defaultFontManager.getFamilyName(0);
        }
        if (defaultTypeface) paragraphFontSources.push({ face: defaultTypeface, bytes });
      }
    } catch (error) {
      console.warn('[CanvasKitLayerRenderer] 기본 CJK 폰트 로딩 실패:', error);
    }
    let symbolFallbackTypeface: Typeface | null = null;
    const symbolFallbackFontUrl = options.symbolFallbackFontUrl
      ?? 'fonts/D2Coding-Regular.woff2';
    try {
      const response = await fetch(symbolFallbackFontUrl);
      if (response.ok) {
        const bytes = await readBoundedResponseArrayBuffer(response, {
          maxBytes: CanvasKitLayerRenderer.MAX_BUNDLED_FONT_BYTES,
        });
        symbolFallbackTypeface = canvasKit.Typeface.MakeFreeTypeFaceFromData(bytes)
          ?? canvasKit.Typeface.MakeTypefaceFromData(bytes);
        if (symbolFallbackTypeface) paragraphFontSources.push({ face: symbolFallbackTypeface, bytes });
      }
    } catch (error) {
      console.warn('[CanvasKitLayerRenderer] 기호 폴백 폰트 로딩 실패:', error);
    }
    let oldHangulTypeface: CanvasKitLocalTypeface | null = null;
    const oldHangulFontUrl = options.oldHangulFontUrl
      ?? 'fonts/RHWPOldHangul-Regular.woff2';
    let oldHangulNativeTypeface: Typeface | null = null;
    let oldHangulFontManager: FontMgr | null = null;
    try {
      const response = await fetch(oldHangulFontUrl);
      if (response.ok) {
        const bytes = await readBoundedResponseArrayBuffer(response, {
          maxBytes: CanvasKitLayerRenderer.MAX_BUNDLED_FONT_BYTES,
        });
        oldHangulNativeTypeface = canvasKit.Typeface.MakeFreeTypeFaceFromData(bytes)
          ?? canvasKit.Typeface.MakeTypefaceFromData(bytes);
        oldHangulFontManager = canvasKit.FontMgr.FromData(bytes.slice(0));
        const fontFamily = oldHangulFontManager && oldHangulFontManager.countFamilies() > 0
          ? oldHangulFontManager.getFamilyName(0)
          : OLD_HANGUL_FONT_FAMILY;
        if (oldHangulNativeTypeface || oldHangulFontManager) {
          oldHangulTypeface = {
            bytes,
            typeface: oldHangulNativeTypeface,
            fontManager: oldHangulFontManager,
            fontFamily,
          };
          if (oldHangulNativeTypeface) paragraphFontSources.push({ face: oldHangulNativeTypeface, bytes });
          oldHangulNativeTypeface = null;
          oldHangulFontManager = null;
        }
      }
    } catch (error) {
      oldHangulNativeTypeface?.delete?.();
      oldHangulFontManager?.delete?.();
      console.warn('[CanvasKitLayerRenderer] 옛한글 shaping 폰트 로딩 실패:', error);
    }
    const renderer = new CanvasKitLayerRenderer(
      canvasKit,
      renderMode,
      resolvedSurfaceRequest,
      defaultTypeface,
      symbolFallbackTypeface,
      defaultFontManager,
      defaultFontFamily,
      defaultFontUrl,
      options.requirePreparedFontFamilies ?? false,
      oldHangulTypeface,
      oldHangulFontUrl,
      defaultFontBytes,
      webgpuDeviceContext,
      webgpuDevice,
      webgpuInitFailure,
    );
    for (const source of paragraphFontSources) renderer.paragraphFonts.remember(source.face, source.bytes);
    return renderer;
  }

  /** 선택된 문서 폰트를 첫 replay 전에 native Typeface로 등록한다. */
  async prepareBundledFonts(sources: readonly CanvasKitBundledFontSource[]): Promise<number> {
    if (this.disposed || sources.length === 0) return 0;
    const generation = this.documentGeneration;
    let registered = 0;
    for (const source of sources) {
      if (!source.url || source.aliases.length === 0) continue;
      const requiresShapingManager = source.aliases.some(alias => (
        normalizedFontFamily(alias) === normalizedFontFamily(OLD_HANGUL_FONT_FAMILY)
      ));
      let prepared = source.url === this.oldHangulFontUrl && this.oldHangulTypeface
        ? this.oldHangulTypeface
        : source.url === this.defaultFontUrl && (this.defaultTypeface || this.defaultFontManager)
          ? {
            bytes: this.defaultFontBytes,
            typeface: this.defaultTypeface,
            fontManager: this.defaultFontManager,
            fontFamily: this.defaultFontFamily,
          }
          : this.bundledTypefaces.get(source.url) ?? null;
      if (prepared && requiresShapingManager && !prepared.fontManager) {
        throw new Error(`CanvasKit shaping font source 준비 실패: ${source.url}`);
      }
      if (!prepared) {
        if (this.bundledTypefaceLoadFailures.has(source.url)) {
          throw new Error(`CanvasKit font source 준비 실패: ${source.url}`);
        }
        let typeface: Typeface | null = null;
        let fontManager: FontMgr | null = null;
        const request = new AbortController();
        this.bundledFontRequests.add(request);
        try {
          if (this.disposed || generation !== this.documentGeneration) {
            throw new Error('문서 교체로 CanvasKit font 준비가 취소되었습니다');
          }
          const response = await fetch(source.url, { signal: request.signal });
          if (!response.ok) {
            throw new Error(`HTTP ${response.status}`);
          }
          const bytes = await readBoundedResponseArrayBuffer(response, {
            maxBytes: CanvasKitLayerRenderer.MAX_BUNDLED_FONT_BYTES,
            signal: request.signal,
            isCancelled: () => this.disposed || generation !== this.documentGeneration,
            cancelledMessage: '문서 교체로 CanvasKit font 준비가 취소되었습니다',
          });
          if (this.disposed || generation !== this.documentGeneration) {
            throw new Error('문서 교체로 CanvasKit font 준비가 취소되었습니다');
          }
          typeface = this.canvasKit.Typeface.MakeFreeTypeFaceFromData(bytes)
            ?? this.canvasKit.Typeface.MakeTypefaceFromData(bytes);
          fontManager = this.canvasKit.FontMgr.FromData(bytes.slice(0));
          if ((!typeface && !fontManager) || (requiresShapingManager && !fontManager)) {
            throw new Error('CanvasKit이 font payload를 해석하지 못했습니다');
          }
          const fontFamily = fontManager && fontManager.countFamilies() > 0
            ? fontManager.getFamilyName(0)
            : source.aliases[0];
          prepared = { bytes, typeface, fontManager, fontFamily };
          this.bundledTypefaces.set(source.url, prepared);
          this.invalidateFontCaches();
          this.paragraphFonts.remember(typeface, bytes);
          registered += 1;
          typeface = null;
          fontManager = null;
        } catch (error) {
          typeface?.delete?.();
          fontManager?.delete?.();
          if (!request.signal.aborted
            && !this.disposed && generation === this.documentGeneration) {
            this.bundledTypefaceLoadFailures.add(source.url);
          }
          throw new Error(`CanvasKit font source 준비 실패 (${source.url}): ${error}`);
        } finally {
          this.bundledFontRequests.delete(request);
        }
      }
      for (const alias of source.aliases) {
        const key = normalizedFontFamily(alias);
        if (key) {
          const faces = this.bundledTypefaceAliases.get(key) ?? [];
          if (!faces.some(face => face.typeface === prepared.typeface)) {
            faces.push(prepared);
            this.bundledTypefaceAliases.set(key, faces);
            this.bundledFontFamilies.get(key)?.dispose();
            this.bundledFontFamilies.delete(key);
            this.invalidateFontCaches();
          }
        }
      }
      await Promise.resolve();
    }
    return registered;
  }

  /** 현재 문서가 실제로 사용하는 설치 글꼴만 CanvasKit native 객체로 등록한다. */
  async prepareLocalFonts(
    fontNames: readonly string[] | undefined,
    options: { refresh?: boolean } = {},
  ): Promise<number> {
    if (this.disposed || hasHostFontProvider() || (!fontNames?.length && !options.refresh)) return 0;
    const generation = this.documentGeneration;
    if (options.refresh) {
      this.localFontPreparationEpoch += 1;
      this.localTypefacePending.clear();
      this.localTypefaceLoadFailures.clear();
    }
    const epoch = this.localFontPreparationEpoch;
    const pendingRecords = new Map<string, LocalFontRecord>();
    // legacy 이름(`한양중고딕`)은 설치 face 이름이 아니므로 치환 대상(`HY중고딕`)까지 함께 본다.
    const candidateNames = [
      ...(fontNames ?? []),
      ...(fontNames ?? []).flatMap(name => projectedSubstituteTargets(name).map(target => target.face)),
    ];
    const requestedAliases = new Set(candidateNames.map(normalizedFontFamily));
    const requestedRecords = getLocalFontRecords({ includeRegistered: true }).filter(record =>
      record.aliases.some(alias => requestedAliases.has(normalizedFontFamily(alias))),
    );
    const requestedKeys = new Set(requestedRecords.map(localFontFaceKey));
    for (const record of requestedRecords) {
      const faceKey = localFontFaceKey(record);
      if (!faceKey || (!options.refresh && this.localTypefaces.has(faceKey))
        || this.localTypefaceLoadFailures.has(faceKey) || this.localTypefacePending.has(faceKey)) continue;
      pendingRecords.set(faceKey, record);
      this.localTypefacePending.set(faceKey, epoch);
    }

    let registered = 0;
    try {
      const bytesByFace = await loadLocalFontBytesFor([...pendingRecords.values()].map(record => record.fullName));
      for (const [faceKey, record] of pendingRecords) {
        const bytes = bytesByFace.get(faceKey);
        if (this.disposed || generation !== this.documentGeneration || epoch !== this.localFontPreparationEpoch) return registered;
        if ((!options.refresh && this.localTypefaces.has(faceKey)) || this.localTypefaceLoadFailures.has(faceKey)) continue;
        const previous = this.localTypefaces.get(faceKey);
        if (!bytes) {
          if (!previous) this.localTypefaceLoadFailures.add(faceKey);
          continue;
        }
        const bytesDigest = Array.from(blake3(new Uint8Array(bytes))).join(',');
        if (previous?.bytesDigest === bytesDigest) {
          if (JSON.stringify(previous.record) !== JSON.stringify(record)) this.invalidateFontCaches();
          previous.record = record;
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
          this.localTypefaces.set(faceKey, { typeface, fontManager, fontFamily, record, bytesDigest });
          previous?.typeface?.delete?.();
          previous?.fontManager?.delete?.();
          this.invalidateFontCaches();
          this.paragraphFonts.remember(typeface, bytes);
          registered += 1;
        } catch (error) {
          typeface?.delete?.();
          fontManager?.delete?.();
          this.localTypefaceLoadFailures.add(faceKey);
          console.warn(`[CanvasKitLayerRenderer] ${record.displayName} local Typeface 등록 실패:`, error);
        }
        // native font parsing은 동기 작업이므로 face 사이에서 paint/event loop에 양보한다.
        await new Promise<void>(resolve => globalThis.setTimeout(resolve, 0));
      }
      if (options.refresh && generation === this.documentGeneration && epoch === this.localFontPreparationEpoch) {
        for (const [faceKey, previous] of this.localTypefaces) {
          if (requestedKeys.has(faceKey)) continue;
          previous.typeface?.delete?.();
          previous.fontManager?.delete?.();
          this.localTypefaces.delete(faceKey);
          this.invalidateFontCaches();
        }
      }
    } finally {
      for (const faceKey of pendingRecords.keys()) {
        if (this.localTypefacePending.get(faceKey) === epoch) {
          this.localTypefacePending.delete(faceKey);
        }
      }
    }
    return registered;
  }

  /** Host records have already been selected from actual text styles. */
  async prepareHostFonts(records: readonly LocalFontRecord[]): Promise<number> {
    const documentGeneration = this.documentGeneration;
    const hostGeneration = getHostFontState().generation;
    let registered = 0;
    for (const record of records) {
      const key = localFontFaceKey(record);
      if (!record.hostReference || this.localTypefaces.has(key)
        || this.localTypefaceLoadFailures.has(key) || this.localTypefacePending.has(key)) continue;
      this.localTypefacePending.set(key, documentGeneration);
      try {
        const data = await loadRendererLocalFont(record);
        if (this.disposed || documentGeneration !== this.documentGeneration
          || hostGeneration !== getHostFontState().generation) return registered;
        const bytes = data ? canvasKitFontFaceData(data.bytes, data.faceIndex ?? 0) : null;
        if (!bytes) { this.localTypefaceLoadFailures.add(key); continue; }
        let typeface: Typeface | null = null;
        let fontManager: FontMgr | null = null;
        try {
          typeface = this.canvasKit.Typeface.MakeFreeTypeFaceFromData(bytes)
            ?? this.canvasKit.Typeface.MakeTypefaceFromData(bytes);
          fontManager = this.canvasKit.FontMgr.FromData(bytes.slice(0));
          if (!typeface) {
            fontManager?.delete?.();
            this.localTypefaceLoadFailures.add(key);
            continue;
          }
          this.localTypefaces.set(key, { typeface, fontManager,
            fontFamily: fontManager && fontManager.countFamilies() > 0 ? fontManager.getFamilyName(0) : record.family,
            hostFace: record.hostReference.face, record, bytes: bytes.slice(0) });
          this.invalidateFontCaches();
          this.paragraphFonts.remember(typeface, bytes);
          registered++;
        } catch (error) {
          typeface?.delete?.(); fontManager?.delete?.();
          this.localTypefaceLoadFailures.add(key);
          console.warn('[CanvasKit] Host face preparation failed:', error);
        }
      } finally {
        if (this.localTypefacePending.get(key) === documentGeneration) this.localTypefacePending.delete(key);
      }
    }
    return registered;
  }

  /** 브라우저 image decode가 필요한 자원은 첫 surface replay 전에 준비한다. */
  preparePageResources(
    tree: PageLayerTree | PageLayerTreeV2,
    resolveImageBytes?: (key: string) => Uint8Array | null,
    documentGeneration?: number,
  ): Promise<void> | null {
    if (documentGeneration !== undefined) this.syncDocumentGeneration(documentGeneration);
    tree = normalizeTextV2Tree(tree);
    const previousResources = this.currentResources;
    const previousResolver = this.currentImageBytesResolver;
    this.currentResources = tree.resources;
    this.currentImageBytesResolver = resolveImageBytes;
    const nodes = [tree.root];
    let work = 0;
    try {
      while (nodes.length > 0 && ++work <= 50_000) {
        const node = nodes.pop()!;
        if (node.kind === 'group') nodes.push(...node.children);
        else if (node.kind === 'clipRect') nodes.push(node.child);
        else for (const op of node.ops) {
          if (op.type === 'image') this.imageForOp(op);
          else if (op.type === 'pageBackground' && op.image) {
            this.imageForOp(this.pageBackgroundImageOp(op));
          }
        }
      }
    } finally {
      this.currentResources = previousResources;
      this.currentImageBytesResolver = previousResolver;
    }
    const pending = [...this.pendingImageLoads.values()].map(load => load.promise);
    return pending.length === 0 ? null : Promise.all(pending).then(() => {});
  }

  renderPage(
    inputTree: PageLayerTree | PageLayerTreeV2,
    targetCanvas: HTMLCanvasElement,
    scale: number,
    pageInfo?: PageInfo,
    resolveFontBytes?: FontBytesResolver,
    documentGeneration = 0,
    resolveImageBytes?: (key: string) => Uint8Array | null,
  ): HTMLCanvasElement {
    if (this.disposed) {
      throw new Error('CanvasKit renderer가 이미 dispose되었습니다');
    }
    this.syncDocumentGeneration(documentGeneration);
    this.unsupportedOps.clear();
    this.currentImageFailures.clear();
    this.currentFontSubstitutions.clear();
    this.equationRoutes.clear(); this.textRecoveries.clear(); this.textFailures.clear(); this.imageRecoveries.clear();
    this.textVariantReports.clear(); this.textVariantReportParts = 0; this.textV2ValidationIssues = []; this.patternReplayOps = 0;
    this.resetReplayFeatureCounts();
    this.lastRenderError = null;
    this.lastRenderCompleted = false;
    let surface: SkSurface | null = null;
    let renderedCanvas = targetCanvas;
    const renderStartedAt = performance.now();
    try {
      let tree: PageLayerTree;
      try { tree = normalizeTextV2Tree(inputTree); }
      catch (error) {
        this.textV2ValidationIssues = [{ code: 'layerTreeValidationFailed', severity: 'error',
          message: (error instanceof Error ? error.message : String(error)).slice(0, 1024) }];
        throw error;
      }
      const pageExtent = { x: 0, y: 0, width: tree.pageWidth, height: tree.pageHeight };
      this.currentPageExtent = this.boundsAreDrawable(pageExtent) ? pageExtent : null;
      const surfaceTarget = this.surfaceCache.get(targetCanvas, () => {
        const created = this.makeSurface(targetCanvas);
        return { ...created, backend: this.surfaceBackend ?? 'software', fallbackReason: this.surfaceFallbackReason };
      });
      this.surfaceBackend = surfaceTarget.backend;
      this.surfaceFallbackReason = surfaceTarget.fallbackReason;
      surface = surfaceTarget.surface;
      renderedCanvas = surfaceTarget.canvas;
      const canvas = surface.getCanvas();
      this.currentResources = tree.resources;
      this.currentProfile = tree.profile ?? 'screen';
      this.currentImageBytesResolver = resolveImageBytes;
      this.currentFontResources = tree.fontResources;
      this.glyphRunFonts.registerResources(
        tree.fontResources,
        tree.resources,
        resolveFontBytes,
        documentGeneration,
      );
      this.currentShowParagraphMarks = tree.outputOptions?.showParagraphMarks === true;
      this.currentShowControlCodes = tree.outputOptions?.showControlCodes === true;
      this.currentClipEnabled = canvasKitCanonicalClipEnabled(
        tree.buildOptions?.clipEnabled,
        tree.outputOptions?.clipEnabled,
      );
      this.selectedTextVariantOps = new WeakSet<LayerPaintOp>();
      this.selectTextVariants(tree.root, canvas);
      this.currentTreeFingerprint = replayFingerprint(tree);
      this.currentPictureContext = this.currentTreeFingerprint
        ? `${this.currentTreeFingerprint}:${this.documentGeneration}:${this.localFontRevision}:${this.imageResourceRevision}:${JSON.stringify(this.glyphRunFonts.diagnostics())}`
        : null;
      if (this.currentTreeFingerprint) {
        const fingerprints = this.treeFingerprints.get(inputTree as PageLayerTree) ?? new Set<string>();
        if (!fingerprints.has(this.currentTreeFingerprint) && fingerprints.size >= 32) {
          const oldest = fingerprints.values().next().value!;
          this.staticPictures.deleteWhere((_key, entry) => entry.treeFingerprint === oldest);
          fingerprints.delete(oldest);
        }
        fingerprints.add(this.currentTreeFingerprint);
        this.treeFingerprints.set(inputTree as PageLayerTree, fingerprints);
      }
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
      for (const replayPlane of CANVASKIT_REPLAY_PLANES) {
        this.renderNode(canvas, tree.root, tree.profile ?? 'screen', replayPlane, null);
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
      this.surfaceCache.clear();
      this.recordRenderFailure(error);
      throw error;
    } finally {
      this.currentTreeFingerprint = null;
      this.currentPictureContext = null;
      this.currentPageExtent = null;
      this.currentOp = null;
      this.currentProfile = 'screen';
      this.preferRasterDepth = 0;
      this.currentResources = undefined;
      this.currentImageBytesResolver = undefined;
      this.currentFontResources = undefined;
      this.currentShowParagraphMarks = false;
      this.currentShowControlCodes = false;
      this.currentClipEnabled = true;
      this.lastRenderDurationMs = performance.now() - renderStartedAt;
      this.renderCount += 1;
    }
    return renderedCanvas;
  }

  releaseLayerTree(tree: PageLayerTree | PageLayerTreeV2): void {
    const fingerprints = this.treeFingerprints.get(tree as PageLayerTree);
    if (fingerprints) this.staticPictures.deleteWhere((_key, entry) => fingerprints.has(entry.treeFingerprint));
    this.treeFingerprints.delete(tree as PageLayerTree);
  }

  private syncDocumentGeneration(generation: number): void {
    if (this.sourceDocumentGeneration !== undefined && this.sourceDocumentGeneration !== generation) {
      this.resetDocumentResources();
    }
    this.sourceDocumentGeneration = generation;
  }

  resetDocumentResources(): void {
    this.documentGeneration += 1;
    this.sourceDocumentGeneration = undefined;
    this.localFontPreparationEpoch += 1;
    this.localFontRevision += 1;
    this.surfaceCache.clear();
    this.staticPictures.clear(true);
    this.textBlobs.clear(true);
    this.failedTextBlobKeys.clear();
    this.fallbackFamilyCache.clear();
    this.imageResourceRevision += 1;
    this.cancelDocumentPreparation();
    for (const entry of this.imageCache.values()) entry.image?.delete?.();
    this.imageCache.clear();
    this.imageRecoveryByKey.clear();
    this.resetImageEffectDiagnostics(); this.resetImageDiagnostics(); this.resetTextReplayDiagnostics(); this.resetEquationReplayDiagnostics(); this.resetPatternDiagnostics();
    this.textVariantReports.clear(); this.textVariantReportParts = 0; this.textV2ValidationIssues = []; this.imageRecoveries.clear();
    this.mipmapImages.clear(true);
    this.imageCachePixels = 0;
    this.imageDecodeFailures.clear();
    this.svgGlyphPathCache.clear();
    this.svgGlyphParseFailures.clear();
    this.currentResources = undefined;
    this.currentImageBytesResolver = undefined;
    this.currentFontResources = undefined;
    this.selectedTextVariantOps = new WeakSet<LayerPaintOp>();
    this.glyphRunFonts.clear();
    for (const { typeface, fontManager } of this.localTypefaces.values()) {
      typeface?.delete?.();
      fontManager?.delete?.();
    }
    this.localTypefaces.clear();
    this.localTypefaceLoadFailures.clear();
    this.localTypefacePending.clear();
    for (const { typeface, fontManager } of this.bundledTypefaces.values()) {
      typeface?.delete?.();
      fontManager?.delete?.();
    }
    this.bundledTypefaces.clear();
    this.bundledTypefaceAliases.clear();
    for (const family of this.bundledFontFamilies.values()) family.dispose();
    this.bundledFontFamilies.clear();
    this.bundledTypefaceLoadFailures.clear();
    this.retainPreparedParagraphFonts();
    this.imageCacheHits = 0;
    this.imageCacheMisses = 0;
    this.imageCacheEvictions = 0;
    this.imageFailureCacheHits = 0;
    this.currentImageFailures.clear();
    this.currentFontSubstitutions.clear();
    this.resetReplayFeatureCounts();
    this.renderCount = 0;
    this.lastRenderDurationMs = null;
  }

  cancelDocumentPreparation(): void {
    for (const load of this.pendingImageLoads.values()) load.cancel();
    this.pendingImageLoads.clear();
    for (const request of this.bundledFontRequests) {
      request.abort(new Error('문서 교체로 CanvasKit font 준비가 취소되었습니다'));
    }
    this.bundledFontRequests.clear();
  }

  diagnostics(): CanvasKitRenderDiagnostics {
    const lastUnsupportedOps = [...this.unsupportedOps].sort();
    const lastExpectedUnsupportedOps = lastUnsupportedOps.filter(isExpectedCanvasKitUnsupportedOp);
    const lastUnexpectedUnsupportedOps = lastUnsupportedOps.filter(
      (op) => !isExpectedCanvasKitUnsupportedOp(op),
    );
    const surfaceFallbackReason = this.surfaceFallbackReason ?? this.surfaceRequest.unsupportedReason ?? null;
    const fontSubstitutions = [...this.currentFontSubstitutions.values()]
      .map((substitution) => ({ ...substitution }));
    const glyphRunFontDiagnostics = this.glyphRunFonts.diagnostics();
    const readinessBlockers: CanvasKitReadinessBlocker[] = [];
    if (!this.lastRenderCompleted) readinessBlockers.push('renderNotCompleted');
    if (this.lastRenderError !== null) readinessBlockers.push('renderError');
    if (lastUnexpectedUnsupportedOps.length > 0) readinessBlockers.push('unexpectedUnsupportedOps');
    if (this.currentImageFailures.size > 0) readinessBlockers.push('imageReplayFailure');
    if (this.localTypefacePending.size > 0) readinessBlockers.push('localFontsPending');
    if (this.pendingImageLoads.size > 0) readinessBlockers.push('imageReplayFailure');
    return {
      surfaceCreated: this.surfaceCache.created,
      surfaceReused: this.surfaceCache.reused,
      staticPictureCacheEntries: this.staticPictures.size,
      staticPictureCacheBytes: this.staticPictures.bytes,
      staticPictureCacheHits: this.staticPictures.hits,
      staticPictureCacheMisses: this.staticPictures.misses,
      staticPictureCacheEvictions: this.staticPictures.evictions,
      textBlobCacheEntries: this.textBlobs.size,
      textBlobCacheBytes: this.textBlobs.bytes,
      textBlobCacheHits: this.textBlobs.hits,
      textBlobCacheMisses: this.textBlobs.misses,
      fallbackFamilyCacheEntries: this.fallbackFamilyCache.size,
      paragraphFontEntries: this.paragraphFonts.diagnostics().entries,
      paragraphFontBytes: this.paragraphFonts.diagnostics().bytes,
      paragraphProviderBuilds: this.paragraphFonts.diagnostics().builds,
      paragraphFontRejections: this.paragraphFonts.diagnostics().rejections,
      pendingImageLoads: this.pendingImageLoads.size,
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
      lastRenderDurationMs: this.lastRenderDurationMs,
      renderCount: this.renderCount,
      imageCacheEntries: this.imageCache.size,
      imageCacheLimit: CanvasKitLayerRenderer.MAX_IMAGE_CACHE_ENTRIES,
      imageCachePixels: this.imageCachePixels,
      imageCachePixelLimit: CanvasKitLayerRenderer.MAX_IMAGE_CACHE_PIXELS,
      imageCacheHits: this.imageCacheHits,
      imageCacheMisses: this.imageCacheMisses,
      imageCacheEvictions: this.imageCacheEvictions,
      mipmapCacheEntries: this.mipmapImages.size,
      mipmapCacheBytes: this.mipmapImages.bytes,
      mipmapCacheHits: this.mipmapImages.hits,
      mipmapCacheMisses: this.mipmapImages.misses,
      imageFailureCacheHits: this.imageFailureCacheHits,
      imageFailures: [...this.currentImageFailures.values()].map((failure) => ({ ...failure })),
      localTypefaceCount: this.localTypefaces.size,
      localTypefaceLoadFailureCount: this.localTypefaceLoadFailures.size,
      localTypefacePendingCount: this.localTypefacePending.size,
      bundledTypefaceCount: this.bundledTypefaces.size,
      bundledTypefaceLoadFailureCount: this.bundledTypefaceLoadFailures.size,
      glyphRunFontBlobCount: glyphRunFontDiagnostics.blobs,
      glyphRunFontBlobBytes: glyphRunFontDiagnostics.bytes,
      glyphRunTypefaceCount: glyphRunFontDiagnostics.typefaces,
      glyphRunFontCount: glyphRunFontDiagnostics.fonts,
      fontSubstitutionLimit: CanvasKitLayerRenderer.MAX_FONT_SUBSTITUTION_DIAGNOSTICS,
      unregisteredFontFallbacks: fontSubstitutions.filter(
        (substitution) => substitution.kind === 'unregisteredFallback',
      ).length,
      fontSubstitutions,
      replayFeatureCounts: { ...this.currentReplayFeatureCounts },
      textVariantDiagnosticGroups: this.textVariantReports.values.length,
      textVariantDiagnosticParts: this.textVariantReportParts,
      replayDiagnosticsDropped: this.textVariantReports.dropped + this.equationRoutes.dropped + this.textRecoveries.dropped
        + this.textFailures.dropped + this.imageRecoveries.dropped,
    };
  }

  getImageDiagnostics() {
    return { cacheHits: this.imageCacheHits, cacheMisses: this.imageCacheMisses,
      failureCacheHits: this.imageFailureCacheHits, pendingLoads: this.pendingImageLoads.size,
      imagesDecoded: this.imagesDecoded, failureAttempts: this.imageFailureAttempts, pendingAccesses: this.pendingImageAccesses,
      recoveries: this.imageRecoveries.snapshot(),
      failures: [...this.currentImageFailures.values()].map(failure => ({ ...failure })),
      droppedRecoveries: this.imageRecoveries.dropped };
  }

  getImageEffectDiagnostics() { return { ...this.imageEffectDiagnostics }; }
  resetImageEffectDiagnostics(): void { this.imageEffectDiagnostics = emptyImageEffectDiagnostics(); }
  resetImageDiagnostics(): void {
    this.imageCacheHits = 0; this.imageCacheMisses = 0; this.imageFailureCacheHits = 0;
    this.imagesDecoded = 0; this.imageFailureAttempts = 0; this.pendingImageAccesses = 0;
    this.currentImageFailures.clear(); this.imageRecoveries.clear();
  }

  getTextReplayDiagnostics() {
    const fontSubstitutions = [...this.currentFontSubstitutions.values()].map(item => ({ ...item }));
    return { constructionFailures: this.textBlobConstructionFailures, failureCacheHits: this.textBlobFailureCacheHits,
      fallbackDraws: this.textBlobFallbackDraws,
      unregisteredFontFallbacks: fontSubstitutions.filter(item => item.kind === 'unregisteredFallback').length,
      fontSubstitutions, recoveries: this.textRecoveries.snapshot(), failures: this.textFailures.snapshot(),
      droppedDiagnostics: this.textRecoveries.dropped + this.textFailures.dropped };
  }

  resetTextReplayDiagnostics(): void {
    this.failedTextBlobKeys.clear(); this.textRecoveries.clear(); this.textFailures.clear(); this.currentFontSubstitutions.clear();
    this.textBlobConstructionFailures = 0; this.textBlobFailureCacheHits = 0; this.textBlobFallbackDraws = 0;
  }

  getEquationReplayDiagnostics() {
    const routes = this.equationRoutes.snapshot();
    return { svgReplays: routes.filter(item => item.route === 'svg').length,
      layoutReplays: routes.filter(item => item.route === 'layout').length,
      fallbackReplays: routes.filter(item => item.route === 'layout' && item.reason !== 'layoutRequested').length,
      failedReplays: routes.filter(item => item.route === 'failed').length, routes, droppedDiagnostics: this.equationRoutes.dropped };
  }
  resetEquationReplayDiagnostics(): void { this.equationRoutes.clear(); }
  getPatternDiagnostics() { return { backend: 'vector' as const, replayedOps: this.patternReplayOps }; }
  resetPatternDiagnostics(): void { this.patternReplayOps = 0; }
  getSurfaceDiagnostics() {
    return { created: this.surfaceCache.created, reused: this.surfaceCache.reused, backend: this.surfaceBackend,
      preference: this.surfaceRequest.preference, fallbackReason: this.surfaceFallbackReason };
  }
  getTextVariantSelectionDiagnostics() { return this.textVariantReports.snapshot(); }
  getTextV2ValidationDiagnostics() { return this.textV2ValidationIssues.map(issue => ({ ...issue })); }

  /** 준비된 CanvasKit 객체만 읽어 실제 text replay 후보의 glyph 보유 여부를 판정한다. */
  fontDecisionEvidence(
    record: FontDecisionTraceRecordV1,
    pageUsesGlyphResources = false,
  ): CanvasKitFontDecisionEvidence {
    const requested = record.paint.canvaskit.requested
      ?? record.layoutName.normalizedFace
      ?? record.document.face
      ?? '';
    const character = record.source.character;
    const requestedFamily = primaryFontFamily(requested);
    const normalized = normalizedFontFamily(requestedFamily);
    const localRecord = resolveRendererLocalFont(requestedFamily);
    const localKey = localRecord ? localFontFaceKey(localRecord) : '';
    const local = localKey ? this.localTypefaces.get(localKey) ?? null : null;
    const bundled = this.findBundledTypeface(normalized);
    const primary = local ?? bundled ?? (
      normalized === normalizedFontFamily(this.defaultFontFamily) || normalized === 'noto sans kr'
        ? (this.defaultTypeface || this.defaultFontManager ? {
            typeface: this.defaultTypeface,
            fontManager: this.defaultFontManager,
            fontFamily: this.defaultFontFamily,
          } : null)
        : null
    );
    const primarySource: CanvasKitFontDecisionEvidence['source'] = local
      ? 'local'
      : bundled
        ? 'bundled'
        : primary
          ? 'default'
          : null;
    const candidates = [
      requestedFamily,
      primary?.fontFamily ?? '',
      this.defaultFontFamily ?? 'CanvasKit default',
      'CanvasKit symbol fallback',
    ].filter((family, index, all) => family && all.indexOf(family) === index);
    const failures: string[] = [];
    const sourceRecordProvided = Number.isSafeInteger(record.source.sectionIndex)
      && Number.isSafeInteger(record.source.paragraphIndex)
      && Number.isSafeInteger(record.source.charOffset);
    if (!sourceRecordProvided || pageUsesGlyphResources) {
      const capabilities = sourceRecordProvided ? ['sourceRecordProvided'] : [];
      const joinFailures = ['backendJoinMissing'];
      if (pageUsesGlyphResources) {
        capabilities.push('canvaskitGlyphResourceSnapshotAvailable');
        joinFailures.push('canvaskitGlyphResourceSourceUnresolved');
      }
      return {
        status: 'notObserved', certainty: 'notObserved', requested: requestedFamily,
        candidates: requestedFamily ? [requestedFamily] : [], resolved: null, source: null,
        capabilities,
        failures: joinFailures,
      };
    }
    const sourceCapabilities = ['sourceRecordProvided'];
    if (localKey && this.localTypefacePending.has(localKey)) failures.push('canvaskitLocalSfntPending');
    if (localKey && this.localTypefaceLoadFailures.has(localKey)) failures.push('canvaskitLocalSfntUnavailable');

    const codePointCount = Array.from(character).length;
    const observe = (typeface: Typeface | null, family: string, source: NonNullable<CanvasKitFontDecisionEvidence['source']>) => {
      if (!typeface || codePointCount !== 1) return null;
      const font = this.makeFont(typeface, 16);
      try {
        return (font.getGlyphIDs(character, 1)[0] ?? 0) !== 0
          ? { family, source }
          : null;
      } finally {
        font.delete();
      }
    };
    const selected = observe(primary?.typeface ?? null, primary?.fontFamily ?? requestedFamily, primarySource ?? 'default')
      ?? (primary?.typeface !== this.defaultTypeface
        ? observe(this.defaultTypeface, this.defaultFontFamily ?? 'CanvasKit default', 'default')
        : null)
      ?? (primary?.typeface !== this.symbolFallbackTypeface && this.defaultTypeface !== this.symbolFallbackTypeface
        ? observe(this.symbolFallbackTypeface, 'CanvasKit symbol fallback', 'symbol')
        : null);
    if (selected) {
      return {
        status: 'complete', certainty: 'resolved', requested: requestedFamily, candidates,
        resolved: selected.family, source: selected.source,
        capabilities: [
          ...sourceCapabilities,
          'canvaskitSfntPrepared',
          'canvaskitGlyphCoverageObserved',
        ],
        failures,
      };
    }
    if (primary?.fontManager && !primary.typeface) {
      failures.push('canvaskitGlyphCoverageUnobservable');
      return {
        status: 'notObserved', certainty: 'planned', requested: requestedFamily, candidates,
        resolved: primary.fontFamily, source: primarySource,
        capabilities: [
          ...sourceCapabilities,
          'canvaskitSfntPrepared',
          'canvaskitShapingManagerPrepared',
        ],
        failures,
      };
    }
    failures.push(primary ? 'canvaskitGlyphMissingAllCandidates' : 'canvaskitSfntAbsent');
    return {
      status: 'complete', certainty: 'observed', requested: requestedFamily, candidates,
      resolved: null, source: null,
      capabilities: [...sourceCapabilities, 'canvaskitSnapshotObserved'], failures,
    };
  }

  private resetReplayFeatureCounts(): void {
    this.currentReplayFeatureCounts = {
      dashedStrokes: 0,
      glyphRuns: 0,
      tabLeaders: 0,
      charOverlaps: 0,
      verticalPresentationPunctuation: 0,
      verticalTextRuns: 0,
    };
  }

  recordRenderFailure(error: unknown, resetReplayState = false): void {
    if (resetReplayState) {
      this.unsupportedOps.clear();
      this.currentImageFailures.clear();
      this.currentFontSubstitutions.clear();
      this.equationRoutes.clear(); this.textRecoveries.clear(); this.textFailures.clear(); this.imageRecoveries.clear();
      this.textVariantReports.clear(); this.textVariantReportParts = 0; this.textV2ValidationIssues = []; this.patternReplayOps = 0;
      this.resetReplayFeatureCounts();
      this.surfaceBackend = null;
      this.surfaceFallbackReason = null;
    }
    this.lastRenderCompleted = false;
    this.lastRenderError = error instanceof Error ? error.message : String(error);
    this.unsupportedOps.add('renderPage');
  }

  dispose(): void {
    this.disposed = true;
    this.resetDocumentResources();
    this.defaultTypeface?.delete();
    this.symbolFallbackTypeface?.delete();
    this.defaultFontManager?.delete();
    this.oldHangulTypeface?.typeface?.delete?.();
    this.oldHangulTypeface?.fontManager?.delete?.();
    this.paragraphFonts.clear();
    this.webgpuDeviceContext?.delete();
    this.webgpuDevice?.destroy();
  }

  private makeSurface(
    targetCanvas: HTMLCanvasElement,
  ): CanvasKitSurfaceTarget {
    this.surfaceBackend = null;
    this.surfaceFallbackReason = this.surfaceRequest.unsupportedReason ?? null;
    if (this.surfaceRequest.preference === 'webgpu') {
      this.surfaceFallbackReason ??= this.webgpuInitFailure;
      if (this.webgpuDeviceContext) {
        let context: ReturnType<typeof this.canvasKit.MakeGPUCanvasContext> | null = null;
        try {
          context = this.canvasKit.MakeGPUCanvasContext(this.webgpuDeviceContext, targetCanvas);
          const surface = context && this.canvasKit.MakeGPUCanvasSurface(context, this.canvasKit.ColorSpace.SRGB, targetCanvas.width, targetCanvas.height);
          if (surface) {
            this.surfaceBackend = 'webgpu';
            return { surface, canvas: targetCanvas };
          }
          this.surfaceFallbackReason = 'webgpuSurfaceUnavailable';
        } catch { this.surfaceFallbackReason = 'webgpuSurfaceCreationFailed'; }
      } else this.surfaceFallbackReason ??= 'webgpuSurfaceUnsupported';
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

  private selectTextVariants(node: LayerNode, canvas: SkCanvas): void {
    if (node.kind === 'group') {
      for (const child of node.children) this.selectTextVariants(child, canvas);
      return;
    }
    if (node.kind === 'clipRect') {
      this.selectTextVariants(node.child, canvas);
      return;
    }

    const selected = selectLayerTextVariantsForLeaf(
      node.ops,
      op => this.glyphOutlineReplayStatus(op),
      op => this.glyphRunReplayStatus(op, canvas),
      report => {
        if (this.textVariantReports.values.length >= 256 || this.textVariantReportParts >= 4096) { this.textVariantReports.dropped++; return; }
        const remaining = 4096 - this.textVariantReportParts;
        if (report.parts.length > remaining) { report.parts = report.parts.slice(0, remaining); report.truncated = true; }
        report.renderProfile = this.currentProfile;
        this.textVariantReportParts += report.parts.length; this.textVariantReports.add(report);
      },
    );
    for (const op of selected) {
      this.selectedTextVariantOps.add(op);
    }
    const groups = new Set(node.ops.flatMap(op => 'variant' in op && op.variant?.equivalenceGroup
      ? [op.variant.equivalenceGroup] : []));
    for (const op of selected) if ('variant' in op && op.variant?.equivalenceGroup) groups.delete(op.variant.equivalenceGroup);
    if (groups.size > 0) this.unsupportedOps.add('textVariant:noReplayableVariant');
  }

  private glyphRunVariantReplayable(op: LayerGlyphRunOp, canvas: SkCanvas): boolean {
    return canvasKitCanvasSupportsGlyphRunReplay(canvas)
      && this.glyphRunFonts.replayStatus(op, this.currentFontResources).replayable;
  }

  private glyphRunReplayStatus(op: LayerGlyphRunOp, canvas: SkCanvas): LayerTextVariantReplayStatus {
    const status = this.glyphRunFonts.replayStatus(op, this.currentFontResources);
    const supported = canvasKitCanvasSupportsGlyphRunReplay(canvas);
    const report = Object.fromEntries(Object.entries(status.report).map(([key, value]) => [key, typeof value === 'string' ? value.slice(0, 1024) : value]));
    const reason = !supported ? 'backendDoesNotSupportVariant' : status.replayable ? undefined : status.reason;
    return { replayable: status.replayable && supported, reason,
      fontVerification: { ...report, faceKey: op.shapeKey?.fontInstance?.faceKey?.slice(0, 1024),
        ...(status.replayable ? { blobKey: status.blob.id, portability: status.blob.portability, expectedDigest: status.blob.digest?.value } : {}),
        replayEligible: status.replayable && supported, reason } };
  }

  private glyphOutlineVariantReplayable(op: LayerGlyphOutlineOp): boolean {
    if (op.diagnostics?.strictVisualEligible !== true || !glyphOutlinePaintIsSupported(op)) return false;
    const status = glyphOutlinePayloadStatus(op, {
      allowMonochromeFillStroke: true,
      allowColrv0ColorLayers: true,
      allowColrv1Stage1ColorGraph: true,
      allowBitmapGlyph: true,
      allowSvgGlyph: true,
    });
    if (!status.supported) return false;
    if (op.payloadKind === 'bitmapGlyph') {
      const imageOp = this.bitmapGlyphImageOp(op);
      return imageOp !== null && this.imageForOp(imageOp) !== null;
    }
    if (op.payloadKind === 'svgGlyph') {
      return this.staticSvgGlyphPathLayers(op) !== null;
    }
    return op.payloadKind === 'colorLayers'
      || op.payloadKind === 'monochromeFill'
      || op.payloadKind === 'monochromeFillStroke';
  }

  private glyphOutlineReplayStatus(op: LayerGlyphOutlineOp): LayerTextVariantReplayStatus {
    const payload = glyphOutlinePayloadStatus(op, { allowMonochromeFillStroke: true, allowColrv0ColorLayers: true,
      allowColrv1Stage1ColorGraph: true, allowBitmapGlyph: true, allowSvgGlyph: true });
    const paintStyleSupported = glyphOutlinePaintIsSupported(op);
    const replayable = this.glyphOutlineVariantReplayable(op);
    const reason = !paintStyleSupported ? 'unsupportedPaintEffect' : !payload.supported ? payload.reason
      : op.diagnostics?.strictVisualEligible !== true ? 'glyphOutlineNotStrictEligible' : replayable ? undefined : 'outlineResourceNotReplayable';
    return { replayable, reason, details: payload.detail, outlineEligibility: {
      strictVisualEligible: op.diagnostics?.strictVisualEligible === true, payloadSupported: payload.supported,
      paintStyleSupported, replayEligible: replayable, reason } };
  }

  private layerResourceIndex(
    id: number | string | undefined,
    keys: string[] | undefined,
    length: number,
  ): number | null {
    if (typeof id === 'number' && Number.isInteger(id) && id >= 0 && id < length) return id;
    if (typeof id !== 'string') return null;
    const index = keys?.indexOf(id) ?? -1;
    return index >= 0 && index < length ? index : null;
  }

  private bitmapGlyphImageOp(op: LayerGlyphOutlineOp): LayerImageOp | null {
    const payload = op.bitmapGlyph;
    const resources = this.currentResources;
    const index = this.layerResourceIndex(
      payload?.imageResourceId ?? payload?.imageRef,
      resources?.imageKeys,
      resources?.images?.length ?? 0,
    );
    if (!payload || index === null || !payload.placement) return null;
    const resource = resources?.images?.[index];
    const resourceKey = resources?.imageKeys?.[index];
    const payloadResourceKey = glyphOutlinePayloadResourceKey(op);
    let bytes: Uint8Array;
    try {
      const resolved = this.imageResourceBytes(resource);
      if (!resolved || resolved.length * 4 / 3 > CanvasKitLayerRenderer.MAX_BITMAP_GLYPH_BASE64_LENGTH) return null;
      bytes = resolved;
    } catch {
      return null;
    }
    if (
      typeof resourceKey !== 'string'
      || payloadResourceKey === null
      || op.payloadResourceKey !== `${payloadResourceKey}:resource:${resourceKey}`
      || !layerResourceKeyMatches('img', resourceKey, bytes)
    ) {
      return null;
    }
    return {
      type: 'image',
      bbox: payload.placement,
      imageRef: index,
      sourceImageKey: resourceKey,
      fillMode: 'fitToSize',
    };
  }

  private staticSvgGlyphPathLayers(op: LayerGlyphOutlineOp): StaticSvgPathLayer[] | null {
    const payload = op.svgGlyph;
    const resources = this.currentResources;
    const index = this.layerResourceIndex(
      payload?.vectorResourceId ?? payload?.svgRef,
      resources?.svgKeys,
      resources?.svgFragments?.length ?? 0,
    );
    if (!payload || index === null) return null;
    const fragment = resources?.svgFragments?.[index];
    const resourceKey = resources?.svgKeys?.[index];
    const payloadResourceKey = glyphOutlinePayloadResourceKey(op);
    if (typeof fragment !== 'string'
      || fragment.length > CanvasKitLayerRenderer.MAX_STATIC_SVG_GLYPH_BYTES) {
      return null;
    }
    const fragmentBytes = new TextEncoder().encode(fragment);
    if (
      fragmentBytes.byteLength > CanvasKitLayerRenderer.MAX_STATIC_SVG_GLYPH_BYTES
      || typeof resourceKey !== 'string'
      || payloadResourceKey === null
      || op.payloadResourceKey !== `${payloadResourceKey}:resource:${resourceKey}`
      || !layerResourceKeyMatches('svg', resourceKey, fragmentBytes)
    ) {
      return null;
    }
    const currentColor = op.paintStyle?.color ?? '#000000';
    if (currentColor.length > 256) return null;
    const svgCacheKey = `${resourceKey}:${currentColor}`;
    const cached = this.svgGlyphPathCache.get(svgCacheKey);
    if (cached) {
      this.svgGlyphPathCache.delete(svgCacheKey);
      this.svgGlyphPathCache.set(svgCacheKey, cached);
      return cached;
    }
    if (this.svgGlyphParseFailures.has(svgCacheKey)) return null;

    const layers = parseStaticSvgPathLayers(fragment, currentColor);
    if (layers.length === 0) {
      this.rememberSvgGlyphParseFailure(svgCacheKey);
      return null;
    }
    if (!staticSvgPathLayersAreReplayable(
      layers,
      pathData => this.canvasKit.Path.MakeFromSVGString(pathData),
    )) {
      this.rememberSvgGlyphParseFailure(svgCacheKey);
      return null;
    }
    if (this.svgGlyphPathCache.size >= CanvasKitLayerRenderer.MAX_SVG_GLYPH_CACHE_ENTRIES) {
      const oldestKey = this.svgGlyphPathCache.keys().next().value as string | undefined;
      if (oldestKey !== undefined) this.svgGlyphPathCache.delete(oldestKey);
    }
    this.svgGlyphPathCache.set(svgCacheKey, layers);
    return layers;
  }

  private rememberSvgGlyphParseFailure(resourceKey: string): void {
    if (this.svgGlyphParseFailures.size >= CanvasKitLayerRenderer.MAX_SVG_GLYPH_CACHE_ENTRIES) {
      const oldestKey = this.svgGlyphParseFailures.values().next().value as string | undefined;
      if (oldestKey !== undefined) this.svgGlyphParseFailures.delete(oldestKey);
    }
    this.svgGlyphParseFailures.add(resourceKey);
  }

  private renderNode(
    canvas: SkCanvas,
    node: LayerNode,
    profile: LayerRenderProfile,
    replayPlane: CanvasKitReplayPlane,
    inheritedLayer: LayerInfo | null = null,
  ): void {
    const preferRaster = node.kind === 'group' && node.cacheHint === 'preferRaster';
    if (preferRaster) this.preferRasterDepth++;
    try {
      this.renderNodeContents(canvas, node, profile, replayPlane, inheritedLayer);
    } finally { if (preferRaster) this.preferRasterDepth--; }
  }

  private renderNodeContents(canvas: SkCanvas, node: LayerNode, profile: LayerRenderProfile,
    replayPlane: CanvasKitReplayPlane, inheritedLayer: LayerInfo | null): void {
    const activeLayer = node.layer ?? inheritedLayer;
    if (node.kind === 'group') {
      const recordingBounds = this.staticPictureBounds(node);
      if (!this.recordingStaticPicture && this.currentPictureContext && this.currentTreeFingerprint
        && (node.cacheHint === 'staticSubtree' || node.cacheHint === 'preferVectorRecording')
        && recordingBounds) {
        const nodeKey = replayFingerprint([node, activeLayer]);
        if (nodeKey) {
          const key = `${this.currentPictureContext}:${profile}:${replayPlane}:${nodeKey}`;
          const cached = this.staticPictures.get(key);
          if (cached) {
            canvas.drawPicture(cached.picture);
            this.restoreStaticPictureDiagnostics(cached.diagnostics);
            return;
          }
          const recorder = new this.canvasKit.PictureRecorder();
          let picture: SkPicture | null = null;
          let finished = false;
          const before = { ...this.currentReplayFeatureCounts };
          const diagnosticStart = { equations: this.equationRoutes.values.length, textRecoveries: this.textRecoveries.values.length,
            textFailures: this.textFailures.values.length, imageRecoveries: this.imageRecoveries.values.length, patternOps: this.patternReplayOps };
          this.recordingStaticPicture = true;
          try {
            const recording = recorder.beginRecording(this.rect(recordingBounds));
            for (const child of node.children) this.renderNode(recording, child, profile, replayPlane, activeLayer);
            picture = recorder.finishRecordingAsPicture();
            finished = true;
            canvas.drawPicture(picture);
            const metadata: StaticPictureDiagnostics = {
              unsupported: [...this.unsupportedOps],
              images: [...this.currentImageFailures],
              substitutions: [...this.currentFontSubstitutions.values()],
              equations: this.equationRoutes.values.slice(diagnosticStart.equations),
              textRecoveries: this.textRecoveries.values.slice(diagnosticStart.textRecoveries),
              textFailures: this.textFailures.values.slice(diagnosticStart.textFailures),
              imageRecoveries: this.imageRecoveries.values.slice(diagnosticStart.imageRecoveries),
              patternOps: this.patternReplayOps - diagnosticStart.patternOps,
              features: {
                dashedStrokes: this.currentReplayFeatureCounts.dashedStrokes - before.dashedStrokes,
                glyphRuns: this.currentReplayFeatureCounts.glyphRuns - before.glyphRuns,
                tabLeaders: this.currentReplayFeatureCounts.tabLeaders - before.tabLeaders,
                charOverlaps: this.currentReplayFeatureCounts.charOverlaps - before.charOverlaps,
                verticalPresentationPunctuation: this.currentReplayFeatureCounts.verticalPresentationPunctuation - before.verticalPresentationPunctuation,
                verticalTextRuns: this.currentReplayFeatureCounts.verticalTextRuns - before.verticalTextRuns,
              },
            };
            // SkPicture's own estimate excludes retained images/fonts, so charge those as well.
            const bytes = Math.ceil(picture.approximateBytesUsed() + this.imageCachePixels * 4
              + this.mipmapImages.bytes + this.glyphRunFonts.diagnostics().bytes + JSON.stringify(metadata).length * 2);
            if (this.pendingImageLoads.size === 0 && this.localTypefacePending.size === 0
              && this.staticPictures.set(key, { picture, treeFingerprint: this.currentTreeFingerprint, diagnostics: metadata }, bytes)) {
              picture = null;
            }
          } finally {
            this.recordingStaticPicture = false;
            if (!finished) { try { picture = recorder.finishRecordingAsPicture(); } catch { /* failed recording */ } }
            picture?.delete();
            recorder.delete();
          }
          return;
        }
      }
      for (const child of node.children) {
        this.renderNode(canvas, child, profile, replayPlane, activeLayer);
      }
      return;
    }
    if (node.kind === 'clipRect') {
      if (!this.currentClipEnabled) {
        this.renderNode(canvas, node.child, profile, replayPlane, activeLayer);
        return;
      }
      this.renderClipNode(canvas, node, profile, replayPlane, activeLayer);
      return;
    }
    this.renderLeaf(canvas, node, profile, replayPlane, activeLayer);
  }

  private staticPictureBounds(node: LayerNode): LayerBounds | null {
    const page = this.currentPageExtent;
    if (!page) return null;
    const visual = node.visualBounds;
    // Font-size-derived ink estimates are not proof of every glyph's outline.
    // Always retain the page extent, including legacy trees without visualBounds.
    if (!visual || !this.boundsAreDrawable(visual)) return page;
    const x = Math.min(page.x, visual.x), y = Math.min(page.y, visual.y);
    const width = Math.max(page.x + page.width, visual.x + visual.width) - x;
    const height = Math.max(page.y + page.height, visual.y + visual.height) - y;
    const union = { x, y, width, height };
    return this.boundsAreDrawable(union) ? union : page;
  }

  private restoreStaticPictureDiagnostics(diagnostics: StaticPictureDiagnostics): void {
    for (const op of diagnostics.unsupported) this.unsupportedOps.add(op);
    for (const [key, failure] of diagnostics.images) this.currentImageFailures.set(key, failure);
    for (const substitution of diagnostics.substitutions) this.recordFontSubstitution(substitution);
    for (const item of diagnostics.equations) this.equationRoutes.add(item);
    for (const item of diagnostics.textRecoveries) this.textRecoveries.add(item);
    for (const item of diagnostics.textFailures) this.textFailures.add(item);
    for (const item of diagnostics.imageRecoveries) this.imageRecoveries.add(item);
    this.patternReplayOps += diagnostics.patternOps;
    this.currentReplayFeatureCounts.dashedStrokes += diagnostics.features.dashedStrokes;
    this.currentReplayFeatureCounts.glyphRuns += diagnostics.features.glyphRuns;
    this.currentReplayFeatureCounts.tabLeaders += diagnostics.features.tabLeaders;
    this.currentReplayFeatureCounts.charOverlaps += diagnostics.features.charOverlaps;
    this.currentReplayFeatureCounts.verticalPresentationPunctuation += diagnostics.features.verticalPresentationPunctuation;
    this.currentReplayFeatureCounts.verticalTextRuns += diagnostics.features.verticalTextRuns;
  }

  private renderClipNode(
    canvas: SkCanvas,
    node: LayerClipNode,
    profile: LayerRenderProfile,
    replayPlane: CanvasKitReplayPlane,
    inheritedLayer: LayerInfo | null,
  ): void {
    canvas.save();
    canvas.clipRect(
      canvasKitCanonicalClipRect(node.clip, (x, y, width, height) =>
        this.canvasKit.XYWHRect(x, y, width, height),
      ),
      this.canvasKit.ClipOp?.Intersect ?? 0,
      true,
    );
    this.renderNode(canvas, node.child, profile, replayPlane, inheritedLayer);
    canvas.restore();
  }

  private renderLeaf(
    canvas: SkCanvas,
    node: LayerLeafNode,
    profile: LayerRenderProfile,
    replayPlane: CanvasKitReplayPlane,
    inheritedLayer: LayerInfo | null,
  ): void {
    const activeLayer = node.layer ?? inheritedLayer;
    for (const op of node.ops) {
      if (layerPaintOpReplayPlane(op, activeLayer) !== replayPlane) {
        continue;
      }
      const equivalenceGroup = 'variant' in op ? op.variant?.equivalenceGroup : undefined;
      if (equivalenceGroup && !this.selectedTextVariantOps.has(op)) {
        continue;
      }
      this.currentOp = op;
      try { this.renderOp(canvas, op, profile); } finally { this.currentOp = null; }
    }
  }

  private renderOp(canvas: SkCanvas, op: LayerPaintOp, profile: LayerRenderProfile): void {
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
          baseline: op.baseline ?? op.fontSize ?? 7,
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
        this.renderEquation(canvas, op);
        return;
      case 'controlLabel':
        this.renderTextRun(canvas, {
          type: 'textRun',
          bbox: op.bbox,
          text: op.label,
          baseline: op.fontSize ?? 10,
          style: {
            fontFamily: 'sans-serif',
            fontSize: op.fontSize ?? 10,
            color: op.color ?? '#cc3333',
          },
        });
        return;
      case 'rawSvg':
        this.unsupportedOps.add('rawSvg:unsupportedDirectReplay');
        return;
      case 'charOverlap':
        this.renderCharOverlap(canvas, op);
        return;
      case 'tabLeader':
        this.renderTabLeader(canvas, op);
        return;
      case 'textControlMark':
        this.renderTextControlMark(canvas, op);
        return;
      case 'textDecoration':
        this.renderTextDecoration(canvas, op);
        return;
      case 'glyphRun':
        this.renderGlyphRun(canvas, op);
        return;
      case 'glyphOutline': {
        const status = glyphOutlinePayloadStatus(op, {
          allowMonochromeFillStroke: true,
          allowColrv0ColorLayers: true,
          allowColrv1Stage1ColorGraph: true,
          allowBitmapGlyph: true,
          allowSvgGlyph: true,
        });
        if (status.supported && this.glyphOutlineVariantReplayable(op)) {
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
    // PageBackgroundNode::render와 같이 이미지가 gradient/solid보다 우선한다.
    if (op.image) {
      this.renderImage(canvas, this.pageBackgroundImageOp(op));
    } else if (op.backgroundColor && !op.gradient) {
      const paint = this.makeFillPaint(op.backgroundColor);
      canvas.drawRect(this.rect(op.bbox), paint);
      paint.delete?.();
    }
    const gradientShader = op.image ? null : this.makeShapeGradientShader(op.gradient, op.bbox);
    if (gradientShader) {
      const paint = this.makeFillPaint('#000000');
      try {
        (paint as unknown as { setShader: (shader: unknown) => void }).setShader(gradientShader);
        canvas.drawRect(this.rect(op.bbox), paint);
      } finally {
        (gradientShader as { delete?: () => void }).delete?.();
        paint.delete?.();
      }
    }
    if (op.borderColor && (op.borderWidth ?? 0) > 0) {
      const paint = this.makeStrokePaint(op.borderColor, op.borderWidth ?? 1);
      canvas.drawRect(this.rect(op.bbox), paint);
      paint.delete?.();
    }
  }

  private pageBackgroundImageOp(op: LayerPageBackgroundOp): LayerImageOp {
    return {
      type: 'image',
      bbox: op.bbox,
      ...op.image,
      imageRef: op.image?.resourceId,
    };
  }

  private renderRectangle(canvas: SkCanvas, op: LayerRectangleOp): void {
    this.drawStyledShape(canvas, op.bbox, op.style, (paint) => {
      const cornerRadius = op.cornerRadius ?? 0;
      if (cornerRadius > 0) {
        canvas.drawRRect(this.canvasKit.RRectXY(this.rect(op.bbox), cornerRadius, cornerRadius), paint);
      } else {
        canvas.drawRect(this.rect(op.bbox), paint);
      }
    }, op.gradient);
  }

  private renderEllipse(canvas: SkCanvas, op: LayerEllipseOp): void {
    this.drawStyledShape(canvas, op.bbox, op.style, (paint) => {
      canvas.drawOval(this.rect(op.bbox), paint);
    }, op.gradient);
  }

  private renderLine(canvas: SkCanvas, op: LayerLineOp): void {
    const style = op.style ?? {};
    const color = style.color ?? '#000000';
    const width = style.width ?? 1;
    const x1 = op.x1;
    const y1 = op.y1;
    const x2 = op.x2;
    const y2 = op.y2;
    if (![x1, y1, x2, y2, width].every(Number.isFinite)) {
      this.unsupportedOps.add('line:invalidGeometry');
      return;
    }
    const shadow = this.resolvedShadow(style.shadow);
    if (shadow) {
      canvas.save();
      try {
        canvas.translate(shadow.offsetX, shadow.offsetY);
        this.drawCompoundLine(
          canvas,
          x1,
          y1,
          x2,
          y2,
          { ...style, color: shadow.color, width },
          shadow.opacity,
        );
      } finally {
        canvas.restore();
      }
    }
    this.drawCompoundLine(canvas, x1, y1, x2, y2, style, 1);
    // A zero-length straight line has no endpoint direction, matching the producer.
    if (x1 !== x2 || y1 !== y2) this.drawLineArrows(canvas, op.arrowHeads, style, color, width);
  }

  private renderPath(canvas: SkCanvas, op: LayerPathOp): void {
    const path = this.pathFromCommands(op.commands ?? [], op.bbox.x, op.bbox.y);
    const style: LayerShapeStyle = op.style ?? (op.lineStyle ? {} : {
      strokeColor: '#000000',
      strokeWidth: 1,
      fillColor: null,
    });
    const replayStyle: LayerShapeStyle = {
      ...style,
      strokeColor: style.strokeColor ?? op.lineStyle?.color,
      strokeWidth: op.lineStyle?.width ?? style.strokeWidth,
      strokeDash: op.lineStyle?.dash ?? style.strokeDash,
      shadow: style.shadow ?? op.lineStyle?.shadow,
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
    this.drawStyledPath(canvas, path, replayStyle, op.bbox, op.gradient);
    if (op.lineStyle) {
      this.drawLineArrows(canvas, op.arrowHeads, op.lineStyle,
        op.lineStyle.color ?? replayStyle.strokeColor ?? '#000000',
        op.lineStyle.width ?? replayStyle.strokeWidth ?? 1);
    }
    if (needsTransform) {
      canvas.restore();
    }
    path.delete?.();
  }

  private pathFromCommands(commands: readonly LayerPathCommand[], x = 0, y = 0): Path {
    const builder = new this.canvasKit.PathBuilder();
    try {
      for (const command of commands) [x, y] = this.applyPathCommand(builder, command, x, y);
      return builder.detach();
    } finally { builder.delete(); }
  }

  private applyPathCommand(path: Pick<PathBuilder, 'arcToRotated' | 'close' | 'cubicTo' | 'lineTo' | 'moveTo'>, command: LayerPathCommand, currentX: number, currentY: number): [number, number] {
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
    if (!op.base64 && op.imageRef === undefined && !op.sourceImageKey) {
      this.recordImageFailure(op, 'dataMissing', null);
      this.unsupportedOps.add('image:dataMissing');
      return;
    }
    const image = this.imageForOp(op);
    if (!image) {
      if (this.pendingImageLoads.size > 0) return;
      this.unsupportedOps.add('image:decodeFailed');
      return;
    }
    this.withImageTransform(canvas, op.bbox, op.transform, () => this.drawImageOp(canvas, image, op));
  }

  private renderGlyphRun(canvas: SkCanvas, op: LayerGlyphRunOp): void {
    const font = this.glyphRunFonts.font(op, this.currentFontResources);
    if (!font) {
      this.unsupportedOps.add('glyphRun:replayInvariant');
      return;
    }
    const style = op.paintStyle;
    const paint = this.makeFillPaint(style.color ?? '#000000');
    const effects: SkPaint[] = [];
    const draw = (current: SkPaint, x = 0, y = 0) => {
      if (!drawCanvasKitGlyphRun(canvas, op, font, current, x, y)) throw new Error('glyphRun:replayFailed');
    };
    const effectPaint = (color: string) => {
      const effect = this.makeFillPaint(color); effects.push(effect); return effect;
    };
    try {
      const size = style.fontSize ?? op.shapeKey.fontInstance.sizePx;
      if (style.emboss || style.engrave) {
        const offset = Math.max(size / 20, 1);
        draw(effectPaint(style.emboss ? '#ffffff' : '#808080'), -offset, -offset);
        draw(effectPaint(style.emboss ? '#808080' : '#ffffff'), offset, offset);
        draw(paint);
      } else {
        if ((style.shadowType ?? 0) > 0) {
          draw(effectPaint(style.shadowColor || '#000000'), style.shadowOffsetX ?? 0, style.shadowOffsetY ?? 0);
        }
        if ((style.outlineType ?? 0) > 0) {
          const stroke = this.makeStrokePaint(style.color ?? '#000000', Math.max(size / 25, 0.5));
          effects.push(stroke); stroke.setStrokeJoin(this.canvasKit.StrokeJoin.Round);
          draw(effectPaint('#ffffff')); draw(stroke);
        } else draw(paint);
      }
      this.currentReplayFeatureCounts.glyphRuns += 1;
    } catch {
      this.unsupportedOps.add('glyphRun:replayFailed');
    } finally {
      for (const effect of effects) effect.delete();
      paint.delete?.();
    }
  }

  private renderGlyphOutline(canvas: SkCanvas, op: LayerGlyphOutlineOp): void {
    if (op.payloadKind === 'bitmapGlyph') {
      this.renderBitmapGlyphOutline(canvas, op);
      return;
    }
    if (op.payloadKind === 'svgGlyph') {
      this.renderSvgGlyphOutline(canvas, op);
      return;
    }
    if (op.payloadKind === 'monochromeFill' || op.payloadKind === 'monochromeFillStroke') {
      this.renderMonochromeGlyphOutline(canvas, op);
      return;
    }
    if (op.colorLayers?.colorFormat === 'colrV0') {
      const status = glyphOutlinePayloadStatus(op, { allowColrv0ColorLayers: true });
      if (!status.supported) {
        this.unsupportedOps.add('glyphOutline:replayInvariant');
        return;
      }
      canvas.save();
      try {
        const placement = this.affineToCanvasKitMatrix(op.placement?.runToPage);
        if (placement) canvas.concat(placement);
        for (const layer of op.colorLayers.layers ?? []) {
          const path = this.pathFromCommands(layer.commands ?? []);
          const paint = new this.canvasKit.Paint();
          canvas.save();
          try {
            const transform = this.affineToCanvasKitMatrix(layer.transformToRun);
            if (transform) canvas.concat(transform);
            this.applyGlyphPathFillRule(path, layer.fillRule);
            paint.setAntiAlias(true);
            paint.setStyle(this.canvasKit.PaintStyle.Fill);
            const color = this.resolvedColor(layer.fill ?? {});
            color[3] *= layer.opacity ?? 1;
            paint.setColor(color);
            canvas.drawPath(path, paint);
          } finally { canvas.restore(); paint.delete(); path.delete(); }
        }
      } finally { canvas.restore(); }
      return;
    }
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

  private renderBitmapGlyphOutline(canvas: SkCanvas, op: LayerGlyphOutlineOp): void {
    const imageOp = this.bitmapGlyphImageOp(op);
    const image = imageOp ? this.imageForOp(imageOp) : null;
    if (!imageOp || !image) {
      this.unsupportedOps.add('glyphOutline:bitmapReplayInvariant');
      return;
    }
    canvas.save();
    try {
      const transform = op.bitmapGlyph?.transformToRun;
      const matrix = this.affineToCanvasKitMatrix(transform);
      if (matrix) (canvas as unknown as { concat: (matrix: number[]) => void }).concat(matrix);
      this.drawImageOp(canvas, image, imageOp, op.bitmapGlyph?.filtering === 'nearest' ? 'nearest' : 'linear');
    } finally {
      canvas.restore();
    }
  }

  private renderSvgGlyphOutline(canvas: SkCanvas, op: LayerGlyphOutlineOp): void {
    const payload = op.svgGlyph;
    const viewBox = payload?.viewBox;
    const placement = payload?.fontEmBox ?? op.bbox;
    const layers = this.staticSvgGlyphPathLayers(op);
    if (!payload || !viewBox || !layers || !this.boundsAreDrawable(placement) || !this.boundsAreDrawable(viewBox)) {
      this.unsupportedOps.add('glyphOutline:svgReplayInvariant');
      return;
    }
    canvas.save();
    try {
      const payloadMatrix = this.affineToCanvasKitMatrix(payload.transformToRun);
      if (payloadMatrix) {
        (canvas as unknown as { concat: (matrix: number[]) => void }).concat(payloadMatrix);
      }
      canvas.translate(placement.x, placement.y);
      canvas.scale(placement.width / viewBox.width, placement.height / viewBox.height);
      canvas.translate(-viewBox.x, -viewBox.y);
      this.drawStaticSvgPathLayers(canvas, layers);
    } finally {
      canvas.restore();
    }
  }

  private drawStaticSvgPathLayers(canvas: SkCanvas, layers: readonly StaticSvgPathLayer[]): void {
    for (const layer of layers) {
      canvas.save();
      let path: Path | null = null;
      try {
        const layerMatrix = this.affineToCanvasKitMatrix(layer.transform);
        if (layerMatrix) {
          (canvas as unknown as { concat: (matrix: number[]) => void }).concat(layerMatrix);
        }
        path = this.canvasKit.Path.MakeFromSVGString(layer.pathData);
        if (!path) throw new Error('정적 SVG path decode 실패');
        this.applyGlyphPathFillRule(path, layer.fillRule);
        if (layer.fill !== null) {
          let paint: SkPaint | null = null;
          try {
            paint = this.makeFillPaint(layer.fill, layer.opacity);
            canvas.drawPath(path, paint);
          } finally {
            paint?.delete?.();
          }
        }
        if (layer.stroke) {
          const stroke = layer.stroke;
          let paint: SkPaint | null = null;
          let effect: ReturnType<typeof this.canvasKit.PathEffect.MakeDash> | null = null;
          try {
            paint = this.makeStrokePaint(stroke.color, stroke.width, stroke.opacity);
            paint.setStrokeJoin(this.canvasKit.StrokeJoin[
              stroke.lineJoin === 'round' ? 'Round' : stroke.lineJoin === 'bevel' ? 'Bevel' : 'Miter'
            ]);
            paint.setStrokeCap(this.canvasKit.StrokeCap[
              stroke.lineCap === 'round' ? 'Round' : stroke.lineCap === 'square' ? 'Square' : 'Butt'
            ]);
            paint.setStrokeMiter(stroke.miterLimit);
            effect = stroke.dashArray
              ? this.canvasKit.PathEffect.MakeDash(stroke.dashArray, stroke.dashOffset)
              : null;
            if (effect) paint.setPathEffect(effect);
            canvas.drawPath(path, paint);
          } finally {
            effect?.delete?.();
            paint?.delete?.();
          }
        }
      } finally {
        path?.delete?.();
        canvas.restore();
      }
    }
  }

  private renderMonochromeGlyphOutline(canvas: SkCanvas, op: LayerGlyphOutlineOp): void {
    const matrix = this.affineToCanvasKitMatrix(op.placement?.runToPage);
    if (!matrix || !op.paths?.length) {
      this.unsupportedOps.add('glyphOutline:replayInvariant');
      return;
    }
    const fill = this.makeFillPaint(op.paintStyle?.color ?? '#000000');
    const stroke = op.payloadKind === 'monochromeFillStroke' && op.stroke
      ? this.makeStrokePaint(op.stroke.color ?? op.paintStyle?.color ?? '#000000', op.stroke.width ?? 1)
      : null;
    if (stroke) {
      // The admission contract currently proves only the miter/butt strict subset.
      stroke.setStrokeJoin(this.canvasKit.StrokeJoin.Miter);
      stroke.setStrokeCap(this.canvasKit.StrokeCap.Butt);
      stroke.setStrokeMiter(op.stroke!.miterLimit!);
    }
    canvas.save();
    try {
      (canvas as unknown as { concat: (matrix: number[]) => void }).concat(matrix);
      for (const outline of op.paths) {
        const path = this.pathFromCommands(outline.commands ?? []);
        try {
          this.applyGlyphPathFillRule(path, outline.fillRule);
          if (stroke && op.stroke?.paintOrder === 'strokeThenFill') canvas.drawPath(path, stroke);
          canvas.drawPath(path, fill);
          if (stroke && op.stroke?.paintOrder !== 'strokeThenFill') canvas.drawPath(path, stroke);
        } finally {
          path.delete?.();
        }
      }
    } finally {
      canvas.restore();
      stroke?.delete?.();
      fill.delete?.();
    }
  }

  private applyGlyphPathFillRule(path: Path, fillRule: string | undefined): void {
    path.setFillType(fillRule === 'evenodd' ? this.canvasKit.FillType.EvenOdd : this.canvasKit.FillType.Winding);
  }

  private renderColorPaintGraphNode(
    canvas: SkCanvas,
    nodesById: Map<number, LayerColorGraphNode>,
    nodeId: number,
    visited: Set<number>,
    budget = { remaining: 4096 },
    depth = 0,
  ): void {
    if (visited.has(nodeId) || depth >= 64 || --budget.remaining < 0) {
      this.unsupportedOps.add('glyphOutline:replayInvariant');
      return;
    }
    visited.add(nodeId);
    try {
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
          this.renderColorPaintGraphNode(canvas, nodesById, transformNode.childNodeId, visited, budget, depth + 1);
        } finally {
          canvas.restore();
        }
        return;
      }
      if (node.kind === 'composite' && node.composite?.mode === 'sourceOver') {
        const { sourceNodeId, backdropNodeId } = node.composite;
        if (sourceNodeId === undefined || backdropNodeId === undefined) {
          this.unsupportedOps.add('glyphOutline:replayInvariant');
          return;
        }
        this.renderColorPaintGraphNode(canvas, nodesById, backdropNodeId, visited, budget, depth + 1);
        this.renderColorPaintGraphNode(canvas, nodesById, sourceNodeId, visited, budget, depth + 1);
        return;
      }
      if (node.kind === 'clip' && node.clip?.clipCommands && node.clip.childNodeId !== undefined) {
        const path = this.pathFromCommands(node.clip.clipCommands);
        canvas.save();
        try {
          this.applyFillRule(path, node.clip.fillRule);
          canvas.clipPath(path, this.canvasKit.ClipOp.Intersect, true);
          this.renderColorPaintGraphNode(canvas, nodesById, node.clip.childNodeId, visited, budget, depth + 1);
        } finally {
          canvas.restore();
          path.delete?.();
        }
        return;
      }
      const pathNode = node.solidPath ?? node.linearGradientPath ?? node.radialGradientPath ?? node.sweepGradientPath;
      if (!pathNode?.commands) {
        this.unsupportedOps.add('glyphOutline:replayInvariant');
        return;
      }
      const path = this.pathFromCommands(pathNode.commands);
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
    } finally {
      visited.delete(nodeId);
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

  private applyFillRule(path: Path, fillRule: string | undefined): void {
    if (fillRule === 'evenodd') {
      (path as unknown as { setFillType?: (fillType: unknown) => void }).setFillType?.(this.canvasKit.FillType.EvenOdd);
    }
  }

  private resolvedColor(color: { rgba?: number[] }): Color {
    const rgba = color.rgba ?? [0, 0, 0, 1];
    return this.canvasKit.Color(
      clampUnit(rgba[0]) * 255,
      clampUnit(rgba[1]) * 255,
      clampUnit(rgba[2]) * 255,
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

  private drawImageOp(canvas: SkCanvas, image: SkImage, op: LayerImageOp, bitmapFiltering?: 'nearest' | 'linear'): void {
    if (imageCropSelectionIsEmpty(op.crop)) return;
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

    const crop = this.croppedEffectImages.has(image) ? null : canvasKitImageSourceRect(
      imageWidth,
      imageHeight,
      op.crop,
      op.originalSizeHu,
    );
    const opacity = Number.isFinite(op.opacity) ? Math.max(0, Math.min(1, op.opacity ?? 1)) : 1;
    const drawImage = (dstX: number, dstY: number, dstW: number, dstH: number) => {
      const src = crop
        ? this.canvasKit.XYWHRect(crop.x, crop.y, crop.width, crop.height)
        : this.canvasKit.XYWHRect(0, 0, imageWidth, imageHeight);
      // imageForOp rejects failed preprocessing before replay. Only binary
      // source-pixel effects need nearest to preserve their threshold/dither;
      // grayscale and tone retain the profile's ordinary sampling policy.
      const binaryEffectWasApplied = !!canvasKitImageEffectKey(op)
        && (op.effect === 'blackWhite' || op.effect === 'pattern8x8');
      const useMipmaps = bitmapFiltering === undefined && !binaryEffectWasApplied
        && this.currentProfile !== 'fastPreview' && this.preferRasterDepth === 0
        && (this.renderMode === 'compat' || this.currentProfile === 'print' || this.currentProfile === 'highQuality')
        && ((crop?.width ?? imageWidth) > dstW * 1.2 || (crop?.height ?? imageHeight) > dstH * 1.2);
      this.drawImageRect(canvas, image, src, this.canvasKit.XYWHRect(dstX, dstY, dstW, dstH), opacity,
        bitmapFiltering ?? (binaryEffectWasApplied ? 'nearest' : 'linear'), useMipmaps);
    };

    const fillMode = op.fillMode ?? 'fitToSize';
    if (canvasKitImageFillModeStretches(fillMode)) {
      drawImage(op.bbox.x, op.bbox.y, op.bbox.width, op.bbox.height);
      return;
    }
    // [#7235] `none`·`zoom` 은 영역에 맞춰 종횡비를 지키며 축소해 가운데 놓는다.
    // 종횡비 기준은 잘린 뒤의 픽셀 영역이다 — 타일 기준인 `originalSize`(HWP
    // shape_attr) 가 아니다. SVG 의 `preserveAspectRatio="xMidYMid meet"` 와 같다.
    if (canvasKitImageFillModeContains(fillMode)) {
      const effectCrop = this.croppedEffectImages.get(image);
      const fit = canvasKitImageContainRect(op.bbox,
        crop?.width ?? effectCrop?.width ?? imageWidth, crop?.height ?? effectCrop?.height ?? imageHeight);
      drawImage(fit.x, fit.y, fit.width, fit.height);
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

  private drawImageRect(canvas: SkCanvas, image: SkImage, source: Rect, dest: Rect, opacity = 1,
    filtering: 'nearest' | 'linear' = 'linear', useMipmaps = false): void {
    const paint = new this.canvasKit.Paint();
    paint.setAntiAlias?.(true);
    if (opacity < 1) {
      paint.setAlphaf(opacity);
    }
    let sampled = image;
    let temporary: SkImage | null = null;
    if (useMipmaps && typeof image.makeCopyWithDefaultMipmaps === 'function') {
      let key = this.imageNativeKeys.get(image);
      if (!key) { key = `nativeImage:${this.nextImageKey++}`; this.imageNativeKeys.set(image, key); }
      const cached = this.mipmapImages.get(key);
      if (cached) sampled = cached;
      else {
        try {
          const copy = image.makeCopyWithDefaultMipmaps();
          if (copy) {
            sampled = copy;
            if (!this.mipmapImages.set(key, copy, Math.ceil(image.width() * image.height() * 16 / 3))) temporary = copy;
          }
        } catch { /* Mipmaps are optional; the original native image remains valid. */ }
      }
    }
    try {
      if (typeof canvas.drawImageRectOptions === 'function') {
        canvas.drawImageRectOptions(sampled, source, dest,
          filtering === 'nearest' ? this.canvasKit.FilterMode.Nearest : this.canvasKit.FilterMode.Linear,
          sampled !== image ? this.canvasKit.MipmapMode.Linear : this.canvasKit.MipmapMode.None, paint);
      } else canvas.drawImageRect(image, source, dest, paint);
    } finally {
      temporary?.delete();
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

  private recordTextRunCoverageGaps(op: LayerTextRunOp, codePoints: readonly string[]): boolean {
    const style = op.style ?? {};
    const decorationsAreExternal = op.legacyVisuals?.decorations === 'mirror';
    if (!decorationsAreExternal && style.underline && style.underline !== 'none') {
      this.unsupportedOps.add('textRun:textDecoration');
    }
    if (!decorationsAreExternal && style.strikethrough) {
      this.unsupportedOps.add('textRun:textDecoration');
    }
    if (!decorationsAreExternal && style.emphasisDot && style.emphasisDot !== 0) {
      this.unsupportedOps.add('textRun:emphasisDot');
    }
    const replayText = op.displayText ?? op.text;
    const hasOldHangul = codePoints.some((character) => {
      const codePoint = character.codePointAt(0) ?? 0;
      return (codePoint >= 0x1100 && codePoint <= 0x11ff)
        || (codePoint >= 0xa960 && codePoint <= 0xa97f)
        || (codePoint >= 0xd7b0 && codePoint <= 0xd7ff);
    });
    const requiresUnsupportedShaping = textRequiresComplexShaping(replayText)
      || (textRunHasPaintEffects(style) && hasOldHangul);
    if (requiresUnsupportedShaping) {
      this.unsupportedOps.add('textRun:scriptTextRequiresShaping');
    }
    return requiresUnsupportedShaping;
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
    if (op.charOverlap && op.legacyVisuals?.charOverlap === 'mirror') {
      return;
    }
    if (op.charOverlap) {
      this.renderCharOverlap(canvas, {
        type: 'charOverlap',
        bbox: op.bbox,
        text: op.text,
        baseline: op.baseline ?? op.style?.fontSize ?? 12,
        rotation: op.rotation ?? 0,
        isVertical: op.isVertical === true,
        style: op.style ?? {},
        positions: op.positions ?? [],
        positionsComplete: op.positions !== undefined,
        charOverlap: op.charOverlap,
      });
      return;
    }
    const replayText = op.displayText ?? op.text;
    const replayPositions = op.displayText !== undefined ? op.displayPositions : op.positions;
    if (!replayText) return;
    const replayCodePoints: string[] = [];
    for (const character of replayText) {
      if (replayCodePoints.length >= CanvasKitLayerRenderer.MAX_TEXT_RUN_CODE_POINTS) {
        this.unsupportedOps.add('textRun:visualItemLimitExceeded');
        return;
      }
      replayCodePoints.push(character);
    }
    const style = op.style ?? {};
    if (this.recordTextRunCoverageGaps(op, replayCodePoints)) return;
    const ratio = style.ratio ?? 1;
    const outlineType = style.outlineType ?? 0;
    const shadowType = style.shadowType ?? 0;
    const shadowOffsetX = style.shadowOffsetX ?? 0;
    const shadowOffsetY = style.shadowOffsetY ?? 0;
    const shadeColor = (style.shadeColor ?? '#ffffff').toLowerCase();
    const baseFontSize = style.fontSize ?? Math.max(1, op.bbox.height || 12);
    const baseline = op.baseline ?? baseFontSize;
    const rotation = op.rotation ?? 0;
    if (![op.bbox.x, op.bbox.y, op.bbox.width, op.bbox.height, ratio, outlineType,
      shadowType, shadowOffsetX, shadowOffsetY, baseFontSize, baseline, rotation]
      .every(Number.isFinite)
      || op.bbox.width < 0
      || op.bbox.height < 0
      || ratio <= 0
      || baseFontSize <= 0
      || !Number.isInteger(outlineType)
      || outlineType < 0
      || !Number.isInteger(shadowType)
      || shadowType < 0) {
      this.unsupportedOps.add('textRun:invalidGeometry');
      return;
    }
    const verticalPresentationText = op.isVertical
      && op.orientation !== 'vertical-sideways'
      ? VERTICAL_PRESENTATION_BASE_TEXT.get(replayText)
      : undefined;
    const glyphReplayText = verticalPresentationText ?? replayText;
    const codePoints = verticalPresentationText === undefined
      ? replayCodePoints
      : [verticalPresentationText];
    const replayUtf16Offsets = new Uint32Array(codePoints.length + 1);
    for (let index = 0; index < codePoints.length; index++) replayUtf16Offsets[index + 1] = replayUtf16Offsets[index] + codePoints[index].length;
    const hasOldHangul = codePoints.some((codePoint) => {
      const code = codePoint.codePointAt(0) ?? 0;
      return (code >= 0x1100 && code <= 0x11ff)
        || (code >= 0xa960 && code <= 0xa97f)
        || (code >= 0xd7b0 && code <= 0xd7ff);
    });
    const effectPaints: SkPaint[] = [];
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
      : op.bbox.y + baseline;
    const needsPreservedAdvances = style.superscript || style.subscript;
    const hasLayoutPositions = replayPositions?.length === codePoints.length + 1
      && replayPositions.every(Number.isFinite);
    const requestedFontFamily = primaryFontFamily(style.fontFamily);
    const preparedTypeface = this.findPreparedTypeface(requestedFontFamily, style);
    if (requestedFontFamily && !preparedTypeface && this.requirePreparedFontFamilies) {
      throw new Error(`CanvasKit font family가 준비되지 않았습니다: ${requestedFontFamily}`);
    }
    if (requestedFontFamily && !preparedTypeface) {
      this.recordFontSubstitution({
        requestedFamily: requestedFontFamily,
        resolvedFamily: this.defaultFontFamily ?? 'CanvasKit default',
        source: 'unregisteredDefault',
        kind: 'unregisteredFallback',
      });
    }
    const typeface = preparedTypeface?.typeface ?? this.defaultTypeface;
    const fontManager = preparedTypeface?.fontManager ?? this.defaultFontManager;
    const paint = this.makeFillPaint(style.color ?? '#000000');
    let font: Font | null = null;
    const fallbackFonts: Font[] = [];
    let boxedPuaFont: Font | null = null;
    let boxedPuaStrokePaint: SkPaint | null = null;
    let canvasSaved = false;
    try {
      paint.setAntiAlias?.(true);
      if (!typeface && !fontManager && !this.symbolFallbackTypeface
        && /[^\u0000-\u00ff]/.test(replayText)) {
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

      {
        const adjustFont = (target: Font, selected: CanvasKitLocalTypeface | null = preparedTypeface) => {
          const adjustable = target as Font & {
            setEmbolden?: (enabled: boolean) => void;
            setSkewX?: (skew: number) => void;
            setScaleX?: (scale: number) => void;
          };
          const physical = target === font && preparedTypeface?.record
            ? localFontFaceStyle(preparedTypeface.record) : null;
          adjustable.setEmbolden?.(selected?.syntheticStyle?.bold
            ?? (style.bold === true && (selected?.hostFace?.weight ?? physical?.weight ?? 400) < 600));
          const physicalItalic = selected?.hostFace
            ? ['italic', 'oblique'].includes(selected.hostFace.slant ?? '') : physical?.italic === true;
          adjustable.setSkewX?.((selected?.syntheticStyle?.italic ?? (style.italic === true && !physicalItalic)) ? -0.2 : 0);
          adjustable.setScaleX?.(ratio);
        };
        font = this.makeFont(typeface, fontSize);
        adjustFont(font);
        const candidateFonts = [font];
        const candidateFontSources: CanvasKitFontSubstitutionDiagnostic['source'][] = [
          'unregisteredDefault',
        ];
        const candidateFontFamilies = [
          preparedTypeface?.fontFamily ?? this.defaultFontFamily ?? 'CanvasKit default',
        ];
        let candidateGlyphIds: Uint16Array[] = [];
        const fallbackSpans: Array<{ start: number; end: number; fontIndex: number }> = [];
        let oldHangulTypeface: CanvasKitLocalTypeface | null = null;
        if (hasLayoutPositions) {
          const primaryGlyphIds = font.getGlyphIDs(glyphReplayText, codePoints.length);
          candidateGlyphIds = [primaryGlyphIds];
          oldHangulTypeface = hasOldHangul
            ? this.findPreparedTypeface(OLD_HANGUL_FONT_FAMILY)
            : null;
          if (primaryGlyphIds.some(glyphId => glyphId === 0)
            && this.defaultTypeface !== null
            && typeface !== this.defaultTypeface) {
            const defaultFont = this.makeFont(this.defaultTypeface, fontSize);
            adjustFont(defaultFont, null);
            fallbackFonts.push(defaultFont);
            candidateFonts.push(defaultFont);
            candidateFontSources.push('missingGlyphDefault');
            candidateFontFamilies.push(this.defaultFontFamily ?? 'CanvasKit default');
            candidateGlyphIds.push(defaultFont.getGlyphIDs(glyphReplayText, codePoints.length));
          }
          if (codePoints.some((_, index) => candidateGlyphIds.every(ids => (ids[index] ?? 0) === 0))
            && this.symbolFallbackTypeface !== null
            && typeface !== this.symbolFallbackTypeface
            && this.defaultTypeface !== this.symbolFallbackTypeface) {
            const symbolFont = this.makeFont(this.symbolFallbackTypeface, fontSize);
            adjustFont(symbolFont, null);
            fallbackFonts.push(symbolFont);
            candidateFonts.push(symbolFont);
            candidateFontSources.push('missingGlyphSymbol');
            candidateFontFamilies.push('CanvasKit symbol fallback');
            candidateGlyphIds.push(symbolFont.getGlyphIDs(glyphReplayText, codePoints.length));
          }
          const selectedFontIndices = codePoints.map((codePoint, index) => {
            const code = codePoint.codePointAt(0) ?? 0;
            if ((code >= 0x1100 && code <= 0x11ff)
              || (code >= 0xa960 && code <= 0xa97f)
              || (code >= 0xd7b0 && code <= 0xd7ff)) {
              return -2;
            }
            const candidateIndex = candidateGlyphIds.findIndex(ids => (ids[index] ?? 0) !== 0);
            if (candidateIndex >= 0) return candidateIndex;
            return code >= 0xF02B1 && code <= 0xF02C4 ? -1 : 0;
          });
          for (const fontIndex of new Set(selectedFontIndices)) {
            if (fontIndex > 0) {
              this.recordFontSubstitution({
                requestedFamily: requestedFontFamily || this.defaultFontFamily || 'CanvasKit default',
                resolvedFamily: candidateFontFamilies[fontIndex],
                source: candidateFontSources[fontIndex],
                kind: 'glyphCoverageFallback',
              });
            } else if (fontIndex === -2 && oldHangulTypeface?.fontManager) {
              this.recordFontSubstitution({
                requestedFamily: requestedFontFamily || this.defaultFontFamily || 'CanvasKit default',
                resolvedFamily: oldHangulTypeface.fontFamily ?? OLD_HANGUL_FONT_FAMILY,
                source: 'oldHangul',
                kind: 'glyphCoverageFallback',
              });
            }
          }
          let spanStart = 0;
          while (spanStart < codePoints.length) {
            const fontIndex = selectedFontIndices[spanStart];
            let spanEnd = spanStart + 1;
            if (fontIndex !== -1) {
              while (spanEnd < codePoints.length && selectedFontIndices[spanEnd] === fontIndex) {
                spanEnd += 1;
              }
            }
            fallbackSpans.push({ start: spanStart, end: spanEnd, fontIndex });
            if (fallbackSpans.length > CanvasKitLayerRenderer.MAX_TEXT_RUN_FALLBACK_SPANS) {
              this.unsupportedOps.add('textRun:fallbackSpanLimitExceeded');
              return;
            }
            spanStart = spanEnd;
          }
        }

        const drawPass = (
          fillPaint: SkPaint,
          offsetX = 0,
          offsetY = 0,
          strokePaint?: SkPaint,
        ) => {
          if (verticalPresentationText !== undefined) {
            const selectedFont = hasLayoutPositions
              ? candidateFonts[fallbackSpans[0]?.fontIndex ?? 0] ?? font!
              : font!;
            const glyphIds = selectedFont.getGlyphIDs(verticalPresentationText, 1);
            const glyphBounds = (
              selectedFont as Font & {
                getGlyphBounds?: (ids: Uint16Array) => Float32Array;
              }
            ).getGlyphBounds?.(glyphIds) ?? new Float32Array();
            const left = glyphBounds[0] ?? 0;
            const top = glyphBounds[1] ?? -fontSize;
            const right = glyphBounds[2] ?? fontSize;
            const bottom = glyphBounds[3] ?? 0;
            const advance = hasLayoutPositions
              ? replayPositions![1] - replayPositions![0]
              : op.bbox.width;
            const targetCenterX = originX
              + (Number.isFinite(advance) ? advance : op.bbox.width) / 2;
            const targetCenterY = originY - baseline + baselineShift + op.bbox.height / 2;
            canvas.save();
            try {
              canvas.translate(targetCenterX + offsetX, targetCenterY + offsetY);
              canvas.rotate(90, 0, 0);
              this.drawCachedText(canvas,
                verticalPresentationText,
                -(left + right) / 2,
                -(top + bottom) / 2,
                fillPaint,
                selectedFont,
              );
              if (strokePaint) {
                this.drawCachedText(canvas,
                  verticalPresentationText,
                  -(left + right) / 2,
                  -(top + bottom) / 2,
                  strokePaint,
                  selectedFont,
                );
              }
            } finally {
              canvas.restore();
            }
            return;
          }

          if (!hasLayoutPositions) {
            const y = originY + baselineShift + offsetY;
            this.drawCachedText(canvas, glyphReplayText, originX + offsetX, y, fillPaint, font!);
            if (strokePaint) {
              this.drawCachedText(canvas, glyphReplayText, originX + offsetX, y, strokePaint, font!);
            }
            return;
          }

          let hasMissingGlyph = false;
          for (const { start: runStart, end: runEnd, fontIndex } of fallbackSpans) {
            if (fontIndex === -2) {
              if (!this.renderShapedScriptText(
                canvas,
                codePoints.slice(runStart, runEnd).join(''),
                style.color ?? '#000000',
                fontSize,
                originX + replayPositions![runStart] + offsetX,
                originY + offsetY,
                baselineShift,
                oldHangulTypeface?.fontManager ?? null,
                oldHangulTypeface?.fontFamily ?? OLD_HANGUL_FONT_FAMILY,
                style.bold === true,
                style.italic === true,
              )) {
                hasMissingGlyph = true;
              }
              continue;
            }
            if (fontIndex === -1) {
              const codePoint = codePoints[runStart].codePointAt(0) ?? 0;
              const displayNumber = String(codePoint - 0xF02B0);
              const boxSize = Math.max(1, fontSize * 0.72);
              const boxX = originX + replayPositions![runStart];
              const boxY = originY + baselineShift - fontSize * 0.76;
              boxedPuaStrokePaint ??= this.makeStrokePaint(
                style.color ?? '#000000',
                Math.max(0.6, fontSize * 0.04),
              );
              boxedPuaFont ??= this.makeFont(
                this.symbolFallbackTypeface ?? this.defaultTypeface ?? typeface,
                Math.max(1, fontSize * 0.5),
              );
              adjustFont(boxedPuaFont, null);
              const numberGlyphIds = boxedPuaFont.getGlyphIDs(
                displayNumber,
                displayNumber.length,
              );
              const numberWidth = (boxedPuaFont.getGlyphWidths(numberGlyphIds) ?? [])
                .reduce((sum, width) => sum + width, 0);
              canvas.drawRect(
                this.canvasKit.XYWHRect(
                  boxX + offsetX,
                  boxY + offsetY,
                  boxSize,
                  boxSize,
                ),
                boxedPuaStrokePaint,
              );
              this.drawCachedText(canvas,
                displayNumber,
                boxX + (boxSize - numberWidth) / 2 + offsetX,
                boxY + boxSize * 0.72 + offsetY,
                fillPaint,
                boxedPuaFont,
              );
              continue;
            }
            const runGlyphIds = new Uint16Array(runEnd - runStart);
            const runPositions = new Float32Array((runEnd - runStart) * 2);
            for (let index = runStart; index < runEnd; index += 1) {
              const glyphId = candidateGlyphIds[fontIndex][index] ?? 0;
              runGlyphIds[index - runStart] = glyphId;
              runPositions[(index - runStart) * 2] = replayPositions![index];
              runPositions[(index - runStart) * 2 + 1] = baselineShift;
              hasMissingGlyph ||= glyphId === 0;
            }
            this.drawCachedGlyphs(canvas,
              runGlyphIds,
              runPositions,
              originX + offsetX,
              originY + offsetY,
              candidateFonts[fontIndex],
              fillPaint,
              { start: replayUtf16Offsets[runStart], length: replayUtf16Offsets[runEnd] - replayUtf16Offsets[runStart] },
            );
            if (strokePaint) {
              this.drawCachedGlyphs(canvas,
                runGlyphIds,
                runPositions,
                originX + offsetX,
                originY + offsetY,
                candidateFonts[fontIndex],
                strokePaint,
                { start: replayUtf16Offsets[runStart], length: replayUtf16Offsets[runEnd] - replayUtf16Offsets[runStart] },
              );
            }
          }
          if (hasMissingGlyph) this.unsupportedOps.add('textRun:glyphMapping');
        };

        if (!hasLayoutPositions && needsPreservedAdvances) {
          this.unsupportedOps.add('textRun:layoutPositions');
        }
        const textWidth = hasLayoutPositions
          ? replayPositions!.at(-1) ?? op.bbox.width
          : op.bbox.width;
        if (textWidth > 0 && shadeColor !== '#ffffff' && shadeColor !== '#000000') {
          const shadePaint = this.makeFillPaint(shadeColor);
          effectPaints.push(shadePaint);
          canvas.drawRect(
            this.canvasKit.XYWHRect(
              originX,
              originY + baselineShift - fontSize,
              textWidth,
              fontSize * 1.2,
            ),
            shadePaint,
          );
        }

        if (style.emboss || style.engrave) {
          const offset = Math.max(fontSize / 20, 1);
          const firstPaint = this.makeFillPaint(style.emboss ? '#ffffff' : '#808080');
          effectPaints.push(firstPaint);
          const secondPaint = this.makeFillPaint(style.emboss ? '#808080' : '#ffffff');
          effectPaints.push(secondPaint);
          drawPass(firstPaint, -offset, -offset);
          drawPass(secondPaint, offset, offset);
          drawPass(paint);
        } else {
          if (shadowType > 0) {
            const shadowPaint = this.makeFillPaint(style.shadowColor ?? style.color ?? '#000000');
            effectPaints.push(shadowPaint);
            drawPass(shadowPaint, shadowOffsetX, shadowOffsetY);
          }
          if (outlineType > 0) {
            const outlineFillPaint = this.makeFillPaint('#ffffff');
            effectPaints.push(outlineFillPaint);
            const outlineStrokePaint = this.makeStrokePaint(
              style.color ?? '#000000',
              Math.max(fontSize / 25, 0.5),
            );
            effectPaints.push(outlineStrokePaint);
            drawPass(outlineFillPaint, 0, 0, outlineStrokePaint);
          } else {
            drawPass(paint);
          }
        }
      }
    } finally {
      try {
        if (canvasSaved) canvas.restore();
      } finally {
        font?.delete?.();
        for (const fallbackFont of fallbackFonts) fallbackFont.delete?.();
        (boxedPuaFont as Font | null)?.delete?.();
        (boxedPuaStrokePaint as SkPaint | null)?.delete?.();
        for (const effectPaint of effectPaints) effectPaint.delete?.();
        paint.delete?.();
      }
    }
    if (op.isVertical) {
      this.currentReplayFeatureCounts.verticalTextRuns += 1;
    }
    if (verticalPresentationText !== undefined) {
      this.currentReplayFeatureCounts.verticalPresentationPunctuation += 1;
    }
  }

  private renderCharOverlap(canvas: SkCanvas, op: LayerCharOverlapOp): void {
    if (typeof op.text !== 'string' || !op.charOverlap || !Array.isArray(op.positions)) {
      this.unsupportedOps.add('charOverlap:invalidGeometry');
      return;
    }
    if (op.positionsComplete !== true
      || op.positions.length > CanvasKitLayerRenderer.MAX_TEXT_SPECIAL_VISUAL_ITEMS + 1) {
      this.unsupportedOps.add('charOverlap:visualItemLimitExceeded');
      return;
    }
    const chars: string[] = [];
    for (const ch of op.text) {
      if (chars.length >= CanvasKitLayerRenderer.MAX_TEXT_SPECIAL_VISUAL_ITEMS) {
        this.unsupportedOps.add('charOverlap:visualItemLimitExceeded');
        return;
      }
      chars.push(ch);
    }
    if (op.positions.length !== chars.length + 1) {
      this.unsupportedOps.add('charOverlap:invalidGeometry');
      return;
    }
    if (chars.length === 0) return;
    if (op.isVertical) {
      this.unsupportedOps.add('textRun:verticalText');
      return;
    }
    const style = op.style ?? {};
    const rawFontSize = style.fontSize ?? (op.bbox.height || 12);
    if (![op.baseline, op.rotation, rawFontSize, op.charOverlap.innerCharSize]
      .every(Number.isFinite)
      || op.positions.some(position => !Number.isFinite(position))
      || rawFontSize <= 0
      || !Number.isInteger(op.charOverlap.borderType)
      || op.charOverlap.borderType < 0
      || op.charOverlap.borderType > 4
      || !Number.isInteger(op.charOverlap.innerCharSize)
      || op.charOverlap.innerCharSize < -128
      || op.charOverlap.innerCharSize > 127) {
      this.unsupportedOps.add('charOverlap:invalidGeometry');
      return;
    }
    const fontSize = Math.max(1, rawFontSize);
    const rawRatio = op.charOverlap.innerCharSize > 0
      ? op.charOverlap.innerCharSize / 100
      : op.charOverlap.innerCharSize < 0
        ? 1 + op.charOverlap.innerCharSize * 0.1
        : 1;
    const innerFontSize = Math.max(1, fontSize * Math.min(4, Math.max(0.1, rawRatio)));
    const requestedFontFamily = primaryFontFamily(style.fontFamily);
    const preparedTypeface = this.findPreparedTypeface(requestedFontFamily, style);
    if (requestedFontFamily && !preparedTypeface && this.requirePreparedFontFamilies) {
      throw new Error(`CanvasKit font family가 준비되지 않았습니다: ${requestedFontFamily}`);
    }
    const primaryTypeface = preparedTypeface?.typeface ?? this.defaultTypeface;

    const overlapDigits: Array<[number, number]> = [];
    for (const ch of chars) {
      const codePoint = ch.codePointAt(0) ?? 0;
      const digit = codePoint >= 0xF0289 && codePoint <= 0xF0291
        ? [0, codePoint - 0xF0288] as [number, number]
        : codePoint >= 0xF0292 && codePoint <= 0xF029B
          ? [1, codePoint - 0xF0292] as [number, number]
          : codePoint >= 0xF0491 && codePoint <= 0xF0499
            ? [0, codePoint - 0xF0490] as [number, number]
            : codePoint >= 0xF049A && codePoint <= 0xF04A3
              ? [1, codePoint - 0xF049A] as [number, number]
              : codePoint >= 0xF04A4 && codePoint <= 0xF04AD
                ? [2, codePoint - 0xF04A4] as [number, number]
                : null;
      if (!digit) {
        overlapDigits.length = 0;
        break;
      }
      overlapDigits.push(digit);
    }
    const decodedNumber = overlapDigits.length === chars.length
      ? overlapDigits
          .sort(([left], [right]) => left - right)
          .map(([, digit]) => String.fromCharCode(0x30 + digit))
          .join('')
      : null;

    let rendered = false;
    const draw = (originX: number, originY: number) => {
      const boxSize = fontSize;
      const centerY = originY + op.bbox.height - boxSize / 2;
      const drawCell = (
        displayText: string,
        centerX: number,
        drawShape: boolean,
        horizontalScale?: number,
      ) => {
        const borderType = horizontalScale !== undefined && op.charOverlap.borderType === 0
          ? 1
          : op.charOverlap.borderType;
        const reversed = borderType === 2 || borderType === 4;
        const circle = borderType === 1 || borderType === 2;
        const rectangle = borderType === 3 || borderType === 4;
        const textColor = reversed ? '#ffffff' : style.color ?? '#000000';
        if (drawShape && (circle || rectangle)) {
          let fill: SkPaint | null = null;
          let stroke: SkPaint | null = null;
          try {
            fill = reversed ? this.makeFillPaint('#000000') : null;
            stroke = this.makeStrokePaint(
              reversed ? '#000000' : style.color ?? '#000000',
              0.8,
            );
            if (circle) {
              const radiusY = boxSize / 2;
              const radiusX = radiusY * 0.85;
              const oval = this.canvasKit.XYWHRect(
                centerX - radiusX,
                centerY - radiusY,
                radiusX * 2,
                radiusY * 2,
              );
              if (fill) canvas.drawOval(oval, fill);
              canvas.drawOval(oval, stroke);
            } else {
              const rect = this.canvasKit.XYWHRect(
                centerX - boxSize / 2,
                centerY - boxSize / 2,
                boxSize,
                boxSize,
              );
              if (fill) canvas.drawRect(rect, fill);
              canvas.drawRect(rect, stroke);
            }
          } finally {
            fill?.delete?.();
            stroke?.delete?.();
          }
        }

        let textFont = this.makeFont(primaryTypeface, innerFontSize);
        let fallbackCandidate: Font | null = null;
        let paint: SkPaint | null = null;
        const adjustFont = (target: Font, selected: CanvasKitLocalTypeface | null = preparedTypeface) => {
          const adjustable = target as Font & {
            setEmbolden?: (enabled: boolean) => void;
            setSkewX?: (skew: number) => void;
          };
          adjustable.setEmbolden?.(selected?.syntheticStyle?.bold ?? (style.bold === true && (selected?.hostFace?.weight ?? 0) < 600));
          adjustable.setSkewX?.((selected?.syntheticStyle?.italic ?? (style.italic === true && !['italic', 'oblique'].includes(selected?.hostFace?.slant ?? ''))) ? -0.2 : 0);
        };
        try {
          adjustFont(textFont);
          let glyphIds = textFont.getGlyphIDs(displayText, Array.from(displayText).length);
          if (glyphIds.some(glyphId => glyphId === 0)) {
            for (const fallbackTypeface of [this.defaultTypeface, this.symbolFallbackTypeface]) {
              if (!fallbackTypeface || fallbackTypeface === primaryTypeface) continue;
              fallbackCandidate = this.makeFont(fallbackTypeface, innerFontSize);
              adjustFont(fallbackCandidate, null);
              const fallbackGlyphIds = fallbackCandidate.getGlyphIDs(
                displayText,
                Array.from(displayText).length,
              );
              if (fallbackGlyphIds.every(glyphId => glyphId !== 0)) {
                textFont.delete?.();
                textFont = fallbackCandidate;
                fallbackCandidate = null;
                glyphIds = fallbackGlyphIds;
                break;
              }
              fallbackCandidate.delete?.();
              fallbackCandidate = null;
            }
          }
          if (glyphIds.some(glyphId => glyphId === 0)) {
            this.unsupportedOps.add('textRun:glyphMapping');
          }
          const widths = textFont.getGlyphWidths(glyphIds) ?? [];
          const measuredWidth = widths.reduce((sum, width) => sum + width, 0);
          const scaleX = horizontalScale ?? 1;
          if (scaleX < 1) {
            textFont.setScaleX(scaleX);
          }
          const drawWidth = measuredWidth * scaleX;
          paint = this.makeFillPaint(textColor);
          const textY = (horizontalScale !== undefined ? centerY - fontSize * 0.08 : centerY)
            + innerFontSize * 0.35;
          this.drawCachedText(canvas, displayText, centerX - Math.max(drawWidth, 1) / 2, textY, paint, textFont);
          rendered = true;
        } finally {
          paint?.delete?.();
          fallbackCandidate?.delete?.();
          textFont.delete?.();
        }
      };

      if (decodedNumber !== null) {
        const horizontalScale = decodedNumber.length > 1
          ? 0.7 / decodedNumber.length * 2
          : 1;
        drawCell(decodedNumber, originX + boxSize / 2, true, horizontalScale);
        return;
      }
      const centerX = chars.length > 1 ? originX + op.bbox.width / 2 : originX + boxSize / 2;
      chars.forEach((ch, index) => {
        const codePoint = ch.codePointAt(0) ?? 0;
        const displayText = codePoint >= 0x2460 && codePoint <= 0x2473
          ? String(codePoint - 0x2460 + 1)
          : codePoint >= 0xF02CE && codePoint <= 0xF02E1
            ? String(codePoint - 0xF02CD)
          : codePoint === 0xF012B
            ? '(인)'
            : codePoint === 0xF031C
              ? '■'
              : codePoint === 0xF02FC
                ? '►'
                : codePoint === 0xF03C5
                  ? '□'
              : ch;
        drawCell(displayText, centerX, index === 0);
      });
    };

    this.withHorizontalTextVisualOrigin(canvas, op.bbox, op.rotation ?? 0, 'charOverlap', draw);
    if (rendered) this.currentReplayFeatureCounts.charOverlaps++;
  }

  private renderTextControlMark(canvas: SkCanvas, op: LayerTextControlMarkOp): void {
    if (op.isVertical) {
      this.unsupportedOps.add('textRun:verticalText');
      return;
    }
    if (!Array.isArray(op.marks)) {
      this.unsupportedOps.add('textControlMark:invalidGeometry');
      return;
    }
    if (op.marksComplete !== true
      || op.marks.length > CanvasKitLayerRenderer.MAX_TEXT_SPECIAL_VISUAL_ITEMS) {
      this.unsupportedOps.add('textControlMark:visualItemLimitExceeded');
      return;
    }
    if (![op.baseline, op.rotation].every(Number.isFinite)
      || op.marks.some(mark => !['space', 'tab', 'paragraphEnd', 'lineBreakEnd'].includes(mark.kind)
        || ![mark.x, mark.y, mark.fontSize].every(Number.isFinite)
        || mark.fontSize <= 0)) {
      this.unsupportedOps.add('textControlMark:invalidGeometry');
      return;
    }
    if (!this.currentShowParagraphMarks && !this.currentShowControlCodes) return;
    const draw = (originX: number, originY: number) => {
      const baselineY = originY + op.baseline;
      let paint: SkPaint | null = null;
      try {
        paint = this.makeStrokePaint('#0066ff', 0.75);
        for (const mark of op.marks) {
          const x = originX + mark.x;
          const y = baselineY + mark.y;
          const size = mark.fontSize;
          if (mark.kind === 'space') {
            canvas.drawLine(x, y - size * 0.45, x + size * 0.25, y - size * 0.15, paint);
            canvas.drawLine(x + size * 0.25, y - size * 0.15, x + size * 0.5, y - size * 0.45, paint);
          } else if (mark.kind === 'tab') {
            const lineY = y - size * 0.3;
            const tipX = x + size * 0.85;
            canvas.drawLine(x, lineY, tipX, lineY, paint);
            canvas.drawLine(tipX, lineY, tipX - size * 0.25, lineY - size * 0.2, paint);
            canvas.drawLine(tipX, lineY, tipX - size * 0.25, lineY + size * 0.2, paint);
          } else if (mark.kind === 'paragraphEnd') {
            const topY = y - size * 0.8;
            const turnX = x + size * 0.4;
            const arrowY = y - size * 0.25;
            canvas.drawLine(turnX, topY, turnX, arrowY, paint);
            canvas.drawLine(turnX, arrowY, x, arrowY, paint);
            canvas.drawLine(x, arrowY, x + size * 0.2, arrowY - size * 0.18, paint);
            canvas.drawLine(x, arrowY, x + size * 0.2, arrowY + size * 0.18, paint);
          } else {
            const lineX = x + size * 0.25;
            const tipY = y - size * 0.1;
            canvas.drawLine(lineX, y - size * 0.85, lineX, tipY, paint);
            canvas.drawLine(lineX, tipY, lineX - size * 0.18, tipY - size * 0.22, paint);
            canvas.drawLine(lineX, tipY, lineX + size * 0.18, tipY - size * 0.22, paint);
          }
        }
      } finally {
        paint?.delete?.();
      }
    };

    this.withHorizontalTextVisualOrigin(canvas, op.bbox, op.rotation, 'textControlMark', draw);
  }

  private renderTabLeader(canvas: SkCanvas, op: LayerTabLeaderOp): void {
    if (!Array.isArray(op.leaders)) {
      this.unsupportedOps.add('tabLeader:invalidGeometry');
      return;
    }
    if (op.leadersComplete !== true
      || op.leaders.length > CanvasKitLayerRenderer.MAX_TEXT_SPECIAL_VISUAL_ITEMS) {
      this.unsupportedOps.add('tabLeader:visualItemLimitExceeded');
      return;
    }
    if (![op.baseline, op.fontSize, op.rotation].every(Number.isFinite)
      || op.fontSize <= 0
      || op.leaders.some(leader => ![leader.startX, leader.endX].every(Number.isFinite)
        || leader.endX < leader.startX
        || !Number.isInteger(leader.fillType)
        || leader.fillType < 0)) {
      this.unsupportedOps.add('tabLeader:invalidGeometry');
      return;
    }
    let rendered = false;
    const draw = (originX: number, originY: number) => {
      const baselineY = originY + op.baseline;
      for (const leader of op.leaders) {
        if (leader.fillType === 0 || leader.endX <= leader.startX) continue;
        const x1 = originX + leader.startX;
        const x2 = originX + leader.endX;
        const y = baselineY - op.fontSize * 0.35;
        switch (leader.fillType) {
          case 1:
            this.drawTextVisualStroke(canvas, x1, y, x2, y, op.color, 0.5);
            break;
          case 2:
            this.drawTextVisualStroke(canvas, x1, y, x2, y, op.color, 0.5, [3, 3]);
            break;
          case 3:
            this.drawTextVisualStroke(canvas, x1, y, x2, y, op.color, 1, [0.1, 3], true);
            break;
          case 4:
            this.drawTextVisualStroke(canvas, x1, y, x2, y, op.color, 0.5, [6, 2, 1, 2]);
            break;
          case 5:
            this.drawTextVisualStroke(canvas, x1, y, x2, y, op.color, 0.5, [6, 2, 1, 2, 1, 2]);
            break;
          case 6:
            this.drawTextVisualStroke(canvas, x1, y, x2, y, op.color, 0.5, [8, 4]);
            break;
          case 7:
            this.drawTextVisualStroke(canvas, x1, y, x2, y, op.color, 1.6, [0, 4], true);
            break;
          case 8:
            this.drawTextVisualStroke(canvas, x1, y - 1, x2, y - 1, op.color, 0.3);
            this.drawTextVisualStroke(canvas, x1, y + 1, x2, y + 1, op.color, 0.3);
            break;
          case 9:
            this.drawTextVisualStroke(canvas, x1, y - 1.2, x2, y - 1.2, op.color, 0.3);
            this.drawTextVisualStroke(canvas, x1, y + 0.8, x2, y + 0.8, op.color, 0.8);
            break;
          case 10:
            this.drawTextVisualStroke(canvas, x1, y - 0.8, x2, y - 0.8, op.color, 0.8);
            this.drawTextVisualStroke(canvas, x1, y + 1.2, x2, y + 1.2, op.color, 0.3);
            break;
          case 11:
            this.drawTextVisualStroke(canvas, x1, y - 2, x2, y - 2, op.color, 0.3);
            this.drawTextVisualStroke(canvas, x1, y, x2, y, op.color, 0.8);
            this.drawTextVisualStroke(canvas, x1, y + 2, x2, y + 2, op.color, 0.3);
            break;
          default:
            break;
        }
        if (leader.fillType >= 1 && leader.fillType <= 11) rendered = true;
      }
    };

    this.withHorizontalTextVisualOrigin(canvas, op.bbox, op.rotation, 'tabLeader', draw);
    if (rendered) this.currentReplayFeatureCounts.tabLeaders++;
  }

  private renderTextDecoration(canvas: SkCanvas, op: LayerTextDecorationOp): void {
    const decoration = op.decoration;
    if (!decoration
      || !Array.isArray(decoration.positions)
      || !['underline', 'strikethrough', 'emphasisDot'].includes(decoration.kind)) {
      this.unsupportedOps.add('textDecoration:invalidGeometry');
      return;
    }
    if (decoration.positionsComplete !== true
      || decoration.positions.length > CanvasKitLayerRenderer.MAX_TEXT_SPECIAL_VISUAL_ITEMS + 1) {
      this.unsupportedOps.add('textDecoration:visualItemLimitExceeded');
      return;
    }
    if (![decoration.baseline, decoration.rotation, decoration.fontSize, decoration.ratio]
      .every(Number.isFinite)
      || decoration.fontSize <= 0
      || decoration.ratio <= 0
      || decoration.positions.some(position => !Number.isFinite(position))
      || !Number.isInteger(decoration.shape)
      || decoration.shape < 0
      || !Number.isInteger(decoration.emphasisDot)
      || decoration.emphasisDot < 0
      || !['none', 'bottom', 'top'].includes(decoration.underline)) {
      this.unsupportedOps.add('textDecoration:invalidGeometry');
      return;
    }
    const textWidth = decoration.positions.at(-1) ?? 0;
    if (!Number.isFinite(textWidth) || textWidth < 0) {
      this.unsupportedOps.add('textDecoration:invalidGeometry');
      return;
    }
    const drawLineShape = (originX: number, y: number) => {
      const x2 = originX + textWidth;
      switch (decoration.shape) {
        case 7:
          this.drawTextVisualStroke(canvas, originX, y - 1, x2, y - 1, decoration.color, 0.7);
          this.drawTextVisualStroke(canvas, originX, y + 1, x2, y + 1, decoration.color, 0.7);
          break;
        case 8:
          this.drawTextVisualStroke(canvas, originX, y - 1.2, x2, y - 1.2, decoration.color, 0.5);
          this.drawTextVisualStroke(canvas, originX, y + 0.8, x2, y + 0.8, decoration.color, 1.2);
          break;
        case 9:
          this.drawTextVisualStroke(canvas, originX, y - 0.8, x2, y - 0.8, decoration.color, 1.2);
          this.drawTextVisualStroke(canvas, originX, y + 1.2, x2, y + 1.2, decoration.color, 0.5);
          break;
        case 10:
          this.drawTextVisualStroke(canvas, originX, y - 1.5, x2, y - 1.5, decoration.color, 0.5);
          this.drawTextVisualStroke(canvas, originX, y, x2, y, decoration.color, 0.5);
          this.drawTextVisualStroke(canvas, originX, y + 1.5, x2, y + 1.5, decoration.color, 0.5);
          break;
        case 11:
          this.drawTextVisualStroke(canvas, originX, y, x2, y, decoration.color, 0.7, [], false, 1.5, 6);
          break;
        case 12:
          this.drawTextVisualStroke(canvas, originX, y - 1, x2, y - 1, decoration.color, 0.5, [], false, 1.2, 6);
          this.drawTextVisualStroke(canvas, originX, y + 1, x2, y + 1, decoration.color, 0.5, [], false, 1.2, 6);
          break;
        default: {
          const dash = decoration.shape === 1 ? [3, 3]
            : decoration.shape === 2 ? [1, 2]
              : decoration.shape === 3 ? [6, 2, 1, 2]
                : decoration.shape === 4 ? [6, 2, 1, 2, 1, 2]
                  : decoration.shape === 5 ? [8, 4]
                    : decoration.shape === 6 ? [0.1, 2.5]
                      : [];
          this.drawTextVisualStroke(
            canvas,
            originX,
            y,
            x2,
            y,
            decoration.color,
            1,
            dash,
            decoration.shape === 6,
          );
        }
      }
    };
    const draw = (originX: number, originY: number) => {
      const baselineY = originY + decoration.baseline;
      if (decoration.kind === 'underline') {
        const y = decoration.underline === 'top'
          ? baselineY - decoration.fontSize + 1
          : baselineY + 2;
        drawLineShape(originX, y);
        return;
      }
      if (decoration.kind === 'strikethrough') {
        drawLineShape(originX, baselineY - decoration.fontSize * 0.3);
        return;
      }
      if (decoration.emphasisDot === 0) return;
      const dotSize = Math.max(1, decoration.fontSize * 0.3);
      let fillPaint: SkPaint | null = null;
      let strokePaint: SkPaint | null = null;
      try {
        fillPaint = this.makeFillPaint(decoration.color);
        strokePaint = this.makeStrokePaint(
          decoration.color,
          Math.max(dotSize * 0.12, 0.75),
        );
        for (const position of decoration.positions.slice(0, -1)) {
          const x = originX + position + decoration.fontSize * decoration.ratio * 0.5;
          const y = baselineY - decoration.fontSize * 1.05;
          const centerY = y - dotSize * 0.45;
          if (decoration.emphasisDot === 1) {
            canvas.drawCircle(x, centerY, Math.max(dotSize * 0.48, 1), fillPaint);
          } else if (decoration.emphasisDot === 2) {
            canvas.drawCircle(x, centerY, Math.max(dotSize * 0.48, 1), strokePaint);
          } else if (decoration.emphasisDot === 3) {
            canvas.drawLine(x - dotSize * 0.45, centerY - dotSize * 0.2, x, centerY + dotSize * 0.25, strokePaint);
            canvas.drawLine(x, centerY + dotSize * 0.25, x + dotSize * 0.45, centerY - dotSize * 0.2, strokePaint);
          } else if (decoration.emphasisDot === 4) {
            canvas.drawLine(x - dotSize * 0.5, centerY, x - dotSize * 0.15, centerY - dotSize * 0.22, strokePaint);
            canvas.drawLine(x - dotSize * 0.15, centerY - dotSize * 0.22, x + dotSize * 0.15, centerY + dotSize * 0.22, strokePaint);
            canvas.drawLine(x + dotSize * 0.15, centerY + dotSize * 0.22, x + dotSize * 0.5, centerY, strokePaint);
          } else if (decoration.emphasisDot === 5) {
            canvas.drawCircle(x, centerY, Math.max(dotSize * 0.22, 0.75), fillPaint);
          } else if (decoration.emphasisDot === 6) {
            const radius = Math.max(dotSize * 0.18, 0.7);
            canvas.drawCircle(x, centerY - radius * 1.5, radius, fillPaint);
            canvas.drawCircle(x, centerY + radius * 1.5, radius, fillPaint);
          }
        }
      } finally {
        strokePaint?.delete?.();
        fillPaint?.delete?.();
      }
    };

    this.withHorizontalTextVisualOrigin(
      canvas,
      op.bbox,
      decoration.rotation,
      'textDecoration',
      draw,
    );
  }

  private withHorizontalTextVisualOrigin(
    canvas: SkCanvas,
    bbox: LayerBounds,
    rotation: number,
    opType: 'charOverlap' | 'textControlMark' | 'tabLeader' | 'textDecoration',
    draw: (originX: number, originY: number) => void,
  ): void {
    if (![bbox.x, bbox.y, bbox.width, bbox.height, rotation].every(Number.isFinite)
      || bbox.width < 0
      || bbox.height < 0) {
      this.unsupportedOps.add(`${opType}:invalidGeometry`);
      return;
    }
    if (rotation !== 0) {
      canvas.save();
      try {
        canvas.rotate(rotation, bbox.x + bbox.width / 2, bbox.y + bbox.height / 2);
        draw(bbox.x, bbox.y);
      } finally {
        canvas.restore();
      }
      return;
    }
    draw(bbox.x, bbox.y);
  }

  private drawTextVisualStroke(
    canvas: SkCanvas,
    x1: number,
    y1: number,
    x2: number,
    y2: number,
    color: string,
    width: number,
    dash: number[] = [],
    roundCap = false,
    waveHeight = 0,
    waveWidth = 0,
  ): void {
    const paint = this.makeStrokePaint(color, width);
    let effect: ReturnType<typeof this.canvasKit.PathEffect.MakeDash> | null = null;
    let path: Path | null = null;
    try {
      if (roundCap) paint.setStrokeCap(this.canvasKit.StrokeCap.Round);
      if (dash.length > 0) {
        effect = this.canvasKit.PathEffect.MakeDash(dash, 0);
        if (effect) paint.setPathEffect(effect);
      }
      if (waveHeight > 0 && waveWidth > 0) {
        const builder = new this.canvasKit.PathBuilder();
        try {
          builder.moveTo(x1, y1);
          let cursor = x1;
          let up = true;
          const step = Math.max(
            waveWidth,
            (x2 - x1) / CanvasKitLayerRenderer.MAX_TEXT_VISUAL_WAVE_SEGMENTS,
          );
          while (cursor < x2) {
            const next = Math.min(cursor + step, x2);
            builder.quadTo((cursor + next) / 2, up ? y1 - waveHeight : y1 + waveHeight, next, y1);
            cursor = next;
            up = !up;
          }
          path = builder.detach();
        } finally {
          builder.delete?.();
        }
        canvas.drawPath(path, paint);
      } else {
        canvas.drawLine(x1, y1, x2, y2, paint);
      }
    } finally {
      path?.delete?.();
      effect?.delete?.();
      paint.delete?.();
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
    bold: boolean,
    italic: boolean,
  ): boolean {
    if (!fontManager) return false;
    const textStyle = {
      color: this.color(color),
      fontSize,
      ...(fontFamily ? { fontFamilies: [fontFamily] } : {}),
      ...(this.canvasKit.FontWeight && this.canvasKit.FontSlant ? {
        fontStyle: {
          weight: bold ? this.canvasKit.FontWeight.Bold : this.canvasKit.FontWeight.Normal,
          slant: italic ? this.canvasKit.FontSlant.Italic : this.canvasKit.FontSlant.Upright,
        },
      } : {}),
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

  /** Availability reads the same prepared candidates as actual text replay.
   * Installed/host faces need not occur in the portable bundled catalog. */
  hasPreparedFontFamily(family: string): boolean {
    return [false, true].some(bold => [false, true].some(italic => (
      this.findPreparedTypeface(family, { bold, italic }) !== null
    )));
  }

  private findBundledTypeface(key: string, style?: LayerTextStyle): CanvasKitLocalTypeface | null {
    const faces = this.bundledTypefaceAliases.get(key);
    if (!faces?.length) return null;
    if (faces.length === 1) return faces[0];
    let family = this.bundledFontFamilies.get(key);
    if (!family) {
      const bytes = faces.map(face => face.bytes).filter((data): data is ArrayBuffer => data !== undefined);
      if (bytes.length !== faces.length) throw new Error(`CanvasKit family bytes가 준비되지 않았습니다: ${key}`);
      family = new CanvasKitBundledFontFamily(this.canvasKit, key, bytes);
      this.bundledFontFamilies.set(key, family);
    }
    return { ...family.select(style?.bold === true, style?.italic === true),
      paragraphTypefaces: faces.map(face => face.typeface) };
  }

  private findPreparedTypeface(fontFamily: string | undefined, style?: LayerTextStyle): CanvasKitLocalTypeface | null {
    const key = `${normalizedFontFamily(fontFamily)}:${style?.bold === true}:${style?.italic === true}:${getHostFontState().generation}`;
    if (this.fallbackFamilyCache.has(key)) {
      const cached = this.fallbackFamilyCache.get(key)!;
      this.fallbackFamilyCache.delete(key);
      this.fallbackFamilyCache.set(key, cached);
      return cached;
    }
    const result = this.resolvePreparedTypeface(fontFamily, style);
    if (this.fallbackFamilyCache.size >= 256) {
      const oldest = this.fallbackFamilyCache.keys().next().value;
      if (oldest !== undefined) this.fallbackFamilyCache.delete(oldest);
    }
    this.fallbackFamilyCache.set(key, result);
    return result;
  }

  private invalidateFontCaches(): void {
    this.localFontRevision += 1;
    this.fallbackFamilyCache.clear();
    this.textBlobs.clear();
    this.failedTextBlobKeys.clear();
    this.staticPictures.clear();
    this.retainPreparedParagraphFonts();
  }

  private retainPreparedParagraphFonts(): void {
    this.paragraphFonts.retain(new Set([this.defaultTypeface, this.symbolFallbackTypeface, this.oldHangulTypeface?.typeface ?? null,
      ...[...this.localTypefaces.values(), ...this.bundledTypefaces.values()].map(face => face.typeface)]));
  }

  private textBlobFontKey(font: Font): string | null {
    try {
      const faceId = this.fontTypefaceIds.get(font);
      return faceId === undefined ? null : [faceId, font.getSize(), font.getScaleX(), font.getSkewX(), font.isEmbolden()].join(':');
    } catch { return null; }
  }

  private makeFont(typeface: Typeface | null, size: number): Font {
    const font = new this.canvasKit.Font(typeface, size);
    let faceId = typeface ? this.typefaceIds.get(typeface) : 0;
    if (faceId === undefined && typeface) {
      faceId = this.nextTypefaceId++;
      this.typefaceIds.set(typeface, faceId);
    }
    this.fontTypefaceIds.set(font, faceId ?? 0);
    return font;
  }

  private rememberTextBlobFailure(key: string): void {
    this.textBlobConstructionFailures++;
    if (this.failedTextBlobKeys.size >= 2048) {
      const oldest = this.failedTextBlobKeys.values().next().value;
      if (oldest !== undefined) this.failedTextBlobKeys.delete(oldest);
    }
    this.failedTextBlobKeys.add(key);
  }

  private textReplayDiagnostic(fallback: 'drawText' | 'drawGlyphs', text?: string,
    range?: { start: number; length: number }): CanvasKitTextReplayDiagnostic {
    const op = this.currentOp;
    const opId = op && 'id' in op && typeof op.id === 'string' ? op.id.slice(0, 1024) : null;
    const style = op?.type === 'textRun' ? op.style : undefined;
    const fullRun = op?.type === 'textRun' && text === op.text;
    return { opId, fontFamily: (style?.fontFamily ?? this.defaultFontFamily ?? 'CanvasKit default').slice(0, 1024),
      clusterStartUtf16: range?.start ?? (fullRun ? 0 : null),
      clusterLengthUtf16: range?.length ?? text?.length ?? null, reason: 'textBlobConstructionFailed', fallback };
  }

  private drawCachedText(canvas: SkCanvas, text: string, x: number, y: number, paint: SkPaint, font: Font): void {
    const fontKey = this.textBlobFontKey(font);
    if (!fontKey || !this.canvasKit.TextBlob || typeof canvas.drawTextBlob !== 'function') {
      canvas.drawText(text, x, y, paint, font);
      return;
    }
    const key = `text:${fontKey}:${text}`;
    let blob = this.textBlobs.get(key);
    let retained = !!blob;
    if (!blob && this.failedTextBlobKeys.has(key)) this.textBlobFailureCacheHits++;
    if (!blob && !this.failedTextBlobKeys.has(key)) {
      try { blob = this.canvasKit.TextBlob.MakeFromText(text, font) ?? undefined; } catch { /* optimization unavailable */ }
      if (blob) retained = this.textBlobs.set(key, blob, key.length * 2 + text.length * 32);
      else this.rememberTextBlobFailure(key);
    }
    try {
      if (blob) canvas.drawTextBlob(blob, x, y, paint);
      else {
        const diagnostic = this.textReplayDiagnostic('drawText', text);
        try { canvas.drawText(text, x, y, paint, font); this.textBlobFallbackDraws++; this.textRecoveries.add(diagnostic); }
        catch (error) { this.textFailures.add({ ...diagnostic, reason: 'simpleTextFallbackFailed' }); throw error; }
      }
    } finally { if (blob && !retained) blob.delete(); }
  }

  private drawCachedGlyphs(canvas: SkCanvas, glyphs: Uint16Array, positions: Float32Array, x: number, y: number, font: Font, paint: SkPaint,
    range?: { start: number; length: number }): void {
    const fontKey = this.textBlobFontKey(font);
    const runKey = fontKey ? replayFingerprint([glyphs, positions]) : null;
    if (!fontKey || !runKey || !this.canvasKit.TextBlob?.MakeFromRSXformGlyphs || typeof canvas.drawTextBlob !== 'function') {
      canvas.drawGlyphs(glyphs, positions, x, y, font, paint);
      return;
    }
    const key = `glyphs:${fontKey}:${runKey}`;
    let blob = this.textBlobs.get(key);
    let retained = !!blob;
    if (!blob && this.failedTextBlobKeys.has(key)) this.textBlobFailureCacheHits++;
    if (!blob && !this.failedTextBlobKeys.has(key)) {
      const transforms = new Float32Array(glyphs.length * 4);
      for (let index = 0; index < glyphs.length; index++) {
        transforms[index * 4] = 1;
        transforms[index * 4 + 2] = positions[index * 2];
        transforms[index * 4 + 3] = positions[index * 2 + 1];
      }
      try { blob = this.canvasKit.TextBlob.MakeFromRSXformGlyphs(glyphs, transforms, font) ?? undefined; } catch { /* original glyph replay remains available */ }
      if (blob) retained = this.textBlobs.set(key, blob, key.length * 2 + glyphs.length * 48);
      else this.rememberTextBlobFailure(key);
    }
    try {
      if (blob) canvas.drawTextBlob(blob, x, y, paint);
      else {
        const diagnostic = this.textReplayDiagnostic('drawGlyphs', undefined, range);
        try { canvas.drawGlyphs(glyphs, positions, x, y, font, paint); this.textBlobFallbackDraws++; this.textRecoveries.add(diagnostic); }
        catch (error) { this.textFailures.add({ ...diagnostic, reason: 'simpleTextFallbackFailed' }); throw error; }
      }
    } finally { if (blob && !retained) blob.delete(); }
  }

  private resolvePreparedTypeface(fontFamily: string | undefined, style?: LayerTextStyle): CanvasKitLocalTypeface | null {
    const key = normalizedFontFamily(fontFamily);
    if (!key) return null;
    const record = resolveRendererLocalFont(primaryFontFamily(fontFamily), hostFontStyle(style));
    let local = record ? this.localTypefaces.get(localFontFaceKey(record)) ?? null : null;
    if (!hasHostFontProvider()) {
      let bestScore = Number.POSITIVE_INFINITY;
      for (const candidate of this.localTypefaces.values()) {
        if (!candidate.record?.aliases.some(alias => normalizedFontFamily(alias) === key)) continue;
        const physical = localFontFaceStyle(candidate.record);
        const score = (physical.italic === (style?.italic === true) ? 0 : 2000)
          + Math.abs(physical.weight - (style?.bold === true ? 700 : 400));
        if (score < bestScore) { local = candidate; bestScore = score; }
      }
    }
    const bundled = this.findBundledTypeface(key, style);
    if (key === normalizedFontFamily(OLD_HANGUL_FONT_FAMILY)) {
      return [this.oldHangulTypeface, local, bundled]
        .find(candidate => candidate?.fontManager) ?? null;
    }
    if (local) return local;
    if (bundled) return bundled;
    if (key === normalizedFontFamily(this.defaultFontFamily) || key === 'noto sans kr') {
      return this.defaultTypeface || this.defaultFontManager
        ? {
            typeface: this.defaultTypeface,
            fontManager: this.defaultFontManager,
            fontFamily: this.defaultFontFamily,
          }
        : null;
    }
    return null;
  }

  private recordFontSubstitution(diagnostic: CanvasKitFontSubstitutionDiagnostic): void {
    diagnostic = { ...diagnostic, requestedFamily: diagnostic.requestedFamily.slice(0, 1024), resolvedFamily: diagnostic.resolvedFamily.slice(0, 1024) };
    if (diagnostic.opId === undefined && this.currentOp && 'id' in this.currentOp && typeof this.currentOp.id === 'string') {
      diagnostic = { ...diagnostic, opId: this.currentOp.id.slice(0, 1024) };
    }
    const key = JSON.stringify([
      diagnostic.opId,
      diagnostic.requestedFamily,
      diagnostic.resolvedFamily,
      diagnostic.source,
    ]);
    if (this.currentFontSubstitutions.has(key)
      || this.currentFontSubstitutions.size < CanvasKitLayerRenderer.MAX_FONT_SUBSTITUTION_DIAGNOSTICS) {
      this.currentFontSubstitutions.set(key, diagnostic);
    }
  }

  private renderEquation(canvas: SkCanvas, op: LayerEquationOp): void {
    const record = (route: CanvasKitEquationReplayDiagnostic['route'], reason: CanvasKitEquationReplayDiagnostic['reason']) => {
      this.equationRoutes.add({ svgResourceId: op.svgResourceId ?? null, hasInlineSvg: typeof op.svgContent === 'string',
        bbox: { ...op.bbox }, route, reason });
    };
    this.equationSvgFailureReason = op.svgResourceId === undefined && typeof op.svgContent !== 'string'
      ? 'layoutRequested' : 'svgResourceMissing';
    if (this.renderEquationSvg(canvas, op)) { record('svg', 'svgReplayed'); return; }
    const svgReason = this.equationSvgFailureReason;
    if (!op.layoutBox || !this.boundsAreDrawable(op.bbox)) {
      this.unsupportedOps.add('equation:unsupportedDirectReplay');
      record('failed', !this.boundsAreDrawable(op.bbox) ? 'invalidEquationBounds' : 'invalidLayout');
      return;
    }
    const scaleX = op.scaleX !== undefined && Number.isFinite(op.scaleX) ? op.scaleX : op.layoutBox.width > 0 && op.bbox.width > 0
      ? op.bbox.width / op.layoutBox.width
      : 1;
    const budget: EquationRenderBudget = {
      remainingNodes: CanvasKitLayerRenderer.MAX_EQUATION_LAYOUT_NODES,
    };
    const recorder = new this.canvasKit.PictureRecorder();
    let picture: ReturnType<typeof recorder.finishRecordingAsPicture> | null = null;
    let recordingFinished = false;
    let replayed = false;
    try {
      const recordingCanvas = recorder.beginRecording(this.rect({
        ...op.bbox, width: Math.max(op.bbox.width, op.layoutBox.width * scaleX),
      }));
      recordingCanvas.save();
      recordingCanvas.translate(op.bbox.x, op.bbox.y);
      if (Math.abs(scaleX - 1) > 0.01) recordingCanvas.scale(scaleX, 1);
      try {
        replayed = this.renderEquationBox(
          recordingCanvas,
          op.layoutBox,
          0,
          0,
          op.color ?? '#000000',
          Math.max(1, op.fontSize ?? op.bbox.height),
          false,
          false,
          0,
          budget,
        );
      } finally {
        recordingCanvas.restore();
      }
      picture = recorder.finishRecordingAsPicture();
      recordingFinished = true;
      if (replayed) canvas.drawPicture(picture);
    } catch {
      replayed = false;
    } finally {
      if (!recordingFinished) {
        try {
          picture = recorder.finishRecordingAsPicture();
        } catch {
          picture = null;
        }
      }
      picture?.delete?.();
      recorder.delete?.();
    }
    if (!replayed) {
      this.unsupportedOps.add('equation:invalidLayout');
    }
    record(replayed ? 'layout' : 'failed', replayed ? svgReason : 'invalidLayout');
  }

  /** 정적 SVG는 먼저 기록을 완료해야만 표시한다. 부분 성공 후 layout을 겹치지 않는다. */
  private renderEquationSvg(canvas: SkCanvas, op: LayerEquationOp): boolean {
    const fragment = op.svgResourceId === undefined
      ? op.svgContent
      : this.currentResources?.svgFragments?.[op.svgResourceId];
    if (typeof fragment !== 'string') return false;
    if (!this.boundsAreDrawable(op.bbox)) { this.equationSvgFailureReason = 'invalidEquationBounds'; return false; }
    this.equationSvgFailureReason = 'svgPayloadUnsupported';
    if (fragment.length > CanvasKitLayerRenderer.MAX_STATIC_SVG_GLYPH_BYTES) return false;
    const paths = parseStaticSvgPathLayers(fragment, op.color ?? '#000000');
    const texts = parseStaticSvgTextLayers(fragment);
    if (paths.length + texts.length === 0 || paths.length + texts.length > 4096
      || (paths.length > 0 && !staticSvgPathLayersAreReplayable(paths, data => this.canvasKit.Path.MakeFromSVGString(data)))) {
      return false;
    }
    this.equationSvgFailureReason = 'svgPathDecodeFailed';
    const recorder = new this.canvasKit.PictureRecorder();
    let picture: ReturnType<typeof recorder.finishRecordingAsPicture> | null = null;
    let finished = false;
    try {
      const recording = recorder.beginRecording(this.rect(op.bbox));
      recording.save();
      let replayed = false;
      try {
        // Equation SVG fragment는 이미 equation-local px 좌표다. bbox-fit은 중복 축척이다.
        recording.translate(op.bbox.x, op.bbox.y);
        this.drawStaticSvgPathLayers(recording, paths);
        replayed = texts.every(layer => this.drawStaticSvgTextLayer(recording, layer));
      } finally {
        recording.restore();
      }
      picture = recorder.finishRecordingAsPicture();
      finished = true;
      if (replayed) canvas.drawPicture(picture);
      return replayed;
    } catch {
      return false;
    } finally {
      if (!finished) {
        try { picture = recorder.finishRecordingAsPicture(); } catch { /* 기록 실패 */ }
      }
      picture?.delete();
      recorder.delete();
    }
  }

  private drawStaticSvgTextLayer(canvas: SkCanvas, layer: StaticSvgTextLayer): boolean {
    if (layer.text.length > CanvasKitLayerRenderer.MAX_EQUATION_TEXT_LENGTH) return false;
    const candidates = [...[layer.fontFamily, 'Noto Sans KR', 'Noto Sans CJK KR', 'NanumGothic', 'D2Coding',
      'NanumGothicCoding', 'Noto Serif KR', 'Noto Serif CJK KR'].map(family =>
      this.findPreparedTypeface(family, {
        bold: layer.fontWeight === 'bold', italic: layer.fontStyle === 'italic',
      })), this.oldHangulTypeface, {
      typeface: this.defaultTypeface,
      fontManager: this.defaultFontManager,
      fontFamily: this.defaultFontFamily,
    }, { typeface: this.symbolFallbackTypeface, fontManager: null, fontFamily: null }]
      .filter((face): face is CanvasKitLocalTypeface => !!face?.typeface);
    const combined = this.paragraphFonts.source(candidates.map(face => face.paragraphTypefaces ?? face.typeface));
    let triedCombined = false;
    for (const candidate of candidates) {
      const font = this.makeFont(candidate.typeface, layer.fontSize);
      let builder: ReturnType<typeof this.canvasKit.ParagraphBuilder.Make> | null = null;
      let paragraph: Paragraph | null = null;
      try {
        const glyphs = font.getGlyphIDs(layer.text, Array.from(layer.text).length);
        const source = triedCombined ? null : combined;
        triedCombined = true;
        if (!source && (!candidate.fontManager || !glyphs || glyphs.some(id => id === 0))) continue;
        const style = new this.canvasKit.ParagraphStyle({
          maxLines: 1,
          textStyle: {
            color: this.color(layer.fill, layer.opacity),
            fontSize: layer.fontSize,
            ...(source ? { fontFamilies: source.families }
              : candidate.fontFamily ? { fontFamilies: [candidate.fontFamily] } : {}),
            fontStyle: {
              weight: layer.fontWeight === 'bold' ? this.canvasKit.FontWeight.Bold : this.canvasKit.FontWeight.Normal,
              slant: layer.fontStyle === 'italic' ? this.canvasKit.FontSlant.Italic : this.canvasKit.FontSlant.Upright,
            },
          },
        });
        builder = source ? this.canvasKit.ParagraphBuilder.MakeFromFontProvider(style, source.provider)
          : this.canvasKit.ParagraphBuilder.Make(style, candidate.fontManager!);
        builder.addText(layer.text);
        paragraph = builder.build();
        paragraph.layout(CanvasKitLayerRenderer.MAX_SHAPED_TEXT_WIDTH);
        if (source && paragraph.unresolvedCodepoints().length > 0) continue;
        const width = paragraph.getLongestLine();
        const height = paragraph.getHeight();
        const baseline = paragraph.getAlphabeticBaseline();
        if (![width, height, baseline].every(Number.isFinite)) continue;
        canvas.save();
        try {
          const transform = this.affineToCanvasKitMatrix(layer.transform);
          if (transform) canvas.concat(transform);
          canvas.drawParagraph(paragraph,
            layer.x - (layer.textAnchor === 'middle' ? width / 2 : layer.textAnchor === 'end' ? width : 0),
            layer.y - (layer.dominantBaseline === 'middle' ? height / 2 : baseline));
        } finally { canvas.restore(); }
        return true;
      } finally {
        paragraph?.delete();
        builder?.delete();
        font.delete();
      }
    }
    return false;
  }

  private renderEquationBox(
    canvas: SkCanvas,
    layout: LayerEquationLayoutBox,
    parentX: number,
    parentY: number,
    color: string,
    fontSize: number,
    italic: boolean,
    bold: boolean,
    depth: number,
    budget: EquationRenderBudget,
  ): boolean {
    if (
      depth > CanvasKitLayerRenderer.MAX_EQUATION_LAYOUT_DEPTH
      || budget.remainingNodes <= 0
      || !this.equationBoxIsFinite(layout)
    ) {
      return false;
    }
    budget.remainingNodes -= 1;
    const x = parentX + layout.x;
    const y = parentY + layout.y;
    const child = (box: LayerEquationLayoutBox, size = fontSize, childItalic = italic, childBold = bold) => (
      this.renderEquationBox(canvas, box, x, y, color, size, childItalic, childBold, depth + 1, budget)
    );

    switch (layout.kind.type) {
      case 'hancomGlyphs': {
        const run = layout.kind;
        const typeface = this.findPreparedTypeface('HyhwpEQ')?.typeface;
        const glyphs = Array.from(run.glyphs);
        const shears = run.shears ?? glyphs.map(() => 0);
        if (!typeface || glyphs.length !== run.positions.length
          || glyphs.length !== shears.length || !shears.every(Number.isFinite)
          || glyphs.length > CanvasKitLayerRenderer.MAX_EQUATION_TEXT_LENGTH
          || !run.positions.every(Number.isFinite) || !Number.isFinite(run.baselineShift)) return false;
        const font = new this.canvasKit.Font(typeface, fontSize);
        font.setHinting(this.canvasKit.FontHinting.None);
        font.setSubpixel(true);
        const paint = this.makeFillPaint(color);
        try {
          const ids = font.getGlyphIDs(run.glyphs, glyphs.length);
          if (!ids || ids.some(id => id === 0)) return false;
          glyphs.forEach((glyph, index) => {
            canvas.save();
            try {
              canvas.translate(x + run.positions[index], y + layout.baseline + run.baselineShift);
              canvas.concat([1, -shears[index], 0, 0, 1, 0, 0, 0, 1]);
              canvas.drawText(glyph, 0, 0, paint, font);
            } finally { canvas.restore(); }
          });
          return true;
        } finally { font.delete(); paint.delete(); }
      }
      case 'row':
        return layout.kind.children.every((box) => child(box));
      case 'text':
      case 'number':
      case 'symbol':
      case 'mathSymbol':
        return this.drawEquationText(
          canvas,
          layout.kind.text,
          x,
          y + layout.baseline,
          this.equationFontSizeFromBox(layout, fontSize),
          color,
          layout.kind.type === 'text' || italic,
          bold,
          layout.width,
          layout.kind.type === 'symbol',
          layout.kind.type === 'text' || layout.kind.type === 'number' ? '함초롬바탕' : 'Latin Modern Math',
        );
      case 'function':
        return this.drawEquationText(
          canvas,
          layout.kind.name,
          x,
          y + layout.baseline,
          this.equationFontSizeFromBox(layout, fontSize),
          color,
          italic,
          bold,
          layout.width,
          false,
          '함초롬바탕',
        );
      case 'fraction':
        return child(layout.kind.numer)
          && this.drawEquationLine(
            canvas,
            x + fontSize * 0.05,
            y + layout.baseline,
            x + layout.width - fontSize * 0.05,
            y + layout.baseline,
            color,
            fontSize * 0.04,
          )
          && child(layout.kind.denom);
      case 'atop':
        return child(layout.kind.top) && child(layout.kind.bottom);
      case 'sqrt': {
        const bodyLeft = x + layout.kind.body.x - fontSize * 0.1;
        const midX = bodyLeft - fontSize * 0.15;
        const midY = y + layout.height;
        const startX = midX - fontSize * 0.3;
        const startY = y + layout.height * 0.6;
        const tickX = startX - fontSize * 0.1;
        const tickY = startY - fontSize * 0.05;
        const linesDrawn = this.drawEquationLine(canvas, tickX, tickY, startX, startY, color, fontSize * 0.04)
          && this.drawEquationLine(canvas, startX, startY, midX, midY, color, fontSize * 0.04)
          && this.drawEquationLine(canvas, midX, midY, bodyLeft, y, color, fontSize * 0.04)
          && this.drawEquationLine(canvas, bodyLeft, y, x + layout.width, y, color, fontSize * 0.04);
        const indexDrawn = layout.kind.index
          ? child(layout.kind.index, fontSize * 0.7, false, false)
          : true;
        return linesDrawn && indexDrawn && child(layout.kind.body);
      }
      case 'superscript':
        return child(layout.kind.base)
          && child(layout.kind.sup, fontSize * 0.7);
      case 'subscript':
        return child(layout.kind.base)
          && child(layout.kind.sub, fontSize * 0.7);
      case 'subSup':
        return child(layout.kind.base)
          && child(layout.kind.sub, fontSize * 0.7)
          && child(layout.kind.sup, fontSize * 0.7);
      case 'bigOp': {
        const opSize = fontSize * 1.5;
        const supHeight = layout.kind.sup ? layout.kind.sup.height + fontSize * 0.05 : 0;
        const symbolDrawn = this.drawEquationText(
          canvas,
          layout.kind.symbol,
          x,
          y + supHeight + opSize * 0.8,
          opSize,
          color,
          false,
          false,
          layout.width,
          true,
        );
        const supDrawn = layout.kind.sup
          ? child(layout.kind.sup, fontSize * 0.7, false, false)
          : true;
        const subDrawn = layout.kind.sub
          ? child(layout.kind.sub, fontSize * 0.7, false, false)
          : true;
        return symbolDrawn && supDrawn && subDrawn;
      }
      case 'limit': {
        const size = this.equationFontSizeFromBox(layout, fontSize);
        const limitDrawn = this.drawEquationText(
          canvas,
          layout.kind.isUpper ? 'Lim' : 'lim',
          x,
          y + size * 0.8,
          size,
          color,
          false,
          false,
          layout.width,
          false,
          '함초롬바탕',
        );
        return limitDrawn && (layout.kind.sub
          ? child(layout.kind.sub, fontSize * 0.7, false, false)
          : true);
      }
      case 'matrix': {
        let rendered = true;
        if (layout.kind.style !== 'plain') {
          const brackets = layout.kind.style === 'paren'
            ? ['(', ')']
            : layout.kind.style === 'bracket'
              ? ['[', ']']
              : ['|', '|'];
          rendered = this.drawEquationBracket(canvas, brackets[0], x, y, layout.height, color, fontSize)
            && this.drawEquationBracket(canvas, brackets[1], x + layout.width, y, layout.height, color, fontSize);
        }
        for (const row of layout.kind.cells) {
          for (const cell of row) rendered = child(cell) && rendered;
        }
        return rendered;
      }
      case 'rel':
        return child(layout.kind.over)
          && child(layout.kind.arrow)
          && (layout.kind.under ? child(layout.kind.under) : true);
      case 'eqAlign':
        return layout.kind.rows.every((row) => child(row.left) && child(row.right));
      case 'paren':
        return (layout.kind.left
          ? this.drawEquationBracket(canvas, layout.kind.left, x, y, layout.height, color, fontSize)
          : true)
          && child(layout.kind.body)
          && (layout.kind.right
            ? this.drawEquationBracket(canvas, layout.kind.right, x + layout.width, y, layout.height, color, fontSize)
            : true);
      case 'decoration':
        return child(layout.kind.body)
          && this.drawEquationDecoration(
            canvas,
            layout.kind.decoration,
            x + layout.kind.body.x + layout.kind.body.width / 2,
            y + fontSize * 0.05,
            layout.kind.body.width,
            color,
            fontSize,
          );
      case 'fontStyle': {
        if (!['roman', 'italic', 'bold'].includes(layout.kind.fontStyle)) return false;
        const nextItalic = layout.kind.fontStyle === 'roman'
          ? false
          : layout.kind.fontStyle === 'italic'
            || layout.kind.fontStyle === 'calligraphy'
            || layout.kind.fontStyle === 'fraktur'
            || italic;
        const nextBold = layout.kind.fontStyle === 'roman'
          ? false
          : layout.kind.fontStyle === 'bold'
            || layout.kind.fontStyle === 'blackboard'
            || bold;
        return child(layout.kind.body, fontSize, nextItalic, nextBold);
      }
      case 'space':
      case 'newline':
      case 'empty':
        return true;
    }
  }

  private equationBoxIsFinite(layout: LayerEquationLayoutBox): boolean {
    return Number.isFinite(layout.x)
      && Number.isFinite(layout.y)
      && Number.isFinite(layout.width)
      && Number.isFinite(layout.height)
      && Number.isFinite(layout.baseline)
      && layout.width >= 0
      && layout.height >= 0;
  }

  private equationFontSizeFromBox(layout: LayerEquationLayoutBox, baseFontSize: number): number {
    return Math.max(1, layout.height > 0 ? layout.height : baseFontSize);
  }

  private drawEquationText(
    canvas: SkCanvas,
    text: string,
    x: number,
    baselineY: number,
    fontSize: number,
    color: string,
    italic: boolean,
    bold: boolean,
    targetWidth: number,
    centered: boolean,
    fontFamily = 'Latin Modern Math',
  ): boolean {
    if (
      !text
      || text.length > CanvasKitLayerRenderer.MAX_EQUATION_TEXT_LENGTH
      || ![x, baselineY, fontSize, targetWidth].every(Number.isFinite)
    ) {
      return false;
    }
    return this.drawStaticControlText(canvas, text, { x, y: baselineY, fontSize: Math.max(1, fontSize),
      color, italic, bold, targetWidth, centered,
      fontFamilies: [/[\u1100-\u11ff\u3130-\u318f\ua960-\ua97f\uac00-\ud7ff]/u.test(text) ? '함초롬바탕' : fontFamily,
        'Latin Modern Math', '함초롬바탕', 'Noto Serif KR', 'Noto Serif CJK KR', 'Noto Sans KR', 'Noto Sans CJK KR'] });
  }

  /** Shape static labels without recomputing producer-owned positions or equation widths. */
  private drawStaticControlText(canvas: SkCanvas, text: string, options: {
    x: number; y: number; fontSize: number; color: string; fontFamilies: readonly string[];
    centered?: boolean; verticalCenter?: boolean; targetWidth?: number; bold?: boolean; italic?: boolean;
  }): boolean {
    const { x, y, fontSize, color, bold = false, italic = false, targetWidth } = options;
    const nativeFinite = (...values: number[]) => values.every(value => Number.isFinite(value) && Math.abs(value) <= 3.4028234663852886e38);
    if (!text || text.length > CanvasKitLayerRenderer.MAX_EQUATION_TEXT_LENGTH
      || !nativeFinite(x, y, fontSize, targetWidth ?? 0)
      || fontSize <= 0 || (targetWidth !== undefined && targetWidth < 0)) return false;
    const candidates = [...options.fontFamilies.map(family => this.findPreparedTypeface(family, { bold, italic })), this.oldHangulTypeface,
      { typeface: this.defaultTypeface, fontManager: this.defaultFontManager, fontFamily: this.defaultFontFamily },
      { typeface: this.symbolFallbackTypeface, fontManager: null, fontFamily: null }];
    const seen = new Set<Typeface | null>();
    const combined = this.paragraphFonts.source(candidates.map(candidate => candidate?.paragraphTypefaces ?? candidate?.typeface ?? null));
    let triedCombined = false;
    const needsShaping = textRequiresComplexShaping(text) || /[\u1100-\u11ff\ua960-\ua97f\ud7b0-\ud7ff]/u.test(text);
    for (const candidate of candidates) {
      if (!candidate || seen.has(candidate.typeface)) continue;
      seen.add(candidate.typeface);
      const font = this.makeFont(candidate.typeface, fontSize);
      let builder: ReturnType<typeof this.canvasKit.ParagraphBuilder.Make> | null = null;
      let paragraph: Paragraph | null = null;
      let paint: SkPaint | null = null;
      try {
        const glyphs = font.getGlyphIDs(text, Array.from(text).length);
        const source = triedCombined ? null : combined;
        triedCombined = true;
        const fullCoverage = glyphs && !glyphs.some(id => id === 0);
        if (!source && !fullCoverage) continue;
        const metrics = options.verticalCenter ? font.getMetrics() : null;
        const baseline = options.verticalCenter
          ? y - ((metrics?.ascent ?? -fontSize * 0.8) + (metrics?.descent ?? fontSize * 0.2)) / 2 : y;
        if (!nativeFinite(baseline)) continue;
        if ((source || candidate.fontManager) && this.canvasKit.ParagraphBuilder && this.canvasKit.ParagraphStyle) {
          try {
            const style = new this.canvasKit.ParagraphStyle({ maxLines: 1, textStyle: {
              color: this.color(color), fontSize,
              ...(source ? { fontFamilies: source.families }
                : candidate.fontFamily ? { fontFamilies: [candidate.fontFamily] } : {}),
              fontStyle: { weight: bold ? this.canvasKit.FontWeight.Bold : this.canvasKit.FontWeight.Normal,
                slant: italic ? this.canvasKit.FontSlant.Italic : this.canvasKit.FontSlant.Upright },
            } });
            builder = source ? this.canvasKit.ParagraphBuilder.MakeFromFontProvider(style, source.provider)
              : this.canvasKit.ParagraphBuilder.Make(style, candidate.fontManager!);
            builder.addText(text); paragraph = builder.build();
            paragraph.layout(CanvasKitLayerRenderer.MAX_SHAPED_TEXT_WIDTH);
            if (source && paragraph.unresolvedCodepoints().length > 0) continue;
            const width = paragraph.getLongestLine(), alphabeticBaseline = paragraph.getAlphabeticBaseline(), height = paragraph.getHeight();
            const scale = targetWidth !== undefined && targetWidth > 0 && width > 0 ? targetWidth / width : 1;
            const left = options.centered ? x + ((targetWidth ?? 0) - width * scale) / 2 : x;
            if (!nativeFinite(width, height, alphabeticBaseline, scale, left) || width < 0 || height < 0 || scale <= 0) continue;
            canvas.save();
            try {
              canvas.translate(left, baseline); canvas.scale(scale, 1);
              canvas.drawParagraph(paragraph, 0, -alphabeticBaseline);
            } finally { canvas.restore(); }
            return true;
          } catch { /* A nominal-safe label may still use the same prepared native font. */ }
        }
        if (needsShaping || !fullCoverage) continue;
        const widths = font.getGlyphWidths(glyphs!) ?? [];
        const width = widths.reduce((sum, advance) => sum + advance, 0);
        const scale = targetWidth !== undefined && targetWidth > 0 && width > 0 ? targetWidth / width : 1;
        const left = options.centered ? x + ((targetWidth ?? 0) - width * scale) / 2 : x;
        if (!nativeFinite(width, scale, left) || width < 0 || scale <= 0) continue;
        font.setScaleX(scale);
        (font as Font & { setEmbolden?: (enabled: boolean) => void }).setEmbolden?.(bold);
        (font as Font & { setSkewX?: (skew: number) => void }).setSkewX?.(italic ? -0.2 : 0);
        paint = this.makeFillPaint(color);
        this.drawCachedText(canvas, text, left, baseline, paint, font);
        return true;
      } finally { paragraph?.delete(); builder?.delete(); paint?.delete(); font.delete(); }
    }
    return false;
  }

  private drawEquationLine(
    canvas: SkCanvas,
    x1: number,
    y1: number,
    x2: number,
    y2: number,
    color: string,
    width: number,
  ): boolean {
    if (![x1, y1, x2, y2, width].every(Number.isFinite)) return false;
    const paint = this.makeStrokePaint(color, Math.max(0.5, width));
    try {
      canvas.drawLine(x1, y1, x2, y2, paint);
      return true;
    } finally {
      paint.delete?.();
    }
  }

  private drawEquationBracket(
    canvas: SkCanvas,
    bracket: string,
    x: number,
    y: number,
    height: number,
    color: string,
    fontSize: number,
  ): boolean {
    const width = Math.max(fontSize * 0.3, 1);
    if (bracket === '|') {
      return this.drawEquationLine(canvas, x, y, x, y + height, color, fontSize * 0.04);
    }
    return this.drawEquationText(
      canvas,
      bracket,
      x - width / 2,
      y + height * 0.7,
      Math.max(height, fontSize),
      color,
      false,
      false,
      width,
      true,
    );
  }

  private drawEquationDecoration(
    canvas: SkCanvas,
    decoration: string,
    centerX: number,
    y: number,
    width: number,
    color: string,
    fontSize: number,
  ): boolean {
    const halfWidth = width / 2;
    const strokeWidth = Math.max(fontSize * 0.03, 0.5);
    switch (decoration) {
      case 'hat':
        return this.drawEquationLine(canvas, centerX - halfWidth * 0.6, y + fontSize * 0.15, centerX, y, color, strokeWidth)
          && this.drawEquationLine(canvas, centerX, y, centerX + halfWidth * 0.6, y + fontSize * 0.15, color, strokeWidth);
      case 'bar':
      case 'overline':
      case 'strikeThrough':
        return this.drawEquationLine(canvas, centerX - halfWidth, y + fontSize * 0.05, centerX + halfWidth, y + fontSize * 0.05, color, strokeWidth);
      case 'underline':
      case 'under':
        return this.drawEquationLine(canvas, centerX - halfWidth, y + fontSize * 1.1, centerX + halfWidth, y + fontSize * 1.1, color, strokeWidth);
      case 'vec':
      case 'dyad': {
        const lineY = y + fontSize * 0.05;
        const startX = centerX - halfWidth;
        const endX = centerX + halfWidth;
        let ok = this.drawEquationLine(canvas, startX, lineY, endX, lineY, color, strokeWidth)
          && this.drawEquationLine(canvas, endX - fontSize * 0.1, lineY - fontSize * 0.06, endX, lineY, color, strokeWidth)
          && this.drawEquationLine(canvas, endX, lineY, endX - fontSize * 0.1, lineY + fontSize * 0.06, color, strokeWidth);
        if (decoration === 'dyad') {
          ok = ok
            && this.drawEquationLine(canvas, startX + fontSize * 0.1, lineY - fontSize * 0.06, startX, lineY, color, strokeWidth)
            && this.drawEquationLine(canvas, startX, lineY, startX + fontSize * 0.1, lineY + fontSize * 0.06, color, strokeWidth);
        }
        return ok;
      }
      case 'dot':
      case 'dDot': {
        const paint = this.makeFillPaint(color);
        const radius = Math.max(fontSize * 0.03, 1);
        try {
          if (decoration === 'dot') {
            canvas.drawCircle(centerX, y + fontSize * 0.06, radius, paint);
          } else {
            canvas.drawCircle(centerX - fontSize * 0.1, y + fontSize * 0.06, radius, paint);
            canvas.drawCircle(centerX + fontSize * 0.1, y + fontSize * 0.06, radius, paint);
          }
          return true;
        } finally {
          paint.delete?.();
        }
      }
      default:
        return false;
    }
  }

  private renderFormObject(canvas: SkCanvas, op: LayerFormObjectOp): void {
    const rejection = formDrawingReplayRejection(op);
    if (rejection) { this.unsupportedOps.add(`formObject:${rejection}`); return; }
    const drawing = op.drawing!;
    for (const primitive of drawing.primitives) {
      const paint = primitive.kind === 'polyline' && !primitive.closed
        ? this.makeStrokePaint(primitive.color, primitive.width)
        : this.makeFillPaint(primitive.color);
      try {
        if (primitive.kind === 'rect') {
          canvas.drawRect(this.rect(primitive.bbox), paint);
        } else if (primitive.kind === 'circle') {
          canvas.drawCircle(primitive.x, primitive.y, primitive.radius, paint);
        } else if (primitive.points.length > 0) {
          const path = this.pathFromCommands([
            { type: 'moveTo', x: primitive.points[0][0], y: primitive.points[0][1] },
            ...primitive.points.slice(1).map(([x, y]) => ({ type: 'lineTo' as const, x, y })),
            ...(primitive.closed ? [{ type: 'closePath' as const }] : []),
          ]);
          try { canvas.drawPath(path, paint); } finally { path.delete(); }
        }
      } finally { paint.delete(); }
    }
    const label = drawing.label;
    if (label?.text && !this.drawStaticControlText(canvas, label.text, {
      x: label.x, y: label.baseline, fontSize: label.fontSize, color: label.color,
      bold: label.bold, italic: label.italic,
      fontFamilies: [label.fontFamily, 'Noto Sans KR', 'Noto Sans CJK KR'],
    })) this.unsupportedOps.add('formObject:textReplayFailed');
  }

  private renderPlaceholder(canvas: SkCanvas, op: LayerPlaceholderOp): void {
    if (op.kind === 'missingPicture') {
      if (![op.bbox.x, op.bbox.y, op.bbox.width, op.bbox.height].every(Number.isFinite)
        || op.bbox.width <= 0 || op.bbox.height <= 0) return;
      const paint = this.makeStrokePaint(op.strokeColor ?? '#999999', 1);
      const dash = 5;
      const gap = 3;
      const horizontalStep = Math.max(
        dash + gap,
        op.bbox.width / CanvasKitLayerRenderer.MAX_PLACEHOLDER_DASH_SEGMENTS_PER_AXIS,
      );
      const verticalStep = Math.max(
        dash + gap,
        op.bbox.height / CanvasKitLayerRenderer.MAX_PLACEHOLDER_DASH_SEGMENTS_PER_AXIS,
      );
      try {
        for (let x = op.bbox.x; x < op.bbox.x + op.bbox.width; x += horizontalStep) {
          const end = Math.min(x + horizontalStep * dash / (dash + gap), op.bbox.x + op.bbox.width);
          canvas.drawLine(x, op.bbox.y, end, op.bbox.y, paint);
          canvas.drawLine(x, op.bbox.y + op.bbox.height, end, op.bbox.y + op.bbox.height, paint);
        }
        for (let y = op.bbox.y; y < op.bbox.y + op.bbox.height; y += verticalStep) {
          const end = Math.min(y + verticalStep * dash / (dash + gap), op.bbox.y + op.bbox.height);
          canvas.drawLine(op.bbox.x, y, op.bbox.x, end, paint);
          canvas.drawLine(op.bbox.x + op.bbox.width, y, op.bbox.x + op.bbox.width, end, paint);
        }
      } finally {
        paint.delete?.();
      }
      const icon = Math.max(14, Math.min(36, Math.min(op.bbox.width, op.bbox.height) * 0.4));
      const ix = op.bbox.x + (op.bbox.width - icon) / 2;
      const iy = op.bbox.y + (op.bbox.height - icon * 0.75) / 2;
      const iconBounds = this.canvasKit.XYWHRect(ix, iy, icon, icon * 0.75);
      let iconFill: SkPaint | null = null;
      let iconStroke: SkPaint | null = null;
      let missingStroke: SkPaint | null = null;
      try {
        iconFill = this.makeFillPaint('#ffffff');
        iconStroke = this.makeStrokePaint('#888888', 1);
        missingStroke = this.makeStrokePaint('#cc4444', 1.5);
        canvas.drawRect(iconBounds, iconFill);
        canvas.drawRect(iconBounds, iconStroke);
        canvas.drawLine(ix + icon * 0.08, iy + icon * 0.62, ix + icon * 0.32, iy + icon * 0.30, iconStroke);
        canvas.drawLine(ix + icon * 0.32, iy + icon * 0.30, ix + icon * 0.52, iy + icon * 0.62, iconStroke);
        canvas.drawLine(ix + icon * 0.52, iy + icon * 0.62, ix + icon * 0.68, iy + icon * 0.42, iconStroke);
        canvas.drawLine(ix + icon * 0.68, iy + icon * 0.42, ix + icon * 0.92, iy + icon * 0.62, iconStroke);
        canvas.drawCircle(ix + icon * 0.72, iy + icon * 0.20, icon * 0.07, iconStroke);
        canvas.drawLine(ix, iy + icon * 0.75, ix + icon, iy, missingStroke);
      } finally {
        missingStroke?.delete?.();
        iconStroke?.delete?.();
        iconFill?.delete?.();
      }
      return;
    }
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
    gradient?: LayerGradientFill,
  ): void {
    const shadow = this.resolvedShadow(style?.shadow);
    if (shadow) {
      canvas.save();
      canvas.translate(shadow.offsetX, shadow.offsetY);
      const shadowPaint = this.makeFillPaint(shadow.color, shadow.opacity);
      draw(shadowPaint);
      shadowPaint.delete?.();
      if (style?.strokeColor && (style.strokeWidth ?? 0) > 0) {
        const shadowStroke = this.makeStrokePaint(shadow.color, style.strokeWidth ?? 1, shadow.opacity);
        try {
          this.drawStrokeWithDash(style.strokeDash, shadowStroke, () => draw(shadowStroke));
        } finally {
          shadowStroke.delete?.();
        }
      }
      canvas.restore();
    }
    const fillShader = this.makeShapeGradientShader(gradient, bounds);
    if (fillShader) {
      const paint = this.makeFillPaint(style?.fillColor ?? '#000000', style?.opacity);
      try {
        (paint as unknown as { setShader: (shader: unknown) => void }).setShader(fillShader);
        draw(paint);
      } finally {
        (fillShader as { delete?: () => void }).delete?.();
        paint.delete?.();
      }
    } else if (style?.pattern) {
      this.drawPatternFill(canvas, bounds, style.pattern, style.opacity ?? 1, draw);
    } else if (style?.fillColor) {
      const paint = this.makeFillPaint(style.fillColor, style.opacity);
      draw(paint);
      paint.delete?.();
    }
    if (style?.strokeColor && (style.strokeWidth ?? 0) > 0) {
      const paint = this.makeStrokePaint(style.strokeColor, style.strokeWidth ?? 1, style.opacity);
      try {
        this.drawStrokeWithDash(style.strokeDash, paint, () => draw(paint));
      } finally {
        paint.delete?.();
      }
    }
    if (!fillShader && !style?.fillColor && !style?.strokeColor && !style?.pattern) {
      const paint = this.makeStrokePaint('#000000', 1);
      draw(paint);
      paint.delete?.();
    }
  }

  private drawStyledPath(
    canvas: SkCanvas,
    path: Path,
    style: LayerShapeStyle,
    bounds?: LayerBounds,
    gradient?: LayerGradientFill,
  ): void {
    let resolvedBounds = bounds ?? { x: 0, y: 0, width: 0, height: 0 };
    if (!bounds) {
      const raw = (path as Path & { getBounds?: () => Float32Array | number[] }).getBounds?.();
      if (raw && raw.length >= 4) {
        resolvedBounds = {
          x: raw[0],
          y: raw[1],
          width: Math.max(0, raw[2] - raw[0]),
          height: Math.max(0, raw[3] - raw[1]),
        };
      }
    }
    this.drawStyledShape(canvas, resolvedBounds, style, (paint) => {
      canvas.drawPath(path, paint);
    }, gradient);
  }

  private makeShapeGradientShader(
    gradient: LayerGradientFill | undefined,
    bounds: LayerBounds,
  ): unknown | null {
    const colors = gradient?.colors;
    if (!gradient || !Array.isArray(colors) || colors.length < 2) {
      return null;
    }
    const lastIndex = Math.max(1, colors.length - 1);
    const shaderColors = colors.map((css) => {
      const { r, g, b, a } = parseCssColor(css);
      return [r / 255, g / 255, b / 255, a];
    });
    const positions = colors.map((_, index) => {
      const raw = index < (gradient.positions?.length ?? 0)
        ? gradient.positions?.[index]
        : index / lastIndex;
      return Math.max(0, Math.min(1, Number.isFinite(raw) ? (raw as number) : index / lastIndex));
    });
    const shaderApi = this.canvasKit.Shader as unknown as {
      MakeLinearGradient?: (...args: unknown[]) => unknown;
      MakeRadialGradient?: (...args: unknown[]) => unknown;
    };
    const gradientType = gradient.gradientType ?? 1;
    if (gradientType >= 2 && gradientType <= 4) {
      const cx = bounds.x + bounds.width * ((gradient.centerX ?? 50) / 100);
      const cy = bounds.y + bounds.height * ((gradient.centerY ?? 50) / 100);
      const radius = Math.max(bounds.width, bounds.height) / 2;
      return shaderApi.MakeRadialGradient?.(
        [cx, cy],
        radius,
        shaderColors,
        positions,
        this.canvasKit.TileMode.Clamp,
      ) ?? null;
    }
    const [x0, y0, x1, y1] = shapeGradientLinearCoords(gradient.angle ?? 0, bounds);
    return shaderApi.MakeLinearGradient?.(
      [x0, y0],
      [x1, y1],
      shaderColors,
      positions,
      this.canvasKit.TileMode.Clamp,
    ) ?? null;
  }

  private resolvedShadow(shadow: LayerShapeStyle['shadow']): { color: string; offsetX: number; offsetY: number; opacity: number } | null {
    if (!shadow) return null;
    const offsetX = shadow.offsetX ?? 0;
    const offsetY = shadow.offsetY ?? 0;
    if (![offsetX, offsetY].every(Number.isFinite)) return null;
    const alpha = Number.isFinite(shadow.alpha) ? Number(shadow.alpha) : 0;
    return {
      color: shadow.color ?? '#000000',
      offsetX,
      offsetY,
      opacity: Math.max(0, Math.min(1, 1 - alpha / 255)),
    };
  }

  private drawPatternFill(
    _canvas: SkCanvas,
    _bounds: LayerBounds,
    pattern: NonNullable<LayerShapeStyle['pattern']>,
    opacity: number,
    draw: (paint: SkPaint) => void,
  ): void {
    const kind = pattern.patternType ?? 0;
    const bg = parseCssColor(pattern.backgroundColor ?? '#ffffff');
    const fg = parseCssColor(pattern.patternColor ?? '#000000');
    const bytes = new Uint8Array(8 * 8 * 4);
    for (let y = 0; y < 8; y++) {
      for (let x = 0; x < 8; x++) {
        const horizontal = y === 0;
        const vertical = x === 0;
        const slash = (x + y) % 8 === 0;
        const backslash = x === y;
        const ink = kind === 1 ? horizontal : kind === 2 ? vertical
          : kind === 3 ? slash : kind === 4 ? backslash
          : kind === 5 ? horizontal || vertical : kind === 6 ? slash || backslash : false;
        const color = ink ? fg : bg;
        bytes.set([color.r, color.g, color.b, Math.round(color.a * 255)], (y * 8 + x) * 4);
      }
    }
    const image = this.canvasKit.MakeImage({
      width: 8, height: 8, colorType: this.canvasKit.ColorType.RGBA_8888,
      alphaType: this.canvasKit.AlphaType.Unpremul, colorSpace: this.canvasKit.ColorSpace.SRGB,
    }, bytes, 8 * 4);
    if (!image) return;
    const scale = (pattern.tileSize ?? 7.68) / 8;
    const shader = image.makeShaderOptions(
      this.canvasKit.TileMode.Repeat, this.canvasKit.TileMode.Repeat,
      this.canvasKit.FilterMode.Linear, this.canvasKit.MipmapMode.None,
      [scale, 0, -scale / 2, 0, scale, -scale / 2, 0, 0, 1],
    );
    const paint = this.makeFillPaint('#ffffff', opacity);
    try {
      paint.setShader(shader);
      // 무늬도 원래 도형 경로를 채워 괘선·타원·path의 소유 영역을 보존한다.
      draw(paint);
    } finally {
      paint.delete(); shader.delete(); image.delete();
    }
    this.patternReplayOps++;
  }

  private compoundLineSegments(lineType: string | undefined): Array<[number, number]> {
    switch (lineType) {
      case 'double':
        return [[0.30, -0.35], [0.30, 0.35]];
      case 'thickThinDouble':
        return [[0.4, -0.30], [0.2, 0.40]];
      case 'thinThickDouble':
        return [[0.2, -0.40], [0.4, 0.30]];
      case 'thinThickThinTriple':
        return [[0.15, -0.425], [0.30, 0.0], [0.15, 0.425]];
      default:
        return [[1, 0]];
    }
  }

  private drawCompoundLine(
    canvas: SkCanvas,
    x1: number,
    y1: number,
    x2: number,
    y2: number,
    style: LayerLineStyle,
    opacity: number,
  ): void {
    const width = style.width ?? 1;
    const color = style.color ?? '#000000';
    const dx = x2 - x1;
    const dy = y2 - y1;
    const lineLen = Math.hypot(dx, dy);
    const nx = lineLen > 0 ? -dy / lineLen : 0;
    const ny = lineLen > 0 ? dx / lineLen : 1;
    for (const [widthRatio, offsetRatio] of this.compoundLineSegments(style.lineType)) {
      const paint = this.makeStrokePaint(color, Math.max(0.3, width * widthRatio), opacity);
      const ox = nx * width * offsetRatio;
      const oy = ny * width * offsetRatio;
      try {
        this.drawStrokeWithDash(style.dash, paint, () => {
          canvas.drawLine(x1 + ox, y1 + oy, x2 + ox, y2 + oy, paint);
        });
      } finally {
        paint.delete?.();
      }
    }
  }

  private drawLineArrows(
    canvas: SkCanvas,
    heads: LayerLineOp['arrowHeads'],
    style: LayerLineStyle,
    color: string,
    width: number,
  ): void {
    if (!heads && ((style.startArrow && style.startArrow !== 'none')
      || (style.endArrow && style.endArrow !== 'none'))) {
      this.unsupportedOps.add('arrow:geometryMissing');
      return;
    }
    for (const head of heads ?? []) {
      const path = this.pathFromCommands(head.commands, 0, 0);
      const fill = this.makeFillPaint(head.filled ? color : '#ffffff');
      const outline = head.filled ? null : this.makeStrokePaint(color, Math.max(0.5, width * 0.3), 1);
      try {
        canvas.drawPath(path, fill);
        if (outline) canvas.drawPath(path, outline);
      } finally {
        outline?.delete?.();
        fill.delete?.();
        path.delete?.();
      }
    }
  }

  private drawStrokeWithDash(
    dash: LayerStrokeDash | undefined,
    paint: SkPaint,
    draw: () => void,
  ): void {
    const intervals = dash === undefined || dash === 'solid'
      ? null
      : dash === 'dash'
        ? [6, 3]
        : dash === 'dot'
          ? [2, 2]
          : dash === 'dashDot'
            ? [6, 3, 2, 3]
            : dash === 'dashDotDot'
              ? [6, 3, 2, 3, 2, 3]
              : undefined;
    if (intervals === undefined) {
      this.unsupportedOps.add(`strokeDash:${String(dash)}`);
      return;
    }
    if (intervals === null) {
      draw();
      return;
    }

    const effect = this.canvasKit.PathEffect.MakeDash(intervals, 0);
    if (!effect) {
      this.unsupportedOps.add('strokeDash:pathEffectUnavailable');
      return;
    }
    try {
      paint.setPathEffect(effect);
      draw();
      this.currentReplayFeatureCounts.dashedStrokes += 1;
    } finally {
      effect.delete?.();
    }
  }

  private imageForOp(op: LayerImageOp): SkImage | null {
    const base64 = op.base64 ?? '';
    if (base64.length > CANVASKIT_MAX_ENCODED_IMAGE_BASE64_LENGTH) {
      this.recordImageFailure(op, 'encodedImageRejected', null);
      return null;
    }
    const resourceIndex = this.layerResourceIndex(op.imageRef, this.currentResources?.imageKeys,
      this.currentResources?.images?.length ?? 0);
    const resourceKey = resourceIndex === null ? undefined : this.currentResources?.imageKeys?.[resourceIndex];
    let sourceKey = canvasKitImageCacheKey(resourceKey ? { sourceImageKey: resourceKey } : op, this.documentGeneration);
    const resource = resourceIndex === null ? undefined : this.currentResources?.images?.[resourceIndex];
    let providedBytes: Uint8Array | null = null;
    // Object transport owns copies, but callers can still mutate their replay input.
    // Validate supplied bytes before a native cache hit; source keys alone are not proof.
    if (resource !== undefined || base64) {
      try { providedBytes = this.imageResourceBytes(resource ?? base64); }
      catch { this.recordImageFailure(op, 'base64DecodeFailed', null); return null; }
      if (!providedBytes || (resourceKey && !layerResourceKeyMatches('img', resourceKey, providedBytes))) {
        this.recordImageFailure(op, 'encodedImageRejected', null); return null;
      }
      const contentKey = replayFingerprint(providedBytes);
      if (!contentKey) { this.recordImageFailure(op, 'encodedImageRejected', null); return null; }
      sourceKey = `${sourceKey ?? 'provided'}:bytes:${contentKey}`;
    }
    const effectKey = canvasKitImageEffectKey(op);
    if (!sourceKey) {
      this.recordImageFailure(op, 'cacheKeyMissing', null);
      return null;
    }
    if (effectKey === null) {
      this.recordImageFailure(op, 'imageEffectFailed', null);
      return null;
    }
    const effectSourceKey = effectKey ? JSON.stringify([op.crop, op.originalSizeHu, op.fillMode]) : '';
    const key = `${sourceKey}:effect:${effectKey}:crop:${effectSourceKey}`;
    const cached = this.imageCache.get(key);
    if (cached) {
      this.imageCache.delete(key);
      this.imageCache.set(key, cached);
      this.imageCacheHits += 1;
      if (effectKey) this.imageEffectDiagnostics.cacheHits++;
      const recovery = this.imageRecoveryByKey.get(key);
      if (recovery) this.imageRecoveries.add(recovery);
      return cached.image;
    }
    const cachedFailure = this.imageDecodeFailures.get(key);
    if (cachedFailure) {
      this.imageCacheHits += 1;
      this.imageFailureCacheHits += 1;
      this.recordImageFailure(op, cachedFailure, key);
      return null;
    }
    if (this.pendingImageLoads.has(key)) { this.pendingImageAccesses++; return null; }
    this.imageCacheMisses += 1;
    if (effectKey) this.imageEffectDiagnostics.cacheMisses++;
    let bytes: Uint8Array;
    try {
      const sourceBytes = resource === undefined && !base64 && op.sourceImageKey
        ? this.currentImageBytesResolver?.(op.sourceImageKey) : undefined;
      const resolved = providedBytes ?? this.imageResourceBytes(sourceBytes ?? undefined);
      if (!resolved) {
        this.recordImageFailure(op, 'dataMissing', key);
        return null;
      }
      bytes = resolved;
      if (resourceKey && !layerResourceKeyMatches('img', resourceKey, bytes)) {
        this.recordImageFailure(op, 'encodedImageRejected', key);
        return null;
      }
    } catch {
      this.recordImageFailure(op, 'base64DecodeFailed', key);
      return null;
    }
    const encodedHeader = replayableEncodedImageHeader(bytes);
    if (!encodedHeader) {
      this.recordImageFailure(op, 'encodedImageRejected', key);
      return null;
    }
    let image: SkImage | null = null;
    if (encodedHeader.format !== 'svg') {
      try { image = this.canvasKit.MakeImageFromEncoded(bytes); } catch { /* bounded browser decode 시도 */ }
    }
    if (!image && encodedImageHasStableFrame(bytes, encodedHeader)) {
      if (this.pendingImageLoads.size >= CanvasKitLayerRenderer.MAX_IMAGE_CACHE_ENTRIES) {
        this.recordImageFailure(op, 'encodedImageRejected', key);
        return null;
      }
      const generation = this.documentGeneration;
      const effectOp = { ...op, crop: op.crop ? { ...op.crop } : undefined,
        originalSizeHu: op.originalSizeHu ? [...op.originalSizeHu] as [number, number] : undefined };
      const load = decodeBrowserImage(this.canvasKit, bytes, encodedHeader);
      const pending = {
        cancel: load.cancel,
        promise: load.promise.then(decoded => {
          if (this.disposed || generation !== this.documentGeneration || this.pendingImageLoads.get(key) !== pending) {
            decoded?.delete();
            return;
          }
          this.pendingImageLoads.delete(key);
          let image = decoded;
          if (image) this.imagesDecoded++;
          if (image && effectKey) {
            image = this.applyImageEffect(decoded!, effectOp);
            decoded!.delete();
          }
          if (!image) this.recordImageFailure(op, decoded ? 'imageEffectFailed' : 'imageDecodeFailed', key);
          else {
            const recovery: CanvasKitImageRecoveryDiagnostic = {
              source: op.sourceImageKey ? 'sourceKey' : op.imageRef !== undefined ? 'resource' : 'inline',
              sourceImageKey: boundedCanvasKitSourceImageKey(op.sourceImageKey), imageRef: op.imageRef ?? null,
              reason: encodedHeader.format === 'svg' ? 'staticSvgRequiresBrowserPreparation' : 'encodedImageDecodeFailed',
              fallback: 'browserImageSource', format: encodedHeader.format };
            this.imageRecoveryByKey.set(key, recovery); this.imageRecoveries.add(recovery);
            this.retainImage(key, image, image.width() * image.height());
          }
        }),
      };
      this.pendingImageLoads.set(key, pending);
      return null;
    }
    if (!image) {
      this.recordImageFailure(op, 'imageDecodeFailed', key);
      return null;
    }
    const imageWithDimensions = image as SkImage & { width?: (() => number) | number; height?: (() => number) | number };
    const width = typeof imageWithDimensions.width === 'function' ? imageWithDimensions.width() : imageWithDimensions.width;
    const height = typeof imageWithDimensions.height === 'function' ? imageWithDimensions.height() : imageWithDimensions.height;
    const decodedPixels = typeof width === 'number' && typeof height === 'number'
      ? width * height
      : Number.POSITIVE_INFINITY;
    if (!decodedImageMatchesEncodedHeader(encodedHeader, width, height)) {
      image.delete?.();
      this.recordImageFailure(op, 'decodedDimensionsMismatch', key);
      return null;
    }
    if (effectKey) {
      const processed = this.applyImageEffect(image, op);
      image.delete?.();
      image = processed;
      if (!image) {
        this.recordImageFailure(op, 'imageEffectFailed', key);
        return null;
      }
    }
    this.imagesDecoded++;
    return this.retainImage(key, image, effectKey ? image.width() * image.height() : decodedPixels);
  }

  private applyImageEffect(original: SkImage, op: LayerImageOp): SkImage | null {
    const started = performance.now();
    const crop = canvasKitEffectCrop(original, op);
    const image = makeCanvasKitEffectImage(this.canvasKit, original, op, route => {
      if (route === 'direct') this.imageEffectDiagnostics.directImageReadbackPreprocesses++;
      else this.imageEffectDiagnostics.nativeSurfacePreprocesses++;
    });
    const elapsed = performance.now() - started;
    this.imageEffectDiagnostics.preprocessTimeMs += elapsed;
    this.imageEffectDiagnostics.maxPreprocessTimeMs = Math.max(this.imageEffectDiagnostics.maxPreprocessTimeMs, elapsed);
    if (!image) this.imageEffectDiagnostics.preprocessFailures++;
    else {
      const pixels = image.width() * image.height();
      this.imageEffectDiagnostics.preprocessedPixels += pixels; this.imageEffectDiagnostics.preprocessedBytes += pixels * 4;
      this.imageEffectDiagnostics.maxPreprocessedBytes = Math.max(this.imageEffectDiagnostics.maxPreprocessedBytes, pixels * 4);
    }
    if (image && crop) this.croppedEffectImages.set(image, { width: crop.width, height: crop.height });
    return image;
  }

  private imageResourceBytes(value: Uint8Array | number[] | string | undefined): Uint8Array | null {
    if (typeof value === 'string') {
      if (!value || value.length > CANVASKIT_MAX_ENCODED_IMAGE_BASE64_LENGTH) return null;
      return base64ToBytes(value);
    }
    if (!value || value.length > CANVASKIT_MAX_ENCODED_IMAGE_BASE64_LENGTH * 3 / 4) return null;
    if (value instanceof Uint8Array) return value;
    if (!value.every(byte => Number.isInteger(byte) && byte >= 0 && byte <= 255)) return null;
    return new Uint8Array(value);
  }

  private retainImage(key: string, image: SkImage, decodedPixels: number): SkImage {
    this.imageResourceRevision += 1;
    while (this.imageCache.size >= CanvasKitLayerRenderer.MAX_IMAGE_CACHE_ENTRIES
      || this.imageCachePixels + decodedPixels > CanvasKitLayerRenderer.MAX_IMAGE_CACHE_PIXELS) {
      const oldestKey = this.imageCache.keys().next().value as string | undefined;
      if (oldestKey === undefined) break;
      const oldest = this.imageCache.get(oldestKey);
      const mipmapKey = oldest ? this.imageNativeKeys.get(oldest.image) : undefined;
      if (mipmapKey) this.mipmapImages.delete(mipmapKey);
      oldest?.image.delete?.();
      this.imageCache.delete(oldestKey);
      this.imageRecoveryByKey.delete(oldestKey);
      this.imageCachePixels = Math.max(0, this.imageCachePixels - (oldest?.pixels ?? 0));
      this.imageCacheEvictions += 1;
    }
    this.imageCache.set(key, { image, pixels: decodedPixels });
    this.imageCachePixels += decodedPixels;
    return image;
  }

  private recordImageFailure(
    op: LayerImageOp,
    reason: CanvasKitImageFailureReason,
    key: string | null,
  ): void {
    this.imageFailureAttempts++;
    if (key) {
      if (!this.imageDecodeFailures.has(key)
        && this.imageDecodeFailures.size >= CanvasKitLayerRenderer.MAX_IMAGE_FAILURE_CACHE_ENTRIES) {
        const oldestKey = this.imageDecodeFailures.keys().next().value as string | undefined;
        if (oldestKey !== undefined) this.imageDecodeFailures.delete(oldestKey);
      }
      this.imageDecodeFailures.set(key, reason);
    }

    const sourceImageKey = boundedCanvasKitSourceImageKey(op.sourceImageKey);
    const imageRef = (
      (typeof op.imageRef === 'number' && Number.isSafeInteger(op.imageRef))
      || (
        typeof op.imageRef === 'string'
        && op.imageRef.length > 0
        && op.imageRef.length <= 256
        && !/[\u0000-\u001f\u007f]/.test(op.imageRef)
      )
    ) ? op.imageRef : null;
    const source = sourceImageKey
      ? 'sourceKey'
      : imageRef !== null
        ? 'resource'
        : op.base64
          ? 'inline'
          : 'missing';
    const diagnosticKey = key
      ?? `${source}:${sourceImageKey ?? String(imageRef ?? op.base64?.length ?? 0)}:${reason}`;
    if (this.currentImageFailures.has(diagnosticKey)
      || this.currentImageFailures.size >= CanvasKitLayerRenderer.MAX_IMAGE_FAILURE_CACHE_ENTRIES) {
      return;
    }
    this.currentImageFailures.set(diagnosticKey, {
      source,
      sourceImageKey,
      imageRef,
      reason,
    });
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

function shapeGradientLinearCoords(
  angle: number,
  bounds: LayerBounds,
): [number, number, number, number] {
  const { x, y, width: w, height: h } = bounds;
  const normalized = ((angle % 360) + 360) % 360;
  switch (normalized) {
    case 0:
      return [x, y, x, y + h];
    case 45:
      return [x, y, x + w, y + h];
    case 90:
      return [x, y, x + w, y];
    case 135:
      return [x, y + h, x + w, y];
    case 180:
      return [x, y + h, x, y];
    case 225:
      return [x + w, y + h, x, y];
    case 270:
      return [x + w, y, x, y];
    case 315:
      return [x + w, y, x, y + h];
    default: {
      const rad = (normalized * Math.PI) / 180;
      const cx = x + w / 2;
      const cy = y + h / 2;
      return [
        cx - Math.sin(rad) * w / 2,
        cy - Math.cos(rad) * h / 2,
        cx + Math.sin(rad) * w / 2,
        cy + Math.cos(rad) * h / 2,
      ];
    }
  }
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
