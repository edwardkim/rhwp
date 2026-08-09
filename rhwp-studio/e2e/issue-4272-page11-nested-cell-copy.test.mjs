#!/usr/bin/env node

import assert from 'node:assert/strict';
import { mkdirSync, writeFileSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { loadHwpFile, runTest, setTestCase, waitForCanvas } from './helpers.mjs';

const E2E_DIR = path.dirname(fileURLToPath(import.meta.url));
const REPO_ROOT = path.resolve(E2E_DIR, '..', '..');
const OUTPUT_DIR = path.join(REPO_ROOT, 'output', '4272');
const OUTPUT_JSON = path.join(OUTPUT_DIR, 'page11-child-cell-copy.json');
const OUTPUT_PNG = path.join(OUTPUT_DIR, 'page11-child-cell-copy.png');
const FIXTURE = 'basic/issue2007_nested_cell_pagination_42065.hwp';
const PAGE_INDEX = 10;
const TARGET_PREFIX = '행하여야 하며';

runTest('#4272 물리 11쪽 자식 표 셀 텍스트 복사', async ({ page }) => {
  setTestCase('최내곽 문단 22 선택이 평면 cellParaIndex 0으로 퇴행하지 않음');
  const consoleMessages = [];
  page.on('console', (message) => {
    if (message.type() === 'warn' || message.type() === 'error') {
      consoleMessages.push({ type: message.type(), text: message.text() });
    }
  });

  const loaded = await loadHwpFile(page, FIXTURE);
  assert.equal(loaded.pageCount, 17, '#4069 17쪽 pagination 계약');
  await waitForCanvas(page, 30000);

  const target = await page.evaluate(({ pageIndex, prefix }) => {
    const wasm = window.__wasm;
    const tree = JSON.parse(wasm.doc.getPageLayerTree(pageIndex));
    let run = null;
    const visit = (value) => {
      if (run || !value || typeof value !== 'object') return;
      const width = value.bbox?.width ?? value.bbox?.w;
      const height = value.bbox?.height ?? value.bbox?.h;
      if (typeof value.text === 'string' && value.text.startsWith(prefix)
          && Number.isFinite(width) && Number.isFinite(height)) {
        run = { text: value.text, bbox: value.bbox, width, height };
        return;
      }
      for (const child of Object.values(value)) visit(child);
    };
    visit(tree);
    if (!run) throw new Error('물리 11쪽 대상 TextRun을 찾지 못했습니다');

    const y = run.bbox.y + run.height / 2;
    const point = (ratio) => {
      const x = run.bbox.x + run.width * ratio;
      return { ratio, x, y, hit: wasm.hitTest(pageIndex, x, y) };
    };
    const start = point(0.2);
    const end = point(0.8);
    if ((start.hit?.cellPath?.length ?? 0) !== 3) throw new Error('깊이 3 시작점이 아닙니다');
    if (JSON.stringify(start.hit.cellPath) !== JSON.stringify(end.hit.cellPath)) {
      throw new Error('선택 끝점의 cellPath가 다릅니다');
    }
    const count = end.hit.charOffset - start.hit.charOffset;
    const expected = wasm.getTextInCellByPath(
      start.hit.sectionIndex,
      start.hit.parentParaIndex,
      JSON.stringify(start.hit.cellPath),
      start.hit.charOffset,
      count,
    );
    return { ...run, start, end, expected, count, depth: start.hit.cellPath.length };
  }, { pageIndex: PAGE_INDEX, prefix: TARGET_PREFIX });

  const points = await page.evaluate(({ pageIndex, target }) => {
    const input = window.__inputHandler;
    input.activateWithCaretPosition();
    input.cursor.clearSelection();
    const container = document.getElementById('scroll-container');
    const content = document.getElementById('scroll-content');
    const view = window.__canvasView;
    const virtualScroll = view.virtualScroll;
    const zoom = view.getZoom?.() ?? 1;
    const pageOffset = virtualScroll.getPageOffset(pageIndex);
    container.scrollTop = Math.max(0, pageOffset + target.start.y - container.clientHeight / 2);
    const contentRect = content.getBoundingClientRect();
    const pageLeft = virtualScroll.getPageLeftResolved(pageIndex, content.clientWidth);
    const toClient = (sample) => ({
      x: contentRect.left + pageLeft + sample.x * zoom,
      y: contentRect.top + pageOffset + sample.y * zoom,
    });

    const wasm = window.__wasm;
    const original = wasm.copySelectionInCellByPath;
    wasm.__issue4272Page11CopyProbe = { calls: [], original };
    wasm.copySelectionInCellByPath = function (...args) {
      const result = original.apply(this, args);
      wasm.__issue4272Page11CopyProbe.calls.push({ args, result });
      return result;
    };
    return { start: toClient(target.start), end: toClient(target.end) };
  }, { pageIndex: PAGE_INDEX, target });

  await page.mouse.move(points.start.x, points.start.y);
  await page.mouse.down();
  await page.mouse.move(points.end.x, points.end.y, { steps: 16 });
  await page.mouse.up();
  await page.keyboard.down('Control');
  await page.keyboard.press('c');
  await page.keyboard.up('Control');
  await page.evaluate(() => new Promise((resolve) => setTimeout(resolve, 200)));

  const observed = await page.evaluate(() => {
    const wasm = window.__wasm;
    const probe = wasm.__issue4272Page11CopyProbe;
    const result = {
      selection: window.__inputHandler.cursor.getSelectionOrdered?.() ?? null,
      highlightCount: document.querySelectorAll('.selection-layer > div').length,
      clipboardText: wasm.getClipboardText(),
      hasInternalClipboard: wasm.hasInternalClipboard(),
      copyCalls: probe?.calls ?? [],
    };
    if (probe?.original) wasm.copySelectionInCellByPath = probe.original;
    delete wasm.__issue4272Page11CopyProbe;
    return result;
  });

  mkdirSync(OUTPUT_DIR, { recursive: true });
  await page.screenshot({ path: OUTPUT_PNG, fullPage: false });
  writeFileSync(OUTPUT_JSON, `${JSON.stringify({
    fixture: FIXTURE,
    pageIndex: PAGE_INDEX,
    target,
    observed,
    consoleMessages,
  }, null, 2)}\n`);

  assert.equal(target.depth, 3);
  assert.equal(target.start.hit.cellPath.at(-1).cellParaIndex, 22, '최내곽 문단 22');
  assert.ok(target.count > 0, '비어 있지 않은 선택 범위');
  assert.equal(observed.clipboardText, target.expected, '물리 11쪽 선택 plain text');
  assert.equal(observed.hasInternalClipboard, true);
  assert.ok(observed.highlightCount >= 1, '선택 하이라이트 표시');
  assert.equal(observed.copyCalls.length, 1, 'Ctrl+C path API 1회');
  assert.equal(observed.copyCalls[0].args[3], 22, '복사 시작 최내곽 cellParaIndex');
  assert.equal(observed.copyCalls[0].args[5], 22, '복사 끝 최내곽 cellParaIndex');
  assert.deepEqual(consoleMessages, []);
  console.log(`Evidence: ${OUTPUT_JSON}`);
  console.log(`Evidence: ${OUTPUT_PNG}`);
});
