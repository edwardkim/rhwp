import type { LayerGlyphOutlineOp, LayerPaintOp } from '@/core/types';
import type { StaticSvgPathLayer } from '../static-svg-path-layers';

export function staticSvgPathLayersAreReplayable(
  layers: readonly StaticSvgPathLayer[],
  makePath: (pathData: string) => { delete?: () => void } | null,
): boolean {
  if (layers.length === 0) return false;
  for (const layer of layers) {
    let path: { delete?: () => void } | null = null;
    try {
      path = makePath(layer.pathData);
    } catch {
      return false;
    }
    if (!path) return false;
    path.delete?.();
  }
  return true;
}

export function selectLayerTextVariantsForLeaf(
  ops: readonly LayerPaintOp[],
  canReplayGlyphOutline: (op: LayerGlyphOutlineOp) => boolean,
): Set<LayerPaintOp> {
  const selected = new Set<LayerPaintOp>();
  const groups = new Map<string, LayerPaintOp[]>();
  for (const op of ops) {
    const group = 'variant' in op ? op.variant?.equivalenceGroup : undefined;
    if (!group) continue;
    const variants = groups.get(group) ?? [];
    variants.push(op);
    groups.set(group, variants);
  }
  for (const variants of groups.values()) {
    const outline = variants.find((op): op is LayerGlyphOutlineOp => (
      op.type === 'glyphOutline' && canReplayGlyphOutline(op)
    ));
    if (outline) {
      selected.add(outline);
      continue;
    }
    const fallback = variants.find((op) => (
      ('variant' in op && op.variant?.isDefaultFallback === true) || op.type === 'textRun'
    ));
    if (fallback) selected.add(fallback);
  }
  return selected;
}
