/**
 * E2E 테스트 — 문서를 바꿔 열면 눈금자가 새 문서를 그린다
 *
 * 종전에는 문서 로드 뒤 눈금자를 깨우는 이벤트가 `cursor-para-changed` 하나뿐이었고,
 * 그 핸들러는 문단 여백·들여쓰기가 이전 문서와 같으면 그리기를 건너뛴다. 그래서 여백이
 * 같은 문서를 잇달아 열면 눈금자가 한 번도 다시 그려지지 않고 앞 문서의(또는 문서를 열기
 * 전 빈) 눈금이 그대로 남았다. CanvasView 가 `document-view-loaded` 로 알리고 눈금자가
 * 그 알림에 크기·눈금을 다시 잡는 것을 확인한다.
 *
 * 검증 항목:
 * 1. 세로 A4 문서를 연 뒤 가로 A4 문서를 열면 눈금자가 새 쪽 폭을 그린다
 * 2. 앞 문서의 눈금이 그대로 남아 있지 않다
 */

import { runTest, assert } from './helpers.mjs';

process.env.VITE_URL = process.env.VITE_URL || 'http://localhost:7700';

const PORTRAIT_DOC = '누름틀-2024.hwp';            // A4 세로 — 쪽 폭 794px
const LANDSCAPE_DOC = '253E164F57A1BC6934-empty.hwp'; // A4 가로 — 쪽 폭 1123px

/** 사용자 경로(loadBytes → initializeDocument)로 문서를 연다. */
async function openDocument(page, filename) {
  const result = await page.evaluate(async (fname) => {
    const resp = await fetch(`/samples/${encodeURIComponent(fname)}`);
    if (!resp.ok) return { error: `HTTP ${resp.status}` };
    const bytes = new Uint8Array(await resp.arrayBuffer());
    const requestId = `ruler-switch-${Math.random().toString(36).slice(2)}`;
    const done = new Promise((resolve) => {
      const off = window.__eventBus.on('open-document-bytes:done', (payload) => {
        if (payload.requestId !== requestId) return;
        off();
        resolve(payload);
      });
    });
    window.__eventBus.emit('open-document-bytes', {
      bytes, fileName: fname, fileHandle: null, skipUnsavedGuard: true, requestId,
    });
    const outcome = await done;
    await new Promise((r) => setTimeout(r, 1500));
    return outcome;
  }, filename);
  if (result.error || result.ok === false) {
    throw new Error(`문서 열기 실패 (${filename}): ${result.error ?? '알 수 없음'}`);
  }
}

/** 가로 눈금자 가운데 줄에서 용지가 차지한 구간의 좌우 끝을 읽는다. */
const readRulerPaperBand = () => {
  const canvas = document.getElementById('h-ruler');
  const dpr = window.devicePixelRatio || 1;
  const row = Math.floor(canvas.height / 2);
  const data = canvas.getContext('2d').getImageData(0, row, canvas.width, 1).data;
  const colorAt = (x) => `${data[x * 4]},${data[x * 4 + 1]},${data[x * 4 + 2]}`;
  const outside = colorAt(0); // 용지 밖은 언제나 여백 톤이다
  let left = -1;
  let right = -1;
  for (let x = 0; x < canvas.width; x++) {
    if (colorAt(x) !== outside) { left = x; break; }
  }
  for (let x = canvas.width - 1; x >= 0; x--) {
    if (colorAt(x) !== outside) { right = x; break; }
  }
  const pageInfo = window.__wasm.getPageInfo(0);
  return {
    bandWidth: left < 0 ? 0 : Math.round((right - left) / dpr),
    pageWidth: pageInfo.width,
    zoom: window.__canvasView.getViewportManager().getZoom(),
  };
};

runTest('문서 전환 시 눈금자 갱신', async ({ page }) => {
  // 첫 실행 스킨 안내 모달이 상태 표시줄 클릭을 삼킨다 — 선택을 마친 상태로 시작한다.
  await page.evaluate(() => {
    localStorage.setItem('rhwp-settings', JSON.stringify({
      version: 1,
      theme: { mode: 'system', skin: 'default', skinChosen: true },
    }));
  });
  await page.reload({ waitUntil: 'networkidle0' });
  await page.evaluate(() => new Promise(r => setTimeout(r, 1500)));

  await openDocument(page, PORTRAIT_DOC);
  const portrait = await page.evaluate(readRulerPaperBand);
  assert(portrait.bandWidth > 0, `TC0: 첫 문서에서 눈금자가 용지를 그림 (${portrait.bandWidth}px)`);

  await openDocument(page, LANDSCAPE_DOC);
  const landscape = await page.evaluate(readRulerPaperBand);

  const expected = landscape.pageWidth * landscape.zoom;
  assert(Math.abs(landscape.bandWidth - expected) <= 30,
    `TC1: 눈금자가 새 문서의 쪽 폭을 그림 (${landscape.bandWidth}px vs 기대 ${Math.round(expected)}px)`);
  assert(landscape.bandWidth !== portrait.bandWidth,
    `TC2: 앞 문서의 눈금이 남지 않음 (앞 ${portrait.bandWidth}px → 지금 ${landscape.bandWidth}px)`);
});
