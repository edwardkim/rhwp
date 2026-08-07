#!/usr/bin/env node

import { mkdirSync, writeFileSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { assert, runTest, setTestCase, waitForCanvas } from './helpers.mjs';

const E2E_DIR = path.dirname(fileURLToPath(import.meta.url));
const REPO_ROOT = path.resolve(E2E_DIR, '..', '..');
const FIXTURE = path.join(
  REPO_ROOT,
  'samples',
  'basic',
  'issue2007_nested_cell_pagination_42065.hwp',
);
const OUTPUT_DIR = path.join(REPO_ROOT, 'output', '4159');
const OUTPUT_PAGE = path.join(OUTPUT_DIR, 'issue2007_p003_bottom_border_canvas2d.png');
const OUTPUT_CROP = path.join(OUTPUT_DIR, 'issue2007_p003_bottom_border_crop.png');
const OUTPUT_TREE = path.join(OUTPUT_DIR, 'render_tree_003_fixed.json');

runTest('#4159 종료 재귀 중첩 표 bottom border Canvas2D', async ({ page }) => {
  setTestCase('issue2007 물리 3쪽 종료 표 수평선 clip 포섭');
  const input = await page.$('#file-input');
  if (!input) throw new Error('file-input not found');
  await input.uploadFile(FIXTURE);
  await page.waitForFunction(
    () => window.__wasm?.getSourceFormat?.() === 'hwp' && window.__wasm.pageCount === 17,
    { timeout: 30000 },
  );
  await waitForCanvas(page, 30000);

  const result = await page.evaluate(() => {
    const doc = window.__wasm?.doc;
    if (!doc || typeof doc.renderPageToCanvas !== 'function') {
      throw new Error('WASM Canvas2D renderer not found');
    }

    const tree = JSON.parse(doc.getPageLayerTree(2));
    const lines = [];
    const visit = (value) => {
      if (!value || typeof value !== 'object') return;
      if (value.type === 'line') lines.push(value);
      for (const child of Object.values(value)) visit(child);
    };
    visit(tree);
    const targets = lines.filter((line) => {
      const width = line.bbox?.width ?? line.bbox?.w ?? 0;
      const height = line.bbox?.height ?? line.bbox?.h ?? 0;
      return line.bbox?.y > 820 && width > 500 && height <= 2;
    });
    if (targets.length !== 1) {
      throw new Error(`page 3 terminal bottom Line count=${targets.length}`);
    }
    const target = targets[0];
    const targetWidth = target.bbox.width ?? target.bbox.w;
    const targetHeight = target.bbox.height ?? target.bbox.h;

    const canvas = document.createElement('canvas');
    const scale = 2;
    doc.renderPageToCanvas(2, canvas, scale);

    const padX = 2;
    const padY = 2;
    const left = Math.max(0, Math.floor((target.bbox.x - padX) * scale));
    const top = Math.max(0, Math.floor((target.bbox.y - padY) * scale));
    const width = Math.min(
      canvas.width - left,
      Math.ceil((targetWidth + padX * 2) * scale),
    );
    const height = Math.min(
      canvas.height - top,
      Math.ceil((targetHeight + padY * 2) * scale),
    );
    const pixels = canvas.getContext('2d', { willReadFrequently: true })
      ?.getImageData(left, top, width, height).data;
    if (!pixels) throw new Error('Canvas2D pixels unavailable');

    let maxRowInk = 0;
    for (let y = 0; y < height; y += 1) {
      let rowInk = 0;
      for (let x = 0; x < width; x += 1) {
        const offset = (y * width + x) * 4;
        const ink = pixels[offset + 3] > 32
          && pixels[offset] + pixels[offset + 1] + pixels[offset + 2] < 690;
        if (ink) rowInk += 1;
      }
      maxRowInk = Math.max(maxRowInk, rowInk);
    }

    const crop = document.createElement('canvas');
    crop.width = width;
    crop.height = height;
    crop.getContext('2d')?.drawImage(
      canvas,
      left,
      top,
      width,
      height,
      0,
      0,
      width,
      height,
    );

    return {
      pageCount: window.__wasm.pageCount,
      treeJson: JSON.stringify(tree, null, 2),
      pageDataUrl: canvas.toDataURL('image/png'),
      cropDataUrl: crop.toDataURL('image/png'),
      targetBbox: target.bbox,
      sampleWidth: width,
      maxRowInk,
    };
  });

  mkdirSync(OUTPUT_DIR, { recursive: true });
  writeFileSync(OUTPUT_TREE, result.treeJson);
  writeFileSync(OUTPUT_PAGE, Buffer.from(result.pageDataUrl.split(',')[1], 'base64'));
  writeFileSync(OUTPUT_CROP, Buffer.from(result.cropDataUrl.split(',')[1], 'base64'));

  assert(result.pageCount === 17, `#4069 pagination 유지 (${result.pageCount}쪽)`);
  assert(
    result.maxRowInk >= result.sampleWidth * 0.8,
    `물리 3쪽 종료 수평선 픽셀 ${result.maxRowInk}/${result.sampleWidth}: ${JSON.stringify(result.targetBbox)}`,
  );
  console.log(`  Evidence: ${OUTPUT_PAGE}`);
  console.log(`  Evidence: ${OUTPUT_CROP}`);
  console.log(`  Evidence: ${OUTPUT_TREE}`);
});
