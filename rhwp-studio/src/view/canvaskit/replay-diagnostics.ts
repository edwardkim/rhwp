import type { LayerBounds } from '../../core/types.ts';

/** Diagnostic metadata is copied out; native handles and whole document strings never enter it. */
export class BoundedReplayLog<T> {
  readonly values: T[] = [];
  dropped = 0;
  add(value: T): void { if (this.values.length < 4096) this.values.push(value); else this.dropped++; }
  clear(): void { this.values.length = 0; this.dropped = 0; }
  snapshot(): T[] { return structuredClone(this.values); }
}

export interface CanvasKitEquationReplayDiagnostic {
  svgResourceId: number | null;
  hasInlineSvg: boolean;
  bbox: LayerBounds;
  route: 'svg' | 'layout' | 'failed';
  reason: 'svgReplayed' | 'layoutRequested' | 'svgResourceMissing' | 'svgPayloadUnsupported'
    | 'invalidEquationBounds' | 'svgPathDecodeFailed' | 'invalidLayout';
}

export interface CanvasKitTextReplayDiagnostic {
  opId: string | null;
  fontFamily: string;
  /** Null means this helper has no exact source-cluster join, not UTF-16 offset zero. */
  clusterStartUtf16: number | null;
  clusterLengthUtf16: number | null;
  reason: 'textBlobConstructionFailed' | 'simpleTextFallbackFailed';
  fallback: 'drawText' | 'drawGlyphs';
}

export interface CanvasKitImageRecoveryDiagnostic {
  source: 'sourceKey' | 'resource' | 'inline';
  sourceImageKey: string | null;
  imageRef: number | string | null;
  reason: 'encodedImageDecodeFailed' | 'staticSvgRequiresBrowserPreparation';
  fallback: 'browserImageSource';
  format: string;
}

export interface CanvasKitImageEffectDiagnostics {
  cacheHits: number;
  cacheMisses: number;
  preprocessFailures: number;
  fallbackToOriginal: 0;
  preprocessedPixels: number;
  preprocessedBytes: number;
  maxPreprocessedBytes: number;
  preprocessTimeMs: number;
  maxPreprocessTimeMs: number;
  directImageReadbackPreprocesses: number;
  nativeSurfacePreprocesses: number;
}

export function emptyImageEffectDiagnostics(): CanvasKitImageEffectDiagnostics {
  return { cacheHits: 0, cacheMisses: 0, preprocessFailures: 0, fallbackToOriginal: 0,
    preprocessedPixels: 0, preprocessedBytes: 0, maxPreprocessedBytes: 0,
    preprocessTimeMs: 0, maxPreprocessTimeMs: 0, directImageReadbackPreprocesses: 0, nativeSurfacePreprocesses: 0 };
}

export interface LayerTextVariantReplayStatus {
  replayable: boolean;
  reason?: string;
  details?: string;
  fontVerification?: Record<string, string | number | boolean | undefined>;
  outlineEligibility?: { strictVisualEligible: boolean; payloadSupported: boolean; paintStyleSupported: boolean;
    replayEligible: boolean; reason?: string };
}

export interface LayerTextVariantGroupReport {
  backend: 'canvaskit';
  renderProfile?: string;
  equivalenceGroup: string;
  selectedVariantId: string;
  selectedVariantKind: string;
  selectedReason: 'glyphRunStrictEligible' | 'glyphOutlineStrictProfile' | 'defaultTextRunFallback' | 'noSupportedVariant';
  anchorOpId?: string;
  partsExpected: number;
  /** Selected parts proven eligible before replay; this is not an executed draw count. */
  partsSelected: number;
  rejectedVariants: Array<{ variantId: string; variantKind: string; reasons: string[] }>;
  parts: Array<LayerTextVariantReplayStatus & { equivalenceGroup: string; variantId: string; variantKind: string;
    partIndex: number; partCount: number }>;
  fontVerification?: LayerTextVariantReplayStatus['fontVerification'];
  outlineEligibility?: LayerTextVariantReplayStatus['outlineEligibility'];
  truncated?: boolean;
}
