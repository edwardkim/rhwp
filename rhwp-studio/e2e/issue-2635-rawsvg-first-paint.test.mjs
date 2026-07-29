/**
 * E2E 회귀 — #2635: 순수 RawSvg 차트가 첫 화면에 늦게 표시되는 회귀
 *
 * 쪼개진원형은 embedded data:image 없이 순수 SVG로 구성된다. 따라서 일반
 * image prefetch 완료 신호에만 의존하면 1500ms fallback까지 차트가 비어 있다.
 * 이 검증은 helper의 기본 1500ms 안정화 대기를 우회해 400ms 안의 합성 화면을 측정한다.
 */
import { PNG } from 'pngjs';
import {
  assert,
  captureCanvasScreenshot,
  runTest,
  setTestCase,
} from './helpers.mjs';

const SAMPLE = 'chart/원형/쪼개진원형.hwp';
const OUT_PATH = '../output/e2e/issue-2635/rawsvg-first-paint.png';
const FIRST_PAINT_LIMIT_MS = 400;

function coloredPixelRatio(buffer) {
  const png = PNG.sync.read(buffer);
  const total = png.width * png.height;
  let colored = 0;
  for (let i = 0; i < png.data.length; i += 4) {
    const r = png.data[i];
    const g = png.data[i + 1];
    const b = png.data[i + 2];
    const a = png.data[i + 3];
    if (a < 16) continue;
    const max = Math.max(r, g, b);
    const min = Math.min(r, g, b);
    if (max - min > 40 && max > 40) colored += 1;
  }
  return colored / total;
}

async function loadWithoutSettleDelay(page) {
  return page.evaluate(async (filename) => {
    const response = await fetch(`/samples/${filename.split('/').map(encodeURIComponent).join('/')}`);
    if (!response.ok) throw new Error(`샘플 fetch 실패: HTTP ${response.status}`);
    const bytes = new Uint8Array(await response.arrayBuffer());
    const startedAt = performance.now();
    const info = window.__wasm.loadDocument(bytes, filename);
    await window.__canvasView.loadDocument();
    return {
      pageCount: info.pageCount,
      documentLoadAndInitialRenderMs: performance.now() - startedAt,
    };
  }, SAMPLE);
}

runTest('#2635 순수 RawSvg 차트 조기 첫 화면 표시', async ({ page }) => {
  setTestCase('쪼개진원형 400ms 이내 첫 화면 표시');
  const loaded = await loadWithoutSettleDelay(page);
  assert(loaded.pageCount === 1, `쪼개진원형 페이지 수(${loaded.pageCount}) == 1`);
  await page.waitForSelector('#scroll-container canvas', { timeout: 10000 });
  await page.evaluate((delay) => new Promise((resolve) => setTimeout(resolve, delay)), FIRST_PAINT_LIMIT_MS);

  const { buffer } = await captureCanvasScreenshot(
    page,
    OUT_PATH,
    'RawSvg first-paint composite',
    '#scroll-container',
  );
  const ratio = coloredPixelRatio(buffer);
  assert(
    ratio > 0.003,
    `RawSvg 차트 유채색 픽셀 ${(ratio * 100).toFixed(3)}% > 0.300% (${FIRST_PAINT_LIMIT_MS}ms 이내)`,
  );
});
