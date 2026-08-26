/**
 * E2E 테스트 — 상태 표시줄 쪽 표시가 문서 쪽번호를 따른다 (#5749)
 *
 * 검증 항목:
 * 1. 새 번호가 없는 문서는 물리 순번과 같은 숫자를 보여준다(회귀 없음)
 * 2. `쪽 > 새 번호로 시작`(NewNumber)을 넣으면 상태 표시줄이 그 번호를 따른다
 * 3. 전체 쪽수(분모)는 물리 쪽수를 유지한다
 */

import { runTest, loadHwpFile, assert } from './helpers.mjs';

process.env.VITE_URL = process.env.VITE_URL || 'http://localhost:7700';

const SAMPLE = '쪽기준.hwp';

const readStatus = (pageIndex) => {
  window.__eventBus.emit('current-page-changed', pageIndex, window.__wasm.pageCount);
  const info = window.__wasm.getPageInfo(pageIndex);
  return {
    pageNumber: info.pageNumber,
    statusBar: document.getElementById('sb-page').textContent,
    pageCount: window.__wasm.pageCount,
  };
};

runTest('상태 표시줄 쪽 표시', async ({ page }) => {
  await loadHwpFile(page, SAMPLE);

  // ── TC1: 새 번호가 없으면 물리 순번과 같다 ─────────────────
  const before = await page.evaluate(readStatus, 0);
  assert(before.pageNumber === 1,
    `TC1: 첫 쪽의 문서 쪽번호는 1 (${before.pageNumber})`);
  assert(before.statusBar === `1 / ${before.pageCount} 쪽`,
    `TC1: 상태 표시줄이 물리 순번과 같음 (${before.statusBar})`);

  // ── TC2: 새 번호로 시작을 넣으면 그 번호를 따른다 ──────────
  const inserted = await page.evaluate(() => {
    const res = window.__wasm.insertNewNumber(0, 0, 0, 7);
    return typeof res === 'string' ? res.slice(0, 80) : String(res);
  });
  assert(inserted.includes('"ok":true'), `TC2: 새 번호 삽입 (${inserted})`);

  const after = await page.evaluate(readStatus, 0);
  assert(after.pageNumber === 7,
    `TC2: 문서 쪽번호가 7로 다시 시작 (${after.pageNumber})`);
  assert(after.statusBar.startsWith('7 / '),
    `TC2: 상태 표시줄이 문서 쪽번호를 따름 (${after.statusBar})`);

  // ── TC3: 분모는 물리 쪽수 ──────────────────────────────────
  assert(after.statusBar === `7 / ${after.pageCount} 쪽`,
    `TC3: 전체 쪽수는 물리 쪽수 유지 (${after.statusBar}, pageCount=${after.pageCount})`);
});
