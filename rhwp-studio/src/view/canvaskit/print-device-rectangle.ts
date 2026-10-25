import type { Canvas } from 'canvaskit-wasm';
import type { LayerBounds } from '../../core/types.ts';
import * as wasmExports from '@wasm/rhwp.js';

type ClipMapper = (rectangle: Float64Array, transform: Float64Array) => Float64Array | null;

/** clip과 채움의 규칙을 섞지 않고 각각의 공통 Rust 결과를 소비한다. */
function mapRectangle(canvas: Canvas, clip: LayerBounds,
  name: 'mapPrintBitmapClip' | 'mapPrintRectangleFill' | 'mapPrintBitmapImage'): LayerBounds {
  const m = canvas.getTotalMatrix?.();
  if (!m || m[6] !== 0 || m[7] !== 0 || m[8] !== 1) return clip;
  const mapper = (wasmExports as unknown as Partial<Record<typeof name, ClipMapper>>)[name];
  if (!mapper) throw new Error('Print 장치 사각형을 지원하는 fresh WASM이 필요합니다.');
  const mapped = mapper(new Float64Array([clip.x, clip.y, clip.width, clip.height]),
    new Float64Array([m[0], m[3], m[1], m[4], m[2], m[5]]));
  return mapped ? { x: mapped[0], y: mapped[1], width: mapped[2], height: mapped[3] } : clip;
}

export function printRectangleClip(canvas: Canvas, clip: LayerBounds): LayerBounds {
  return mapRectangle(canvas, clip, 'mapPrintBitmapClip');
}

export function printRectangleFill(canvas: Canvas, rectangle: LayerBounds): LayerBounds {
  return mapRectangle(canvas, rectangle, 'mapPrintRectangleFill');
}

export function printBitmapImage(canvas: Canvas, rectangle: LayerBounds): LayerBounds {
  return mapRectangle(canvas, rectangle, 'mapPrintBitmapImage');
}

/** 띠와 외곽선의 결합 계산은 Native·Canvas2D와 같은 Rust 함수를 사용한다. */
export function printPicturePenWidth(canvas: Canvas, pen: readonly [number, number]): number | undefined {
  const m = canvas.getTotalMatrix?.();
  if (!m || m[6] !== 0 || m[7] !== 0 || m[8] !== 1) return undefined;
  type PenMapper = (pen: Float64Array, transform: Float64Array) => number | undefined;
  const mapper = (wasmExports as unknown as { printPicturePenWidth?: PenMapper }).printPicturePenWidth;
  if (!mapper) throw new Error('Print 그림 외곽 pen을 지원하는 fresh WASM이 필요합니다.');
  return mapper(new Float64Array(pen), new Float64Array([m[0], m[3], m[1], m[4]]));
}
