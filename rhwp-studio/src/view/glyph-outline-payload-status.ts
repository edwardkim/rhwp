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

// Exact largest finite IEEE-754 binary32 value, shared with native Skia.
const MAX_GLYPH_SCALAR = 3.4028234663852886e38;

function isPortableGlyphScalar(value: unknown): value is number {
  return typeof value === 'number' && Number.isFinite(value) && Math.abs(value) <= MAX_GLYPH_SCALAR;
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
  if ((op.placement?.baselineY !== undefined && !isPortableGlyphScalar(op.placement.baselineY))
    || !isOptionalFiniteAffine(op.placement?.runToPage)
    || ((payloadKind === 'monochromeFill' || payloadKind === 'monochromeFillStroke')
      && (!Array.isArray(op.paths) || op.paths.length === 0
        || !op.paths.every((path) => isValidPathCommands(path?.commands))))) {
    return { payloadKind, supported: false, reason: 'unsupportedOutlinePayload' };
  }
  switch (payloadKind) {
    case 'monochromeFill':
      return { payloadKind, supported: true };
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

export function glyphOutlinePayloadResourceKey(op: LayerGlyphOutlineOp): string | null {
  const payloadKind = op.payloadKind ?? 'monochromeFill';
  if (!hasExclusivePayloadFamily(op, payloadKind)) {
    return null;
  }
  switch (payloadKind) {
    case 'colorLayers':
      return hasColorLayersResourceKeyContract(op) && op.colorLayers
        ? colorLayersResourceKey(op.colorLayers)
        : null;
    case 'bitmapGlyph':
      return hasBitmapGlyphContract(op) && op.bitmapGlyph
        ? bitmapGlyphResourceKey(op.bitmapGlyph)
        : null;
    case 'svgGlyph':
      return hasSvgGlyphContract(op) && op.svgGlyph
        ? svgGlyphResourceKey(op.svgGlyph)
        : null;
    default:
      return null;
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
      && colorLayers.layers.every((layer) => isValidPathCommands(layer?.commands) && isOptionalFiniteAffine(layer.transformToRun))
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
          && isPortableGlyphScalar(node.linearGradientPath.gradient?.x0)
          && isPortableGlyphScalar(node.linearGradientPath.gradient?.y0)
          && isPortableGlyphScalar(node.linearGradientPath.gradient?.x1)
          && isPortableGlyphScalar(node.linearGradientPath.gradient?.y1)
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
          && isPortableGlyphScalar(node.radialGradientPath.gradient?.cx)
          && isPortableGlyphScalar(node.radialGradientPath.gradient?.cy)
          && isPortableGlyphScalar(node.radialGradientPath.gradient?.radius)
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
          && isPortableGlyphScalar(node.sweepGradientPath.gradient?.cx)
          && isPortableGlyphScalar(node.sweepGradientPath.gradient?.cy)
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

function hasColorLayersResourceKeyContract(op: LayerGlyphOutlineOp): boolean {
  const colorLayers = op.colorLayers;
  if (!colorLayers) {
    return false;
  }
  if (colorLayers.colorFormat === 'colrV0') {
    return hasColrv0ResolvedLayerContract(colorLayers);
  }
  if (colorLayers.colorFormat === 'colrV1') {
    return hasSupportedColrv1GraphContract(op);
  }
  return false;
}

function hasColrv0ResolvedLayerContract(colorLayers: NonNullable<LayerGlyphOutlineOp['colorLayers']>): boolean {
  const layers = colorLayers.layers ?? [];
  return colorLayers.colorFormat === 'colrV0'
    && colorLayers.paintGraph === undefined
    && layers.length > 0
    && layers.every((layer) => (
      isValidPathCommands(layer?.commands)
      && isValidResolvedColor(layer.fill)
      && isSupportedFillRule(layer.fillRule)
      && isValidTextRange(layer.sourceRangeUtf8)
      && isNonEmptyTextRange(layer.glyphRange)
      && isOptionalFiniteAffine(layer.transformToRun)
      && (layer.opacity === undefined || Number.isFinite(layer.opacity))
    ));
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
  return transform !== undefined && transform !== null
    && isPortableGlyphScalar(transform.a)
    && isPortableGlyphScalar(transform.b)
    && isPortableGlyphScalar(transform.c)
    && isPortableGlyphScalar(transform.d)
    && isPortableGlyphScalar(transform.e)
    && isPortableGlyphScalar(transform.f);
}

function isOptionalFiniteAffine(transform: { a?: number; b?: number; c?: number; d?: number; e?: number; f?: number } | undefined): boolean {
  return transform === undefined || isFiniteAffine(transform);
}

function isValidPathCommands(commands: unknown[] | undefined): boolean {
  return Array.isArray(commands) && commands.length > 0 && commands.every((value) => {
    if (!value || typeof value !== 'object') return false;
    const command = value as Record<string, unknown>;
    switch (command.type) {
      case 'moveTo':
      case 'lineTo':
        return [command.x, command.y].every(isPortableGlyphScalar);
      case 'curveTo':
        return [command.x1, command.y1, command.x2, command.y2, command.x3, command.y3].every(isPortableGlyphScalar);
      case 'arcTo':
        return [command.rx, command.ry, command.rotation, command.x, command.y].every(isPortableGlyphScalar);
      case 'closePath':
        return true;
      default:
        return false;
    }
  });
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
  return isPortableGlyphScalar(startAngleDegrees)
    && isPortableGlyphScalar(endAngleDegrees)
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
    && isPortableGlyphScalar(stroke.width)
    && (stroke.width ?? 0) > 0
    && stroke.join === 'miter'
    && stroke.cap === 'butt'
    && isPortableGlyphScalar(stroke.miterLimit)
    && (stroke.miterLimit ?? 0) >= 1
    && (stroke.paintOrder === 'fillThenStroke' || stroke.paintOrder === 'strokeThenFill');
}

function hasBitmapGlyphContract(op: LayerGlyphOutlineOp): boolean {
  const glyph = op.bitmapGlyph;
  return !!glyph
    && typeof glyph.imageRef === 'number'
    && glyph.scalingPolicy !== 'backendDefault'
    && glyph.placement !== undefined
    && isPositiveBounds(glyph.placement)
    && isOptionalFiniteAffine(glyph.transformToRun);
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
    && isPositiveBounds(glyph.viewBox)
    && (glyph.intrinsicSize === undefined || (glyph.intrinsicSize !== null
      && isPortableGlyphScalar(glyph.intrinsicSize.width) && glyph.intrinsicSize.width > 0
      && isPortableGlyphScalar(glyph.intrinsicSize.height) && glyph.intrinsicSize.height > 0
    ))
    && isOptionalFiniteAffine(glyph.transformToRun);
}

function colorLayersResourceKey(colorLayers: NonNullable<LayerGlyphOutlineOp['colorLayers']>): string {
  const graph = colorLayers.paintGraph;
  const graphKey = graph
    ? `graph:root:${graph.rootNodeId ?? '-'}:nodes:${(graph.nodes ?? []).map((node) => [
      node.nodeId ?? '-',
      node.kind ?? '-',
      textRangeKey(node.glyphRange),
      fontColorGlyphRefKey(node.sourceFontRef),
    ].join(':')).join('|')}`
    : `layers:${(colorLayers.layers ?? []).map((layer) => [
      layer.layerIndex ?? '-',
      layer.glyphId ?? '-',
      textRangeKey(layer.glyphRange),
      textRangeKey(layer.sourceRangeUtf8),
      fontColorGlyphRefKey(layer.sourceFontRef),
      layer.paletteIndex ?? '-',
    ].join(':')).join('|')}`;
  return [
    'glyphPayload:colorLayers',
    `format:${colorLayers.colorFormat ?? '-'}`,
    `source:${fontColorGlyphRefKey(colorLayers.sourceFontRef)}`,
    `palette:${paletteRefKey(colorLayers.paletteRef)}`,
    `range:${textRangeKey(colorLayers.sourceRangeUtf8)}`,
    `glyphRange:${textRangeKey(colorLayers.glyphRange)}`,
    graphKey,
  ].join(':');
}

function bitmapGlyphResourceKey(glyph: NonNullable<LayerGlyphOutlineOp['bitmapGlyph']>): string {
  return [
    'glyphPayload:bitmapGlyph',
    `imageRef:${glyph.imageRef ?? '-'}`,
    `range:${textRangeKey(glyph.sourceRangeUtf8)}`,
    `glyphRange:${textRangeKey(glyph.glyphRange)}`,
    `placement:${boundsKey(glyph.placement)}`,
    `alphaPremultiplied:${glyph.alphaPremultiplied === true}`,
    `scaling:${glyph.scalingPolicy ?? '-'}`,
    `filtering:${glyph.filtering ?? '-'}`,
    `transform:${affineKey(glyph.transformToRun)}`,
  ].join(':');
}

function svgGlyphResourceKey(glyph: NonNullable<LayerGlyphOutlineOp['svgGlyph']>): string {
  return [
    'glyphPayload:svgGlyph',
    `svgRef:${glyph.svgRef ?? '-'}`,
    `range:${textRangeKey(glyph.sourceRangeUtf8)}`,
    `glyphRange:${textRangeKey(glyph.glyphRange)}`,
    `viewBox:${boundsKey(glyph.viewBox)}`,
    `intrinsicSize:${glyph.intrinsicSize ? `${fixed(glyph.intrinsicSize.width)},${fixed(glyph.intrinsicSize.height)}` : '-'}`,
    `staticSanitized:${glyph.staticSanitized === true}`,
    `script:${glyph.scriptAllowed === true}`,
    `animation:${glyph.animationAllowed === true}`,
    `external:${glyph.externalResourcesAllowed === true}`,
    `interactive:${glyph.interactivityAllowed === true}`,
    `transform:${affineKey(glyph.transformToRun)}`,
  ].join(':');
}

function fontColorGlyphRefKey(ref: { faceKey?: string; glyphId?: number; paletteIndex?: number; colorFormat?: string } | undefined): string {
  if (!ref) return '-';
  return [
    `face:${ref.faceKey ?? '-'}`,
    `glyph:${ref.glyphId ?? '-'}`,
    `palette:${ref.paletteIndex ?? '-'}`,
    `format:${ref.colorFormat ?? '-'}`,
  ].join(':');
}

function paletteRefKey(ref: { id?: string; index?: number; cpalDigest?: string } | undefined): string {
  if (!ref) return '-';
  return [
    `id:${ref.id ?? '-'}`,
    `index:${ref.index ?? '-'}`,
    `digest:${ref.cpalDigest ?? '-'}`,
  ].join(':');
}

function textRangeKey(range: { start?: number; end?: number } | undefined): string {
  return range ? `${range.start ?? '-'}..${range.end ?? '-'}` : '-';
}

function boundsKey(bounds: { x?: number; y?: number; width?: number; height?: number } | undefined): string {
  return bounds ? [bounds.x, bounds.y, bounds.width, bounds.height].map(fixedBounds).join(',') : '-';
}

function affineKey(transform: { a?: number; b?: number; c?: number; d?: number; e?: number; f?: number } | undefined): string {
  return transform ? [transform.a, transform.b, transform.c, transform.d, transform.e, transform.f].map(fixed).join(',') : '-';
}

function fixed(value: number | undefined): string {
  return Number.isFinite(value) ? (value ?? 0).toFixed(6) : '-';
}

function fixedBounds(value: number | undefined): string {
  return Number.isFinite(value) ? (value ?? 0).toFixed(3) : '-';
}

function isPositiveBounds(bounds: { x?: number; y?: number; width?: number; height?: number }): boolean {
  return bounds !== null && isPortableGlyphScalar(bounds.x)
    && isPortableGlyphScalar(bounds.y)
    && isPortableGlyphScalar(bounds.width)
    && isPortableGlyphScalar(bounds.height)
    && bounds.width > 0
    && bounds.height > 0
    && isPortableGlyphScalar(bounds.x + bounds.width)
    && isPortableGlyphScalar(bounds.y + bounds.height);
}
