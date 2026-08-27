import test from 'node:test';
import assert from 'node:assert/strict';

import { resolveRulerScale } from '../src/view/ruler-scale.ts';

const PX_PER_MM = 96 / 25.4;

function assertNiceStep(value: number): void {
  const exponent = Math.floor(Math.log10(value));
  const normalized = value / (10 ** exponent);
  assert.ok(
    [1, 2, 5].some((candidate) => Math.abs(candidate - normalized) < 1e-9),
    `${value}mm는 1·2·5 × 10ⁿ 단계여야 한다`,
  );
}

test('대표 배율의 숫자와 세부 눈금은 최소 화면 간격을 지킨다', () => {
  for (const zoom of [0.1, 0.2, 0.25, 0.5, 1, 5]) {
    const scale = resolveRulerScale(zoom);
    assert.ok(
      scale.labelStepMm * PX_PER_MM * zoom >= 30,
      `${zoom * 100}% 숫자 간격이 30px보다 작다`,
    );
    assert.ok(
      scale.tickStepMm * PX_PER_MM * zoom >= 3.5,
      `${zoom * 100}% 세부 눈금 간격이 3.5px보다 작다`,
    );
    assert.ok(scale.labelStepMm >= 10, '숫자는 1cm보다 촘촘하게 표시하지 않는다');
    assertNiceStep(scale.labelStepMm);
    assertNiceStep(scale.tickStepMm);
  }
});

test('배율을 높이면 눈금 단위는 같거나 더 촘촘해진다', () => {
  const scales = [0.1, 0.2, 0.25, 0.5, 1, 5].map(resolveRulerScale);
  for (let i = 1; i < scales.length; i++) {
    assert.ok(scales[i].labelStepMm <= scales[i - 1].labelStepMm);
    assert.ok(scales[i].tickStepMm <= scales[i - 1].tickStepMm);
  }
});

test('잘못된 배율도 유한한 표시 단계로 정규화한다', () => {
  for (const zoom of [0, -1, Number.NaN, Number.POSITIVE_INFINITY]) {
    const scale = resolveRulerScale(zoom);
    assert.ok(Number.isFinite(scale.labelStepMm) && scale.labelStepMm > 0);
    assert.ok(Number.isFinite(scale.tickStepMm) && scale.tickStepMm > 0);
  }
});

