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
const OUTPUT_DIR = path.join(REPO_ROOT, 'output', '536');
const OUTPUT_PNG = path.join(OUTPUT_DIR, 'issue2007_p002_canvas2d.png');

runTest('#536 Canvas2D 한컴 사각 안 숫자 PUA 폴백', async ({ page }) => {
  setTestCase('issue2007 2쪽 U+F02B1 사각형+1');
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
    const canvas = document.createElement('canvas');
    const scale = 2;
    doc.renderPageToCanvas(1, canvas, scale);

    const tree = JSON.parse(doc.getPageLayerTree(1));
    let target = null;
    const visit = (value) => {
      if (!value || typeof value !== 'object' || target) return;
      if (value.type === 'textRun' && value.text?.startsWith('\u{F02B1}')) {
        target = value;
        return;
      }
      for (const child of Object.values(value)) visit(child);
    };
    visit(tree);
    if (!target) throw new Error('page 2 U+F02B1 TextRun not found');

    const fontSize = target.style?.fontSize ?? 12;
    const baselineY = target.bbox.y + (target.baseline ?? fontSize);
    const boxSize = Math.max(1, fontSize * 0.72);
    const boxX = target.bbox.x;
    const boxY = baselineY - fontSize * 0.76;
    const pad = 2;
    const left = Math.max(0, Math.floor((boxX - pad) * scale));
    const top = Math.max(0, Math.floor((boxY - pad) * scale));
    const width = Math.min(
      canvas.width - left,
      Math.ceil((boxSize + pad * 2) * scale),
    );
    const height = Math.min(
      canvas.height - top,
      Math.ceil((boxSize + pad * 2) * scale),
    );
    const pixels = canvas.getContext('2d', { willReadFrequently: true })
      ?.getImageData(left, top, width, height).data;
    if (!pixels) throw new Error('Canvas2D pixels unavailable');

    let minX = width;
    let minY = height;
    let maxX = -1;
    let maxY = -1;
    let centerInk = 0;
    for (let y = 0; y < height; y += 1) {
      for (let x = 0; x < width; x += 1) {
        const offset = (y * width + x) * 4;
        const ink = pixels[offset + 3] > 32
          && pixels[offset] + pixels[offset + 1] + pixels[offset + 2] < 690;
        if (!ink) continue;
        minX = Math.min(minX, x);
        minY = Math.min(minY, y);
        maxX = Math.max(maxX, x);
        maxY = Math.max(maxY, y);
        if (
          x > width * 0.3 && x < width * 0.7
          && y > height * 0.25 && y < height * 0.75
        ) centerInk += 1;
      }
    }

    return {
      dataUrl: canvas.toDataURL('image/png'),
      pageCount: window.__wasm.pageCount,
      rawText: target.text,
      charOverlap: target.charOverlap ?? null,
      fontSize,
      boxSize,
      inkWidth: maxX >= minX ? maxX - minX + 1 : 0,
      inkHeight: maxY >= minY ? maxY - minY + 1 : 0,
      centerInk,
      expectedPixels: boxSize * scale,
    };
  });

  mkdirSync(OUTPUT_DIR, { recursive: true });
  writeFileSync(OUTPUT_PNG, Buffer.from(result.dataUrl.split(',')[1], 'base64'));

  assert(result.pageCount === 17, `#4122 pagination 유지 (${result.pageCount}쪽)`);
  assert(result.rawText.startsWith('\u{F02B1}'), 'IR은 U+F02B1 raw PUA를 보존');
  assert(result.charOverlap === null, '대상 표식은 CharOverlap op가 아닌 TextRun');
  assert(
    result.inkWidth >= result.expectedPixels * 0.82
      && result.inkWidth <= result.expectedPixels * 1.18,
    `사각형 잉크 폭 ${result.inkWidth}px ≈ ${result.expectedPixels.toFixed(1)}px`,
  );
  assert(
    result.inkHeight >= result.expectedPixels * 0.82
      && result.inkHeight <= result.expectedPixels * 1.18,
    `사각형 잉크 높이 ${result.inkHeight}px ≈ ${result.expectedPixels.toFixed(1)}px`,
  );
  assert(result.centerInk >= 3, `사각형 내부 숫자 1 잉크 존재 (${result.centerInk}px)`);
  console.log(`  Evidence: ${OUTPUT_PNG}`);
});
