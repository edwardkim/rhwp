export function inspectCanvasKitRuntimeImageFailures(runtime) {
  const imageFailures = runtime !== null
    && typeof runtime === 'object'
    && Array.isArray(runtime.imageFailures)
    ? runtime.imageFailures
    : null;
  return {
    available: imageFailures !== null,
    failures: imageFailures ?? [],
    hasFailures: (imageFailures?.length ?? 0) > 0,
  };
}

// A missing font is a product fallback contract, not a CanvasKit replay success.
// Require both the actual selection failure and an independently resolved font plan.
export function inspectCanvasKitFontFallback(sample, backend, diagnostics) {
  if (sample.canvaskitReadinessGate === true || !backend.startsWith('canvaskit-')) return null;
  const runtime = diagnostics?.runtime;
  const selection = runtime?.selection;
  const plan = diagnostics?.fontPlan;
  const reason = 'canvaskitResourcePreparationFailed';
  const prefix = 'CanvasKit font family가 준비되지 않았습니다: ';
  if (runtime?.activeBackend !== 'canvas2d'
    || runtime.backendFallbackReason !== reason
    || runtime.request?.backend?.backend !== 'canvaskit'
    || selection?.requestedBackend !== 'canvaskit'
    || selection.effectiveBackend !== 'canvas2d'
    || selection.selectionReason !== reason || selection.fallbackReason !== reason
    || selection.initializationError
    || diagnostics.replayPlanError || diagnostics.canvaskitRender?.lastRenderError
    || (diagnostics.canvaskitRender?.lastUnexpectedUnsupportedOps ?? []).length > 0
    || (diagnostics.canvaskitRender?.imageFailures ?? []).length > 0
    || typeof selection.selectionError !== 'string'
    || !selection.selectionError.startsWith(prefix)
    || plan?.complete !== true || !Array.isArray(plan.unavailableFonts)
    || !Array.isArray(plan.requiredFontFamilies)) return null;
  const missingFonts = selection.selectionError.slice(prefix.length).split(', ');
  if (!missingFonts.length || missingFonts.some(font => !font
    || !plan.unavailableFonts.includes(font) || !plan.requiredFontFamilies.includes(font))) return null;
  return { reason, missingFonts, actualBackend: 'canvas2d' };
}
