const PX_PER_MM = 96 / 25.4;
const MIN_LABEL_SPACING_PX = 30;
const MIN_TICK_SPACING_PX = 3.5;
const MIN_LABEL_STEP_MM = 10;

export interface RulerScale {
  labelStepMm: number;
  tickStepMm: number;
}

/** 주어진 값 이상인 가장 가까운 `1·2·5 × 10ⁿ` 단계를 고른다. */
export function niceStepCeil(value: number): number {
  const safeValue = Number.isFinite(value) && value > 0 ? value : 1;
  const exponent = Math.floor(Math.log10(safeValue));
  const power = 10 ** exponent;
  const normalized = safeValue / power;

  if (normalized <= 1) return power;
  if (normalized <= 2) return 2 * power;
  if (normalized <= 5) return 5 * power;
  return 10 * power;
}

/** 가로·세로 눈금자가 공유하는 화면 밀도 기반 표시 단계. */
export function resolveRulerScale(zoom: number): RulerScale {
  const safeZoom = Number.isFinite(zoom) && zoom > 0 ? zoom : 1;
  const pixelsPerMm = PX_PER_MM * safeZoom;
  return {
    labelStepMm: niceStepCeil(Math.max(
      MIN_LABEL_STEP_MM,
      MIN_LABEL_SPACING_PX / pixelsPerMm,
    )),
    tickStepMm: niceStepCeil(MIN_TICK_SPACING_PX / pixelsPerMm),
  };
}

