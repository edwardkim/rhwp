#!/usr/bin/env node

import { mkdirSync, writeFileSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { assert, runTest, setTestCase, waitForCanvas } from './helpers.mjs';

const E2E_DIR = path.dirname(fileURLToPath(import.meta.url));
const REPO_ROOT = path.resolve(E2E_DIR, '..', '..');
const FIXTURE = path.join(REPO_ROOT, 'samples', 'basic', 'pau-004.hwp');
const OUTPUT_DIR = path.join(REPO_ROOT, 'output', 'pau-004');
const OUTPUT_PAGE = path.join(OUTPUT_DIR, 'pau004_p001_canvas2d.png');
const OUTPUT_CROP = path.join(OUTPUT_DIR, 'pau004_p001_f02fb_crop.png');
const OUTPUT_TREE = path.join(OUTPUT_DIR, 'render_tree_001.json');

runTest('U+F02FB 작은 오른쪽 방향 삼각형 Canvas2D 투영', async ({ page }) => {
  setTestCase('#4224 pau-004 일반 TextRun U+F02FB');
  const input = await page.$('#file-input');
  if (!input) throw new Error('file-input not found');
  await input.uploadFile(FIXTURE);
  await page.waitForFunction(
    () => window.__wasm?.getSourceFormat?.() === 'hwp' && window.__wasm.pageCount === 1,
    { timeout: 30000 },
  );
  await waitForCanvas(page, 30000);

  const result = await page.evaluate(() => {
    const doc = window.__wasm?.doc;
    if (!doc || typeof doc.renderPageToCanvas !== 'function') {
      throw new Error('WASM Canvas2D renderer not found');
    }

    const tree = JSON.parse(doc.getPageLayerTree(0));
    const textRuns = [];
    const visit = (value) => {
      if (!value || typeof value !== 'object') return;
      if (value.type === 'textRun') textRuns.push(value);
      for (const child of Object.values(value)) visit(child);
    };
    visit(tree);
    const target = textRuns.find((run) => run.text?.startsWith('\u{F02FB}'));
    if (!target) throw new Error('pau-004 U+F02FB TextRun not found');

    const fillTextCalls = [];
    const proto = CanvasRenderingContext2D.prototype;
    const originalFillText = proto.fillText;
    proto.fillText = function captureFillText(text, x, y, ...rest) {
      fillTextCalls.push({ text: String(text), x, y });
      return originalFillText.call(this, text, x, y, ...rest);
    };

    const canvas = document.createElement('canvas');
    const scale = 2;
    try {
      doc.renderPageToCanvas(0, canvas, scale);
    } finally {
      proto.fillText = originalFillText;
    }

    const pad = 8;
    const height = target.bbox.height ?? target.bbox.h;
    const width = target.bbox.width ?? target.bbox.w;
    const left = Math.max(0, Math.floor((target.bbox.x - pad) * scale));
    const top = Math.max(0, Math.floor((target.bbox.y - pad) * scale));
    const crop = document.createElement('canvas');
    crop.width = Math.min(Math.ceil((width + pad * 2) * scale), canvas.width - left);
    crop.height = Math.min(Math.ceil((height + pad * 2) * scale), canvas.height - top);
    crop.getContext('2d')?.drawImage(
      canvas,
      left,
      top,
      crop.width,
      crop.height,
      0,
      0,
      crop.width,
      crop.height,
    );

    return {
      treeJson: JSON.stringify(tree, null, 2),
      pageDataUrl: canvas.toDataURL('image/png'),
      cropDataUrl: crop.toDataURL('image/png'),
      pageCount: window.__wasm.pageCount,
      rawText: target.text,
      charOverlap: target.charOverlap ?? null,
      fillTextCalls,
    };
  });

  mkdirSync(OUTPUT_DIR, { recursive: true });
  writeFileSync(OUTPUT_TREE, result.treeJson);
  writeFileSync(OUTPUT_PAGE, Buffer.from(result.pageDataUrl.split(',')[1], 'base64'));
  writeFileSync(OUTPUT_CROP, Buffer.from(result.cropDataUrl.split(',')[1], 'base64'));

  assert(result.pageCount === 1, `페이지 수 유지 (${result.pageCount}쪽)`);
  assert(result.rawText === '\u{F02FB}아름다운', 'IR은 U+F02FB 원문을 보존');
  assert(result.charOverlap == null, '대상은 CharOverlap이 아닌 일반 TextRun');
  assert(
    result.fillTextCalls.some((call) => call.text.includes('▸')),
    `Canvas2D가 작은 오른쪽 방향 삼각형을 출력: ${JSON.stringify(result.fillTextCalls)}`,
  );
  assert(
    !result.fillTextCalls.some((call) => call.text.includes('\u{F02FB}')),
    `Canvas2D raw U+F02FB 출력 금지: ${JSON.stringify(result.fillTextCalls)}`,
  );
  assert(
    result.fillTextCalls.map((call) => call.text).join('').includes('▸아름다운'),
    `삼각형 뒤 본문 순서 보존: ${JSON.stringify(result.fillTextCalls)}`,
  );
  console.log(`  Evidence: ${OUTPUT_PAGE}`);
  console.log(`  Evidence: ${OUTPUT_CROP}`);
  console.log(`  Evidence: ${OUTPUT_TREE}`);
});
