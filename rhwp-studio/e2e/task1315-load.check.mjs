/**
 * Task #1315 4단계 — roundtrip 산출 HWPX의 rhwp-studio 로드 확인 (임시 검증 스크립트)
 *
 * 대표 rt 파일 8건을 bytes 직접 주입(window.__wasm.loadDocument)으로 로드하고
 * 원본과 페이지 수를 비교한다. /samples fetch 제한과 무관하게 동작한다.
 * 호스트 Chrome CDP에 연결하여 작업지시자가 화면으로 직접 확인할 수 있다.
 *
 * 실행: CHROME_CDP=http://localhost:19222 node e2e/task1315-load.check.mjs
 */
import { readFileSync } from 'fs';
import puppeteer from 'puppeteer-core';

const VITE_URL = process.env.VITE_URL || 'http://localhost:7700';
const CHROME_CDP = process.env.CHROME_CDP || 'http://localhost:19222';

const REPO = '..';
const PAIRS = [
  ['blank_hwpx', `${REPO}/samples/hwpx/blank_hwpx.hwpx`, `${REPO}/output/poc/task1315/blank_hwpx.rt.hwpx`],
  ['para-001', `${REPO}/samples/hwpx/para-001.hwpx`, `${REPO}/output/poc/task1315/para-001.rt.hwpx`],
  ['basic-table-01', `${REPO}/samples/hwpx/basic-table-01.hwpx`, `${REPO}/output/poc/task1315/basic-table-01.rt.hwpx`],
  ['ta-pic-001-r', `${REPO}/samples/hwpx/ta-pic-001-r.hwpx`, `${REPO}/output/poc/task1315/ta-pic-001-r.rt.hwpx`],
  ['form-002', `${REPO}/samples/hwpx/form-002.hwpx`, `${REPO}/output/poc/task1315/form-002.rt.hwpx`],
  ['footnote-01', `${REPO}/samples/hwpx/footnote-01.hwpx`, `${REPO}/output/poc/task1315/footnote-01.rt.hwpx`],
  ['math-001', `${REPO}/samples/hwpx/math-001.hwpx`, `${REPO}/output/poc/task1315/math-001.rt.hwpx`],
  ['보도자료-2025-1q', `${REPO}/samples/hwpx/2025년 1분기 해외직접투자 보도자료f.hwpx`, `${REPO}/output/poc/task1315/2025년 1분기 해외직접투자 보도자료f.rt.hwpx`],
];

async function loadBytes(page, filePath, fname) {
  const b64 = readFileSync(filePath).toString('base64');
  return await page.evaluate(async ({ data, name }) => {
    try {
      const bin = atob(data);
      const bytes = new Uint8Array(bin.length);
      for (let i = 0; i < bin.length; i++) bytes[i] = bin.charCodeAt(i);
      const docInfo = window.__wasm?.loadDocument(bytes, name);
      if (!docInfo) return { error: 'loadDocument returned null' };
      window.__canvasView?.loadDocument?.();
      await new Promise(r => setTimeout(r, 1200));
      return { pageCount: docInfo.pageCount };
    } catch (e) {
      return { error: e.message || String(e) };
    }
  }, { data: b64, name: fname });
}

const browser = await puppeteer.connect({ browserURL: CHROME_CDP, defaultViewport: null });
const page = await browser.newPage();
let failures = 0;
try {
  await page.goto(VITE_URL, { waitUntil: 'networkidle0', timeout: 30000 });
  await page.waitForFunction(() => !!window.__wasm && !!window.__canvasView, { timeout: 15000 });

  console.log('name\torig_pages\trt_pages\trt_load');
  for (const [name, origPath, rtPath] of PAIRS) {
    const orig = await loadBytes(page, origPath, `${name}.hwpx`);
    const rt = await loadBytes(page, rtPath, `${name}.rt.hwpx`);
    const ok = !rt.error;
    if (!ok) failures++;
    console.log(`${name}\t${orig.error ? `ERR:${orig.error}` : orig.pageCount}\t${rt.error ? `ERR:${rt.error}` : rt.pageCount}\t${ok ? 'OK' : 'FAIL'}`);
  }
} finally {
  // 마지막 문서 화면을 작업지시자가 볼 수 있도록 탭은 닫지 않고 disconnect만 한다.
  browser.disconnect();
}
process.exit(failures > 0 ? 1 : 0);
