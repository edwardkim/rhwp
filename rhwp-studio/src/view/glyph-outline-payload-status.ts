import type { LayerGlyphOutlineOp } from '@/core/types';

type LayerColorPaintGraphNode = NonNullable<
  NonNullable<NonNullable<LayerGlyphOutlineOp['colorLayers']>['paintGraph']>['nodes']
>[number];

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

const COLRV1_SUPPORTED_NODE_KINDS = new Set([
  'solidPath',
  'linearGradientPath',
  'radialGradientPath',
  'sweepGradientPath',
  'transform',
]);

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
    const unsupported = nodes.find((node) => !COLRV1_SUPPORTED_NODE_KINDS.has(node.kind ?? ''));
    if (unsupported) {
      return {
        payloadKind,
        supported: false,
        reason: 'unsupportedColorGlyph',
        detail: `colrV1Node:${unsupported.kind ?? 'unknown'}`,
      };
    }
    if (!hasSupportedColrv1GraphContract(op)) {
      return { payloadKind, supported: false, reason: 'unsupportedColorGlyph', detail: 'colrV1InvalidGraph' };
    }
    return options.allowColrv1Stage1ColorGraph
      ? { payloadKind, supported: true }
      : { payloadKind, supported: false, reason: 'unsupportedColorGlyph', detail: 'colrV1GateClosed' };
  }
  return { payloadKind, supported: false, reason: 'unsupportedColorGlyph', detail: `format:${colorLayers.colorFormat ?? 'missing'}` };
}

function hasSupportedColrv1GraphContract(op: LayerGlyphOutlineOp): boolean {
  const colorLayers = op.colorLayers;
  const graph = colorLayers?.paintGraph;
  const nodes = graph?.nodes ?? [];
  const topLevelGlyphRange = colorLayers?.glyphRange;
  if (
    colorLayers?.colorFormat !== 'colrV1'
    || !graph
    || nodes.length === 0
    || nodes.length > 64
    || colorLayers.sourceFontRef === undefined
    || !isValidTextRange(colorLayers.sourceRangeUtf8)
    || !isNonEmptyTextRange(topLevelGlyphRange)
  ) {
    return false;
  }
  const nodesById = new Map<number, NonNullable<typeof nodes[number]>>();
  for (const node of nodes) {
    if (!Number.isInteger(node.nodeId) || node.nodeId === undefined || nodesById.has(node.nodeId)) {
      return false;
    }
    nodesById.set(node.nodeId, node);
  }
  let nodeId = graph.rootNodeId;
  const visited = new Set<number>();
  for (let depth = 0; depth < 64; depth += 1) {
    if (!Number.isInteger(nodeId) || nodeId === undefined || visited.has(nodeId)) {
      return false;
    }
    visited.add(nodeId);
    const node = nodesById.get(nodeId);
    if (!node) {
      return false;
    }
    switch (node.kind) {
      case 'solidPath':
        return visited.size === nodes.length
          && node.solidPath !== undefined
          && node.transform === undefined
          && node.linearGradientPath === undefined
          && node.radialGradientPath === undefined
          && node.sweepGradientPath === undefined
          && isLeafMetadataValid(node)
          && isValidPathCommands(node.solidPath.commands)
          && isValidResolvedColor(node.solidPath.fill)
          && isSupportedFillRule(node.solidPath.fillRule);
      case 'linearGradientPath':
        return visited.size === nodes.length
          && node.solidPath === undefined
          && node.transform === undefined
          && node.radialGradientPath === undefined
          && node.sweepGradientPath === undefined
          && node.linearGradientPath !== undefined
          && isLeafMetadataValid(node)
          && isValidPathCommands(node.linearGradientPath.commands)
          && Number.isFinite(node.linearGradientPath.gradient?.x0)
          && Number.isFinite(node.linearGradientPath.gradient?.y0)
          && Number.isFinite(node.linearGradientPath.gradient?.x1)
          && Number.isFinite(node.linearGradientPath.gradient?.y1)
          && isValidColorGradientStops(node.linearGradientPath.gradient?.stops)
          && isSupportedFillRule(node.linearGradientPath.fillRule);
      case 'radialGradientPath':
        return visited.size === nodes.length
          && node.solidPath === undefined
          && node.transform === undefined
          && node.linearGradientPath === undefined
          && node.sweepGradientPath === undefined
          && node.radialGradientPath !== undefined
          && isLeafMetadataValid(node)
          && isValidPathCommands(node.radialGradientPath.commands)
          && Number.isFinite(node.radialGradientPath.gradient?.cx)
          && Number.isFinite(node.radialGradientPath.gradient?.cy)
          && Number.isFinite(node.radialGradientPath.gradient?.radius)
          && (node.radialGradientPath.gradient?.radius ?? 0) > 0
          && isValidColorGradientStops(node.radialGradientPath.gradient?.stops)
          && isSupportedFillRule(node.radialGradientPath.fillRule);
      case 'sweepGradientPath':
        return visited.size === nodes.length
          && node.solidPath === undefined
          && node.transform === undefined
          && node.linearGradientPath === undefined
          && node.radialGradientPath === undefined
          && node.sweepGradientPath !== undefined
          && isLeafMetadataValid(node)
          && isValidPathCommands(node.sweepGradientPath.commands)
          && Number.isFinite(node.sweepGradientPath.gradient?.cx)
          && Number.isFinite(node.sweepGradientPath.gradient?.cy)
          && isSupportedFullCircleSweepGradient(
            node.sweepGradientPath.gradient?.startAngleDegrees,
            node.sweepGradientPath.gradient?.endAngleDegrees,
          )
          && isValidColorGradientStops(node.sweepGradientPath.gradient?.stops)
          && isSupportedFillRule(node.sweepGradientPath.fillRule);
      case 'transform':
        if (
          node.solidPath !== undefined
          || node.linearGradientPath !== undefined
          || node.radialGradientPath !== undefined
          || node.sweepGradientPath !== undefined
          || node.transform === undefined
          || !Number.isInteger(node.transform.childNodeId)
          || !isFiniteAffine(node.transform.transform)
        ) {
          return false;
        }
        nodeId = node.transform.childNodeId;
        continue;
      default:
        return false;
    }
  }
  return false;
}

function isLeafMetadataValid(node: LayerColorPaintGraphNode): boolean {
  return isValidTextRange(node.sourceRangeUtf8)
    && isNonEmptyTextRange(node.glyphRange)
    && node.sourceFontRef !== undefined;
}

function isValidTextRange(range: { start?: number; end?: number } | undefined): boolean {
  return range !== undefined
    && Number.isInteger(range.start)
    && Number.isInteger(range.end)
    && (range.end ?? -1) >= (range.start ?? 0);
}

function isNonEmptyTextRange(range: { start?: number; end?: number } | undefined): boolean {
  return isValidTextRange(range) && (range?.end ?? 0) > (range?.start ?? 0);
}

function isFiniteAffine(transform: { a?: number; b?: number; c?: number; d?: number; e?: number; f?: number } | undefined): boolean {
  return transform !== undefined
    && Number.isFinite(transform.a)
    && Number.isFinite(transform.b)
    && Number.isFinite(transform.c)
    && Number.isFinite(transform.d)
    && Number.isFinite(transform.e)
    && Number.isFinite(transform.f);
}

function isValidPathCommands(commands: unknown[] | undefined): boolean {
  return Array.isArray(commands) && commands.length > 0;
}

function isValidResolvedColor(color: { rgba?: number[] } | undefined): boolean {
  return Array.isArray(color?.rgba)
    && color.rgba.length === 4
    && color.rgba.every((component) => Number.isFinite(component) && component >= 0 && component <= 1);
}

function isValidColorGradientStops(stops: Array<{ offset?: number; color?: { rgba?: number[] } }> | undefined): boolean {
  if (!Array.isArray(stops) || stops.length < 2) {
    return false;
  }
  let previousOffset = Number.NEGATIVE_INFINITY;
  for (const stop of stops) {
    if (
      !Number.isFinite(stop.offset)
      || (stop.offset ?? -1) < 0
      || (stop.offset ?? 2) > 1
      || (stop.offset ?? -1) < previousOffset
      || !isValidResolvedColor(stop.color)
    ) {
      return false;
    }
    previousOffset = stop.offset ?? previousOffset;
  }
  return true;
}

function isSupportedFullCircleSweepGradient(
  startAngleDegrees: number | undefined,
  endAngleDegrees: number | undefined,
): boolean {
  return Number.isFinite(startAngleDegrees)
    && Number.isFinite(endAngleDegrees)
    && (startAngleDegrees ?? 0) < (endAngleDegrees ?? 0)
    && Math.abs((endAngleDegrees ?? 0) - (startAngleDegrees ?? 0) - 360) <= 1e-9;
}

function isSupportedFillRule(fillRule: string | undefined): boolean {
  return fillRule === 'nonzero' || fillRule === 'evenodd';
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
