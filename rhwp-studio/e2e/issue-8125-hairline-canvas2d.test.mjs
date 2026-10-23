#!/usr/bin/env node

// #8125 WASM Canvas2D 의 장치 한 칸보다 가는 획은 한 칸 굵기로 그린다.
//
// 한컴 Print PDF 를 raster 로 읽으면 0.1 mm(약 0.38 px) 표 괘선도 장치 한 칸을 채운다.
// Canvas2D 가 선언 굵기(하한 0.5 px)대로 anti-alias 하면 괘선의 덮임이 절반 안팎에 그쳐 옅어진다.
// Native 검사(tests/cases/native_fractional_glyph_origin.rs `subpixel_stroke_hairline`)와 같은
// 입력·같은 판정이다: render tree 의 실제 괘선 위치에서 그 괘선을 가로지르는 화소 중 가장 진한
// 화소가 선 색 대비의 절반 이상인지 본다. 조판 좌표·선언 굵기는 그대로다.

import { mkdirSync, writeFileSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { assert, runTest, setTestCase, waitForCanvas } from './helpers.mjs';

const E2E_DIR = path.dirname(fileURLToPath(import.meta.url));
const REPO_ROOT = path.resolve(E2E_DIR, '..', '..');
const FIXTURE = path.join(REPO_ROOT, 'samples', 'basic', 'calendar_year.hwp');
const OUTPUT_DIR = path.join(REPO_ROOT, 'output', '8125');
const OUTPUT_PAGE = path.join(OUTPUT_DIR, 'calendar_year_p001_canvas2d.png');

runTest('#8125 장치 한 칸보다 가는 괘선 Canvas2D', async ({ page }) => {
  setTestCase('calendar_year 1쪽 가는 가로 괘선의 장치 한 칸 덮임');
  const input = await page.$('#file-input');
  if (!input) throw new Error('file-input not found');
  await input.uploadFile(FIXTURE);
  // 빈 새 문서도 한 쪽이라, 표본의 괘선이 layer tree 에 나타날 때까지 기다린다.
  await page.waitForFunction(
    () => window.__wasm?.getSourceFormat?.() === 'hwp' && window.__wasm.pageCount === 1
      && window.__wasm.doc?.getPageLayerTree(0).includes('"type":"line"'),
    { timeout: 30000 },
  );
  await waitForCanvas(page, 30000);

  const result = await page.evaluate(() => {
    const doc = window.__wasm?.doc;
    if (!doc || typeof doc.renderPageToCanvas !== 'function') {
      throw new Error('WASM Canvas2D renderer not found');
    }

    const tree = JSON.parse(doc.getPageLayerTree(0));
    const lines = [];
    const visit = (value) => {
      if (!value || typeof value !== 'object') return;
      if (value.type === 'line') lines.push(value);
      for (const child of Object.values(value)) visit(child);
    };
    visit(tree);
    const thin = lines.filter((line) => Math.abs(line.y1 - line.y2) < 0.01
      && Math.abs(line.x2 - line.x1) > 20
      && line.style.width > 0
      && line.style.width < 1);

    const canvas = document.createElement('canvas');
    doc.renderPageToCanvas(0, canvas, 1);
    const ctx = canvas.getContext('2d', { willReadFrequently: true });
    const pixels = ctx.getImageData(0, 0, canvas.width, canvas.height).data;
    // 투명 화소는 흰 종이 위에 합성한 회색으로 읽는다.
    const ink = (x, y) => {
      const at = (y * canvas.width + x) * 4;
      const gray = (pixels[at] + pixels[at + 1] + pixels[at + 2]) / 3;
      return (255 - gray) * (pixels[at + 3] / 255);
    };

    const faint = [];
    let checked = 0;
    for (const line of thin) {
      const hex = /^#([0-9a-f]{6})$/i.exec(line.style.color);
      if (!hex) continue;
      const value = parseInt(hex[1], 16);
      const lineGray = (((value >> 16) & 0xff) + ((value >> 8) & 0xff) + (value & 0xff)) / 3;
      const contrast = 255 - lineGray;
      if (contrast < 64) continue;
      checked += 1;
      // 괘선 양 끝(교차 괘선)을 피해 가운데 세 지점을 본다.
      for (const t of [0.3, 0.5, 0.7]) {
        const x = Math.floor(line.x1 + (line.x2 - line.x1) * t);
        let darkest = 0;
        for (let y = Math.floor(line.y1 - 2); y <= Math.floor(line.y1 + 2); y += 1) {
          if (y >= 0 && y < canvas.height) darkest = Math.max(darkest, ink(x, y));
        }
        if (darkest < contrast * 0.5) {
          faint.push([+line.y1.toFixed(1), line.style.width, +(darkest / contrast).toFixed(2)]);
        }
      }
    }

    return {
      pageDataUrl: canvas.toDataURL('image/png'),
      thinCount: thin.length,
      checked,
      faint,
    };
  });

  mkdirSync(OUTPUT_DIR, { recursive: true });
  writeFileSync(OUTPUT_PAGE, Buffer.from(result.pageDataUrl.split(',')[1], 'base64'));

  assert(result.checked >= 10, `가는 가로 괘선 ${result.checked}/${result.thinCount}개`);
  assert(
    result.faint.length === 0,
    `장치 한 칸을 덮지 못한 괘선 ${result.faint.length}곳 (y, 굵기, 최대 덮임): ${JSON.stringify(result.faint.slice(0, 6))}`,
  );
  console.log(`  Evidence: ${OUTPUT_PAGE}`);
});
