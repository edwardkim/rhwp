#!/usr/bin/env node

import assert from 'node:assert/strict';
import { mkdirSync, writeFileSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { loadHwpFile, runTest, setTestCase, waitForCanvas } from './helpers.mjs';

const E2E_DIR = path.dirname(fileURLToPath(import.meta.url));
const REPO_ROOT = path.resolve(E2E_DIR, '..', '..');
const OUTPUT_DIR = path.join(REPO_ROOT, 'output', '4272');
const OUTPUT_JSON = path.join(OUTPUT_DIR, 'nested-cell-text-selection.json');
const OUTPUT_PNG = path.join(OUTPUT_DIR, 'nested-cell-text-selection.png');
const COPY_PASTE_PNG = path.join(OUTPUT_DIR, 'nested-cell-copy-paste.png');
const FIXTURE = 'basic/issue2007_nested_cell_pagination_42065.hwp';
const PAGE_INDEX = 4;
const TARGET_TEXT = '23,504';

runTest('#4272 중첩 표 안쪽 셀 텍스트 선택 하이라이트', async ({ page }) => {
  setTestCase('물리 5쪽 깊이 3 cellPath의 실제 마우스 드래그');
  const consoleMessages = [];
  page.on('console', (message) => {
    if (message.type() === 'warn' || message.type() === 'error') {
      consoleMessages.push({ type: message.type(), text: message.text() });
    }
  });

  const loaded = await loadHwpFile(page, FIXTURE);
  assert.equal(loaded.pageCount, 17, '#4069 17쪽 pagination 계약');
  await waitForCanvas(page, 30000);

  const target = await page.evaluate(({ pageIndex, targetText }) => {
    const wasm = window.__wasm;
    if (!wasm?.doc) throw new Error('WASM document not loaded');
    const tree = JSON.parse(wasm.doc.getPageLayerTree(pageIndex));
    const runs = [];
    const visit = (value) => {
      if (!value || typeof value !== 'object') return;
      const width = value.bbox?.width ?? value.bbox?.w;
      const height = value.bbox?.height ?? value.bbox?.h;
      if (value.text === targetText && Number.isFinite(width) && Number.isFinite(height)) {
        runs.push({ text: value.text, bbox: value.bbox, width, height });
      }
      for (const child of Object.values(value)) visit(child);
    };
    visit(tree);

    const candidates = [];
    for (const run of runs) {
      const y = run.bbox.y + run.height / 2;
      const samples = [0.04, 0.12, 0.25, 0.5, 0.75, 0.88, 0.96].map((ratio) => {
        const x = run.bbox.x + run.width * ratio;
        return { ratio, x, y, hit: wasm.hitTest(pageIndex, x, y) };
      }).filter((sample) => (sample.hit?.cellPath?.length ?? 0) >= 2);

      for (const start of samples) {
        for (const end of [...samples].reverse()) {
          if (start.ratio >= end.ratio) continue;
          if (JSON.stringify(start.hit.cellPath) !== JSON.stringify(end.hit.cellPath)) continue;
          const charSpan = Math.abs(end.hit.charOffset - start.hit.charOffset);
          if (charSpan === 0) continue;
          candidates.push({ ...run, start, end, depth: start.hit.cellPath.length, charSpan });
          break;
        }
      }
    }
    candidates.sort((a, b) => b.depth - a.depth || b.charSpan - a.charSpan);
    if (!candidates.length) throw new Error(`${targetText} 중첩 셀 드래그 후보를 찾지 못했습니다`);
    return candidates[0];
  }, { pageIndex: PAGE_INDEX, targetText: TARGET_TEXT });

  await page.evaluate(({ pageIndex, candidate }) => {
    const input = window.__inputHandler;
    input.activateWithCaretPosition();
    input.cursor.clearSelection();
    input.cursor.exitCellSelectionMode();
    const container = document.getElementById('scroll-container');
    const canvasView = window.__canvasView;
    const virtualScroll = canvasView.virtualScroll;
    const zoom = canvasView.getZoom?.() ?? 1;
    const pageOffset = virtualScroll.getPageOffset(pageIndex);
    const targetY = (candidate.start.y + candidate.end.y) * zoom / 2;
    container.scrollTop = Math.max(0, pageOffset + targetY - container.clientHeight / 2);

    const wasm = window.__wasm;
    const original = wasm.getSelectionRectsInCellByPath;
    wasm.__issue4272SelectionProbe = { calls: 0, totalMs: 0, original };
    wasm.getSelectionRectsInCellByPath = function (...args) {
      const startedAt = performance.now();
      wasm.__issue4272SelectionProbe.calls += 1;
      try {
        return original.apply(this, args);
      } finally {
        wasm.__issue4272SelectionProbe.totalMs += performance.now() - startedAt;
      }
    };
  }, { pageIndex: PAGE_INDEX, candidate: target });
  await page.evaluate(() => new Promise((resolve) => {
    requestAnimationFrame(() => requestAnimationFrame(() => setTimeout(resolve, 300)));
  }));

  const points = await page.evaluate(({ pageIndex, candidate }) => {
    const container = document.getElementById('scroll-container');
    const scrollContent = document.getElementById('scroll-content');
    const canvasView = window.__canvasView;
    const virtualScroll = canvasView.virtualScroll;
    const zoom = canvasView.getZoom?.() ?? 1;
    const contentRect = scrollContent.getBoundingClientRect();
    const pageOffset = virtualScroll.getPageOffset(pageIndex);
    const pageLeft = virtualScroll.getPageLeftResolved(pageIndex, scrollContent.clientWidth);
    const toClient = (sample) => ({
      x: contentRect.left + pageLeft + sample.x * zoom,
      y: contentRect.top + pageOffset + sample.y * zoom,
    });
    return {
      start: toClient(candidate.start),
      end: toClient(candidate.end),
      container: container.getBoundingClientRect().toJSON(),
      scrollTop: container.scrollTop,
      zoom,
    };
  }, { pageIndex: PAGE_INDEX, candidate: target });
  assert.ok(points.start.y >= points.container.top && points.start.y <= points.container.bottom);
  assert.ok(points.end.y >= points.container.top && points.end.y <= points.container.bottom);

  await page.mouse.move(points.start.x, points.start.y);
  await page.mouse.down();
  await page.mouse.move(points.end.x, points.end.y, { steps: 16 });
  await page.mouse.up();
  await page.evaluate(() => new Promise((resolve) => requestAnimationFrame(() => setTimeout(resolve, 200))));

  const observed = await page.evaluate(() => {
    const input = window.__inputHandler;
    const wasm = window.__wasm;
    const probe = wasm.__issue4272SelectionProbe;
    const highlights = Array.from(document.querySelectorAll('.selection-layer > div'))
      .filter((element) => getComputedStyle(element).display !== 'none')
      .map((element) => ({
        left: element.style.left,
        top: element.style.top,
        width: element.style.width,
        height: element.style.height,
      }));
    const result = {
      hasSelection: input.hasSelection(),
      selection: input.cursor.getSelectionOrdered?.() ?? null,
      cursor: input.cursor.getPosition(),
      highlightCount: highlights.length,
      highlights,
      pathApiCalls: probe?.calls ?? 0,
      pathApiTotalMs: probe?.totalMs ?? 0,
    };
    if (probe?.original) wasm.getSelectionRectsInCellByPath = probe.original;
    delete wasm.__issue4272SelectionProbe;
    return result;
  });

  await page.screenshot({ path: OUTPUT_PNG, fullPage: false });

  await page.keyboard.down('Control');
  await page.keyboard.press('c');
  await page.keyboard.up('Control');
  await page.evaluate(() => new Promise((resolve) => setTimeout(resolve, 150)));
  const copied = await page.evaluate(() => ({
    text: window.__wasm.getClipboardText(),
    hasInternalClipboard: window.__wasm.hasInternalClipboard(),
  }));

  await page.keyboard.down('Control');
  await page.keyboard.press('v');
  await page.keyboard.up('Control');
  await page.evaluate(() => new Promise((resolve) => requestAnimationFrame(() => setTimeout(resolve, 350))));
  const afterPaste = await page.evaluate(({ candidate, targetText }) => {
    const input = window.__inputHandler;
    const start = candidate.start.hit;
    return {
      text: window.__wasm.getTextInCellByPath(
        start.sectionIndex,
        start.parentParaIndex,
        JSON.stringify(start.cellPath),
        start.charOffset,
        [...targetText].length,
      ),
      cursor: input.cursor.getPosition(),
      hasSelection: input.hasSelection(),
    };
  }, { candidate: target, targetText: TARGET_TEXT });
  await page.screenshot({ path: COPY_PASTE_PNG, fullPage: false });

  mkdirSync(OUTPUT_DIR, { recursive: true });
  writeFileSync(OUTPUT_JSON, `${JSON.stringify({
    fixture: FIXTURE,
    pageIndex: PAGE_INDEX,
    pageCount: loaded.pageCount,
    target,
    points,
    observed,
    copied,
    afterPaste,
    consoleMessages,
  }, null, 2)}\n`);

  assert.equal(target.text, TARGET_TEXT);
  assert.equal(target.depth, 3, '외부→래퍼→자식 표 깊이 3 경로');
  assert.equal(target.charSpan, [...TARGET_TEXT].length, `23,504 전체 선택 (${target.charSpan})`);
  assert.equal(observed.hasSelection, true, '논리 선택 생성');
  assert.ok(observed.selection, '정렬된 선택 범위 존재');
  assert.equal(observed.selection.start.charOffset, target.start.hit.charOffset, '선택 시작 offset');
  assert.equal(observed.selection.end.charOffset, target.end.hit.charOffset, '선택 끝 offset');
  assert.deepEqual(observed.selection.start.cellPath, observed.selection.end.cellPath);
  assert.ok(observed.pathApiCalls >= 1, `경로 기반 rect API 호출 (${observed.pathApiCalls})`);
  assert.ok(observed.pathApiCalls <= 20, `16-step drag에서 중복 폭증 없는 API 호출 (${observed.pathApiCalls})`);
  assert.ok(observed.highlightCount >= 1, `선택 하이라이트 표시 (${observed.highlightCount})`);
  assert.equal(copied.hasInternalClipboard, true, '내부 클립보드 생성');
  assert.equal(copied.text, TARGET_TEXT, '중첩 셀 선택 plain text 복사');
  assert.equal(afterPaste.text, TARGET_TEXT, '선택 범위 Ctrl+V 뒤 텍스트 보존');
  assert.equal(afterPaste.hasSelection, false, '붙여넣기 뒤 선택 해제');
  assert.equal(afterPaste.cursor.charOffset, [...TARGET_TEXT].length, '붙여넣기 뒤 캐럿 offset');
  assert.deepEqual(consoleMessages, [], `브라우저 경고·오류 없음: ${JSON.stringify(consoleMessages)}`);
  console.log(`Evidence: ${OUTPUT_JSON}`);
  console.log(`Evidence: ${OUTPUT_PNG}`);
  console.log(`Evidence: ${COPY_PASTE_PNG}`);
});
