/** DEV opt-in only. These wrappers observe existing calls; never warm a product cache. */
export function budgetProbeBoundaries(
  enabled: boolean, view: object, renderer: object, wasm: object, lru: object,
): [object, string, string][] {
  if (!enabled) return [];
  return [
    [renderer, 'getCanvasSurfaceLayerCount', 'budget.layerCount'],
    [renderer, 'getLayerPlaneSummaryFromOverlayImages', 'budget.overlaySummary'],
    [renderer, 'getLayerPlaneSummaryFromTree', 'budget.treeSummary'],
    [wasm, 'getPageOverlayImages', 'wasm.overlayImages'],
    [wasm, 'getPageLayerTree', 'wasm.layerTree'],
    [view, 'pageSurfaceDescriptor', 'budget.descriptor'],
    [view, 'reconcilePageSurfaceBudget', 'budget.reconcile'],
    [lru, 'reconcile', 'cache.reconcile'],
  ];
}

const PAGE_BOUNDARIES = new Set(['budget.layerCount', 'budget.overlaySummary', 'budget.treeSummary',
  'budget.descriptor', 'wasm.overlayImages', 'wasm.layerTree']);

export function budgetProbePage(name: string, args: unknown[]): number | null {
  // reconcile's budget/phase arguments are NOT page indices.
  return PAGE_BOUNDARIES.has(name) && typeof args[0] === 'number' ? args[0] : null;
}
