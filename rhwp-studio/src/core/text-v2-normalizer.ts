import type { LayerNode, LayerPaintOp, LayerTextVariantMeta, PageLayerTree } from './types.ts';

const TEXT_KINDS = new Set(['textRun', 'glyphRun', 'glyphOutline']);
const SUPPORTED_FEATURES = new Set([
  'text.variants', 'text.paintOrderSlot', 'text.strictVisualFallbackFree',
  'fontResources', 'text.glyphRun', 'text.glyphRun.verticalUpright',
  'text.glyphOutline', 'text.outlineGlyph', 'text.glyphOutline.monochromeFill',
  'text.glyphOutline.monochromeFillStroke', 'text.glyphOutline.colorLayers',
  'text.glyphOutline.colorLayers.colrV0', 'text.glyphOutline.colorLayers.colrV1',
  'text.glyphOutline.colorLayers.colrV1.sourceOver', 'text.glyphOutline.colorLayers.colrV1.clip',
  'text.glyphOutline.bitmapGlyph', 'text.glyphOutline.svgGlyph', 'text.glyphOutline.svgGlyph.fontEmBox',
]);

function object(value: unknown): Record<string, any> {
  if (!value || typeof value !== 'object' || Array.isArray(value)) throw new Error('text v2 object required');
  return value as Record<string, any>;
}
function identifier(value: unknown): value is string {
  return typeof value === 'string' && value.length > 0 && value.length <= 1024;
}
function features(value: unknown): string[] {
  if (!Array.isArray(value) || value.length > 128 || !value.every(identifier)
    || new Set(value).size !== value.length) throw new Error('invalid text v2 feature list');
  return value;
}

/** Validate bounded v2 structure, then expand each exclusive set at its original leaf paint slot. */
export function normalizeTextV2Tree(input: unknown): PageLayerTree {
  const tree = object(input);
  const major = tree.schema?.major ?? tree.schemaVersion ?? 1;
  if (tree.schemaVersion !== undefined && tree.schemaVersion !== major) throw new Error('conflicting layer schema versions');
  if (major === 1) return input as PageLayerTree;
  if (major !== 2) throw new Error(`unsupported layer schema major ${String(major)}`);
  const required = features(tree.requiredFeatures);
  if (!required.includes('text.variants') || !required.includes('text.paintOrderSlot')
    || required.some(feature => !SUPPORTED_FEATURES.has(feature))) throw new Error('unsupported required text v2 feature');
  const strict = tree.textV2?.profile === 'strictVisual'
    && tree.textV2?.strictVisualFallbackFree === true
    && required.includes('text.strictVisualFallbackFree');
  if (!strict && (tree.textV2?.fallbackPolicy === 'none'
    || required.includes('text.strictVisualFallbackFree'))) throw new Error('incomplete strict text v2 declaration');
  const slots = new Set<string>();
  const payloadIds = new Set<string>();
  const activeNodes = new Set<object>();
  let remaining = 50_000;
  const consume = (count = 1) => {
    remaining -= count;
    if (remaining < 0) throw new Error('text v2 work budget exceeded');
  };
  const expand = (raw: unknown): LayerPaintOp[] => {
    const op = object(raw);
    consume();
    if (op.type !== 'text') {
      if (TEXT_KINDS.has(op.type)) throw new Error('schema v2 text must use an exclusive envelope');
      return [op as LayerPaintOp];
    }
    if (!identifier(op.id) || !identifier(op.paintOrderSlotId) || slots.has(op.paintOrderSlotId)
      || op.selectionPolicy !== 'exclusiveVariantSet' || !identifier(op.defaultVariantId)
      || (strict ? op.fallbackPolicy !== 'none' : op.fallbackPolicy !== 'required')
      || !Array.isArray(op.variants) || op.variants.length === 0 || op.variants.length > 64) {
      throw new Error('invalid text v2 paint slot or selection policy');
    }
    const bounds = object(op.bbox);
    if (![bounds.x, bounds.y, bounds.width, bounds.height].every(Number.isFinite)
      || bounds.width < 0 || bounds.height < 0) throw new Error('invalid text v2 bounds');
    slots.add(op.paintOrderSlotId);
    const variantIds = new Set<string>();
    const expanded: LayerPaintOp[] = [];
    let defaultPresent = false;
    for (const rawVariant of op.variants) {
      const variant = object(rawVariant);
      if (!identifier(variant.variantId) || variantIds.has(variant.variantId)
        || !identifier(variant.kind) || !Array.isArray(variant.parts)
        || variant.parts.length === 0 || variant.parts.length > 4096) throw new Error('invalid text v2 variant');
      variantIds.add(variant.variantId);
      const requirements = features(variant.requiredFeatures);
      const supported = TEXT_KINDS.has(variant.kind) && requirements.every(feature => SUPPORTED_FEATURES.has(feature));
      if (strict && variant.kind === 'textRun') throw new Error('strict text v2 cannot contain a TextRun fallback');
      if (variant.variantId === op.defaultVariantId) {
        if (!supported || (!strict && variant.kind !== 'textRun')) throw new Error('invalid text v2 default variant');
        defaultPresent = true;
      }
      const indices = new Set<number>();
      const paintOrder = new Set<number>();
      const parts: Array<{ order: number; payload: LayerPaintOp }> = [];
      for (const rawPart of variant.parts) {
        consume();
        const part = object(rawPart);
        const payload = object(part.payload);
        const order = part.localPaintOrder ?? part.partIndex;
        if (!Number.isSafeInteger(part.partIndex) || part.partIndex < 0 || part.partIndex >= variant.parts.length
          || part.partCount !== variant.parts.length || indices.has(part.partIndex)
          || !Number.isSafeInteger(order) || order < 0 || order > 4096 || paintOrder.has(order)
          || payload.type !== variant.kind) throw new Error('invalid or incomplete text v2 parts');
        indices.add(part.partIndex); paintOrder.add(order);
        const id = payload.id ?? `${op.paintOrderSlotId}:${variant.variantId}:${part.partIndex}`;
        if (!identifier(id) || payloadIds.has(id)) throw new Error('duplicate text v2 payload identity');
        payloadIds.add(id);
        if (!supported) continue; // Future optional variants cannot displace a known compatibility fallback.
        const meta: LayerTextVariantMeta = {
          equivalenceGroup: op.paintOrderSlotId, variantId: variant.variantId, variantKind: variant.kind,
          partIndex: part.partIndex, partCount: part.partCount, localPaintOrder: order,
          isDefaultFallback: !strict && variant.variantId === op.defaultVariantId && variant.kind === 'textRun',
          requires: requirements, quality: variant.quality, anchorOpId: op.paintOrderSlotId,
        };
        parts.push({ order, payload: { ...payload, id, variant: meta } as unknown as LayerPaintOp });
      }
      parts.sort((a, b) => a.order - b.order);
      expanded.push(...parts.map(part => part.payload));
    }
    if (!defaultPresent) throw new Error('missing text v2 default variant');
    return expanded;
  };
  const visit = (raw: unknown, depth: number): LayerNode => {
    consume();
    const node = object(raw);
    if (depth > 64 || activeNodes.has(node)) throw new Error('cyclic or too-deep text v2 layer tree');
    activeNodes.add(node);
    try {
      if (node.kind === 'group') {
        if (!Array.isArray(node.children) || node.children.length > remaining) throw new Error('invalid text v2 group');
        return { ...node, children: node.children.map((child: unknown) => visit(child, depth + 1)) } as LayerNode;
      }
      if (node.kind === 'clipRect') return { ...node, child: visit(node.child, depth + 1) } as LayerNode;
      if (node.kind !== 'leaf' || !Array.isArray(node.ops) || node.ops.length > remaining) throw new Error('invalid text v2 leaf');
      return { ...node, ops: node.ops.flatMap(expand) } as LayerNode;
    } finally { activeNodes.delete(node); }
  };
  return { ...tree, schemaVersion: 1, schemaMinorVersion: 0, schema: { major: 1, minor: 0 },
    normalizedFromSchema: { major: 2, minor: tree.schema?.minor ?? tree.schemaMinorVersion ?? 0 },
    root: visit(tree.root, 0) } as PageLayerTree;
}
