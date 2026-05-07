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
import { runTest, loadHwpFile, screenshot, assert } from './helpers.mjs';

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
      const helperResolved = vs.getPageLeftResolved(i, clientWidth);
      const col = i % Math.max(columns, 1);
      const delta = correct >= 0 ? (buggy - correct) : 0;
      // 헬퍼 기대값: 그리드 모드는 pageLefts[i], 단일 컬럼은 buggy 공식과 동치
      const expectedHelper = correct >= 0 ? correct : buggy;
      const helperDelta = helperResolved - expectedHelper;
      rows.push({ i, col, pw, correct, buggy, helperResolved, helperDelta, delta });
    }
    return { zoom, isGrid, columns, pageCount, clientWidth, rows };
  });

  console.log(`  zoom=${state.zoom}  grid=${state.isGrid}  columns=${state.columns}  pageCount=${state.pageCount}  clientWidth=${state.clientWidth}`);
  console.log(`  | i  | col | pw     | correct(pageLefts[i]) | buggy(formula) | helper | delta_px |`);
  console.log(`  |----|-----|--------|-----------------------|----------------|--------|----------|`);
  for (const r of state.rows.slice(0, 8)) {  // 첫 8 페이지만 출력
    console.log(`  | ${String(r.i).padEnd(2)} | ${String(r.col).padEnd(3)} | ${r.pw.toFixed(1).padEnd(6)} | ${r.correct.toFixed(1).padEnd(21)} | ${r.buggy.toFixed(1).padEnd(14)} | ${r.helperResolved.toFixed(1).padEnd(6)} | ${r.delta.toFixed(1).padEnd(8)} |`);
  }

  // 헬퍼 동치성 assert: 모든 페이지에 대해 |helperDelta| < 0.01
  const maxHelperDelta = Math.max(...state.rows.map(r => Math.abs(r.helperDelta)));
  assert(
    maxHelperDelta < 0.01,
    `[${label}] getPageLeftResolved == 기대값 (max|helperDelta|=${maxHelperDelta.toFixed(4)}px)`
  );
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
    const columns = typeof vs.getColumns === 'function' ? vs.getColumns() : 1;
    const col = pageIdx % Math.max(columns, 1);
    const isLastCol = col === columns - 1;

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
      columns, col, isLastCol,
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

  // fix 검증: CORRECT click → cursor.pos 정상 (모든 col)
  assert(
    afterCorrectClick.pos !== null,
    `[${label}] CORRECT click → cursor.pos !== null`
  );

  // rectPageIdx assert 는 last-col 케이스에서만 strict 하게 검증.
  // non-last col 은 getPageAtY 가 X 무시하고 row 의 last page idx 만 반환하는 별개 결함 (Issue #689)
  // 으로 인해 항상 마지막 col 페이지로 cursor 가 떨어짐. Task #685 의 pageLeft 정정만으로는 해결되지 않는
  // 후속 결함이며, 본 e2e 는 #685 정정 효과 (last col 에서 정확한 pageLefts[i] 적용) 만 검증.
  if (probe.isLastCol) {
    assert(
      afterCorrectClick.rectPageIdx === pageIdx,
      `[${label}] CORRECT click → cursor.rectPageIdx=${afterCorrectClick.rectPageIdx} (기대 ${pageIdx}, last col=${probe.col}/columns=${probe.columns})`
    );
  } else {
    console.log(`  SKIP: [${label}] non-last col rectPageIdx strict assert (col=${probe.col}/columns=${probe.columns}) — getPageAtY X-무시 결함, Issue #689 후속`);
  }

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

  // [3b] zoom=0.25 last col (col=4) click 검증 — pageLeft 정정 효과
  if (stateZ025.columns >= 2 && stateZ025.pageCount > stateZ025.columns - 1) {
    const lastColPage = stateZ025.columns - 1;
    await probeClickAtPage(page, `page ${lastColPage} (zoom=0.25 last col)`, lastColPage, 100, 200);
  }

  // [4] zoom=1.0 (단일 컬럼) - 비교 baseline
  console.log('\n[4] zoom=1.0 변경 (단일 컬럼)');
  await page.evaluate(() => {
    window.__inputHandler.viewportManager.setZoom(1.0);
  });
  await page.evaluate(() => new Promise(r => setTimeout(r, 600)));

  const stateZ10 = await dumpGridState(page, 'zoom=1.0 단일 컬럼 (정상 baseline)');

  // [4b] zoom=1.0 click baseline — 단일 컬럼 모드 click 무회귀 확인
  console.log('\n[4b] zoom=1.0 click baseline');
  await probeClickAtPage(page, 'page 0 (zoom=1.0 single col)', 0, 100, 200);

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
