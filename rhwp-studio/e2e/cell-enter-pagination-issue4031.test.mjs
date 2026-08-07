/**
 * Issue #4031 — pending pagination 중 셀 Enter의 중복 full pagination 제거 계약.
 *
 * 115쪽 거대 표 셀(HWP/HWPX)에서:
 *  1. deferred insert로 pending pagination을 만든 뒤 direct Enter를 실키로 보낸다.
 *     admitted 경로 계약: pre-navigation `wasm.flushDeferredPagination` 0회,
 *     `splitParagraphInCell` 1회, 이후 pending 해소·caret은 새 문단 시작.
 *  2. 사후 flush oracle: split의 동기 pagination이 최신이므로 page count 불변.
 *  3. barrier 대조군: pending 중 ArrowDown은 기존 full flush 1회를 유지한다.
 *
 * 실행 (repo root에서 wasm-pack build 후):
 *   cd rhwp-studio && npm run e2e:issue-4031-cell-enter
 */

import assert from 'node:assert/strict';
import crypto from 'node:crypto';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import {
  closeBrowser,
  closePage,
  createPage,
  launchBrowser,
  loadApp,
  setTestCase,
} from './helpers.mjs';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const REPO_ROOT = path.resolve(__dirname, '../..');

const TARGET = Object.freeze({
  sectionIndex: 0,
  paragraphIndex: 5,
  charOffset: 130,
  parentParaIndex: 0,
  controlIndex: 2,
  cellIndex: 2,
  cellParaIndex: 5,
  cellPath: [{ controlIndex: 2, cellIndex: 2, cellParaIndex: 5 }],
});

const SAMPLES = Object.freeze({
  hwp: path.join(REPO_ROOT, 'samples/issue1949_giant_cell_nested_tables_perf.hwp'),
  hwpx: path.join(REPO_ROOT, 'samples/issue1949_giant_cell_nested_tables_perf.hwpx'),
});

function waitTwoRafs(page) {
  return page.evaluate(
    () =>
      new Promise((resolve) => {
        requestAnimationFrame(() => requestAnimationFrame(() => resolve()));
      }),
  );
}

function clickBlockingModalChoice(page) {
  return page.evaluate(() => {
    const allowed = new Set(['그대로 보기', '대체 글꼴로 보기']);
    const buttons = Array.from(document.querySelectorAll('button'));
    const button = buttons.find((candidate) => {
      const label = candidate.textContent?.trim() ?? '';
      const style = getComputedStyle(candidate);
      return allowed.has(label) && style.display !== 'none' && style.visibility !== 'hidden';
    });
    if (!button) return null;
    const label = button.textContent?.trim() ?? '';
    button.click();
    return label;
  });
}

async function openDocumentThroughApp(page, format) {
  const bytes = readFileSync(SAMPLES[format]);
  const fileName = path.basename(SAMPLES[format]);
  const encoded = bytes.toString('base64');
  const requestId = `issue4031-${format}-${crypto.randomUUID()}`;

  await page.evaluate(({ base64, name, id }) => {
    const binary = atob(base64);
    const payload = new Uint8Array(binary.length);
    for (let index = 0; index < binary.length; index += 1) {
      payload[index] = binary.charCodeAt(index);
    }
    window.__issue4031LoadResult = null;
    const off = window.__eventBus.on('open-document-bytes:done', (result) => {
      if (result?.requestId !== id) return;
      off();
      window.__issue4031LoadResult = result;
    });
    window.__eventBus.emit('open-document-bytes', {
      bytes: payload,
      fileName: name,
      fileHandle: null,
      skipUnsavedGuard: true,
      requestId: id,
    });
  }, { base64: encoded, name: fileName, id: requestId });

  const deadline = Date.now() + 90_000;
  let result = null;
  while (Date.now() < deadline) {
    await clickBlockingModalChoice(page);
    result = await page.evaluate(() => window.__issue4031LoadResult);
    if (result) break;
    await new Promise((resolve) => setTimeout(resolve, 100));
  }
  assert.ok(result, `${format}: open-document-bytes:done timeout`);
  assert.equal(result.ok, true, `${format}: document load failed: ${result.error ?? 'unknown'}`);
  await page.evaluate(() => document.fonts.ready);
  await waitTwoRafs(page);

  const pageCount = await page.evaluate(() => window.__wasm.pageCount);
  assert.equal(pageCount, 115, `${format}: expected 115 pages`);
}

async function moveToTarget(page, target = TARGET) {
  const state = await page.evaluate((pos) => {
    const input = window.__inputHandler;
    input.cursor.clearSelection();
    input.cursor.moveTo(pos);
    input.cursor.resetPreferredX();
    input.updateCaret();
    input.focus();
    return {
      position: input.cursor.getPosition(),
      focused: document.activeElement === input.textarea,
    };
  }, target);
  assert.equal(state.focused, true, 'hidden textarea was not focused');
  assert.equal(state.position.cellParaIndex, target.cellParaIndex, 'target cell paragraph mismatch');
  return state;
}

async function installCounters(page) {
  await page.evaluate(() => {
    const wasm = window.__wasm;
    const counts = { flush: 0, cancel: 0, split: 0, splitByPath: 0 };
    window.__issue4031Counts = counts;
    if (!window.__issue4031Wrapped) {
      window.__issue4031Wrapped = true;
      for (const [method, key] of [
        ['flushDeferredPagination', 'flush'],
        ['cancelDeferredPagination', 'cancel'],
        ['splitParagraphInCell', 'split'],
        ['splitParagraphInCellByPath', 'splitByPath'],
      ]) {
        const original = wasm[method].bind(wasm);
        wasm[method] = (...args) => {
          window.__issue4031Counts[key] += 1;
          return original(...args);
        };
      }
    }
  });
}

function resetCounters(page) {
  return page.evaluate(() => {
    window.__issue4031Counts = { flush: 0, cancel: 0, split: 0, splitByPath: 0 };
  });
}

function readState(page) {
  return page.evaluate((target) => {
    const wasm = window.__wasm;
    const input = window.__inputHandler;
    return {
      counts: { ...window.__issue4031Counts },
      pending: input.hasDeferredPaginationPending(),
      pageCount: wasm.pageCount,
      cursor: input.cursor.getPosition(),
      cellParaCount: wasm.getCellParagraphCount(
        target.sectionIndex,
        target.parentParaIndex,
        target.controlIndex,
        target.cellIndex,
      ),
      targetParaLength: wasm.getCellParagraphLength(
        target.sectionIndex,
        target.parentParaIndex,
        target.controlIndex,
        target.cellIndex,
        target.cellParaIndex,
      ),
    };
  }, TARGET);
}

async function runFormat(browser, format) {
  setTestCase(`issue4031-${format}`);
  const page = await createPage(browser, 1280, 900);
  try {
    await loadApp(page);
    await openDocumentThroughApp(page, format);
    await moveToTarget(page);
    await installCounters(page);

    const before = await readState(page);
    assert.equal(before.pending, false, `${format}: unexpected pending before typing`);

    // 1) deferred insert 3회로 pending pagination을 만든다 (실키 경로).
    await page.keyboard.type('111', { delay: 30 });
    await waitTwoRafs(page);
    const pendingState = await readState(page);
    assert.equal(pendingState.pending, true, `${format}: typing did not defer pagination`);
    assert.equal(
      pendingState.targetParaLength,
      before.targetParaLength + 3,
      `${format}: deferred inserts missing from model`,
    );

    // 2) direct Enter — admitted 경로 계약.
    await resetCounters(page);
    const enterStarted = Date.now();
    await page.keyboard.press('Enter');
    await waitTwoRafs(page);
    const enterElapsedMs = Date.now() - enterStarted;
    const after = await readState(page);

    assert.equal(
      after.counts.flush,
      0,
      `${format}: admitted cell Enter must not run pre-navigation full flush`,
    );
    assert.equal(
      after.counts.split + after.counts.splitByPath,
      1,
      `${format}: exactly one cell paragraph split must run`,
    );
    assert.equal(after.pending, false, `${format}: pending must be cleared by owned completion`);
    assert.equal(
      after.cellParaCount,
      before.cellParaCount + 1,
      `${format}: cell paragraph count must grow by one`,
    );
    assert.equal(
      after.targetParaLength,
      TARGET.charOffset + 3,
      `${format}: split point must sit after the three inserted digits`,
    );
    assert.equal(
      after.cursor.cellParaIndex,
      TARGET.cellParaIndex + 1,
      `${format}: caret must land on the new paragraph`,
    );
    assert.equal(after.cursor.charOffset, 0, `${format}: caret must land at offset 0`);

    // 3) 사후 flush oracle — split이 최신 revision을 계산했으므로 page count 불변.
    const oracle = await page.evaluate(() => {
      const result = window.__wasm.flushDeferredPagination();
      return { status: result.status, pageCount: window.__wasm.pageCount };
    });
    assert.equal(
      oracle.pageCount,
      after.pageCount,
      `${format}: post-split flush must not change page count`,
    );

    // 4) barrier 대조군 — pending 중 ArrowDown은 기존 full flush를 유지한다.
    await page.keyboard.type('1', { delay: 30 });
    await waitTwoRafs(page);
    const navPending = await page.evaluate(() =>
      window.__inputHandler.hasDeferredPaginationPending(),
    );
    assert.equal(navPending, true, `${format}: control typing did not defer pagination`);
    await resetCounters(page);
    await page.keyboard.press('ArrowDown');
    await waitTwoRafs(page);
    const navState = await readState(page);
    assert.equal(
      navState.counts.flush,
      1,
      `${format}: non-Enter boundary key must keep the full-flush barrier`,
    );
    assert.equal(navState.pending, false, `${format}: barrier flush must clear pending`);

    console.log(
      `[issue4031] ${format}: enter_elapsed_ms=${enterElapsedMs} ` +
        `flush=0 split=1 pages=${after.pageCount} oracle=${oracle.status} barrier_flush=1 — OK`,
    );
  } finally {
    await closePage(page);
  }
}

async function main() {
  const browser = await launchBrowser();
  try {
    for (const format of ['hwp', 'hwpx']) {
      await runFormat(browser, format);
    }
    console.log('[issue4031] all formats passed');
  } finally {
    await closeBrowser(browser);
  }
}

main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
