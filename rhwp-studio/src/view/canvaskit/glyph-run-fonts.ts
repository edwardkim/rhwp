import { blake3 } from '@noble/hashes/blake3.js';
import { bytesToHex } from '@noble/hashes/utils.js';
import type { Canvas, CanvasKit, Font, Paint, Typeface } from 'canvaskit-wasm';

import type {
  LayerFontBlobResource,
  LayerFontFaceResource,
  LayerFontResources,
  LayerGlyphRunOp,
  LayerResources,
} from '@/core/types';
import { canvasKitFontFaceData } from './sfnt-face.ts';

const MAX_FONT_BLOB_BYTES = 32 * 1024 * 1024;
const MAX_DOCUMENT_FONT_BLOB_BYTES = 64 * 1024 * 1024;
const MAX_ENCODED_FONT_BLOB_LENGTH = Math.ceil(MAX_FONT_BLOB_BYTES / 3) * 4;
const MAX_FONT_BLOB_RESOURCES = 256;
const MAX_FONT_FACE_RESOURCES = 256;
const MAX_GLYPHS_PER_RUN = 4096;
const MAX_GLYPH_RUN_TYPEFACES = 256;
const MAX_GLYPH_RUN_FONTS = 1024;
const MAX_FLOAT32 = 3.4028234663852886e38;
const BOUNDED_VERTICAL_HWP5_TABLE_CELL_REASON = 'boundedVerticalHwp5TableCellV1';
const BOUNDED_VERTICAL_GLYPH_VARIANT_ID = 'verticalGlyphRun';
const FONT_RESOURCE_KEY = /^font:blake3:(0|[1-9][0-9]*):([0-9a-f]{64})$/;
const BASE64_PAYLOAD = /^(?:[A-Za-z0-9+/]{4})*(?:[A-Za-z0-9+/]{2}==|[A-Za-z0-9+/]{3}=)?$/;

export interface CanvasKitGlyphRunReplayReport {
  replayEligibility: LayerGlyphRunOp['diagnostics']['replayEligibility'];
  quality: LayerGlyphRunOp['diagnostics']['quality'];
  digestMatched?: boolean;
  exactFaceInstantiated?: boolean;
  faceIndexSupported?: boolean;
  variationSupported?: boolean;
  effectSupported?: boolean;
}

export type CanvasKitGlyphRunReplayStatus =
  | {
    replayable: true;
    face: LayerFontFaceResource;
    blob: LayerFontBlobResource;
    report: CanvasKitGlyphRunReplayReport;
  }
  | {
    replayable: false;
    reason: string;
    report: CanvasKitGlyphRunReplayReport;
  };

export type FontBytesResolver = (resourceKey: string) => Uint8Array | null;

export class CanvasKitGlyphRunFontCache {
  private readonly canvasKit: CanvasKit;
  private readonly verifiedFontBlobs = new Map<string, ArrayBuffer>();
  private readonly glyphRunTypefaces = new Map<string, Typeface>();
  private readonly glyphRunFonts = new Map<string, Font>();
  private registeredBlobBytes = 0;
  private documentGeneration: number | null = null;

  constructor(canvasKit: CanvasKit) {
    this.canvasKit = canvasKit;
  }

  registerResources(
    fontResources: LayerFontResources | undefined,
    resources: LayerResources | undefined,
    resolveFontBytes?: FontBytesResolver,
    documentGeneration = 0,
  ): void {
    if (!Number.isSafeInteger(documentGeneration) || documentGeneration < 0) return;
    if (this.documentGeneration !== null && documentGeneration < this.documentGeneration) return;
    if (this.documentGeneration !== null && documentGeneration > this.documentGeneration) {
      this.clear();
    }
    this.documentGeneration = documentGeneration;
    if (
      !Array.isArray(fontResources?.blobs)
      || fontResources.blobs.length === 0
      || fontResources.blobs.length > MAX_FONT_BLOB_RESOURCES
      || !Array.isArray(fontResources.faces)
      || fontResources.faces.length > MAX_FONT_FACE_RESOURCES
      || !Array.isArray(resources?.fontBlobs)
      || resources.fontBlobs.length > MAX_FONT_BLOB_RESOURCES
      || !Array.isArray(resources.fontBlobKeys)
      || resources.fontBlobKeys.length > MAX_FONT_BLOB_RESOURCES
    ) return;
    const byKeyTransport = resources.fontBlobs.length === 0 && resolveFontBytes !== undefined;
    const pending: Array<{ cacheKey: string; buffer: ArrayBuffer }> = [];
    const pendingKeys = new Set<string>();
    let pendingBytes = 0;
    for (const blob of fontResources.blobs) {
      if (
        !blob
        || typeof blob !== 'object'
        || blob.portability !== 'portableBlob'
        || blob.digest?.algorithm !== 'blake3'
        || blob.dataRef?.kind !== 'fontBlob'
      ) {
        if (byKeyTransport) return;
        continue;
      }
      const cacheKey = this.fontBlobCacheKey(blob);
      if (this.verifiedFontBlobs.has(cacheKey) || pendingKeys.has(cacheKey)) continue;
      const resourceIndex = resources.fontBlobKeys?.indexOf(blob.dataRef.id) ?? -1;
      if (resourceIndex < 0) {
        if (byKeyTransport) return;
        continue;
      }
      const keyMatch = FONT_RESOURCE_KEY.exec(blob.dataRef.id);
      const declaredByteLength = keyMatch ? Number(keyMatch[1]) : Number.NaN;
      if (
        !keyMatch
        || keyMatch[2] !== blob.digest.value
        || !Number.isSafeInteger(declaredByteLength)
        || declaredByteLength <= 0
        || declaredByteLength > MAX_FONT_BLOB_BYTES
      ) {
        if (byKeyTransport) return;
        continue;
      }
      const inlinePayload = resources.fontBlobs[resourceIndex];
      let bytes = fontBlobPayloadBytes(inlinePayload);
      if (inlinePayload === undefined && resolveFontBytes) {
        try {
          const resolved = resolveFontBytes(blob.dataRef.id);
          bytes = resolved instanceof Uint8Array ? resolved : null;
        } catch {
          bytes = null;
        }
      }
      if (!bytes || !fontResourceKeyMatches(blob.dataRef.id, blob.digest.value, bytes)) {
        if (byKeyTransport) return;
        continue;
      }
      if (
        this.registeredBlobBytes + pendingBytes + bytes.byteLength > MAX_DOCUMENT_FONT_BLOB_BYTES
        || this.verifiedFontBlobs.size + pending.length >= MAX_FONT_BLOB_RESOURCES
      ) {
        if (byKeyTransport) return;
        continue;
      }
      const copy = new Uint8Array(bytes.byteLength);
      copy.set(bytes);
      pending.push({ cacheKey, buffer: copy.buffer });
      pendingKeys.add(cacheKey);
      pendingBytes += copy.byteLength;
    }
    for (const entry of pending) {
      this.verifiedFontBlobs.set(entry.cacheKey, entry.buffer);
    }
    this.registeredBlobBytes += pendingBytes;
  }

  replayStatus(
    run: LayerGlyphRunOp,
    fontResources: LayerFontResources | undefined,
  ): CanvasKitGlyphRunReplayStatus {
    const report = this.baseReport(run);
    const reject = (
      reason: string,
      details: Partial<CanvasKitGlyphRunReplayReport> = {},
    ): CanvasKitGlyphRunReplayStatus => ({
      replayable: false,
      reason,
      report: { ...report, ...details },
    });

    if (
      !run
      || typeof run.diagnostics !== 'object'
      || run.diagnostics === null
      || typeof run.variant !== 'object'
      || run.variant === null
      || !Array.isArray(run.glyphIds)
      || !Array.isArray(run.positions)
      || (run.advances !== undefined && !Array.isArray(run.advances))
      || !Array.isArray(run.clusters)
      || (run.glyphTransforms !== undefined && !Array.isArray(run.glyphTransforms))
      || typeof run.paintStyle !== 'object'
      || run.paintStyle === null
      || typeof run.shapeKey?.fontInstance !== 'object'
      || run.shapeKey.fontInstance === null
      || typeof run.placement?.runToPage !== 'object'
      || run.placement.runToPage === null
    ) return reject('glyphRunMalformed');
    const instance = run.shapeKey.fontInstance;
    if (run.diagnostics.replayEligibility !== 'portable') return reject('nonPortableGlyphRun');
    if (!run.diagnostics.strictVisualEligible) return reject('strictVisualIneligible');
    if (run.diagnostics.quality !== 'exact' && run.diagnostics.quality !== 'positionAdjusted') {
      return reject('qualityNotStrictEligible');
    }
    if (
      run.diagnostics.quality === 'positionAdjusted'
      && (!Number.isFinite(run.diagnostics.maxResidualAfterAdjustmentPx)
        || run.diagnostics.maxResidualAfterAdjustmentPx > 0.25)
    ) return reject('positionAdjustedResidualTooLarge');
    if (run.diagnostics.missingGlyphCount !== 0) return reject('missingGlyph');
    if (run.diagnostics.clusterMismatchCount !== 0) return reject('clusterMismatch');
    if (run.diagnostics.usedFallbackFontCount !== 0) return reject('fontNotPortable');
    const boundedVertical = isBoundedVerticalHwp5CanvasKitCandidate(run);
    const declaresBoundedVertical = run.diagnostics.reason === BOUNDED_VERTICAL_HWP5_TABLE_CELL_REASON
      || run.variant.variantId === BOUNDED_VERTICAL_GLYPH_VARIANT_ID;
    if (declaresBoundedVertical && !boundedVertical) {
      return reject('boundedVerticalGlyphRunTupleMismatch');
    }
    if (!boundedVertical && run.orientation !== 'horizontal') {
      return reject('verticalGlyphOrientationAuthorityPending');
    }
    if (!boundedVertical && run.glyphTransforms?.length) {
      return reject('glyphTransformAuthorityPending');
    }
    if (instance.syntheticBold !== false || instance.syntheticItalic !== false) {
      return reject('syntheticStyleAuthorityPending');
    }
    if (run.direction !== 'ltr' || run.shapeKey.direction !== 'ltr') {
      return reject('bidiDirectionAuthorityPending');
    }
    if (run.bidiLevel !== 0) return reject('bidiLevelAuthorityPending');
    if (
      !boundedVertical
      && (run.writingMode !== 'horizontal-tb' || run.shapeKey.writingMode !== 'horizontal-tb')
    ) {
      return reject('writingModeAuthorityPending');
    }
    if (!run.glyphIds.length) return reject('emptyGlyphRun');
    if (
      run.glyphIds.length > MAX_GLYPHS_PER_RUN
      || run.positions.length > MAX_GLYPHS_PER_RUN
      || (run.advances?.length ?? 0) > MAX_GLYPHS_PER_RUN
      || run.clusters.length > MAX_GLYPHS_PER_RUN
    ) return reject('glyphRunTooLarge');
    if (run.glyphIds.length !== run.positions.length) return reject('glyphPositionCountMismatch');
    if (run.advances && run.advances.length !== run.glyphIds.length) {
      return reject('glyphAdvanceCountMismatch');
    }
    if (run.glyphIds.some(glyphId => !Number.isInteger(glyphId) || glyphId <= 0 || glyphId > 0xffff)) {
      return reject('glyphIdOutOfRange');
    }
    if (run.positions.some(point => (
      !Number.isFinite(point?.x)
      || !Number.isFinite(point?.y)
      || Math.abs(point.x) > MAX_FLOAT32
      || Math.abs(point.y) > MAX_FLOAT32
    ))) {
      return reject('positionNotFinite');
    }
    if (run.advances?.some(advance => (
      !Number.isFinite(advance?.dx)
      || !Number.isFinite(advance?.dy)
      || Math.abs(advance.dx) > MAX_FLOAT32
      || Math.abs(advance.dy) > MAX_FLOAT32
    ))) {
      return reject('advanceNotFinite');
    }
    const transform = run.placement?.runToPage;
    if (
      !transform
      || !Number.isFinite(run.placement.baselineY ?? 0)
      || ![transform.a, transform.b, transform.c, transform.d, transform.e, transform.f]
        .every(value => Number.isFinite(value) && Math.abs(value) <= MAX_FLOAT32)
      || Math.abs(run.placement.baselineY ?? 0) > MAX_FLOAT32
    ) return reject('placementNotFinite');
    if (!glyphRunPaintIsSupported(run)) return reject('unsupportedPaintEffect', { effectSupported: false });
    if (run.shapeKey.fontInstance.variations?.length) {
      return reject('variationUnsupported', { variationSupported: false });
    }
    const fontSize = run.shapeKey.fontInstance.sizePx;
    if (!Number.isFinite(fontSize) || fontSize <= 0 || fontSize > 4096) {
      return reject('fontInstanceInvalid');
    }

    if (fontResources && (
      !Array.isArray(fontResources.faces)
      || !Array.isArray(fontResources.blobs)
    )) return reject('fontResourceTableMalformed', { exactFaceInstantiated: false });
    if (
      (fontResources?.faces?.length ?? 0) > MAX_FONT_FACE_RESOURCES
      || (fontResources?.blobs?.length ?? 0) > MAX_FONT_BLOB_RESOURCES
    ) return reject('fontResourceTableTooLarge', { exactFaceInstantiated: false });
    const face = fontResources?.faces?.find(candidate => (
      candidate?.id === run.shapeKey.fontInstance.faceKey
    ));
    if (!face) return reject('fontFaceMissing', { exactFaceInstantiated: false });
    if (!Number.isInteger(face.faceIndex) || face.faceIndex < 0) {
      return reject('faceIndexUnsupported', { exactFaceInstantiated: false, faceIndexSupported: false });
    }
    const blob = fontResources?.blobs?.find(candidate => candidate?.id === face.blobKey);
    if (!blob) return reject('fontBlobMissing', { exactFaceInstantiated: false });
    if (
      blob.portability !== 'portableBlob'
      || blob.digest?.algorithm !== 'blake3'
      || blob.dataRef?.kind !== 'fontBlob'
    ) return reject('fontBlobNotPortable', { digestMatched: false, exactFaceInstantiated: false });
    if (!this.verifiedFontBlobs.has(this.fontBlobCacheKey(blob))) {
      return reject('fontBlobNotVerified', { digestMatched: false, exactFaceInstantiated: false });
    }
    const typefaceKey = this.typefaceCacheKey(face, blob);
    if (!this.glyphRunTypefaces.has(typefaceKey)
      && this.glyphRunTypefaces.size >= MAX_GLYPH_RUN_TYPEFACES) {
      return reject('typefaceLimitExceeded', { exactFaceInstantiated: false });
    }
    if (!this.typefaceFor(face, blob)) {
      const bytes = this.verifiedFontBlobs.get(this.fontBlobCacheKey(blob));
      const faceSupported = !!bytes && canvasKitFontFaceData(bytes, face.faceIndex) !== null;
      return reject(faceSupported ? 'fontFaceInstantiationFailed' : 'faceIndexUnsupported', {
        digestMatched: true,
        exactFaceInstantiated: false,
        faceIndexSupported: faceSupported,
      });
    }
    const fontKey = this.fontCacheKey(run, face, blob);
    if (!this.glyphRunFonts.has(fontKey) && this.glyphRunFonts.size >= MAX_GLYPH_RUN_FONTS) {
      return reject('fontInstanceLimitExceeded', { exactFaceInstantiated: true });
    }
    return {
      replayable: true,
      face,
      blob,
      report: {
        ...report,
        digestMatched: true,
        exactFaceInstantiated: true,
        faceIndexSupported: true,
        variationSupported: true,
        effectSupported: true,
      },
    };
  }

  font(run: LayerGlyphRunOp, fontResources: LayerFontResources | undefined): Font | null {
    const status = this.replayStatus(run, fontResources);
    if (!status.replayable) return null;
    const typeface = this.typefaceFor(status.face, status.blob);
    if (!typeface) return null;
    const key = this.fontCacheKey(run, status.face, status.blob);
    const cached = this.glyphRunFonts.get(key);
    if (cached) return cached;
    if (this.glyphRunFonts.size >= MAX_GLYPH_RUN_FONTS) return null;
    const instance = run.shapeKey.fontInstance;
    const font = new this.canvasKit.Font(typeface, instance.sizePx);
    font.setSubpixel(true);
    font.setEmbolden(instance.syntheticBold === true);
    font.setSkewX(instance.syntheticItalic === true ? -0.25 : 0);
    this.glyphRunFonts.set(key, font);
    return font;
  }

  diagnostics(): { blobs: number; typefaces: number; fonts: number; bytes: number } {
    return {
      blobs: this.verifiedFontBlobs.size,
      typefaces: this.glyphRunTypefaces.size,
      fonts: this.glyphRunFonts.size,
      bytes: this.registeredBlobBytes,
    };
  }

  clear(): void {
    for (const font of this.glyphRunFonts.values()) font.delete();
    for (const typeface of this.glyphRunTypefaces.values()) typeface.delete();
    this.glyphRunFonts.clear();
    this.glyphRunTypefaces.clear();
    this.verifiedFontBlobs.clear();
    this.registeredBlobBytes = 0;
    this.documentGeneration = null;
  }

  private typefaceFor(face: LayerFontFaceResource, blob: LayerFontBlobResource): Typeface | null {
    const cacheKey = this.typefaceCacheKey(face, blob);
    const cached = this.glyphRunTypefaces.get(cacheKey);
    if (cached) return cached;
    if (this.glyphRunTypefaces.size >= MAX_GLYPH_RUN_TYPEFACES) return null;
    const blobBytes = this.verifiedFontBlobs.get(this.fontBlobCacheKey(blob));
    if (!blobBytes) return null;
    const faceBytes = canvasKitFontFaceData(blobBytes, face.faceIndex);
    if (!faceBytes) return null;
    let typeface: Typeface | null = null;
    try {
      typeface = this.canvasKit.Typeface.MakeTypefaceFromData(faceBytes.slice(0));
    } catch {
      typeface = null;
    }
    if (!typeface) {
      try {
        typeface = this.canvasKit.Typeface.MakeFreeTypeFaceFromData(faceBytes.slice(0));
      } catch {
        typeface = null;
      }
    }
    if (typeface) this.glyphRunTypefaces.set(cacheKey, typeface);
    return typeface;
  }

  private baseReport(run: LayerGlyphRunOp): CanvasKitGlyphRunReplayReport {
    return {
      replayEligibility: run?.diagnostics?.replayEligibility ?? 'notReplayable',
      quality: run?.diagnostics?.quality ?? 'omitted',
    };
  }

  private fontBlobCacheKey(blob: LayerFontBlobResource): string {
    return `${blob.dataRef?.id ?? blob.id}:${blob.digest?.value ?? 'no-digest'}`;
  }

  private typefaceCacheKey(face: LayerFontFaceResource, blob: LayerFontBlobResource): string {
    return `${this.fontBlobCacheKey(blob)}:face=${face.faceIndex}`;
  }

  private fontCacheKey(
    run: LayerGlyphRunOp,
    face: LayerFontFaceResource,
    blob: LayerFontBlobResource,
  ): string {
    const instance = run.shapeKey.fontInstance;
    return [
      this.typefaceCacheKey(face, blob),
      String(instance.sizePx),
      instance.syntheticBold ? 'bold' : 'regular',
      instance.syntheticItalic ? 'italic' : 'upright',
    ].join('|');
  }
}

export function drawCanvasKitGlyphRun(
  canvas: Canvas,
  run: LayerGlyphRunOp,
  font: Font,
  paint: Paint,
): boolean {
  if (!canvasKitCanvasSupportsGlyphRunReplay(canvas)) return false;
  const glyphs = Uint16Array.from(run.glyphIds);
  const positions = new Float32Array(run.positions.length * 2);
  for (const [index, point] of run.positions.entries()) {
    positions[index * 2] = point.x;
    positions[index * 2 + 1] = point.y;
  }
  const transform = run.placement.runToPage;
  canvas.save();
  try {
    canvas.concat([
      transform.a,
      transform.c,
      transform.e,
      transform.b,
      transform.d,
      transform.f,
      0,
      0,
      1,
    ]);
    canvas.drawGlyphs(glyphs, positions, 0, run.placement.baselineY ?? 0, font, paint);
    return true;
  } catch {
    return false;
  } finally {
    canvas.restore();
  }
}

export function canvasKitCanvasSupportsGlyphRunReplay(canvas: unknown): canvas is Canvas {
  return typeof (canvas as { drawGlyphs?: unknown } | null | undefined)?.drawGlyphs === 'function';
}

function isBoundedVerticalHwp5CanvasKitCandidate(run: LayerGlyphRunOp): boolean {
  const instance = run.shapeKey.fontInstance;
  const requires = run.variant.requires;
  return run.diagnostics.reason === BOUNDED_VERTICAL_HWP5_TABLE_CELL_REASON
    && run.variant.variantId === BOUNDED_VERTICAL_GLYPH_VARIANT_ID
    && run.variant.variantKind === 'glyphRun'
    && run.variant.partIndex === 0
    && run.variant.partCount === 1
    && run.variant.isDefaultFallback === false
    && run.variant.quality === 'exact'
    && Array.isArray(requires)
    && requires.length === 3
    && requires[0] === 'fontResources'
    && requires[1] === 'text.glyphRun'
    && requires[2] === 'text.glyphRun.verticalUpright'
    && run.writingMode === 'vertical-rl'
    && run.shapeKey.writingMode === 'vertical-rl'
    && run.orientation === 'vertical-upright'
    && run.glyphTransforms === undefined
    && run.direction === 'ltr'
    && run.shapeKey.direction === 'ltr'
    && run.bidiLevel === 0
    && run.shapeKey.shapingEngine === 'rustybuzz-q4-vertical-v1'
    && run.shapeKey.fallbackPolicy === 'none'
    && (instance.variations === undefined || instance.variations.length === 0)
    && instance.syntheticBold === false
    && instance.syntheticItalic === false
    && run.diagnostics.quality === 'exact'
    && run.diagnostics.replayEligibility === 'portable'
    && run.diagnostics.strictVisualEligible === true
    && run.diagnostics.clusterMismatchCount === 0
    && run.diagnostics.missingGlyphCount === 0
    && run.diagnostics.usedFallbackFontCount === 0;
}

function fontBlobPayloadBytes(payload: Uint8Array | number[] | string | undefined): Uint8Array | null {
  if (payload instanceof Uint8Array) {
    return payload.byteLength > 0 && payload.byteLength <= MAX_FONT_BLOB_BYTES ? payload : null;
  }
  if (Array.isArray(payload)) {
    if (
      payload.length === 0
      || payload.length > MAX_FONT_BLOB_BYTES
      || payload.some(value => !Number.isInteger(value) || value < 0 || value > 255)
    ) return null;
    return Uint8Array.from(payload);
  }
  if (
    typeof payload !== 'string'
    || payload.length === 0
    || payload.length > MAX_ENCODED_FONT_BLOB_LENGTH
    || payload.length % 4 !== 0
    || !BASE64_PAYLOAD.test(payload)
  ) return null;
  try {
    const binary = globalThis.atob(payload);
    if (binary.length === 0 || binary.length > MAX_FONT_BLOB_BYTES) return null;
    const bytes = new Uint8Array(binary.length);
    for (let index = 0; index < binary.length; index += 1) bytes[index] = binary.charCodeAt(index);
    return bytes;
  } catch {
    return null;
  }
}

function fontResourceKeyMatches(resourceKey: string, digest: string, bytes: Uint8Array): boolean {
  const match = FONT_RESOURCE_KEY.exec(resourceKey);
  if (!match || match[2] !== digest) return false;
  const byteLength = Number(match[1]);
  return Number.isSafeInteger(byteLength)
    && byteLength === bytes.byteLength
    && bytesToHex(blake3(bytes)) === digest;
}

function glyphRunPaintIsSupported(run: LayerGlyphRunOp): boolean {
  const style = run.paintStyle;
  if (
    (style.color !== undefined && typeof style.color !== 'string')
    || (style.shadeColor !== undefined && typeof style.shadeColor !== 'string')
    || (style.ratio !== undefined && (!Number.isFinite(style.ratio) || style.ratio <= 0))
  ) return false;
  const ratio = typeof style.ratio === 'number' && style.ratio > 0 ? style.ratio : 1;
  const shadeColor = (style.shadeColor ?? '#ffffff').toLowerCase();
  return Math.abs(ratio - 1) <= 0.001
    && (style.underline ?? 'none') === 'none'
    && style.strikethrough !== true
    && (style.outlineType ?? 0) === 0
    && (style.shadowType ?? 0) === 0
    && style.emboss !== true
    && style.engrave !== true
    && style.superscript !== true
    && style.subscript !== true
    && (style.emphasisDot ?? 0) === 0
    && shadeColor === '#ffffff';
}
