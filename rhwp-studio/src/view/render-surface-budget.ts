import type { LayerRenderProfile } from '@/core/types';

export const DEFAULT_CANVAS2D_LAYER_COUNT = 4;
export const DEFAULT_VISIBLE_SURFACE_PIXEL_BUDGET = 32_000_000;
/** visible에 prefetch용 25% headroom을 더한 약 153MiB RGBA surface 한도다. */
export const DEFAULT_RETAINED_SURFACE_PIXEL_BUDGET = 40_000_000;
/** scroll 정착 뒤 visible 화질을 올릴 때 허용하는 별도 mandatory surface hard gate다. */
export const DEFAULT_SETTLED_VISIBLE_SURFACE_PIXEL_LIMIT = 64_000_000;
export const SURFACE_BUDGET_RELEASE_RATIO = 0.88;

const DPR_EPSILON = 0.001;
const DEMOTION_STEPS = [2, 1.5, 1] as const;

export type RenderSurfaceTier = 'screen' | 'balanced' | 'economy' | 'export';

export interface RenderSurfacePageInput {
  pageIndex: number;
  width: number;
  height: number;
  /** 페이지별 Canvas surface 수. 없으면 planner의 보수적인 fallback을 사용한다. */
  layerCount?: number;
  visible: boolean;
  focused: boolean;
  distanceFromFocus: number;
  /** interaction 동안 이미 완성된 surface의 DPR을 그대로 쓰는 실행 제약이다. */
  lockedEffectiveDpr?: number;
}

export interface RenderSurfaceBudgetInput {
  pages: readonly RenderSurfacePageInput[];
  zoom: number;
  rawDpr: number;
  layerCount: number;
  renderProfile?: LayerRenderProfile | string;
  visiblePixelBudget?: number;
  retainedPixelBudget?: number;
  previousEffectiveDpr?: ReadonlyMap<number, number>;
}

export interface RenderSurfaceDecision {
  pageIndex: number;
  layerCount: number;
  visible: boolean;
  focused: boolean;
  effectiveDpr: number;
  tier: RenderSurfaceTier;
  surfacePixels: number;
  surfaceBytes: number;
}

export interface RenderSurfaceBudgetPlan {
  decisions: RenderSurfaceDecision[];
  visibleSurfacePixels: number;
  retainedSurfacePixels: number;
  fullQualityVisibleSurfacePixels: number;
  fullQualityRetainedSurfacePixels: number;
  visiblePixelBudget: number;
  retainedPixelBudget: number;
  withinBudget: boolean;
  heldByHysteresis: boolean;
}

interface MutablePageState extends RenderSurfacePageInput {
  layerCount: number;
  steps: number[];
  stepIndex: number;
  interactionLocked: boolean;
}

function positive(value: number, fallback: number): number {
  return Number.isFinite(value) && value > 0 ? value : fallback;
}

function isExportProfile(profile: LayerRenderProfile | string): boolean {
  return profile === 'print' || profile === 'highQuality';
}

/** raw DPR을 보존하고, 필요할 때만 그 아래의 안정된 단계로 내려간다. */
export function renderDprSteps(rawDprValue: number): number[] {
  const rawDpr = positive(rawDprValue, 1);
  const steps = [rawDpr];
  for (const step of DEMOTION_STEPS) {
    if (step < rawDpr - DPR_EPSILON && !steps.some(value => Math.abs(value - step) < DPR_EPSILON)) {
      steps.push(step);
    }
  }
  return steps;
}

export function pageSurfacePixels(
  page: Pick<RenderSurfacePageInput, 'width' | 'height' | 'layerCount'>,
  zoomValue: number,
  dprValue: number,
  layerCountValue: number,
): number {
  const zoom = positive(zoomValue, 1);
  const dpr = positive(dprValue, 1);
  const width = positive(page.width, 1);
  const height = positive(page.height, 1);
  const layerCount = Math.max(
    1,
    Math.round(positive(page.layerCount ?? layerCountValue, positive(layerCountValue, 1))),
  );
  return width * height * zoom * zoom * dpr * dpr * layerCount;
}

function totals(states: readonly MutablePageState[], zoom: number, layerCount: number): {
  visible: number;
  retained: number;
} {
  let visible = 0;
  let retained = 0;
  for (const state of states) {
    const pixels = pageSurfacePixels(
      state,
      zoom,
      state.steps[state.stepIndex] ?? state.steps[0],
      layerCount,
    );
    retained += pixels;
    if (state.visible) visible += pixels;
  }
  return { visible, retained };
}

function previousStepIndex(state: MutablePageState, previous: number | undefined): number {
  if (!Number.isFinite(previous)) return 0;
  const value = previous as number;
  const index = state.steps.findIndex(step => step <= value + DPR_EPSILON);
  return index >= 0 ? index : state.steps.length - 1;
}

function canDemote(state: MutablePageState): boolean {
  return !state.focused
    && !state.interactionLocked
    && state.stepIndex + 1 < state.steps.length;
}

function demotionOrder(
  a: MutablePageState,
  b: MutablePageState,
  preferOffscreen: boolean,
): number {
  if (preferOffscreen && a.visible !== b.visible) return a.visible ? 1 : -1;
  // 같은 우선순위에서는 모든 쪽을 한 단계씩 낮춘 뒤 두 번째 단계를 사용한다.
  if (a.stepIndex !== b.stepIndex) return a.stepIndex - b.stepIndex;
  if (a.distanceFromFocus !== b.distanceFromFocus) {
    return b.distanceFromFocus - a.distanceFromFocus;
  }
  return a.pageIndex - b.pageIndex;
}

function demoteUntil(
  states: MutablePageState[],
  zoom: number,
  layerCount: number,
  kind: 'visible' | 'retained',
  budget: number,
): void {
  const maxIterations = states.reduce((sum, state) => sum + state.steps.length, 0);
  for (let iteration = 0; iteration < maxIterations; iteration++) {
    const current = totals(states, zoom, layerCount);
    if (current[kind] <= budget) return;

    const candidates = states
      .filter(state => canDemote(state) && (kind === 'retained' || state.visible))
      .sort((a, b) => demotionOrder(a, b, kind === 'retained'));
    const candidate = candidates[0];
    if (!candidate) return;
    candidate.stepIndex += 1;
  }
}

function tierFor(effectiveDpr: number, rawDpr: number, exportProfile: boolean): RenderSurfaceTier {
  if (exportProfile) return 'export';
  if (effectiveDpr >= rawDpr - DPR_EPSILON) return 'screen';
  if (effectiveDpr > 1 + DPR_EPSILON) return 'balanced';
  return 'economy';
}

/**
 * 열 수와 무관하게 전체 surface 비용을 먼저 계산한 뒤, 초과분만 비포커스 쪽에서 줄인다.
 * visible 예산은 visible 쪽만, retained 예산은 offscreen prefetch를 먼저 조정한다.
 */
export function planRenderSurfaceBudget(input: RenderSurfaceBudgetInput): RenderSurfaceBudgetPlan {
  const zoom = positive(input.zoom, 1);
  const rawDpr = positive(input.rawDpr, 1);
  const layerCount = Math.max(1, Math.round(positive(input.layerCount, 1)));
  const visiblePixelBudget = positive(
    input.visiblePixelBudget ?? DEFAULT_VISIBLE_SURFACE_PIXEL_BUDGET,
    DEFAULT_VISIBLE_SURFACE_PIXEL_BUDGET,
  );
  const retainedPixelBudget = positive(
    input.retainedPixelBudget ?? DEFAULT_RETAINED_SURFACE_PIXEL_BUDGET,
    DEFAULT_RETAINED_SURFACE_PIXEL_BUDGET,
  );
  const exportProfile = isExportProfile(input.renderProfile ?? 'screen');
  const states: MutablePageState[] = input.pages.map(page => ({
    ...page,
    width: positive(page.width, 1),
    height: positive(page.height, 1),
    layerCount: Math.max(1, Math.round(positive(page.layerCount ?? layerCount, layerCount))),
    distanceFromFocus: Math.max(0, positive(page.distanceFromFocus, 0)),
    steps: renderDprSteps(rawDpr),
    stepIndex: 0,
    // 편집 focus의 raw DPR 보호가 interaction lock보다 강하다.
    interactionLocked: !page.focused && Number.isFinite(page.lockedEffectiveDpr),
  }));

  const fullQuality = totals(states, zoom, layerCount);
  const releaseVisible = visiblePixelBudget * SURFACE_BUDGET_RELEASE_RATIO;
  const releaseRetained = retainedPixelBudget * SURFACE_BUDGET_RELEASE_RATIO;
  const mayPromoteAll = fullQuality.visible <= releaseVisible
    && fullQuality.retained <= releaseRetained;
  let heldByHysteresis = false;

  if (!exportProfile) {
    for (const state of states) {
      if (!state.interactionLocked) continue;
      state.stepIndex = previousStepIndex(state, state.lockedEffectiveDpr);
    }
  }

  if (!exportProfile && !mayPromoteAll && input.previousEffectiveDpr) {
    for (const state of states) {
      if (state.focused || state.interactionLocked) continue;
      state.stepIndex = previousStepIndex(
        state,
        input.previousEffectiveDpr.get(state.pageIndex),
      );
      if (state.stepIndex > 0) heldByHysteresis = true;
    }
  }

  if (!exportProfile) {
    demoteUntil(states, zoom, layerCount, 'visible', visiblePixelBudget);
    demoteUntil(states, zoom, layerCount, 'retained', retainedPixelBudget);
  }

  const finalTotals = totals(states, zoom, layerCount);
  const decisions = states.map(state => {
    const effectiveDpr = state.steps[state.stepIndex] ?? rawDpr;
    const surfacePixels = pageSurfacePixels(state, zoom, effectiveDpr, layerCount);
    return {
      pageIndex: state.pageIndex,
      layerCount: state.layerCount,
      visible: state.visible,
      focused: state.focused,
      effectiveDpr,
      tier: tierFor(effectiveDpr, rawDpr, exportProfile),
      surfacePixels,
      surfaceBytes: surfacePixels * 4,
    } satisfies RenderSurfaceDecision;
  });

  return {
    decisions,
    visibleSurfacePixels: finalTotals.visible,
    retainedSurfacePixels: finalTotals.retained,
    fullQualityVisibleSurfacePixels: fullQuality.visible,
    fullQualityRetainedSurfacePixels: fullQuality.retained,
    visiblePixelBudget,
    retainedPixelBudget,
    withinBudget:
      finalTotals.visible <= visiblePixelBudget
      && finalTotals.retained <= retainedPixelBudget,
    heldByHysteresis,
  };
}

export interface SettledVisibleEffectiveDprInput {
  pages: readonly Pick<
    RenderSurfacePageInput,
    'width' | 'height' | 'layerCount' | 'focused'
  >[];
  zoom: number;
  rawDpr: number;
  layerCount: number;
  absolutePixelLimit?: number;
}

/**
 * scroll 정착 visible 집합을 raw DPR로 복원할 수 있는지 한 번 계산한다.
 * raw가 hard gate를 넘으면 1.5만 한 번 시도하고, 그것도 넘으면 기존 planner에 맡긴다.
 */
export function resolveSettledVisibleEffectiveDpr(
  input: SettledVisibleEffectiveDprInput,
): number | null {
  if (input.pages.length === 0) return null;
  const rawDpr = positive(input.rawDpr, 1);
  const fallbackDpr = Math.min(rawDpr, 1.5);
  const candidates = fallbackDpr < rawDpr - DPR_EPSILON
    ? [rawDpr, fallbackDpr]
    : [rawDpr];
  const absolutePixelLimit = positive(
    input.absolutePixelLimit ?? DEFAULT_SETTLED_VISIBLE_SURFACE_PIXEL_LIMIT,
    DEFAULT_SETTLED_VISIBLE_SURFACE_PIXEL_LIMIT,
  );
  for (const candidate of candidates) {
    const pixels = input.pages.reduce((sum, page) => (
      sum + pageSurfacePixels(
        page,
        input.zoom,
        page.focused ? rawDpr : candidate,
        input.layerCount,
      )
    ), 0);
    if (pixels <= absolutePixelLimit) return candidate;
  }
  return null;
}
