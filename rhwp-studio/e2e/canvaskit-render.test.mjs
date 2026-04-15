import {
  assert,
  comparePngBuffers,
  createNewDocument,
  loadApp,
  loadHwpFile,
  runTest,
  screenshotCanvas,
  setTestCase,
} from './helpers.mjs';

const SAMPLE_CASES = [
  { name: 'blank-new-document', setup: (page) => createNewDocument(page), maxDiffPixels: 0, maxDiffRatio: 0 },
  { name: 'lseg-01-basic', setup: (page) => loadHwpFile(page, 'lseg-01-basic.hwp'), maxDiffPixels: 12000, maxDiffRatio: 0.015 },
  // 표 샘플은 굵은 한글 헤더와 기호 glyph에서 Canvas2D/CanvasKit rasterizer 차이가 조금 더 커진다.
  { name: 'hwp-table-test', setup: (page) => loadHwpFile(page, 'hwp_table_test.hwp'), maxDiffPixels: 22000, maxDiffRatio: 0.025 },
  { name: 'pic-crop-01', setup: (page) => loadHwpFile(page, 'pic-crop-01.hwp'), maxDiffPixels: 9000, maxDiffRatio: 0.01 },
];

async function renderScenario(page, backend, caseInfo) {
  await loadApp(page, `?renderer=${backend}`);
  await caseInfo.setup(page);

  const activeBackend = await page.evaluate(() => window.__renderBackend ?? window.__canvasView?.getRenderBackend?.());
  assert(activeBackend === backend || (backend === 'canvas2d' && activeBackend === 'canvas'), `${caseInfo.name} backend=${backend}`);

  if (backend === 'canvaskit') {
    const layerSummary = await page.evaluate(() => {
      const tree = window.__wasm?.getPageLayerTree?.(0);
      if (!tree) return null;
      const root = tree.root;
      const opCount = root.kind === 'leaf' ? root.ops.length : root.kind === 'group' ? root.children.length : 1;
      return { kind: root.kind, opCount };
    });
    assert(!!layerSummary && layerSummary.opCount > 0, `${caseInfo.name} layer tree exported`);
  }

  return screenshotCanvas(page, `${caseInfo.name}-${backend}`);
}

runTest('CanvasKit 렌더 비교', async ({ page }) => {
  for (const caseInfo of SAMPLE_CASES) {
    setTestCase(caseInfo.name);
    console.log(`\n[${caseInfo.name}] Canvas2D baseline 렌더...`);
    const baseline = await renderScenario(page, 'canvas2d', caseInfo);

    console.log(`[${caseInfo.name}] CanvasKit 렌더...`);
    const canvaskit = await renderScenario(page, 'canvaskit', caseInfo);

    const diff = await comparePngBuffers(baseline.buffer, canvaskit.buffer, {
      diffName: caseInfo.name,
      threshold: 0.08,
      maxDiffPixels: caseInfo.maxDiffPixels,
      maxDiffRatio: caseInfo.maxDiffRatio,
    });

    assert(
      diff.passed,
      `${caseInfo.name} screenshot diff pixels=${diff.diffPixels} ratio=${diff.diffRatio.toFixed(4)}`,
    );
  }
}, { skipLoadApp: true });
