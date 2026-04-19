import {
  assert,
  comparePngBuffers,
  cropPngBuffer,
  createNewDocument,
  getLayerOpBBoxes,
  loadApp,
  loadHwpFile,
  runTest,
  screenshotCanvas,
  setTestCase,
} from './helpers.mjs';

const FULL_PAGE_CASES = [
  { name: 'blank-new-document', setup: (page) => createNewDocument(page) },
  { name: 'lseg-01-basic', setup: (page) => loadHwpFile(page, 'lseg-01-basic.hwp') },
  { name: 'eq-01', setup: (page) => loadHwpFile(page, 'eq-01.hwp') },
  {
    name: 'hwp-table-test',
    setup: (page) => loadHwpFile(page, 'hwp_table_test.hwp'),
    maxDiffRatio: 0.0002,
  },
  { name: 'pic-crop-01', setup: (page) => loadHwpFile(page, 'pic-crop-01.hwp') },
  { name: 'field-01', setup: (page) => loadHwpFile(page, 'field-01.hwp') },
  { name: 'shape-group-02', setup: (page) => loadHwpFile(page, 'shape-group-02.hwp') },
  { name: 'group-drawing-02', setup: (page) => loadHwpFile(page, 'group-drawing-02.hwp') },
];
const CANVASKIT_MODE = process.env.RHWP_CANVASKIT_MODE === 'default' ? 'default' : 'compat';
const TOLERANT_DIFF = {
  ignoreChannelDelta: 8,
  maxDiffRatio: 0.0025,
};
const FEATURE_CASES = [
  {
    name: 'eq-01',
    setup: (page) => loadHwpFile(page, 'eq-01.hwp'),
    opType: 'equation',
    margin: 4,
  },
];

async function renderScenario(page, backend, caseInfo) {
  const search = backend === 'canvaskit'
    ? `?renderer=${backend}&canvaskitMode=${CANVASKIT_MODE}`
    : `?renderer=${backend}`;
  await loadApp(page, search);
  await caseInfo.setup(page);

  const activeBackend = await page.evaluate(() => window.__renderBackend ?? window.__canvasView?.getRenderBackend?.());
  assert(activeBackend === backend || (backend === 'canvas2d' && activeBackend === 'canvas'), `${caseInfo.name} backend=${backend}`);

  if (backend === 'canvaskit') {
    const layerSummary = await page.evaluate(() => {
      const tree = window.__wasm?.getPageLayerTree?.(0);
      if (!tree) return null;
      const root = tree.root;
      const opCount = root.kind === 'leaf' ? root.ops.length : root.kind === 'group' ? root.children.length : 1;
      return {
        kind: root.kind,
        opCount,
        mode: window.__canvaskitRenderMode,
      };
    });
    assert(!!layerSummary && layerSummary.opCount > 0, `${caseInfo.name} layer tree exported`);
    assert(layerSummary?.mode === CANVASKIT_MODE, `${caseInfo.name} canvaskitMode=${CANVASKIT_MODE}`);
  }

  const screenshotName = backend === 'canvaskit'
    ? `${caseInfo.name}-${backend}-${CANVASKIT_MODE}`
    : `${caseInfo.name}-${backend}`;
  return screenshotCanvas(page, screenshotName);
}

runTest('CanvasKit 렌더 비교', async ({ page }) => {
  for (const caseInfo of FULL_PAGE_CASES) {
    setTestCase(caseInfo.name);
    console.log(`\n[${caseInfo.name}] Canvas2D baseline 렌더...`);
    const baseline = await renderScenario(page, 'canvas2d', caseInfo);

    console.log(`[${caseInfo.name}] CanvasKit 렌더...`);
    const canvaskit = await renderScenario(page, 'canvaskit', caseInfo);

    const diff = await comparePngBuffers(baseline.buffer, canvaskit.buffer, {
      diffName: `${caseInfo.name}-${CANVASKIT_MODE}`,
      ignoreChannelDelta: TOLERANT_DIFF.ignoreChannelDelta,
      maxDiffRatio: caseInfo.maxDiffRatio ?? TOLERANT_DIFF.maxDiffRatio,
    });

    assert(
      diff.passed,
      `${caseInfo.name} screenshot exact=${diff.exactDiffPixels} (${diff.exactDiffRatio.toFixed(4)}), tolerant=${diff.tolerantDiffPixels} (${diff.tolerantDiffRatio.toFixed(4)}), raw_tolerant=${diff.rawTolerantDiffPixels} (${diff.rawTolerantDiffRatio.toFixed(4)}), ignored_channel_delta<=${diff.ignoreChannelDelta}, max_channel_delta=${diff.maxChannelDelta}`,
    );
  }

  for (const caseInfo of FEATURE_CASES) {
    setTestCase(`${caseInfo.name}-feature`);
    console.log(`\n[${caseInfo.name}] Canvas2D baseline 기능 렌더...`);
    const baseline = await renderScenario(page, 'canvas2d', caseInfo);

    console.log(`[${caseInfo.name}] CanvasKit 기능 렌더...`);
    const canvaskit = await renderScenario(page, 'canvaskit', caseInfo);

    const boxes = await getLayerOpBBoxes(page, caseInfo.opType);
    assert(boxes.length > 0, `${caseInfo.name} ${caseInfo.opType} bbox exported`);

    for (const [index, box] of boxes.entries()) {
      const bbox = {
        x: box.x - caseInfo.margin,
        y: box.y - caseInfo.margin,
        width: box.width + caseInfo.margin * 2,
        height: box.height + caseInfo.margin * 2,
      };
      const diff = await comparePngBuffers(
        cropPngBuffer(baseline.buffer, bbox),
        cropPngBuffer(canvaskit.buffer, bbox),
        {
          diffName: `${caseInfo.name}-${caseInfo.opType}-${index}-${CANVASKIT_MODE}`,
          ignoreChannelDelta: TOLERANT_DIFF.ignoreChannelDelta,
          maxDiffRatio: TOLERANT_DIFF.maxDiffRatio,
        },
      );
      assert(
        diff.passed,
        `${caseInfo.name} ${caseInfo.opType}[${index}] exact=${diff.exactDiffPixels} (${diff.exactDiffRatio.toFixed(4)}), tolerant=${diff.tolerantDiffPixels} (${diff.tolerantDiffRatio.toFixed(4)}), raw_tolerant=${diff.rawTolerantDiffPixels} (${diff.rawTolerantDiffRatio.toFixed(4)}), ignored_channel_delta<=${diff.ignoreChannelDelta}, max_channel_delta=${diff.maxChannelDelta}`,
      );
    }
  }
}, { skipLoadApp: true });
