import type { LayerPaintOp } from '@/core/types';

export type CanvasKitReplayPlane = 'background' | 'behindText' | 'flow' | 'inFrontOfText';

export const CANVASKIT_REPLAY_PLANES = [
  'background',
  'behindText',
  'flow',
  'inFrontOfText',
] as const satisfies readonly CanvasKitReplayPlane[];

export function layerPaintOpReplayPlane(op: LayerPaintOp): CanvasKitReplayPlane {
  if (op.type === 'pageBackground') {
    return 'background';
  }
  if (op.type === 'image') {
    if (op.wrap === 'behindText') {
      return 'behindText';
    }
    if (op.wrap === 'inFrontOfText') {
      return 'inFrontOfText';
    }
  }
  return 'flow';
}
