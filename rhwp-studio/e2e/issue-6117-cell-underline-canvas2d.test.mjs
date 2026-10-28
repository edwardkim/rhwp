#!/usr/bin/env node

// [#6117] studio 캔버스에서 표 칸 안 밑줄이 칸 우측 괘선을 넘어 그려진다.
//
// #6028 이 정한 계약 — soft-wrap 이 소비한 줄-말미 공백은 장식선 길이에서 뺀다 —
// 은 RenderNode 경로(`svg.rs`, 캔버스의 RenderNode 분기)에만 배선돼 있었다.
// studio 는 layer tree 를 **그대로 재생**하므로 그 배선이 없어, 배분 정렬로 늘어난
// 말미 공백까지 밑줄이 그어졌다. layer→SVG 경로는 RenderNode 로 되돌려 그리기
// 때문에 CLI 에서는 드러나지 않는다.
//
// 이 테스트는 칸 우단 오른쪽 띠의 잉크를 세어 밑줄이 넘어오지 않음을 고정한다.

import { mkdirSync, writeFileSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { assert, runTest, setTestCase, waitForCanvas } from './helpers.mjs';

const E2E_DIR = path.dirname(fileURLToPath(import.meta.url));
const REPO_ROOT = path.resolve(E2E_DIR, '..', '..');
const FIXTURE = path.join(
  REPO_ROOT,
  'samples',
  'issue6117',
  '52690_higher_education_decree.hwp',
);
const OUTPUT_DIR = path.join(REPO_ROOT, 'output', '6117');
const OUTPUT_PNG = path.join(OUTPUT_DIR, '52690_p009_canvas2d.png');
/** 결함이 나타나는 쪽(0-based). */
const PAGE = 8;

runTest('#6117 Canvas2D 표 칸 밑줄이 괘선을 넘지 않는다', async ({ page }) => {
  setTestCase('52690 9쪽 신구조문 대비표 밑줄');
  const input = await page.$('#file-input');
  if (!input) throw new Error('file-input not found');
  await input.uploadFile(FIXTURE);
  await page.waitForFunction(
    () => window.__wasm?.pageCount > 8,
    { timeout: 30000 },
  );
  await waitForCanvas(page, 30000);

  const result = await page.evaluate((pageIndex) => {
    const doc = window.__wasm?.doc;
    if (!doc || typeof doc.renderPageToCanvas !== 'function') {
      throw new Error('WASM Canvas2D renderer not found');
    }
    const canvas = document.createElement('canvas');
    const scale = 2;
    doc.renderPageToCanvas(pageIndex, canvas, scale);

    // 밑줄 run 을 품은 **모든** 표 칸을 모은다 — 결함은 오른쪽 칸(우단 ≈691.7)에서
    // 크게 나므로 한 칸만 보면 놓친다.
    const tree = JSON.parse(doc.getPageLayerTree(pageIndex));
    const cells = new Map();
    const visit = (value, enclosingCell) => {
      if (!value || typeof value !== 'object') return;
      const nextCell = value.groupKind?.kind === 'tableCell' ? value : enclosingCell;
      if (
        value.type === 'textRun'
        && value.style?.underline
        && value.style.underline !== 'none'
        && nextCell
      ) {
        const key = `${nextCell.bounds.x},${nextCell.bounds.y},${nextCell.bounds.width}`;
        if (!cells.has(key)) cells.set(key, nextCell);
      }
      for (const child of Object.values(value)) visit(child, nextCell);
    };
    visit(tree, null);
    if (cells.size === 0) throw new Error('밑줄이 있는 표 칸을 찾지 못했다');

    const ctx = canvas.getContext('2d', { willReadFrequently: true });
    let worst = { maxRunLength: 0, ink: 0, cellRight: 0 };
    for (const cell of cells.values()) {
      const right = cell.bounds.x + cell.bounds.width;
      // 칸 우단 바로 오른쪽 24px 띠(괘선 자체는 1~2px 이라 3px 띄운다).
      const left = Math.max(0, Math.floor((right + 3) * scale));
      const top = Math.max(0, Math.floor(cell.bounds.y * scale));
      const width = Math.min(canvas.width - left, Math.ceil(24 * scale));
      const height = Math.min(canvas.height - top, Math.ceil(cell.bounds.height * scale));
      if (width <= 0 || height <= 0) continue;
      const pixels = ctx?.getImageData(left, top, width, height).data;
      if (!pixels) continue;
      // 넘어온 밑줄은 **괘선에 맞닿아** 오른쪽으로 이어진다. 옆 칸 글자는 칸
      // 여백만큼 떨어져 시작하므로, 띠 왼쪽 끝(괘선 바로 옆)에서 시작하는
      // 연속 잉크만 센다 — 이웃 글자와 확실히 갈린다.
      let ink = 0;
      let maxRunLength = 0;
      const isDark = (offset) => pixels[offset + 3] > 32
        && pixels[offset] + pixels[offset + 1] + pixels[offset + 2] < 690;
      for (let y = 0; y < height; y += 1) {
        const rowStart = y * width * 4;
        if (!isDark(rowStart)) continue;
        let run = 0;
        for (let x = 0; x < width; x += 1) {
          if (!isDark(rowStart + x * 4)) break;
          run += 1;
        }
        ink += run;
        if (run > maxRunLength) maxRunLength = run;
      }
      if (maxRunLength > worst.maxRunLength) {
        worst = { maxRunLength, ink, cellRight: right };
      }
    }

    return {
      dataUrl: canvas.toDataURL('image/png'),
      cellRight: worst.cellRight,
      ink: worst.ink,
      maxRunLength: worst.maxRunLength,
      cellCount: cells.size,
      scale,
    };
  }, PAGE);

  mkdirSync(OUTPUT_DIR, { recursive: true });
  writeFileSync(OUTPUT_PNG, Buffer.from(result.dataUrl.split(',')[1], 'base64'));

  // 밑줄이 넘어오면 칸 우단 오른쪽에 **가로로 이어진** 어두운 픽셀 띠가 생긴다.
  // 옆 칸의 글자는 세로로 흩어진 잉크라 연속 길이가 짧다 — 연속 길이로 가른다.
  assert(
    result.maxRunLength <= 3 * result.scale,
    `괘선에 맞닿아 이어진 밑줄 없음 (worst 칸 우단 ${result.cellRight.toFixed(1)}) `
      + `(칸 ${result.cellCount}개 중 최대 연속 ${result.maxRunLength}px, 잉크 ${result.ink}px)`,
  );
  console.log(`  Evidence: ${OUTPUT_PNG}`);
});
