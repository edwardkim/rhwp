#!/usr/bin/env node

import { mkdirSync, writeFileSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { assert, runTest, setTestCase, waitForCanvas } from './helpers.mjs';

const E2E_DIR = path.dirname(fileURLToPath(import.meta.url));
const REPO_ROOT = path.resolve(E2E_DIR, '..', '..');
const FIXTURE = path.join(REPO_ROOT, 'samples', 'issues', '2809', 'jubo_20260104.hwp');
const OUTPUT_DIR = path.join(REPO_ROOT, 'output', 'e2e', 'task2809');
const OUTPUT_PNG = path.join(OUTPUT_DIR, 'jubo_p2_wasm_canvas_2x.png');
const OUTPUT_EDITOR_PNG = path.join(OUTPUT_DIR, 'jubo_p2_rhwp_editor_100.png');

runTest('Issue #2809 위·아래 Split 문단 속성 차이 회귀', async ({ page }) => {
  setTestCase('이슈 원본 HWP WASM 렌더');
  const input = await page.$('#file-input');
  if (!input) throw new Error('file-input not found');
  await input.uploadFile(FIXTURE);
  await page.waitForFunction(
    () => window.__wasm?.getSourceFormat?.() === 'hwp' && window.__wasm.pageCount === 6,
    { timeout: 30000 },
  );
  await waitForCanvas(page, 30000);

  await page.waitForFunction(
    () => Array.from(document.querySelectorAll('button')).some(
      (candidate) => candidate.textContent?.includes('대체 글꼴로 보기'),
    ),
    { timeout: 5000 },
  ).catch(() => {});
  const dismissedFontDialog = await page.evaluate(() => {
    const button = Array.from(document.querySelectorAll('button')).find(
      (candidate) => candidate.textContent?.includes('대체 글꼴로 보기'),
    );
    if (!(button instanceof HTMLButtonElement)) return false;
    button.click();
    return true;
  });
  if (dismissedFontDialog) {
    await new Promise((resolve) => setTimeout(resolve, 300));
  }

  const result = await page.evaluate(() => {
    const doc = window.__wasm?.doc;
    if (!doc || typeof doc.renderPageToCanvas !== 'function') {
      throw new Error('WASM Canvas 렌더러를 찾을 수 없습니다');
    }
    const canvas = document.createElement('canvas');
    doc.renderPageToCanvas(1, canvas, 2);
    const svg = window.__wasm.renderPageSvg(1);
    const labelRows = Array.from(svg.matchAll(
      /<text x="([0-9.]+)" y="([0-9.]+)"[^>]*>다<\/text>\s*<text x="([0-9.]+)"[^>]*>같<\/text>\s*<text x="([0-9.]+)"[^>]*>이<\/text>/g,
    ), (match) => ({
      xs: [Number(match[1]), Number(match[3]), Number(match[4])],
      y: Number(match[2]),
    }));
    const context = canvas.getContext('2d', { willReadFrequently: true });
    const labelGlyphWidths = labelRows.map((row) => row.xs.map((glyphX) => {
      const scale = 2;
      const left = Math.max(0, Math.floor(glyphX * scale) - 2);
      const top = Math.max(0, Math.floor((row.y - 20) * scale));
      const width = Math.min(canvas.width - left, Math.ceil(30 * scale));
      const height = Math.min(canvas.height - top, Math.ceil(24 * scale));
      const pixels = context?.getImageData(left, top, width, height).data;
      let minX = width;
      let maxX = -1;
      if (pixels) {
        for (let y = 0; y < height; y += 1) {
          for (let x = 0; x < width; x += 1) {
            const offset = (y * width + x) * 4;
            if (pixels[offset] + pixels[offset + 1] + pixels[offset + 2] < 600) {
              minX = Math.min(minX, x);
              maxX = Math.max(maxX, x);
            }
          }
        }
      }
      return maxX >= minX ? maxX - minX + 1 : 0;
    }));
    const layerTree = JSON.parse(window.__wasm.getPageLayerTree(1));
    const layerLabelPositions = [];
    const labelVariantGroups = [];
    const visit = (value) => {
      if (!value || typeof value !== 'object') return;
      if (value.type === 'textRun' && value.text === '다 같 이') {
        layerLabelPositions.push(value.positions ?? []);
        labelVariantGroups.push(value.variant?.equivalenceGroup ?? null);
      }
      for (const child of Object.values(value)) visit(child);
    };
    visit(layerTree);
    return {
      dataUrl: canvas.toDataURL('image/png'),
      width: canvas.width,
      height: canvas.height,
      labels: labelRows.map((row) => row.xs),
      labelGlyphWidths,
      layerLabelPositions,
      labelVariantGroups,
      hasSplitLabel: svg.includes('>다</text>') && svg.includes('>같</text>') && svg.includes('>이</text>'),
    };
  });

  mkdirSync(OUTPUT_DIR, { recursive: true });
  writeFileSync(OUTPUT_PNG, Buffer.from(result.dataUrl.split(',')[1], 'base64'));

  await page.evaluate(() => {
    const view = window.__canvasView;
    view.viewportManager.setZoom(1.0);
    view.container.scrollTop = view.virtualScroll.getPageOffset(1);
    view.updateVisiblePages();
  });
  await page.waitForFunction(() => !!window.__canvasView?.canvasPool?.getCanvas(1), {
    timeout: 30000,
  });
  await new Promise((resolve) => setTimeout(resolve, 500));
  await page.screenshot({ path: OUTPUT_EDITOR_PNG });

  const rendererDiagnostics = await page.evaluate(() => ({
    backend: window.__canvasView?.getRenderBackend?.(),
    session: window.__canvasView?.getRendererSessionDiagnostics?.(),
    page: window.__canvasView?.getCanvasKitRenderDiagnostics?.(1),
  }));
  console.log(`  Renderer diagnostics: ${JSON.stringify(rendererDiagnostics)}`);
  console.log(`  Label variant groups: ${JSON.stringify(result.labelVariantGroups)}`);

  assert(result.width > 1000 && result.height > 1500, `2x canvas 생성 (${result.width}×${result.height})`);
  assert(result.hasSplitLabel, 'WASM SVG에 다/같/이 라벨 존재');
  assert(result.labels.length >= 6, '위·아래 Split 라벨 좌표 추출');
  const topSpan = result.labels[0]?.[2] - result.labels[0]?.[0];
  const bottomSpan = result.labels.at(-1)?.[2] - result.labels.at(-1)?.[0];
  assert(
    Number.isFinite(topSpan) && Number.isFinite(bottomSpan) && Math.abs(topSpan - bottomSpan) < 3,
    `마지막 glyph 잉크 여유를 포함한 위·아래 분배 폭 정합 (${topSpan} / ${bottomSpan})`,
  );
  const topLayerSpan = result.layerLabelPositions[0]?.[4] - result.layerLabelPositions[0]?.[0];
  const bottomLayerSpan = result.layerLabelPositions.at(-1)?.[4]
    - result.layerLabelPositions.at(-1)?.[0];
  assert(
    Number.isFinite(topLayerSpan)
      && Number.isFinite(bottomLayerSpan)
      && Math.abs(topLayerSpan - bottomLayerSpan) < 3,
    `WASM 페이지 레이어 트리의 위·아래 분배 폭 정합 (${topLayerSpan} / ${bottomLayerSpan})`,
  );
  const topGlyphWidth = result.labelGlyphWidths[0]?.[0];
  const topLastGlyphWidth = result.labelGlyphWidths[0]?.[2];
  const bottomGlyphWidth = result.labelGlyphWidths.at(-1)?.[0];
  assert(
    topGlyphWidth > 0
      && bottomGlyphWidth > 0
      && topGlyphWidth / bottomGlyphWidth > 0.8
      && topGlyphWidth / bottomGlyphWidth < 1.2,
    `음수 자간은 글자 위치만 조정하고 glyph 폭은 유지 (${topGlyphWidth} / ${bottomGlyphWidth}px)`,
  );
  assert(
    topLastGlyphWidth > 0 && topLastGlyphWidth / topGlyphWidth > 0.75,
    `위쪽 마지막 glyph가 셀 우측에서 잘리지 않음 (${topLastGlyphWidth} / ${topGlyphWidth}px)`,
  );
  console.log(`  Evidence: ${OUTPUT_PNG}`);
  console.log(`  Editor evidence: ${OUTPUT_EDITOR_PNG}`);
});
