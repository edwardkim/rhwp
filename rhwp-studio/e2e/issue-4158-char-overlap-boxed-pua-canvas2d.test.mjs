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
const OUTPUT_DIR = path.join(REPO_ROOT, 'output', '4158');
const OUTPUT_PAGE = path.join(OUTPUT_DIR, 'issue2007_p010_char_overlap_canvas2d.png');
const OUTPUT_CROP = path.join(OUTPUT_DIR, 'issue2007_p010_char_overlap_crop.png');
const OUTPUT_TREE = path.join(OUTPUT_DIR, 'render_tree_010.json');

runTest('#4158 실제 CharOverlap 사각 숫자 Canvas2D 합성', async ({ page }) => {
  setTestCase('issue2007 10쪽 공정거래위원회 앞 U+F02B1');
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

    const tree = JSON.parse(doc.getPageLayerTree(9));
    const textRuns = [];
    const visit = (value) => {
      if (!value || typeof value !== 'object') return;
      if (value.type === 'textRun') textRuns.push(value);
      for (const child of Object.values(value)) visit(child);
    };
    visit(tree);

    const pageHeight = tree.bbox?.height ?? tree.bbox?.h ?? 1122.5;
    const candidates = textRuns.filter((run) => (
      run.text?.startsWith('\u{F02B1}')
      && run.charOverlap != null
      && run.bbox?.y >= 0
      && run.bbox.y < pageHeight
    ));
    const contextIndexes = textRuns
      .map((run, index) => (run.text?.includes('공정거래위원회') ? index : -1))
      .filter((index) => index >= 0);
    const target = candidates
      .map((candidate) => {
        const index = textRuns.indexOf(candidate);
        const distance = Math.min(
          ...contextIndexes
            .filter((contextIndex) => contextIndex > index)
            .map((contextIndex) => contextIndex - index),
        );
        return { candidate, distance };
      })
      .filter(({ distance }) => Number.isFinite(distance))
      .sort((left, right) => left.distance - right.distance)[0]?.candidate;
    if (!target) throw new Error('page 10 CharOverlap U+F02B1 TextRun not found');

    const fontSize = Math.max(1, target.style?.fontSize ?? 12);
    const targetHeight = target.bbox.height ?? target.bbox.h;
    const expectedCenter = {
      x: target.bbox.x + fontSize / 2,
      y: target.bbox.y + targetHeight - fontSize / 2,
    };
    const fillTextCalls = [];
    const strokeRectCalls = [];
    const proto = CanvasRenderingContext2D.prototype;
    const originalFillText = proto.fillText;
    const originalStrokeRect = proto.strokeRect;
    proto.fillText = function captureFillText(text, x, y, ...rest) {
      fillTextCalls.push({ text: String(text), x, y });
      return originalFillText.call(this, text, x, y, ...rest);
    };
    proto.strokeRect = function captureStrokeRect(x, y, width, height) {
      strokeRectCalls.push({ x, y, width, height });
      return originalStrokeRect.call(this, x, y, width, height);
    };

    const canvas = document.createElement('canvas');
    const scale = 2;
    try {
      doc.renderPageToCanvas(9, canvas, scale);
    } finally {
      proto.fillText = originalFillText;
      proto.strokeRect = originalStrokeRect;
    }

    const nearTarget = (call) => (
      Math.abs(call.x - expectedCenter.x) <= 1
      && Math.abs(call.y - expectedCenter.y) <= 1
    );
    const targetTextCalls = fillTextCalls.filter(nearTarget);
    const targetRectCalls = strokeRectCalls.filter((call) => (
      Math.abs(call.x - target.bbox.x) <= 1
      && Math.abs(call.y - (target.bbox.y + targetHeight - fontSize)) <= 1
      && Math.abs(call.width - fontSize) <= 1
      && Math.abs(call.height - fontSize) <= 1
    ));

    const cropPad = 4;
    const cropLeft = Math.max(0, Math.floor((target.bbox.x - cropPad) * scale));
    const cropTop = Math.max(
      0,
      Math.floor((target.bbox.y + targetHeight - fontSize - cropPad) * scale),
    );
    const cropSize = Math.ceil((fontSize + cropPad * 2) * scale);
    const crop = document.createElement('canvas');
    crop.width = Math.min(cropSize, canvas.width - cropLeft);
    crop.height = Math.min(cropSize, canvas.height - cropTop);
    crop.getContext('2d')?.drawImage(
      canvas,
      cropLeft,
      cropTop,
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
      charOverlap: target.charOverlap,
      contextFound: textRuns.some((run) => run.text?.includes('공정거래위원회')),
      targetTextCalls,
      targetRectCalls,
      candidateBboxes: candidates.map((candidate) => candidate.bbox),
    };
  });

  mkdirSync(OUTPUT_DIR, { recursive: true });
  writeFileSync(OUTPUT_TREE, result.treeJson);
  writeFileSync(OUTPUT_PAGE, Buffer.from(result.pageDataUrl.split(',')[1], 'base64'));
  writeFileSync(OUTPUT_CROP, Buffer.from(result.cropDataUrl.split(',')[1], 'base64'));

  assert(result.pageCount === 17, `페이지 수 유지 (${result.pageCount}쪽)`);
  assert(result.rawText.startsWith('\u{F02B1}'), 'IR은 U+F02B1 raw PUA를 보존');
  assert(result.charOverlap?.borderType === 0, '실제 CharOverlap raw borderType=0 보존');
  assert(result.contextFound, '물리 10쪽 공정거래위원회 문맥 확인');
  assert(
    result.targetTextCalls.some((call) => call.text === '1'),
    `대상 중심에 숫자 1 합성: ${JSON.stringify(result.targetTextCalls)}`,
  );
  assert(
    !result.targetTextCalls.some((call) => call.text.includes('\u{F02B1}')),
    `대상 중심에 raw PUA fillText 금지: ${JSON.stringify(result.targetTextCalls)}`,
  );
  assert(
    result.targetRectCalls.length >= 1,
    `대상 bbox에 사각 테두리 합성: ${JSON.stringify(result.targetRectCalls)}`,
  );
  console.log(`  Evidence: ${OUTPUT_PAGE}`);
  console.log(`  Evidence: ${OUTPUT_CROP}`);
  console.log(`  Evidence: ${OUTPUT_TREE}`);
});
