import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const rootDir = dirname(dirname(fileURLToPath(import.meta.url)));

function source(path: string): string {
  return readFileSync(join(rootDir, path), 'utf8');
}

test('개발용 런타임 import 실패는 Studio 초기화를 중단시키지 않는다', () => {
  const main = source('src/main.ts');

  assert.match(main, /if \(!import\.meta\.env\.DEV \|\| stopDevelopmentRenderRuntime \|\| !canvasView\) return;/);
  assert.match(main, /try \{[\s\S]*await import\('@\/core\/subsecond-runtime'\)/);
  assert.match(
    main,
    /catch \(error\) \{[\s\S]*console\.warn\('\[main\] 개발용 렌더 코드 교체를 시작하지 못했습니다:'/,
  );
  const canvasViewCreatedAt = main.indexOf('canvasView = new CanvasView(');
  const runtimeStartedAt = main.indexOf('await startDevelopmentRenderRuntime();', canvasViewCreatedAt);
  assert.ok(canvasViewCreatedAt >= 0 && runtimeStartedAt > canvasViewCreatedAt);
});

test('일반 Studio 클래스는 개발용 소켓과 감시자를 소유하지 않는다', () => {
  const bridge = source('src/core/wasm-bridge.ts');
  const canvasView = source('src/view/canvas-view.ts');

  assert.doesNotMatch(bridge, /subsecond-runtime|startRenderCodeReload|getRenderCodeReload|disconnectSubsecondDevtools/);
  assert.doesNotMatch(canvasView, /subsecond-runtime|startRenderCodeReloadWatch|renderCodeReloadWatcher|render-code-reloaded/);
  assert.match(
    source('src/main.ts'),
    /canvasView\?\.refreshPages\(\{ throwOnPageInfoError: true \}\)/,
  );
  assert.match(canvasView, /if \(options\.throwOnPageInfoError\) throw e;/);
});

test('비활성화된 자동 투명선 경로의 고아 이벤트를 남기지 않는다', () => {
  assert.doesNotMatch(source('src/engine/input-handler.ts'), /transparent-borders-changed/);
  assert.doesNotMatch(source('src/command/commands/view.ts'), /transparent-borders-changed/);
});

test('Subsecond fidelity harness owns hot line-break provenance and generation-safe scans', () => {
  assert.match(source('src/main.ts'), /visibilityState !== 'visible'\) return Promise\.resolve\(\)/);
  const wasmApi = source('../src/wasm_api.rs');
  const boundaries = source('../src/wasm_api/render_patch_boundary.rs');
  const devApi = source('src/core/rhwp-dev.ts');
  const overlay = source('src/dev/pdf-reference-overlay.ts');
  const fidelity = source('src/dev/fidelity-scan.ts');
  const canvasView = source('src/view/canvas-view.ts');

  assert.match(wasmApi, /js_name = getLineBreakProvenance/);
  assert.match(source('../src/subsecond_dev.rs'), /js_name = getSubsecondPatchEpoch/);
  assert.match(source('../src/subsecond_dev.rs'), /rhwp-subsecond-commit/);
  assert.match(source('src/core/subsecond-runtime.ts'), /SUBSECOND_COMMIT_EVENT = 'rhwp-subsecond-commit'/);
  assert.match(boundaries, /exports \["getLineBreakProvenance"\]/);
  assert.match(devApi, /lineBreakVisible\(/);
  assert.match(devApi, /cellPath/);
  assert.match(overlay, /__rhwpFidelityHarness/);
  assert.match(overlay, /documentGeneration/);
  assert.match(overlay, /getDocumentDigest/);
  assert.match(overlay, /getDocumentGeneration/);
  assert.match(overlay, /referenceGeneration/);
  assert.match(fidelity, /downstreamChangedPageRange/);
  assert.match(overlay, /findFirstLineBreakError/);
  assert.match(overlay, /current\.desiredSrc !== src/);
  assert.match(overlay, /current\.loadingSrc = ''/);
  assert.match(overlay, /current\.imageRetryCount >= 3/);
  assert.match(
    canvasView,
    /nextDiagnosticRenderGeneration\([\s\S]*?decisionChanged,[\s\S]*?changed/,
    'a backend fallback must invalidate an in-flight mixed-backend fidelity scan',
  );
  assert.match(
    canvasView,
    /capturePageForDiagnostics[\s\S]*?fallbackFromResourceFailure[\s\S]*?passesRuntimeReadinessGate[\s\S]*?fallbackFromRuntimeFailure[\s\S]*?commitCanvasKitFallback/,
    'offscreen capture must use the same CanvasKit admission as visible pages',
  );
  assert.match(
    overlay,
    /abandonIfStale[\s\S]*?queueMicrotask[\s\S]*?scanWholeDocument\(trigger\)/,
    'a renderer change during capture must discard and restart the scan',
  );
});

test('layout state owners remain explicit Subsecond jump-table boundaries', () => {
  assert.match(source('../src/lib.rs'), /patch_epoch\(\) == 0 \{\s*\$target/);
  const frame = source('../src/renderer/layout_frame.rs');
  const lineBreaking = source('../src/renderer/composer/line_breaking.rs');
  const layout = source('../src/renderer/layout.rs');
  const paragraphLayout = source('../src/renderer/layout/paragraph_layout.rs');
  const tableLayout = source('../src/renderer/layout/table_layout.rs');
  const partialTable = source('../src/renderer/layout/table_partial.rs');
  const measurement = source('../src/renderer/layout/text_measurement.rs');

  for (const owner of [
    'new_hot_impl',
    'carve_hot_impl',
    'restore_checkpoint_hot_impl',
    'try_admit_stored_rows_hot_impl',
    'commit_carved_row_hot_impl',
    'project_line_segs_hot_impl',
    'project_line_segs_since_hot_impl',
  ]) {
    assert.match(frame, new RegExp(`hot_call!\\([\\s\\S]{0,120}${owner}`), owner);
  }

  for (const owner of [
    'tokenize_paragraph_with_regenerated_space_metric_hot_impl',
    'fill_lines_hot_impl',
    'fill_one_interval_hot_impl',
    'layout_paragraph_in_frame_hot_impl',
    'stored_row_metrics_hot_impl',
    'resolve_stored_line_segs_in_frame_hot_impl',
    'stored_rows_reproduce_frame_expectation_hot_impl',
    'layout_picture_band_hot_impl',
    'reflow_line_segs_hot_impl',
    'recalculate_section_vpos_hot_impl',
  ]) {
    assert.match(lineBreaking, new RegExp(`hot_call!\\([\\s\\S]{0,240}${owner}`), owner);
  }

  for (const owner of [
    'build_render_tree_hot_impl',
    'build_columns_hot_impl',
    'build_single_column_hot_impl',
    'layout_column_item_hot_impl',
  ]) {
    assert.match(layout, new RegExp(`hot_call!\\([\\s\\S]{0,240}${owner}`), owner);
  }

  for (const owner of [
    'layout_inline_table_paragraph_hot_impl',
    'layout_paragraph_hot_impl',
    'layout_partial_paragraph_hot_impl',
    'layout_composed_paragraph_hot_impl',
    'layout_raw_paragraph_hot_impl',
  ]) {
    assert.match(paragraphLayout, new RegExp(`hot_call!\\([\\s\\S]{0,240}${owner}`), owner);
  }

  for (const owner of [
    'layout_table_hot_impl',
    'layout_table_cells_hot_impl',
    'layout_horizontal_cell_paragraphs_hot_impl',
    'resolve_column_widths_hot_impl',
    'resolve_row_heights_hot_impl',
    'resolve_cell_padding_hot_impl',
    'cell_units_hot_impl',
    'cell_units_uncached_hot_impl',
    'advance_row_cut_hot_impl',
    'advance_row_cut_inner_hot_impl',
    'advance_row_block_cut_hot_impl',
    'advance_row_block_cut_with_row_offsets_hot_impl',
  ]) {
    assert.match(tableLayout, new RegExp(`hot_call!\\([\\s\\S]{0,240}${owner}`), owner);
  }

  for (const owner of [
    'layout_partial_table_hot_impl',
    'layout_partial_table_resolved_hot_impl',
    'layout_partial_table_cells_hot_impl',
  ]) {
    assert.match(partialTable, new RegExp(`hot_call!\\([\\s\\S]{0,240}${owner}`), owner);
  }

  for (const owner of [
    'find_next_tab_stop_hot_impl',
    'resolved_to_text_style_hot_impl',
    'estimate_text_width_hot_impl',
    'estimate_text_width_unrounded_hot_impl',
    'hancom_regenerated_space_width_hot_impl',
    'compute_char_positions_hot_impl',
  ]) {
    assert.match(measurement, new RegExp(`hot_call!\\([\\s\\S]{0,160}${owner}`), owner);
  }
});

test('visible line-break diagnostics preserve group ancestry and the rendered TextBox frame', async () => {
  const previousWindow = (globalThis as { window?: unknown }).window;
  const runtime = globalThis as typeof globalThis & {
    window?: typeof globalThis;
    rhwpDev?: {
      lineBreakVisible(page: number, options: { limit: number }): {
        items: unknown[];
        errors: unknown[];
      };
    };
  };
  runtime.window = globalThis;
  const calls: Array<[number, number, string, string]> = [];
  let runs: Array<Record<string, unknown>> = [{
    secIdx: 0,
    paraIdx: 0,
    parentParaIdx: 4,
    controlIdx: 0,
    cellIdx: 0,
    cellParaIdx: 2,
    cellPath: [{ controlIndex: 0, cellIndex: 0, cellParaIndex: 2 }],
    groupPath: [1, 3],
    lineContainerWidthHwp: 16_850,
    x: 400,
  }];
  const doc = {
    getPageTextLayout: () => JSON.stringify({ runs }),
    getLineBreakProvenance: (...args: [number, number, string, string]) => {
      calls.push(args);
      return JSON.stringify({ schemaVersion: 3, status: 'ok' });
    },
  };
  try {
    const { initRhwpDev } = await import('../src/core/rhwp-dev.ts');
    initRhwpDev({ doc, pageCount: 1 } as never);
    const result = runtime.rhwpDev?.lineBreakVisible(2, { limit: 10 });
    assert.equal(result?.errors.length, 0);
    assert.equal(result?.items.length, 1);
    assert.equal(calls.length, 1);
    assert.deepEqual(JSON.parse(calls[0][2]), [
      { controlIndex: 0, cellIndex: 0, cellParaIndex: 2 },
    ]);
    assert.deepEqual(JSON.parse(calls[0][3]), {
      geometry: false,
      measurement: false,
      pageIndex: 2,
      textX: 400,
      groupPath: [1, 3],
      visibleFrameWidthHwp: 16_850,
      geometryMode: 'current-frame',
      maxRows: 128,
      maxCarves: 128,
      maxTokens: 256,
      maxFitDecisions: 512,
    });

    calls.length = 0;
    runs = [{
      secIdx: 2,
      paraIdx: 6,
      parentParaIdx: 379,
      controlIdx: 0,
      cellIdx: 4,
      cellParaIdx: 6,
      cellPath: [{ controlIndex: 0, cellIndex: 4, cellParaIndex: 6 }],
      groupPath: [],
      lineContainerWidthHwp: 27_323,
      x: 130,
    }];
    runtime.rhwpDev?.lineBreakVisible(59, { limit: 10 });
    assert.equal(calls.length, 1);
    assert.deepEqual(JSON.parse(calls[0][3]), {
      geometry: false,
      measurement: false,
      pageIndex: 59,
      textX: 130,
      groupPath: [],
      visibleFrameWidthHwp: 27_323,
      geometryMode: 'current-frame',
      maxRows: 128,
      maxCarves: 128,
      maxTokens: 256,
      maxFitDecisions: 512,
    });

    calls.length = 0;
    runs = [{
      secIdx: 2,
      paraIdx: 0,
      parentParaIdx: 0,
      cellPath: [{ controlIndex: 4, cellIndex: 0, cellParaIndex: 0 }],
      flowContext: 'header',
      x: 89,
    }];
    const headerResult = runtime.rhwpDev?.lineBreakVisible(59, { limit: 10 });
    assert.equal(headerResult?.items.length, 0);
    assert.equal(headerResult?.errors.length, 0);
    assert.equal(calls.length, 0);
  } finally {
    delete runtime.rhwpDev;
    if (previousWindow === undefined) delete runtime.window;
    else runtime.window = previousWindow as typeof globalThis;
  }
});
