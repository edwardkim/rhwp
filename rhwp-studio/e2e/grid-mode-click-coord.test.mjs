/**
 * 보류 ① 그리드 좌표 결함 — 정량 e2e 측정
 *
 * 본질: zoom ≤ 0.5 그리드 모드에서 input-handler-mouse.ts 의 14곳이
 * `pageLeft = (scrollContent.clientWidth - pageDisplayWidth) / 2` 단일 컬럼 가정 사용.
 * 실제 페이지 element 의 left 는 `pageLefts[i]` (canvas-view.ts:158).
 * 두 값의 차이가 click 좌표 어긋남 정도.
 *
 * 본 측정은:
 *   1. 그리드 모드 활성 여부 확인 (zoom=0.5, multi-page)
 *   2. 각 페이지 (col 0/1/2/...) 의 correct vs buggy pageLeft 값 비교
 *   3. delta_px (CSS px 단위) 정량화
 *   4. 실제 click 시 cursor 위치 어긋남 검증 (correct 좌표로 click → 정상, buggy 좌표로 click → 어긋남)
 *
 * 실행:
 *   cd rhwp-studio
 *   npx vite --host 0.0.0.0 --port 7700 &
 *   node e2e/grid-mode-click-coord.test.mjs --mode=headless
 */
import { runTest, loadHwpFile, screenshot } from './helpers.mjs';

async function dumpGridState(page, label) {
  console.log(`\n=== ${label} ===`);
  const state = await page.evaluate(() => {
    const sc = document.querySelector('#scroll-content');
    const ih = window.__inputHandler;
    const vs = ih.virtualScroll;
    const vm = ih.viewportManager;
    const zoom = vm.getZoom();
    const isGrid = vs.isGridMode();
    const columns = vs.getColumns();
    const pageCount = vs.pageCount;
    const clientWidth = sc.clientWidth;
    const rows = [];
    for (let i = 0; i < pageCount; i++) {
      const correct = vs.getPageLeft(i);
      const pw = vs.getPageWidth(i);
      const buggy = (clientWidth - pw) / 2;
      const col = i % Math.max(columns, 1);
      const delta = correct >= 0 ? (buggy - correct) : 0;
      rows.push({ i, col, pw, correct, buggy, delta });
    }
    return { zoom, isGrid, columns, pageCount, clientWidth, rows };
  });

  console.log(`  zoom=${state.zoom}  grid=${state.isGrid}  columns=${state.columns}  pageCount=${state.pageCount}  clientWidth=${state.clientWidth}`);
  console.log(`  | i  | col | pw     | correct(pageLefts[i]) | buggy(formula) | delta_px |`);
  console.log(`  |----|-----|--------|-----------------------|----------------|----------|`);
  for (const r of state.rows.slice(0, 8)) {  // 첫 8 페이지만 출력
    console.log(`  | ${String(r.i).padEnd(2)} | ${String(r.col).padEnd(3)} | ${r.pw.toFixed(1).padEnd(6)} | ${r.correct.toFixed(1).padEnd(21)} | ${r.buggy.toFixed(1).padEnd(14)} | ${r.delta.toFixed(1).padEnd(8)} |`);
  }
  return state;
}

async function probeClickAtPage(page, label, pageIdx, hwpX, hwpY) {
  console.log(`\n--- ${label} (page ${pageIdx}, hwpX=${hwpX}, hwpY=${hwpY}) ---`);

  // correct 좌표 + buggy 좌표 둘 다 계산
  const probe = await page.evaluate(({ pageIdx, hwpX, hwpY }) => {
    const sc = document.querySelector('#scroll-content');
    const ih = window.__inputHandler;
    const vs = ih.virtualScroll;
    const vm = ih.viewportManager;
    const zoom = vm.getZoom();
    const pw = vs.getPageWidth(pageIdx);
    const po = vs.getPageOffset(pageIdx);

    // CORRECT: pageLefts[i] 사용 (실제 페이지 element 위치)
    const correctLeft = vs.getPageLeft(pageIdx);
    const correctLeftDOM = correctLeft >= 0 ? correctLeft : (sc.clientWidth - pw) / 2;
    const correctDocX = correctLeftDOM + hwpX * zoom;
    const correctDocY = po + hwpY * zoom;

    // BUGGY: (clientWidth - pageDisplayWidth) / 2 (input-handler-mouse 의 가정)
    const buggyLeft = (sc.clientWidth - pw) / 2;
    const buggyDocX = buggyLeft + hwpX * zoom;
    const buggyDocY = po + hwpY * zoom;

    // 스크롤 안정화
    const scroller = sc.parentElement;
    scroller.scrollTop = Math.max(0, correctDocY - 200);

    return {
      zoom, pw, po, correctLeft, correctLeftDOM, buggyLeft,
      correctDocX, correctDocY, buggyDocX, buggyDocY,
      delta_x: buggyDocX - correctDocX,
    };
  }, { pageIdx, hwpX, hwpY });

  await page.evaluate(() => new Promise(r => setTimeout(r, 400)));

  // CORRECT 좌표로 click 후 cursor 위치 확인
  const correctClick = await page.evaluate(({ correctDocX, correctDocY }) => {
    const sc = document.querySelector('#scroll-content');
    const cr = sc.getBoundingClientRect();
    return { clientX: cr.left + correctDocX, clientY: cr.top + correctDocY };
  }, probe);

  await page.mouse.click(correctClick.clientX, correctClick.clientY);
  await page.evaluate(() => new Promise(r => setTimeout(r, 250)));

  const afterCorrectClick = await page.evaluate(() => {
    const ih = window.__inputHandler;
    const cur = ih?.cursor;
    const pos = cur?.getPosition?.();
    const rect = cur?.getRect?.();
    return {
      pos: pos ? { sec: pos.sectionIndex, para: pos.paragraphIndex, char: pos.charOffset } : null,
      rectPageIdx: rect?.pageIndex ?? null,
      rectX: rect?.x ?? null,
      rectY: rect?.y ?? null,
    };
  });

  // BUGGY 좌표로 click 후 cursor 위치 확인 (input-handler-mouse 의 현재 동작 모사)
  await page.mouse.click(correctClick.clientX + probe.delta_x, correctClick.clientY);
  await page.evaluate(() => new Promise(r => setTimeout(r, 250)));

  const afterBuggyClick = await page.evaluate(() => {
    const ih = window.__inputHandler;
    const cur = ih?.cursor;
    const pos = cur?.getPosition?.();
    const rect = cur?.getRect?.();
    return {
      pos: pos ? { sec: pos.sectionIndex, para: pos.paragraphIndex, char: pos.charOffset } : null,
      rectPageIdx: rect?.pageIndex ?? null,
      rectX: rect?.x ?? null,
      rectY: rect?.y ?? null,
    };
  });

  console.log(`  zoom=${probe.zoom} pw=${probe.pw.toFixed(1)} po=${probe.po.toFixed(1)}`);
  console.log(`  correctLeft=${probe.correctLeft} correctLeftDOM=${probe.correctLeftDOM.toFixed(1)} buggyLeft=${probe.buggyLeft.toFixed(1)}`);
  console.log(`  delta_x = ${probe.delta_x.toFixed(1)} px (CSS px, click 어긋남 정도)`);
  console.log(`  CORRECT click @(${correctClick.clientX.toFixed(1)}, ${correctClick.clientY.toFixed(1)}) → pos=${JSON.stringify(afterCorrectClick.pos)} rectPage=${afterCorrectClick.rectPageIdx}`);
  console.log(`  BUGGY  click @(${(correctClick.clientX + probe.delta_x).toFixed(1)}, ${correctClick.clientY.toFixed(1)}) → pos=${JSON.stringify(afterBuggyClick.pos)} rectPage=${afterBuggyClick.rectPageIdx}`);

  return { probe, correctClick, afterCorrectClick, afterBuggyClick };
}

runTest('보류 ① 그리드 좌표 결함 — exam_kor.hwp zoom=0.5 정량 측정', async ({ page }) => {
  // 충분히 큰 viewport 로 columns >= 2 보장
  await page.setViewport({ width: 1600, height: 1000 });
  await page.evaluate(() => new Promise(r => setTimeout(r, 200)));

  console.log('[1] exam_kor.hwp 로드');
  const info = await loadHwpFile(page, 'exam_kor.hwp');
  console.log(`  pageCount=${info.pageCount}`);

  await screenshot(page, 'grid-coord-01-loaded');

  // [2] zoom=0.5 → 그리드 모드 활성
  console.log('\n[2] zoom=0.5 변경');
  await page.evaluate(() => {
    window.__inputHandler.viewportManager.setZoom(0.5);
  });
  await page.evaluate(() => new Promise(r => setTimeout(r, 600)));
  await screenshot(page, 'grid-coord-02-zoom05');

  const stateZ05 = await dumpGridState(page, 'zoom=0.5 그리드 상태');

  // [3] zoom=0.25 → columns 더 많음
  console.log('\n[3] zoom=0.25 변경');
  await page.evaluate(() => {
    window.__inputHandler.viewportManager.setZoom(0.25);
  });
  await page.evaluate(() => new Promise(r => setTimeout(r, 600)));
  await screenshot(page, 'grid-coord-03-zoom025');

  const stateZ025 = await dumpGridState(page, 'zoom=0.25 그리드 상태');

  // [4] zoom=1.0 (단일 컬럼) - 비교 baseline
  console.log('\n[4] zoom=1.0 변경 (단일 컬럼)');
  await page.evaluate(() => {
    window.__inputHandler.viewportManager.setZoom(1.0);
  });
  await page.evaluate(() => new Promise(r => setTimeout(r, 600)));

  const stateZ10 = await dumpGridState(page, 'zoom=1.0 단일 컬럼 (정상 baseline)');

  // [5] zoom=0.5 + 실제 click 측정 — col 0/1 페이지 비교
  console.log('\n[5] zoom=0.5 실제 click 측정');
  await page.evaluate(() => {
    window.__inputHandler.viewportManager.setZoom(0.5);
  });
  await page.evaluate(() => new Promise(r => setTimeout(r, 600)));

  // page 0 (col 0) — 임의 좌표 (페이지 좌측 상단)
  await probeClickAtPage(page, 'page 0 (col 0)', 0, 100, 200);
  await screenshot(page, 'grid-coord-04-page0-click');

  // page 1 (col 1) — 동일 페이지 내 좌표
  await probeClickAtPage(page, 'page 1 (col 1)', 1, 100, 200);
  await screenshot(page, 'grid-coord-05-page1-click');

  // page 2 (col 0 if columns=2, col 2 if columns >=3) — 가운데 열 가능성
  if (stateZ05.pageCount >= 3) {
    await probeClickAtPage(page, 'page 2 (col=2 % columns)', 2, 100, 200);
    await screenshot(page, 'grid-coord-06-page2-click');
  }

  // 결과 요약
  console.log('\n=== 측정 결과 요약 ===');
  console.log(`zoom=0.5: columns=${stateZ05.columns}, delta 분포 (page 0..7):`);
  for (const r of stateZ05.rows.slice(0, 8)) {
    console.log(`  page ${r.i} (col ${r.col}): delta_px = ${r.delta.toFixed(1)}`);
  }
  console.log(`zoom=0.25: columns=${stateZ025.columns}, delta 분포 (page 0..7):`);
  for (const r of stateZ025.rows.slice(0, 8)) {
    console.log(`  page ${r.i} (col ${r.col}): delta_px = ${r.delta.toFixed(1)}`);
  }
  console.log(`zoom=1.0 (baseline): columns=${stateZ10.columns}, delta 분포 (page 0..3):`);
  for (const r of stateZ10.rows.slice(0, 4)) {
    console.log(`  page ${r.i} (col ${r.col}): delta_px = ${r.delta.toFixed(1)}`);
  }
});
