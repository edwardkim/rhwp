#!/usr/bin/env node

/**
 * Issue #3820 — Hancom 2020 PDF와 같은 383 physical pages를 HWP/HWPX 모두
 * Studio WASM 경로에서 유지한다.
 *
 * 실행 (repo root에서 fresh WASM build 후):
 *   wasm-pack build --target web --out-dir pkg
 *   cd rhwp-studio && npm run e2e:issue-3820
 */

import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { assert, runTest, setTestCase, waitForCanvas } from './helpers.mjs';

const E2E_DIR = path.dirname(fileURLToPath(import.meta.url));
const REPO_ROOT = path.resolve(E2E_DIR, '..', '..');
const EXPECTED_PAGES = 383;
const FIXTURES = Object.freeze([
  {
    format: 'hwp',
    file: path.join(REPO_ROOT, 'samples', '2025 행정업무운영 편람(최종).hwp'),
    ownerPage: 284,
    ownerText: '홈페이지상의 질의에 대하여',
  },
  {
    format: 'hwpx',
    file: path.join(REPO_ROOT, 'samples', '2025 행정업무운영 편람(최종).hwpx'),
    ownerPage: 143,
    ownerText: '기안문에 작성한 붙임 문서를 첨부',
  },
]);

async function dismissFontDialog(page) {
  await page.evaluate(() => {
    const labels = new Set(['그대로 보기', '대체 글꼴로 보기']);
    const button = Array.from(document.querySelectorAll('button')).find((candidate) =>
      labels.has(candidate.textContent?.trim())
      && getComputedStyle(candidate).display !== 'none'
      && getComputedStyle(candidate).visibility !== 'hidden');
    button?.click();
  });
}

runTest('Issue #3820 HWP/HWPX 383쪽 WASM 페이지 계약', async ({ page }) => {
  for (const fixture of FIXTURES) {
    setTestCase(`#3820 ${fixture.format.toUpperCase()} 383쪽 및 page owner`);
    const input = await page.$('#file-input');
    if (!input) throw new Error('file-input not found');
    await input.uploadFile(fixture.file);
    await page.waitForFunction(
      ({ format, expectedPages }) =>
        window.__wasm?.getSourceFormat?.() === format
        && window.__wasm.pageCount === expectedPages,
      { timeout: 90_000 },
      { format: fixture.format, expectedPages: EXPECTED_PAGES },
    );
    await dismissFontDialog(page);
    await waitForCanvas(page, 30_000);

    const result = await page.evaluate(({ pageIndex, ownerText }) => ({
      pageCount: window.__wasm?.pageCount,
      ownerTree: window.__wasm?.doc?.getPageLayerTree?.(pageIndex) ?? '',
      renderer: window.__canvasView?.getRenderBackend?.(),
      ownerText,
    }), { pageIndex: fixture.ownerPage, ownerText: fixture.ownerText });

    assert(result.pageCount === EXPECTED_PAGES, `${fixture.format}: ${EXPECTED_PAGES}쪽`);
    assert(
      result.ownerTree.includes(fixture.ownerText),
      `${fixture.format}: p${fixture.ownerPage + 1}가 '${fixture.ownerText}'를 소유`,
    );
    console.log(
      `[issue3820] ${fixture.format}: pages=${result.pageCount} `
        + `owner=p${fixture.ownerPage + 1} renderer=${result.renderer ?? 'unknown'} — OK`,
    );
  }
});
