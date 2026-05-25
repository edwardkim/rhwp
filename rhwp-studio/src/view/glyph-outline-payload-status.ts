import type { LayerGlyphOutlineOp } from '@/core/types';

export type GlyphOutlinePayloadRejectReason =
  | 'unsupportedOutlinePayload'
  | 'glyphOutlineStrokeStyleUnsupported'
  | 'unsupportedColorGlyph'
  | 'unsupportedBitmapGlyph'
  | 'unsupportedSvgGlyph';

export interface GlyphOutlinePayloadStatus {
  payloadKind: string;
  supported: boolean;
  reason?: GlyphOutlinePayloadRejectReason;
  detail?: string;
}

export interface GlyphOutlinePayloadStatusOptions {
  allowMonochromeFillStroke?: boolean;
  allowColrv0ColorLayers?: boolean;
  allowColrv1Stage1ColorGraph?: boolean;
  allowBitmapGlyph?: boolean;
  allowSvgGlyph?: boolean;
}

const COLRV1_STAGE1_NODE_KINDS = new Set(['solidPath', 'transform']);

export function glyphOutlinePayloadStatus(
  op: LayerGlyphOutlineOp,
  options: GlyphOutlinePayloadStatusOptions = {},
): GlyphOutlinePayloadStatus {
  const payloadKind = op.payloadKind ?? 'monochromeFill';
  if (!hasExclusivePayloadFamily(op, payloadKind)) {
    return { payloadKind, supported: false, reason: 'unsupportedOutlinePayload', detail: 'mixedPayloadFamily' };
  }
  switch (payloadKind) {
    case 'monochromeFill':
      return { payloadKind, supported: Array.isArray(op.paths) && op.paths.length > 0, reason: op.paths?.length ? undefined : 'unsupportedOutlinePayload' };
    case 'monochromeFillStroke':
      if (!options.allowMonochromeFillStroke) {
        return { payloadKind, supported: false, reason: 'glyphOutlineStrokeStyleUnsupported', detail: 'gateClosed' };
      }
      return isStrictStroke(op.stroke)
        ? { payloadKind, supported: true }
        : { payloadKind, supported: false, reason: 'glyphOutlineStrokeStyleUnsupported' };
    case 'colorLayers':
      return colorLayersStatus(op, options);
    case 'bitmapGlyph':
      return options.allowBitmapGlyph && hasBitmapGlyphContract(op)
        ? { payloadKind, supported: true }
        : { payloadKind, supported: false, reason: 'unsupportedBitmapGlyph' };
    case 'svgGlyph':
      return options.allowSvgGlyph && hasSvgGlyphContract(op)
        ? { payloadKind, supported: true }
        : { payloadKind, supported: false, reason: 'unsupportedSvgGlyph' };
    default:
      return { payloadKind, supported: false, reason: 'unsupportedOutlinePayload', detail: 'unknownPayloadKind' };
  }
}

function colorLayersStatus(
  op: LayerGlyphOutlineOp,
  options: GlyphOutlinePayloadStatusOptions,
): GlyphOutlinePayloadStatus {
  const payloadKind = 'colorLayers';
  const colorLayers = op.colorLayers;
  if (!colorLayers) {
    return { payloadKind, supported: false, reason: 'unsupportedColorGlyph', detail: 'missingColorLayers' };
  }
  if (colorLayers.colorFormat === 'colrV0') {
    return options.allowColrv0ColorLayers && Array.isArray(colorLayers.layers) && colorLayers.layers.length > 0
      ? { payloadKind, supported: true }
      : { payloadKind, supported: false, reason: 'unsupportedColorGlyph', detail: 'colrV0GateClosed' };
  }
  if (colorLayers.colorFormat === 'colrV1') {
    const nodes = colorLayers.paintGraph?.nodes ?? [];
    const unsupported = nodes.find((node) => !COLRV1_STAGE1_NODE_KINDS.has(node.kind ?? ''));
    if (unsupported) {
      return {
        payloadKind,
        supported: false,
        reason: 'unsupportedColorGlyph',
        detail: `colrV1Node:${unsupported.kind ?? 'unknown'}`,
      };
    }
    return options.allowColrv1Stage1ColorGraph && nodes.length > 0
      ? { payloadKind, supported: true }
      : { payloadKind, supported: false, reason: 'unsupportedColorGlyph', detail: 'colrV1GateClosed' };
  }
  return { payloadKind, supported: false, reason: 'unsupportedColorGlyph', detail: `format:${colorLayers.colorFormat ?? 'missing'}` };
}

function hasExclusivePayloadFamily(op: LayerGlyphOutlineOp, payloadKind: string): boolean {
  const families = [
    op.stroke !== undefined,
    op.colorLayers !== undefined,
    op.bitmapGlyph !== undefined,
    op.svgGlyph !== undefined,
  ].filter(Boolean).length;
  if (payloadKind === 'monochromeFill') return families === 0;
  if (payloadKind === 'monochromeFillStroke') return op.stroke !== undefined && families === 1;
  if (payloadKind === 'colorLayers') return op.colorLayers !== undefined && families === 1;
  if (payloadKind === 'bitmapGlyph') return op.bitmapGlyph !== undefined && families === 1;
  if (payloadKind === 'svgGlyph') return op.svgGlyph !== undefined && families === 1;
  return families === 0;
}

function isStrictStroke(stroke: LayerGlyphOutlineOp['stroke']): boolean {
  return !!stroke
    && Number.isFinite(stroke.width)
    && (stroke.width ?? 0) > 0
    && stroke.join === 'miter'
    && stroke.cap === 'butt'
    && Number.isFinite(stroke.miterLimit)
    && (stroke.miterLimit ?? 0) >= 1
    && (stroke.paintOrder === 'fillThenStroke' || stroke.paintOrder === 'strokeThenFill');
}

function hasBitmapGlyphContract(op: LayerGlyphOutlineOp): boolean {
  const glyph = op.bitmapGlyph;
  return !!glyph
    && typeof glyph.imageRef === 'number'
    && glyph.scalingPolicy !== 'backendDefault'
    && glyph.placement !== undefined
    && isPositiveBounds(glyph.placement);
}

function hasSvgGlyphContract(op: LayerGlyphOutlineOp): boolean {
  const glyph = op.svgGlyph;
  return !!glyph
    && typeof glyph.svgRef === 'number'
    && glyph.staticSanitized === true
    && glyph.scriptAllowed !== true
    && glyph.animationAllowed !== true
    && glyph.externalResourcesAllowed !== true
    && glyph.interactivityAllowed !== true
    && glyph.viewBox !== undefined
    && isPositiveBounds(glyph.viewBox);
}

function isPositiveBounds(bounds: { width?: number; height?: number }): boolean {
  return Number.isFinite(bounds.width)
    && Number.isFinite(bounds.height)
    && (bounds.width ?? 0) > 0
    && (bounds.height ?? 0) > 0;
}
