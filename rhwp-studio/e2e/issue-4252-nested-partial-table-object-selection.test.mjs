#!/usr/bin/env node

import assert from 'node:assert/strict';
import { mkdirSync, writeFileSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import {
  loadHwpFile,
  runTest,
  setTestCase,
  waitForCanvas,
} from './helpers.mjs';

const E2E_DIR = path.dirname(fileURLToPath(import.meta.url));
const REPO_ROOT = path.resolve(E2E_DIR, '..', '..');
const OUTPUT_DIR = path.join(REPO_ROOT, 'output', '4252');
const PERF_PHASE = process.env.ISSUE4252_PERF_PHASE || 'after';
const PERF_OUTPUT = path.join(OUTPUT_DIR, `perf-${PERF_PHASE}.json`);
const VISUAL_OUTPUT = path.join(OUTPUT_DIR, 'page5-child-table-object-selection.png');
const FIXTURE = 'basic/issue2007_nested_cell_pagination_42065.hwp';
const EXPECTED_PATH = Object.freeze([
  Object.freeze({ controlIndex: 1, cellIndex: 0, cellParaIndex: 0 }),
  Object.freeze({ controlIndex: 2, cellIndex: 0, cellParaIndex: 12 }),
  Object.freeze({ controlIndex: 0, cellIndex: 0, cellParaIndex: 0 }),
]);
const PAGE2_VALID_PATH = Object.freeze([
  Object.freeze({ controlIndex: 1, cellIndex: 1, cellParaIndex: 0 }),
  Object.freeze({ controlIndex: 5, cellIndex: 0, cellParaIndex: 0 }),
]);

function median(values) {
  const sorted = [...values].sort((a, b) => a - b);
  const middle = Math.floor(sorted.length / 2);
  return sorted.length % 2 ? sorted[middle] : (sorted[middle - 1] + sorted[middle]) / 2;
}

runTest('#4252 재귀 분할 중첩 표 객체 선택 경로', async ({ page }) => {
  setTestCase('물리 5쪽 child table hit → Esc 객체 선택');
  const warnings = [];
  const cursorWarnings = [];
  page.on('console', (message) => {
    if (message.type() === 'warn' && message.text().includes('renderTableObjectSelection')) {
      warnings.push(message.text());
    }
    if (message.type() === 'warn' && message.text().includes('[CursorState] updateRect 실패')) {
      cursorWarnings.push(message.text());
    }
  });

  const loadMs = [];
  for (let iteration = 0; iteration < 5; iteration += 1) {
    const loaded = await loadHwpFile(page, FIXTURE);
    assert.equal(loaded.pageCount, 17, '#4069 17쪽 pagination 계약');
    loadMs.push(loaded.documentLoadAndInitialRenderMs);
  }
  await waitForCanvas(page, 30000);

  const probe = await page.evaluate(({ expectedPath, page2Path }) => {
    const wasm = window.__wasm;
    if (!wasm?.doc) throw new Error('WASM document not loaded');

    const tree = JSON.parse(wasm.doc.getPageLayerTree(4));
    let target = null;
    const visit = (value) => {
      if (!value || typeof value !== 'object' || target) return;
      const width = value.bbox?.width ?? value.bbox?.w;
      const height = value.bbox?.height ?? value.bbox?.h;
      if (value.text === '구 분' && Number.isFinite(width) && Number.isFinite(height)) {
        target = { bbox: value.bbox, width, height };
        return;
      }
      for (const child of Object.values(value)) visit(child);
    };
    visit(tree);
    if (!target) throw new Error('physical page 5 `구 분` TextRun not found');

    const hit = wasm.hitTest(
      4,
      target.bbox.x + target.width / 2,
      target.bbox.y + target.height / 2,
    );

    const measureLookup = (parentParaIndex, pathValue, iterations = 9) => {
      const durations = [];
      let cellCount = 0;
      let error = null;
      for (let index = 0; index < iterations + 1; index += 1) {
        const startedAt = performance.now();
        try {
          const cells = wasm.getTableCellBboxesByPath(
            0,
            parentParaIndex,
            JSON.stringify(pathValue),
          );
          if (index > 0) durations.push(performance.now() - startedAt);
          cellCount = cells.length;
        } catch (caught) {
          if (index > 0) durations.push(performance.now() - startedAt);
          error = caught?.message || String(caught);
          break;
        }
      }
      return { durations, cellCount, error };
    };

    return {
      hit,
      target,
      expectedLookup: measureLookup(7, expectedPath),
      page2Lookup: measureLookup(2, page2Path),
    };
  }, { expectedPath: EXPECTED_PATH, page2Path: PAGE2_VALID_PATH });

  const inputState = await page.evaluate((hit) => {
    const input = window.__inputHandler;
    // loadHwpFile()은 빠른 E2E helper라 main.ts의 initDoc() 입력 활성화 단계를
    // 거치지 않는다. 실제 파일 열기와 동일하게 키보드 입력을 활성화한다.
    input.activateWithCaretPosition();
    input.cursor.clearSelection();
    input.cursor.moveTo(hit);
    input.updateCaret();
    input.focus();
    return {
      active: input.isActive(),
      inCell: input.cursor.isInCell(),
      position: input.cursor.getPosition(),
      focused: document.activeElement === input.textarea,
    };
  }, probe.hit);
  assert.equal(inputState.active, true, 'input handler active');
  assert.equal(inputState.inCell, true, 'hit-test 위치가 셀 내부');
  assert.equal(inputState.focused, true, 'hidden textarea focus');

  await page.evaluate(() => {
    const input = window.__inputHandler;
    const original = input.renderTableObjectSelection;
    const probeState = { calls: 0, totalMs: 0, original };
    input.__issue4252SelectionRenderProbe = probeState;
    input.renderTableObjectSelection = function (...args) {
      const startedAt = performance.now();
      probeState.calls += 1;
      try {
        return original.apply(this, args);
      } finally {
        probeState.totalMs += performance.now() - startedAt;
      }
    };
  });
  const selectionStartedAt = await page.evaluate(() => performance.now());
  await page.keyboard.press('Escape');
  await page.evaluate(() => new Promise((resolve) => requestAnimationFrame(() => resolve())));
  const selection = await page.evaluate((startedAt) => {
    const input = window.__inputHandler;
    const renderer = input.tableObjectRenderer;
    const probeState = input.__issue4252SelectionRenderProbe;
    const result = {
      durationMs: performance.now() - startedAt,
      selected: input.cursor.isInTableObjectSelection(),
      ref: input.cursor.getSelectedTableRef(),
      borders: renderer?.borders?.length ?? 0,
      handles: renderer?.handles?.length ?? 0,
      layerChildren: document.querySelector('.table-object-layer')?.childElementCount ?? 0,
      renderCalls: probeState?.calls ?? 0,
      renderTotalMs: probeState?.totalMs ?? 0,
    };
    if (probeState?.original) input.renderTableObjectSelection = probeState.original;
    delete input.__issue4252SelectionRenderProbe;
    return result;
  }, selectionStartedAt);
  mkdirSync(OUTPUT_DIR, { recursive: true });
  await page.screenshot({ path: VISUAL_OUTPUT, fullPage: false });

  await page.keyboard.press('Escape');
  await page.evaluate(() => new Promise((resolve) => requestAnimationFrame(() => resolve())));
  const parentExit = await page.evaluate(() => ({
    selected: window.__inputHandler.cursor.isInTableObjectSelection(),
    position: window.__inputHandler.cursor.getPosition(),
    rect: window.__inputHandler.cursor.getRect(),
  }));

  const evidence = {
    phase: PERF_PHASE,
    pageCount: 17,
    load: { valuesMs: loadMs, medianMs: median(loadMs) },
    page2ValidLookup: {
      ...probe.page2Lookup,
      medianMs: median(probe.page2Lookup.durations),
    },
    page5ExpectedLookup: {
      ...probe.expectedLookup,
      medianMs: median(probe.expectedLookup.durations),
    },
    hit: probe.hit,
    inputState,
    selection,
    parentExit,
    warnings,
    cursorWarnings,
  };
  writeFileSync(PERF_OUTPUT, `${JSON.stringify(evidence, null, 2)}\n`);
  console.log(`  Evidence: ${PERF_OUTPUT}`);
  console.log(`  Evidence: ${VISUAL_OUTPUT}`);

  assert.equal(probe.hit.parentParaIndex, 7, 'hit-test가 실제 parent paragraph[7]을 보존');
  assert.deepEqual(probe.hit.cellPath, EXPECTED_PATH, 'hit-test가 외부→래퍼→자식 표 전체 경로를 보존');
  assert.equal(probe.expectedLookup.error, null, '물리 5쪽 자식 표 bbox 조회 오류 없음');
  assert.ok(probe.expectedLookup.cellCount >= 55, `자식 표 bbox 55개 이상 (${probe.expectedLookup.cellCount})`);
  assert.equal(selection.selected, true, 'Esc로 자식 표 객체 선택 모드 진입');
  assert.deepEqual(selection.ref?.cellPath, EXPECTED_PATH, '선택 참조가 자식 표 전체 경로를 보존');
  assert.ok(selection.borders >= 1, `표 객체 선택 외곽선 표시 (${selection.borders})`);
  assert.ok(selection.handles >= 8, `표 객체 선택 핸들 표시 (${selection.handles})`);
  assert.equal(selection.renderCalls, 1, 'Esc 한 번에 표 객체 선택 렌더는 한 번만 실행');
  assert.deepEqual(warnings, [], `renderTableObjectSelection 경고 없음: ${warnings.join('\n')}`);
  assert.equal(parentExit.selected, false, '두 번째 Esc로 자식 표 선택 해제');
  assert.deepEqual(
    parentExit.position.cellPath,
    EXPECTED_PATH.slice(0, -1),
    '자식 표 선택 해제 후 부모 wrapper 셀 경로',
  );
  assert.ok(parentExit.rect, 'table-only 부모 문단에서도 caret rect 유지');
  assert.deepEqual(cursorWarnings, [], `CursorState updateRect 경고 없음: ${cursorWarnings.join('\n')}`);
});
