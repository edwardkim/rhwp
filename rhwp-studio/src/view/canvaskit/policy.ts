import type { CanvasKitRenderMode } from '@/view/render-backend';
import type { LayerClipNode, LayerRenderProfile } from '@/core/types';

export function canvaskitClipRightPad(
  renderMode: CanvasKitRenderMode,
  profile: LayerRenderProfile,
  clipKind: LayerClipNode['clipKind'],
  rightOverflowSlop?: number,
): number {
  if (typeof rightOverflowSlop === 'number' && Number.isFinite(rightOverflowSlop)) {
    return Math.max(0, rightOverflowSlop);
  }
  // P16 only mirrors the known legacy text overflow case: HWP body/table-cell clips can
  // trim the right edge of glyphs in fast preview. This is not a general clip inflation
  // policy; broader native/browser parity rules belong with the later CanvasKit coverage work.
  if (renderMode === 'compat' && profile === 'fastPreview' && (clipKind === 'body' || clipKind === 'tableCell')) {
    return 4;
  }
  return 0;
}
