import {
  normalizePageArrangement,
  type PageArrangement,
} from './page-arrangement.ts';

export type PageMovementDirection = 'vertical' | 'horizontal';

export interface PageMovementSettings {
  direction: PageMovementDirection;
  /** 가로 방향에서 세로 휠 입력을 좌우 이동으로 바꿀지 여부 */
  wheelHorizontal: boolean;
}

export const DEFAULT_PAGE_MOVEMENT: PageMovementSettings = {
  direction: 'vertical',
  wheelHorizontal: true,
};

export function normalizePageMovementSettings(value: unknown): PageMovementSettings {
  if (!value || typeof value !== 'object') return { ...DEFAULT_PAGE_MOVEMENT };
  const candidate = value as { direction?: unknown; wheelHorizontal?: unknown };
  return {
    direction: candidate.direction === 'horizontal' ? 'horizontal' : 'vertical',
    wheelHorizontal: typeof candidate.wheelHorizontal === 'boolean'
      ? candidate.wheelHorizontal
      : DEFAULT_PAGE_MOVEMENT.wheelHorizontal,
  };
}

/** 한컴은 가로 쪽 이동을 한 쪽 배치에서만 허용한다. */
export function resolvePageViewSettings(
  arrangementValue: unknown,
  movementValue: unknown,
): { arrangement: PageArrangement; movement: PageMovementSettings } {
  const movement = normalizePageMovementSettings(movementValue);
  return {
    arrangement: movement.direction === 'horizontal'
      ? { kind: 'single' }
      : normalizePageArrangement(arrangementValue),
    movement,
  };
}
