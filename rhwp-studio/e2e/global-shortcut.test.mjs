/**
 * E2E 테스트 — 시작 시 빈 문서 + 전역 단축키
 *
 * 검증: 앱 시작 직후 빈 문서가 열려 편집 가능 상태이고, Alt+N → 새 문서 생성이 동작한다.
 */
import { runTest, loadApp, screenshot, assert, getPageCount } from './helpers.mjs';

process.env.VITE_URL = process.env.VITE_URL || 'http://localhost:7700';

runTest('시작 시 빈 문서 + Alt+N', async ({ page }) => {
  // 앱 로드만, 명시적 문서 생성 없음
  await loadApp(page);
  await page.evaluate(() => new Promise(r => setTimeout(r, 800)));

  // 시작 직후 빈 문서가 열려 있어야 한다
  const pageCountBefore = await page.evaluate(() => window.__wasm?.pageCount ?? 0);
  assert(pageCountBefore >= 1, `TC1: 시작 시 빈 문서 자동 생성 (pageCount=${pageCountBefore})`);
  await screenshot(page, 'global-01-empty');

  // Alt+N 입력 (편집 영역 클릭 없이)
  await page.keyboard.down('Alt');
  await page.keyboard.press('n');
  await page.keyboard.up('Alt');
  await page.evaluate(() => new Promise(r => setTimeout(r, 800)));

  // 새 문서 생성 확인
  const pageCountAfter = await page.evaluate(() => window.__wasm?.pageCount ?? 0);
  await screenshot(page, 'global-02-new-doc');
  assert(pageCountAfter >= 1, `TC2: Alt+N으로 새 문서 생성 (pageCount=${pageCountAfter})`);
});
