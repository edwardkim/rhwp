import type { Canvas } from 'canvaskit-wasm';
import type { LayerBounds } from '../../core/types.ts';
import * as wasmExports from '@wasm/rhwp.js';

type ClipMapper = (rectangle: Float64Array, transform: Float64Array) => Float64Array | null;

/** CanvasKit의 행렬을 읽고 장치 포함 경계는 공통 Rust 결과로만 결정한다. */
export function printBitmapClip(canvas: Canvas, clip: LayerBounds): LayerBounds {
  const m = canvas.getTotalMatrix?.();
  if (!m || m[6] !== 0 || m[7] !== 0 || m[8] !== 1) return clip;
  const mapper = (wasmExports as unknown as { mapPrintBitmapClip?: ClipMapper }).mapPrintBitmapClip;
  if (!mapper) throw new Error('Print bitmap clip을 지원하는 fresh WASM이 필요합니다.');
  const mapped = mapper(new Float64Array([clip.x, clip.y, clip.width, clip.height]),
    new Float64Array([m[0], m[3], m[1], m[4], m[2], m[5]]));
  return mapped ? { x: mapped[0], y: mapped[1], width: mapped[2], height: mapped[3] } : clip;
}
