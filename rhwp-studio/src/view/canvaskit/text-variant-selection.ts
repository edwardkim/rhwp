import type { LayerGlyphOutlineOp, LayerGlyphRunOp, LayerPaintOp } from '@/core/types';
import type { StaticSvgPathLayer } from '../static-svg-path-layers';
import type { LayerTextVariantGroupReport, LayerTextVariantReplayStatus } from './replay-diagnostics.ts';

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
  canReplayGlyphOutline: (op: LayerGlyphOutlineOp) => boolean | LayerTextVariantReplayStatus,
  canReplayGlyphRun: (op: LayerGlyphRunOp) => boolean | LayerTextVariantReplayStatus = () => false,
  onReport?: (report: LayerTextVariantGroupReport) => void,
): Set<LayerPaintOp> {
  const MAX_VARIANT_PARTS = 4096;
  const selected = new Set<LayerPaintOp>();
  const groups = new Map<string, LayerPaintOp[]>();
  for (const op of ops) {
    const group = 'variant' in op ? op.variant?.equivalenceGroup : undefined;
    if (!group) continue;
    const variants = groups.get(group) ?? [];
    variants.push(op);
    groups.set(group, variants);
  }
  for (const [group, variants] of groups) {
    const statuses = new Map<LayerPaintOp, LayerTextVariantReplayStatus>();
    const replayable = (op: LayerPaintOp) => {
      const result = op.type === 'glyphOutline' ? canReplayGlyphOutline(op)
        : op.type === 'glyphRun' ? canReplayGlyphRun(op) : false;
      const status = typeof result === 'boolean' ? { replayable: result } : result;
      statuses.set(op, status);
      return status.replayable;
    };
    const candidates = new Map<string, {
      kind: string;
      expectedPartCount: number;
      parts: Set<number>;
      ops: LayerPaintOp[];
      structurallyValid: boolean;
      duplicatePart: boolean;
      fallback: boolean;
    }>();
    for (const op of variants) {
      if (!('variant' in op) || !op.variant?.variantId) continue;
      const partIndex = op.variant.partIndex ?? 0;
      const partCount = op.variant.partCount ?? 1;
      const declaredKind = op.variant.variantKind ?? op.type;
      const candidate = candidates.get(op.variant.variantId) ?? {
        kind: op.type,
        expectedPartCount: partCount,
        parts: new Set<number>(),
        ops: [],
        structurallyValid: true,
        duplicatePart: false,
        fallback: op.variant.isDefaultFallback === true || op.type === 'textRun',
      };
      candidate.structurallyValid &&= candidate.kind === op.type
        && declaredKind === op.type
        && candidate.expectedPartCount === partCount
        && Number.isInteger(partCount)
        && partCount > 0
        && partCount <= MAX_VARIANT_PARTS
        && Number.isInteger(partIndex)
        && partIndex >= 0
        && partIndex < partCount;
      candidate.duplicatePart ||= candidate.parts.has(partIndex);
      candidate.parts.add(partIndex);
      candidate.ops.push(op);
      candidate.fallback ||= op.variant.isDefaultFallback === true || op.type === 'textRun';
      candidates.set(op.variant.variantId, candidate);
    }
    const complete = [...candidates.values()].filter(candidate => (
      candidate.structurallyValid
      && !candidate.duplicatePart
      && candidate.parts.size === candidate.expectedPartCount
      && Array.from({ length: candidate.expectedPartCount }, (_, index) => index)
        .every(index => candidate.parts.has(index))
    ));
    let chosen: (typeof complete)[number] | undefined;
    for (const kind of ['glyphOutline', 'glyphRun'] as const) {
      chosen = complete.find(candidate => (
        candidate.kind === kind
        && !candidate.fallback
        && candidate.ops.every(replayable)
      ));
      if (chosen) break;
    }
    chosen ??= complete.find(candidate => candidate.kind === 'textRun' && candidate.fallback);
    for (const op of chosen?.ops ?? []) selected.add(op);
    if (onReport) {
      const selectedMeta = chosen?.ops[0] && 'variant' in chosen.ops[0] ? chosen.ops[0].variant : undefined;
      const parts: LayerTextVariantGroupReport['parts'] = [];
      const rejectedVariants: LayerTextVariantGroupReport['rejectedVariants'] = [];
      for (const [id, candidate] of candidates) {
        const reasons = new Set<string>();
        if (!candidate.structurallyValid) reasons.add('variantPartCountMismatch');
        if (candidate.duplicatePart) reasons.add('variantDuplicatePart');
        if (candidate.parts.size !== candidate.expectedPartCount) reasons.add('variantPartsIncomplete');
        for (const op of candidate.ops) {
          const status = statuses.get(op) ?? (candidate.kind === 'textRun' && complete.includes(candidate)
            ? { replayable: true } : { replayable: false, reason: 'variantNotEvaluated' });
          if (status.reason) reasons.add(status.reason);
          const meta = 'variant' in op ? op.variant : undefined;
          if (parts.length < MAX_VARIANT_PARTS) parts.push({ ...status, equivalenceGroup: group.slice(0, 1024),
            variantId: id.slice(0, 1024), variantKind: candidate.kind,
            partIndex: meta?.partIndex ?? 0, partCount: meta?.partCount ?? 1 });
        }
        if (candidate !== chosen && rejectedVariants.length < 64) rejectedVariants.push({
          variantId: id.slice(0, 1024), variantKind: candidate.kind,
          reasons: reasons.size ? [...reasons] : [candidate.kind === 'textRun' ? 'defaultFallbackNotSelected' : 'higherPriorityVariantSelected'],
        });
      }
      const selectedPart = parts.find(part => part.variantId === selectedMeta?.variantId);
      onReport({ backend: 'canvaskit', equivalenceGroup: group.slice(0, 1024),
        selectedVariantId: selectedMeta?.variantId?.slice(0, 1024) ?? '', selectedVariantKind: chosen?.kind ?? '',
        selectedReason: chosen?.kind === 'glyphOutline' ? 'glyphOutlineStrictProfile'
          : chosen?.kind === 'glyphRun' ? 'glyphRunStrictEligible' : chosen ? 'defaultTextRunFallback' : 'noSupportedVariant',
        anchorOpId: selectedMeta?.anchorOpId?.slice(0, 1024), partsExpected: chosen?.expectedPartCount ?? 0,
        partsSelected: chosen?.ops.length ?? 0, parts, rejectedVariants,
        fontVerification: selectedPart?.fontVerification, outlineEligibility: selectedPart?.outlineEligibility,
        truncated: parts.length < variants.length || rejectedVariants.length < candidates.size - (chosen ? 1 : 0) });
    }
  }
  return selected;
}
