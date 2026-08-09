#!/usr/bin/env node

import assert from 'node:assert/strict';
import { mkdirSync, writeFileSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { loadHwpFile, runTest, setTestCase, waitForCanvas } from './helpers.mjs';

const E2E_DIR = path.dirname(fileURLToPath(import.meta.url));
const REPO_ROOT = path.resolve(E2E_DIR, '..', '..');
const OUTPUT_DIR = path.join(REPO_ROOT, 'output', '4272');
const OUTPUT_JSON = path.join(OUTPUT_DIR, 'nested-table-object-copy.json');
const OUTPUT_PNG = path.join(OUTPUT_DIR, 'nested-table-object-copy.png');
const FIXTURE = 'basic/issue2007_nested_cell_pagination_42065.hwp';
const PAGE_INDEX = 4;
const EXPECTED_PATH = Object.freeze([
  Object.freeze({ controlIndex: 1, cellIndex: 0, cellParaIndex: 0 }),
  Object.freeze({ controlIndex: 2, cellIndex: 0, cellParaIndex: 12 }),
  Object.freeze({ controlIndex: 0, cellIndex: 0, cellParaIndex: 0 }),
]);
const EXPECTED_OWNER_PATH = EXPECTED_PATH.slice(0, -1);

runTest('#4272 3중 중첩 표 객체 Ctrl+C', async ({ page }) => {
  setTestCase('물리 5쪽 자식 표 선택 → owner path 기반 표 객체 복사');
  const consoleMessages = [];
  page.on('console', (message) => {
    if (message.type() === 'warn' || message.type() === 'error') {
      consoleMessages.push({ type: message.type(), text: message.text() });
    }
  });

  const loaded = await loadHwpFile(page, FIXTURE);
  assert.equal(loaded.pageCount, 17, '#4069 17쪽 pagination 계약');
  await waitForCanvas(page, 30000);

  const target = await page.evaluate((pageIndex) => {
    const wasm = window.__wasm;
    const tree = JSON.parse(wasm.doc.getPageLayerTree(pageIndex));
    let run = null;
    const visit = (value) => {
      if (run || !value || typeof value !== 'object') return;
      const width = value.bbox?.width ?? value.bbox?.w;
      const height = value.bbox?.height ?? value.bbox?.h;
      if (value.text === '구 분' && Number.isFinite(width) && Number.isFinite(height)) {
        run = { text: value.text, bbox: value.bbox, width, height };
        return;
      }
      for (const child of Object.values(value)) visit(child);
    };
    visit(tree);
    if (!run) throw new Error('물리 5쪽 `구 분` TextRun을 찾지 못했습니다');
    const hit = wasm.hitTest(
      pageIndex,
      run.bbox.x + run.width / 2,
      run.bbox.y + run.height / 2,
    );
    return { ...run, hit };
  }, PAGE_INDEX);
  assert.deepEqual(target.hit.cellPath, EXPECTED_PATH, '깊이 3 자식 표 셀 경로');

  const inputState = await page.evaluate(({ pageIndex, targetValue }) => {
    const input = window.__inputHandler;
    input.activateWithCaretPosition();
    input.cursor.clearSelection();
    input.cursor.moveTo(targetValue.hit);
    input.updateCaret();
    input.focus();

    const container = document.getElementById('scroll-container');
    const view = window.__canvasView;
    const pageOffset = view.virtualScroll.getPageOffset(pageIndex);
    const zoom = view.getZoom?.() ?? 1;
    container.scrollTop = Math.max(
      0,
      pageOffset + (targetValue.bbox.y + targetValue.height / 2) * zoom
        - container.clientHeight / 2,
    );

    const wasm = window.__wasm;
    const originalCopy = wasm.copyControl;
    const originalExport = wasm.exportControlHtml;
    wasm.__issue4272TableCopyProbe = {
      copyCalls: [],
      exportCalls: [],
      originalCopy,
      originalExport,
    };
    wasm.copyControl = function (...args) {
      const result = originalCopy.apply(this, args);
      wasm.__issue4272TableCopyProbe.copyCalls.push({ args, result });
      return result;
    };
    wasm.exportControlHtml = function (...args) {
      const result = originalExport.apply(this, args);
      wasm.__issue4272TableCopyProbe.exportCalls.push({
        args,
        resultLength: result.length,
        containsTable: result.includes('<table'),
      });
      return result;
    };
    return {
      active: input.isActive(),
      inCell: input.cursor.isInCell(),
      position: input.cursor.getPosition(),
      focused: document.activeElement === input.textarea,
    };
  }, { pageIndex: PAGE_INDEX, targetValue: target });

  await page.keyboard.press('Escape');
  await page.evaluate(() => new Promise((resolve) => requestAnimationFrame(resolve)));
  const selected = await page.evaluate(() => ({
    active: window.__inputHandler.cursor.isInTableObjectSelection(),
    ref: window.__inputHandler.cursor.getSelectedTableRef(),
  }));

  await page.keyboard.down('Control');
  await page.keyboard.press('c');
  await page.keyboard.up('Control');
  await page.evaluate(() => new Promise((resolve) => setTimeout(resolve, 200)));

  const observed = await page.evaluate(() => {
    const wasm = window.__wasm;
    const probe = wasm.__issue4272TableCopyProbe;
    const result = {
      selected: window.__inputHandler.cursor.isInTableObjectSelection(),
      ref: window.__inputHandler.cursor.getSelectedTableRef(),
      clipboardText: wasm.getClipboardText(),
      hasInternalClipboard: wasm.hasInternalClipboard(),
      clipboardHasControl: wasm.clipboardHasControl(),
      copyCalls: probe?.copyCalls ?? [],
      exportCalls: probe?.exportCalls ?? [],
    };
    if (probe?.originalCopy) wasm.copyControl = probe.originalCopy;
    if (probe?.originalExport) wasm.exportControlHtml = probe.originalExport;
    delete wasm.__issue4272TableCopyProbe;
    return result;
  });

  mkdirSync(OUTPUT_DIR, { recursive: true });
  await page.screenshot({ path: OUTPUT_PNG, fullPage: false });
  writeFileSync(OUTPUT_JSON, `${JSON.stringify({
    fixture: FIXTURE,
    pageIndex: PAGE_INDEX,
    target,
    inputState,
    selected,
    expectedOwnerPath: EXPECTED_OWNER_PATH,
    observed,
    consoleMessages,
  }, null, 2)}\n`);

  assert.equal(inputState.active, true);
  assert.equal(inputState.inCell, true);
  assert.equal(inputState.focused, true);
  assert.equal(selected.active, true, 'Esc로 자식 표 객체 선택');
  assert.deepEqual(selected.ref?.cellPath, EXPECTED_PATH, '선택 렌더링용 전체 경로 유지');
  assert.equal(observed.selected, true, 'Ctrl+C 뒤 객체 선택 유지');
  assert.equal(observed.copyCalls.length, 1, 'copyControl 1회');
  assert.deepEqual(
    observed.copyCalls[0].args,
    [0, 7, 0, JSON.stringify(EXPECTED_OWNER_PATH)],
    '선택 표 control 0 + 소유 문단 depth 2 경로',
  );
  assert.equal(observed.exportCalls.length, 1, 'exportControlHtml 1회');
  assert.deepEqual(
    observed.exportCalls[0].args,
    [0, 7, 0, JSON.stringify(EXPECTED_OWNER_PATH)],
  );
  assert.equal(observed.exportCalls[0].containsTable, true);
  assert.ok(observed.exportCalls[0].resultLength > 0);
  assert.equal(observed.clipboardText, '[표]');
  assert.equal(observed.hasInternalClipboard, true);
  assert.equal(observed.clipboardHasControl, true);
  assert.deepEqual(consoleMessages, []);
  console.log(`Evidence: ${OUTPUT_JSON}`);
  console.log(`Evidence: ${OUTPUT_PNG}`);
});
