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
  // P16 기본 호환: fast preview에서 본문 오른쪽 글리프 끝이 잘리는 기존 보정만 유지한다.
  if (renderMode === 'compat' && profile === 'fastPreview' && clipKind === 'body') {
    return 4;
  }
  return 0;
}
