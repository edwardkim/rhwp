/**
 * E2E 회귀 — #1456: rhwp-studio 캔버스 차트/OLE(rawSvg) 비동기 디코드 재렌더 안전망
 *
 * 배경: studio 캔버스는 차트/OLE 를 rawSvg PaintOp → web_canvas draw_image(비동기
 * HtmlImageElement)로 래스터화한다. 캐시 미스 시 첫(동기) 렌더는 공백이고, #1181 의
 * 재렌더 안전망(scheduleReRender)은 트리거 카운트(imageCount) > 0 일 때만 작동했다.
 * 그런데 카운트 산정(collectLayerPlaneSummary)이 rawSvg 를 누락 → 차트/OLE 단독
 * 페이지는 즉시 bail → 공백 고착. #1456 에서 rawSvgCount 를 트리거에 반영해 해소.
 *
 * 검증:
 *   케이스 A (비공백): 새 세션에서 차트 HWP 로드 → 캔버스가 유채색 픽셀을 가짐(비공백).
 *     수정 전이면 rawSvg 단독 페이지가 재렌더 트리거 bail → 공백(유채색 ~0) → FAIL.
 *   케이스 B (비재사용/고착): 같은 세션에서 다른 차트로 교체 로드 → 두 캔버스가
 *     유의미하게 다름(B 가 A 의 캐시 픽셀을 재사용하지 않음). #3(CanvasPool) 판정 입력.
 *
 * 실행:
 *   cd rhwp-studio
 *   npx vite --host 127.0.0.1 --port 7700 &
 *   # macOS: CHROME_PATH 지정 필요
 *   CHROME_PATH="/Applications/Google Chrome.app/Contents/MacOS/Google Chrome" \
 *     node e2e/issue-1456-chart-rerender.test.mjs --mode=headless
 */
import { PNG } from 'pngjs';
import {
  runTest,
  loadHwpFile,
  captureCanvasScreenshot,
  assert,
  setTestCase,
} from './helpers.mjs';

const CHART_A = 'chart/세로막대형/묶은세로막대형.hwp';
const CHART_B = 'chart/원형/2차원원형.hwp';
const OUT_DIR = '../output/e2e/issue-1456';

/** 재렌더 타이머(200/600/1500ms) + prefetch 마이크로태스크 발화를 위한 여유 대기 */
async function waitReRender(page, ms = 2200) {
  await page.evaluate((d) => new Promise((r) => setTimeout(r, d)), ms);
}

/** 유채색(채도 충분) 픽셀 비율 — 차트의 막대/라인/조각 색 검출. 흑백 텍스트·백지 배경 제외 */
function coloredPixelRatio(buffer) {
  const png = PNG.sync.read(buffer);
  const total = png.width * png.height;
  if (total === 0) return 0;
  let colored = 0;
  for (let i = 0; i < png.data.length; i += 4) {
    const r = png.data[i];
    const g = png.data[i + 1];
    const b = png.data[i + 2];
    const a = png.data[i + 3];
    if (a < 16) continue; // 투명 배경
    const max = Math.max(r, g, b);
    const min = Math.min(r, g, b);
    if (max - min > 40 && max > 40) colored += 1; // 채도 + 비백 → 유채색
  }
  return colored / total;
}

/** 두 캔버스 PNG 의 픽셀 차이 비율(크기 다르면 명백히 다름=1) */
function pixelDiffRatio(bufA, bufB) {
  const a = PNG.sync.read(bufA);
  const b = PNG.sync.read(bufB);
  if (a.width !== b.width || a.height !== b.height) return 1;
  const total = a.width * a.height;
  if (total === 0) return 0;
  let diff = 0;
  for (let i = 0; i < a.data.length; i += 4) {
    const d =
      Math.abs(a.data[i] - b.data[i]) +
      Math.abs(a.data[i + 1] - b.data[i + 1]) +
      Math.abs(a.data[i + 2] - b.data[i + 2]);
    if (d > 30) diff += 1;
  }
  return diff / total;
}

runTest('#1456 차트/OLE rawSvg 첫 로드 재렌더', async ({ page }) => {
  // ── 케이스 A: 새 세션 차트 HWP 첫 로드 → 비공백 ──────────────
  setTestCase('A: 차트 HWP 첫 로드 비공백');
  const a = await loadHwpFile(page, CHART_A);
  assert(a.pageCount >= 1, `차트 A 로드(pageCount=${a.pageCount})`);
  await waitReRender(page);
  const { buffer: bufA } = await captureCanvasScreenshot(page, `${OUT_DIR}/chartA.png`, 'chart A');
  const ratioA = coloredPixelRatio(bufA);
  assert(ratioA > 0.003, `차트 A 유채색 픽셀 ${(ratioA * 100).toFixed(3)}% > 0.3% (비공백 렌더)`);

  // ── 케이스 B: 같은 세션 다른 차트 교체 로드 → 비재사용 ───────
  setTestCase('B: 다른 차트 비재사용(고착 검증)');
  const b = await loadHwpFile(page, CHART_B);
  assert(b.pageCount >= 1, `차트 B 로드(pageCount=${b.pageCount})`);
  await waitReRender(page);
  const { buffer: bufB } = await captureCanvasScreenshot(page, `${OUT_DIR}/chartB.png`, 'chart B');
  const ratioB = coloredPixelRatio(bufB);
  assert(ratioB > 0.003, `차트 B 유채색 픽셀 ${(ratioB * 100).toFixed(3)}% > 0.3% (비공백 렌더)`);

  const diff = pixelDiffRatio(bufA, bufB);
  assert(diff > 0.02, `차트 B↔A 픽셀 차이 ${(diff * 100).toFixed(2)}% > 2% (B가 A 캐시 미재사용)`);
});
