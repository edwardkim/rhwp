import type { LayerTextStyle } from '../../core/types.ts';

/** 원본 종류가 우선이며 연속 표본은 공통 producer가 확정한 순서를 소비한다. */
export function textShadowOffsets(style: LayerTextStyle): readonly [number, number][] {
  if ((style.shadowType ?? 0) === 0) return [];
  const offsets: [number, number][] = style.shadowType === 2 && style.shadowOffsets !== undefined
    ? style.shadowOffsets
    : [[style.shadowOffsetX ?? 0, style.shadowOffsetY ?? 0]];
  if (!Array.isArray(offsets) || offsets.length === 0 || !offsets.every(offset => Array.isArray(offset)
    && offset.length === 2 && offset.every(Number.isFinite))) {
    throw new Error('Invalid resolved text shadow offsets');
  }
  return offsets;
}
