import test from 'node:test';
import assert from 'node:assert/strict';

import { withCanvasKitSurfaceBlockers } from '../src/core/canvaskit-document-preflight.ts';
import type { CanvasKitDocumentPreflight } from '../src/core/types.ts';
import type { CanvasKitLayerRenderer } from '../src/view/canvaskit-renderer.ts';
import { RendererSession, type RendererSessionOptions } from '../src/view/renderer-session.ts';

function preflight(
  status: CanvasKitDocumentPreflight['status'],
): CanvasKitDocumentPreflight {
  const eligible = status === 'eligible';
  return {
    schemaVersion: 1,
    mode: 'default',
    profile: 'screen',
    status,
    eligible,
    complete: status !== 'incomplete',
    pageCount: 2,
    scannedPages: status === 'incomplete' ? 1 : 2,
    scannedWorkUnits: 12,
    limits: {
      maxPages: 128,
      maxWorkUnits: 50_000,
      maxBlockers: 32,
      maxRequiredFontFamilies: 256,
    },
    summary: {
      totalItems: 10,
      directItems: eligible ? 10 : 9,
      directRequiredItems: eligible ? 0 : 1,
      compatOverlayItems: 0,
      textFallbackItems: 0,
      unsupportedItems: 0,
      hiddenOverlayViolations: eligible ? 0 : 1,
    },
    blockers: eligible ? [] : [{ code: 'hiddenCanvas2dOverlayRequired', pageIndex: 1 }],
    requiredFontFamilies: ['Noto Sans KR'],
    capabilityDigest: `digest-${status}`,
  };
}

function fakeRenderer(
  onDispose: () => void = () => {},
  onReset: () => void = () => {},
  onCancelPreparation: () => void = () => {},
): CanvasKitLayerRenderer {
  return {
    resetDocumentResources: onReset,
    cancelDocumentPreparation: onCancelPreparation,
    dispose: onDispose,
  } as unknown as CanvasKitLayerRenderer;
}

function session(
  backend: 'auto' | 'canvas2d' | 'canvaskit',
  create: () => Promise<CanvasKitLayerRenderer>,
  options: RendererSessionOptions = {},
  source: 'default' | 'url' = 'url',
): RendererSession {
  return new RendererSession(
    { backend, source, ...(source === 'url' ? { requested: backend } : {}) },
    { mode: 'default', source: 'default' },
    { preference: 'auto', requested: 'auto' },
    'screen',
    create,
    options,
  );
}

test('auto selects CanvasKit only after a complete eligible document preflight', async () => {
  let preflightCalls = 0;
  let createCalls = 0;
  const renderer = fakeRenderer();
  const wasm = {
    getCanvasKitDocumentPreflight() {
      preflightCalls += 1;
      return preflight('eligible');
    },
  };
  const rendererSession = session('auto', async () => {
    createCalls += 1;
    return renderer;
  });

  rendererSession.beginDocument('document-a');
  const first = await rendererSession.resolve(wasm as never);
  const cached = await rendererSession.resolve(wasm as never);

  assert.equal(first.backend, 'canvaskit');
  assert.equal(first.canvaskitRenderer, renderer);
  assert.equal(first.diagnostics.selectionReason, 'autoEligible');
  assert.equal(first.diagnostics.documentRevision, 1);
  assert.equal(first.diagnostics.resourceGeneration, 1);
  assert.equal(cached, first);
  assert.equal(preflightCalls, 1);
  assert.equal(createCalls, 1);
});

test('auto fails closed for ineligible and incomplete document reports', async () => {
  let createCalls = 0;
  for (const expected of [
    ['ineligible', 'canvaskitDocumentIneligible', 'autoIneligible'],
    ['incomplete', 'canvaskitDocumentPreflightIncomplete', 'autoPreflightIncomplete'],
  ] as const) {
    const rendererSession = session('auto', async () => {
      createCalls += 1;
      return fakeRenderer();
    });
    rendererSession.beginDocument(`document-${expected[0]}`);
    const selected = await rendererSession.resolve({
      getCanvasKitDocumentPreflight: () => preflight(expected[0]),
    } as never);
    assert.equal(selected.backend, 'canvas2d');
    assert.equal(selected.diagnostics.fallbackReason, expected[1]);
    assert.equal(selected.diagnostics.selectionReason, expected[2]);
  }
  assert.equal(createCalls, 0);
});

test('auto rejects mismatched preflight mode and profile before CanvasKit initialization', async () => {
  let createCalls = 0;
  for (const report of [
    { ...preflight('eligible'), mode: 'compat' as const },
    { ...preflight('eligible'), profile: 'print' as const },
  ]) {
    const rendererSession = session('auto', async () => {
      createCalls += 1;
      return fakeRenderer();
    });
    rendererSession.beginDocument(`mismatch-${report.mode}-${report.profile}`);
    const selected = await rendererSession.resolve({
      getCanvasKitDocumentPreflight: () => report,
    });
    assert.equal(selected.backend, 'canvas2d');
    assert.equal(selected.diagnostics.selectionReason, 'autoPreflightIncomplete');
    assert.match(selected.diagnostics.selectionError ?? '', /요청 불일치/);
  }
  assert.equal(createCalls, 0);
});

test('surface preflight transforms stay lazy and document resources prepare before selection', async () => {
  let createCalls = 0;
  let prepareCalls = 0;
  const blocked = session('auto', async () => {
    createCalls += 1;
    return fakeRenderer();
  }, {
    transformCanvasKitPreflight: report => ({
      ...report,
      status: 'ineligible',
      eligible: false,
    }),
  });
  blocked.beginDocument('surface-blocked');
  assert.equal((await blocked.resolve({
    getCanvasKitDocumentPreflight: () => preflight('eligible'),
  })).backend, 'canvas2d');
  assert.equal(createCalls, 0);

  const prepared = session('auto', async () => {
    createCalls += 1;
    return fakeRenderer();
  }, {
    async prepareCanvasKitDocument(_renderer, report) {
      prepareCalls += 1;
      assert.deepEqual(report.requiredFontFamilies, ['Noto Sans KR']);
    },
  });
  prepared.beginDocument('surface-prepared');
  assert.equal((await prepared.resolve({
    getCanvasKitDocumentPreflight: () => preflight('eligible'),
  })).backend, 'canvaskit');
  assert.equal(createCalls, 1);
  assert.equal(prepareCalls, 1);

  const failed = session('auto', async () => fakeRenderer(), {
    async prepareCanvasKitDocument() {
      throw new Error('font decode failed');
    },
  });
  failed.beginDocument('surface-preparation-failed');
  const fallback = await failed.resolve({
    getCanvasKitDocumentPreflight: () => preflight('eligible'),
  });
  assert.equal(fallback.backend, 'canvas2d');
  assert.equal(fallback.diagnostics.selectionReason, 'canvaskitResourcePreparationFailed');
  assert.equal(fallback.diagnostics.selectionError, 'font decode failed');
});

test('auto re-evaluation permits text marks but keeps structural control markers on Canvas2D', async () => {
  let showControlCodes = false;
  let createCalls = 0;
  const rendererSession = session('auto', async () => {
    createCalls += 1;
    return fakeRenderer();
  }, {
    transformCanvasKitPreflight(report) {
      return withCanvasKitSurfaceBlockers(
        report,
        showControlCodes ? ['viewOption:showControlCodes'] : [],
      );
    },
  });
  const wasm = { getCanvasKitDocumentPreflight: () => preflight('eligible') };

  rendererSession.beginDocument('document-view-marks');
  assert.equal((await rendererSession.resolve(wasm)).backend, 'canvaskit');

  rendererSession.invalidateDocument({ resetResources: false });
  assert.equal((await rendererSession.resolve(wasm)).backend, 'canvaskit');

  showControlCodes = true;
  rendererSession.invalidateDocument({ resetResources: false });
  const controlCodes = await rendererSession.resolve(wasm);
  assert.equal(controlCodes.backend, 'canvas2d');
  assert.equal(
    controlCodes.diagnostics.preflight?.blockers.at(-1)?.detail,
    'viewOption:showControlCodes',
  );

  showControlCodes = false;
  rendererSession.invalidateDocument({ resetResources: false });
  assert.equal((await rendererSession.resolve(wasm)).backend, 'canvaskit');
  assert.equal(createCalls, 1);
});

test('fixed backends bypass auto preflight while distinguishing default and explicit diagnostics', async () => {
  let preflightCalls = 0;
  const wasm = {
    getCanvasKitDocumentPreflight() {
      preflightCalls += 1;
      return preflight('ineligible');
    },
  };

  const canvas2d = session('canvas2d', async () => fakeRenderer());
  canvas2d.beginDocument('document-canvas2d');
  const canvas2dSelection = await canvas2d.resolve(wasm as never);
  assert.equal(canvas2dSelection.backend, 'canvas2d');
  assert.equal(canvas2dSelection.diagnostics.selectionReason, 'explicitCanvas2d');

  const defaultCanvas2d = session('canvas2d', async () => fakeRenderer(), {}, 'default');
  defaultCanvas2d.beginDocument('document-default-canvas2d');
  const defaultSelection = await defaultCanvas2d.resolve(wasm as never);
  assert.equal(defaultSelection.backend, 'canvas2d');
  assert.equal(defaultSelection.diagnostics.request.source, 'default');
  assert.equal(defaultSelection.diagnostics.selectionReason, 'defaultCanvas2d');

  const canvaskit = session('canvaskit', async () => fakeRenderer());
  canvaskit.beginDocument('document-canvaskit');
  const canvaskitSelection = await canvaskit.resolve(wasm as never);
  assert.equal(canvaskitSelection.backend, 'canvaskit');
  assert.equal(canvaskitSelection.diagnostics.requestedBackend, 'canvaskit');
  assert.equal(canvaskitSelection.diagnostics.selectionReason, 'explicitCanvasKit');
  assert.equal(preflightCalls, 0);
});

test('revision invalidation prevents stale decisions from becoming current', async () => {
  let releaseInitialization: ((renderer: CanvasKitLayerRenderer) => void) | null = null;
  let prepareCalls = 0;
  let resetCalls = 0;
  let cancelPreparationCalls = 0;
  const rendererSession = session('auto', () => new Promise((resolve) => {
    releaseInitialization = resolve;
  }), {
    async prepareCanvasKitDocument() {
      prepareCalls += 1;
    },
  });
  const wasm = { getCanvasKitDocumentPreflight: () => preflight('eligible') };

  rendererSession.beginDocument('document-a');
  const stalePromise = rendererSession.resolve(wasm as never);
  rendererSession.invalidateDocument();
  releaseInitialization!(fakeRenderer(
    () => {},
    () => { resetCalls += 1; },
    () => { cancelPreparationCalls += 1; },
  ));
  const stale = await stalePromise;
  const current = await rendererSession.resolve(wasm as never);

  assert.equal(stale.backend, 'canvas2d');
  assert.equal(stale.diagnostics.selectionReason, 'superseded');
  assert.equal(stale.diagnostics.documentRevision, 1);
  assert.equal(stale.diagnostics.resourceGeneration, 1);
  assert.equal(stale.diagnostics.documentDigest, 'document-a');
  assert.equal(rendererSession.isCurrent(stale), false);
  assert.equal(rendererSession.isCurrent(current), true);
  assert.equal(current.diagnostics.documentRevision, 2);
  assert.equal(current.diagnostics.resourceGeneration, 2);
  assert.notEqual(current.diagnostics.decisionKey, stale.diagnostics.decisionKey);
  assert.equal(prepareCalls, 1);

  rendererSession.invalidateDocument();
  assert.equal(resetCalls, 1);
  rendererSession.invalidateDocument({ resetResources: false });
  assert.equal(resetCalls, 1);
  assert.equal(cancelPreparationCalls, 1);
});

test('revision invalidation during document preparation returns a superseded Canvas2D selection', async () => {
  for (const preparationResult of ['resolve', 'reject'] as const) {
    let finishPreparation: (() => void) | null = null;
    const rendererSession = session('auto', async () => fakeRenderer(), {
      prepareCanvasKitDocument: () => new Promise<void>((resolve, reject) => {
        finishPreparation = preparationResult === 'resolve'
          ? resolve
          : () => reject(new Error('stale font failure'));
      }),
    });
    const wasm = { getCanvasKitDocumentPreflight: () => preflight('eligible') };

    rendererSession.beginDocument(`document-prepare-${preparationResult}`);
    const stalePromise = rendererSession.resolve(wasm as never);
    while (!finishPreparation) await Promise.resolve();
    rendererSession.invalidateDocument();
    finishPreparation();

    const stale = await stalePromise;
    assert.equal(stale.backend, 'canvas2d');
    assert.equal(stale.diagnostics.selectionReason, 'superseded');
    assert.equal(stale.diagnostics.fallbackReason, 'canvaskitRevisionInvalidated');
    assert.equal(stale.diagnostics.selectionError, null);
    assert.equal(stale.diagnostics.documentRevision, 1);
    assert.equal(stale.diagnostics.resourceGeneration, 1);
    assert.equal(stale.diagnostics.documentDigest, `document-prepare-${preparationResult}`);
    assert.equal(rendererSession.isCurrent(stale), false);
  }
});

test('auto mutations pin Canvas2D without rescanning until the caller re-evaluates', async () => {
  let preflightCalls = 0;
  const wasm = {
    getCanvasKitDocumentPreflight() {
      preflightCalls += 1;
      return preflight('eligible');
    },
  };
  const rendererSession = session('auto', async () => fakeRenderer());

  rendererSession.beginDocument('document-edit');
  const initial = await rendererSession.resolve(wasm as never);
  assert.equal(initial.backend, 'canvaskit');

  const firstEdit = rendererSession.pinAutoMutationRevision()!;
  const secondEdit = rendererSession.pinAutoMutationRevision()!;
  assert.equal(firstEdit.backend, 'canvas2d');
  assert.equal(secondEdit.backend, 'canvas2d');
  assert.equal(secondEdit.diagnostics.selectionReason, 'autoRevisionPending');
  assert.equal(secondEdit.diagnostics.fallbackReason, 'canvaskitRevisionInvalidated');
  assert.equal(secondEdit.diagnostics.preflight, null);
  assert.equal(preflightCalls, 1);

  rendererSession.invalidateDocument();
  const reevaluated = await rendererSession.resolve(wasm as never);
  assert.equal(reevaluated.backend, 'canvaskit');
  assert.equal(preflightCalls, 2);
});

test('CanvasKit initialization and resource failures converge on explicit Canvas2D fallbacks', async () => {
  const initializationFailure = session('auto', async () => {
    throw new Error('init failed');
  });
  initializationFailure.beginDocument('document-init-failure');
  const failed = await initializationFailure.resolve({
    getCanvasKitDocumentPreflight: () => preflight('eligible'),
  } as never);
  assert.equal(failed.backend, 'canvas2d');
  assert.equal(failed.diagnostics.fallbackReason, 'canvaskitInitializationFailed');
  assert.equal(failed.diagnostics.initializationError, 'init failed');
  assert.equal(failed.diagnostics.selectionError, 'init failed');

  const resourceFailure = session('auto', async () => fakeRenderer());
  resourceFailure.beginDocument('document-resource-failure');
  await resourceFailure.resolve({
    getCanvasKitDocumentPreflight: () => preflight('eligible'),
  } as never);
  const decisionKey = resourceFailure.diagnostics()!.decisionKey;
  const fallback = resourceFailure.fallbackFromResourceFailure(
    new Error('resource failed'),
    decisionKey,
  )!;
  assert.equal(fallback.backend, 'canvas2d');
  assert.equal(fallback.diagnostics.fallbackReason, 'canvaskitResourcePreparationFailed');
  assert.equal(fallback.diagnostics.initializationError, null);
  assert.equal(fallback.diagnostics.selectionError, 'resource failed');
  assert.equal(resourceFailure.isCurrent(fallback), true);

  const staleFailure = session('auto', async () => fakeRenderer());
  staleFailure.beginDocument('document-stale-failure');
  const staleSelection = await staleFailure.resolve({
    getCanvasKitDocumentPreflight: () => preflight('eligible'),
  } as never);
  staleFailure.invalidateDocument();
  assert.equal(
    staleFailure.fallbackFromResourceFailure(
      new Error('late resource failure'),
      staleSelection.diagnostics.decisionKey,
    ),
    null,
  );

  const runtimeFailure = session('auto', async () => fakeRenderer());
  runtimeFailure.beginDocument('document-runtime-failure');
  const runtimeSelection = await runtimeFailure.resolve({
    getCanvasKitDocumentPreflight: () => preflight('eligible'),
  } as never);
  const runtimeFallback = runtimeFailure.fallbackFromRuntimeFailure(
    new Error('surface failed'),
    runtimeSelection.diagnostics.decisionKey,
  )!;
  assert.equal(runtimeFallback.backend, 'canvas2d');
  assert.equal(runtimeFallback.diagnostics.fallbackReason, 'canvaskitRuntimeFailed');

  const explicitRuntimeFailure = session('canvaskit', async () => fakeRenderer());
  explicitRuntimeFailure.beginDocument('document-explicit-runtime-failure');
  const explicitSelection = await explicitRuntimeFailure.resolve({} as never);
  assert.equal(
    explicitRuntimeFailure.fallbackFromRuntimeFailure(
      new Error('surface failed'),
      explicitSelection.diagnostics.decisionKey,
    ),
    null,
  );
});

test('CanvasKit initialization retries at a new document boundary', async () => {
  let attempts = 0;
  const rendererSession = session('auto', async () => {
    attempts += 1;
    if (attempts === 1) throw new Error('transient init failure');
    return fakeRenderer();
  });
  const wasm = { getCanvasKitDocumentPreflight: () => preflight('eligible') };

  rendererSession.beginDocument('document-a');
  const first = await rendererSession.resolve(wasm as never);
  assert.equal(first.backend, 'canvas2d');

  rendererSession.beginDocument('document-b');
  const second = await rendererSession.resolve(wasm as never);
  assert.equal(second.backend, 'canvaskit');
  assert.equal(attempts, 2);
});

test('RendererSession owns and disposes CanvasKit exactly once', async () => {
  let disposeCalls = 0;
  const rendererSession = session('canvaskit', async () => fakeRenderer(() => {
    disposeCalls += 1;
  }));
  rendererSession.beginDocument('document-dispose');
  const selection = await rendererSession.resolve({} as never);

  rendererSession.dispose();
  rendererSession.dispose();
  assert.equal(disposeCalls, 1);
  assert.equal(rendererSession.isCurrent(selection), false);
  await assert.rejects(rendererSession.resolve({} as never), /disposed/);
});
