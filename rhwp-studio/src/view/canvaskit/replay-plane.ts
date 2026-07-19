import type { LayerInfo, LayerPaintOp } from '@/core/types';

export type CanvasKitReplayPlane = 'background' | 'behindText' | 'flow' | 'inFrontOfText';

export const CANVASKIT_REPLAY_PLANES = [
  'background',
  'behindText',
  'flow',
  'inFrontOfText',
] as const satisfies readonly CanvasKitReplayPlane[];

/**
 * 바탕쪽 유래 op 의 replay plane 상한 (#2318).
 *
 * 한컴 의미론: 바탕쪽 개체의 textWrap 은 바탕쪽 내부 개체 간 순서에만 적용되고,
 * 바탕쪽 전체는 항상 본문 뒤에 깔린다. rust 분류기(cap_master_page_plane)와
 * 동일 계약 — layer JSON 의 masterPage provenance 를 소비한다.
 */
function capMasterPagePlane(
  plane: CanvasKitReplayPlane,
  layer?: LayerInfo | null,
): CanvasKitReplayPlane {
  if (plane !== 'background' && layer?.masterPage === true) {
    return 'behindText';
  }
  return plane;
}

export function renderLayerReplayPlane(layer?: LayerInfo | null): CanvasKitReplayPlane {
  let plane: CanvasKitReplayPlane = 'flow';
  if (layer?.textWrap === 'behindText') {
    plane = 'behindText';
  } else if (layer?.textWrap === 'inFrontOfText') {
    plane = 'inFrontOfText';
  }
  return capMasterPagePlane(plane, layer);
}

export function layerPaintOpReplayPlane(
  op: LayerPaintOp,
  layer?: LayerInfo | null,
): CanvasKitReplayPlane {
  if (op.type === 'pageBackground') {
    return 'background';
  }
  if (layer?.textWrap) {
    return renderLayerReplayPlane(layer);
  }
  let plane: CanvasKitReplayPlane = 'flow';
  if (op.type === 'image') {
    if (op.wrap === 'behindText') {
      plane = 'behindText';
    } else if (op.wrap === 'inFrontOfText') {
      plane = 'inFrontOfText';
    }
  }
  return capMasterPagePlane(plane, layer);
}
