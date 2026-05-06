/**
 * Issue #595 진단 e2e
 *
 * 본질: exam_math.hwp 의 수식을 더블클릭했을 때 1페이지(0-based 0) 만 정상,
 * 2페이지(0-based 1) 부터 findPictureAtClick 이 hit 으로 인식하지 못 함.
 *
 * 본 e2e 는 정정 패치 전 단계로, page 1 vs page 2 의 좌표/매칭 결과를
 * 직접 비교하여 본질 가설을 데이터로 검증한다.
 *
 * 실행:
 *   cd rhwp-studio
 *   npx vite --host 0.0.0.0 --port 7700 &
 *   node e2e/issue-595.test.mjs --mode=headless
 *
 * 또는:
 *   node e2e/issue-595.test.mjs --mode=host
 */
import { runTest, loadHwpFile, screenshot } from './helpers.mjs';

// 진단 유틸 — page.evaluate 안에서 좌표 변환 + bbox 매칭 직접 호출
async function probe(page, label, target) {
  console.log(`\n=== ${label} ===`);
  console.log(`  target: paraIdx=${target.paraIdx} ci=${target.ci}, pageInside=(${target.pageX}, ${target.pageY})`);

  // 1) 좌표 변환 + 진단 데이터 캡처 (클릭 전)
  const beforeClick = await page.evaluate((t) => {
    const sc = document.querySelector('#scroll-content');
    const ih = window.__inputHandler;
    const cv = window.__canvasView;
    if (!sc || !ih || !cv) {
      return { error: `missing globals: sc=${!!sc} ih=${!!ih} cv=${!!cv}` };
    }
    // private 접근 (TS private 은 컴파일 시만 — JS 런타임 접근 가능)
    const vs = ih.virtualScroll;
    const vm = ih.viewportManager;
    const wasm = window.__wasm;
    const zoom = vm.getZoom();
    const pageWidth = vs.getPageWidth(t.targetPageIdx);
    const pageHeight = vs.getPageHeight(t.targetPageIdx);
    const pageOffset = vs.getPageOffset(t.targetPageIdx);
    const pageLefts = [];
    const pageOffsets = [];
    const pageHeights = [];
    for (let i = 0; i < vs.pageCount; i++) {
      pageLefts.push(vs.getPageLeft(i));
      pageOffsets.push(vs.getPageOffset(i));
      pageHeights.push(vs.getPageHeight(i));
    }

    // 페이지 내부 좌표 (HWP layout 단위) → DOM 좌표
    // canvas style.left 가 -1 이면 CSS 중앙 정렬 사용
    const pageLeftRaw = vs.getPageLeft(t.targetPageIdx);
    const pageLeftDOM = pageLeftRaw >= 0 ? pageLeftRaw : (sc.clientWidth - pageWidth) / 2;
    const docX = pageLeftDOM + t.pageX * zoom;
    const docY = pageOffset + t.pageY * zoom;

    // 스크롤하여 클릭 좌표가 화면 안에 들어오게
    const scroller = sc.parentElement;
    const targetScrollY = Math.max(0, docY - 200);
    scroller.scrollTop = targetScrollY;

    return {
      zoom,
      pageWidth,
      pageHeight,
      pageOffset,
      pageLeftRaw,
      pageLeftDOM,
      docX,
      docY,
      targetScrollY,
      scrollContentClientWidth: sc.clientWidth,
      scrollerScrollTop: scroller.scrollTop,
      pageOffsets,
      pageHeights,
      pageLefts,
      pageCount: vs.pageCount,
    };
  }, { ...target });

  if (beforeClick.error) throw new Error(`probe failed: ${beforeClick.error}`);

  // 스크롤 안정화 대기
  await page.evaluate(() => new Promise(r => setTimeout(r, 400)));

  // 2) clientX/clientY 계산 (스크롤 후 contentRect 기준)
  const clickPoint = await page.evaluate((d) => {
    const sc = document.querySelector('#scroll-content');
    const cr = sc.getBoundingClientRect();
    const clientX = cr.left + d.docX;
    const clientY = cr.top + d.docY;
    // 안전: clientY 가 viewport 안에 있는지 확인 (스크롤 안정화 후)
    const vph = window.innerHeight;
    return {
      clientX, clientY, crLeft: cr.left, crTop: cr.top,
      viewportHeight: vph,
      inViewport: clientY >= 0 && clientY < vph,
    };
  }, { docX: beforeClick.docX, docY: beforeClick.docY });

  // 3) 클릭 직전 inputHandler.findPictureAtClick 직접 호출 + 좌표 재계산 (클릭 모사)
  const probeResult = await page.evaluate((cp) => {
    const sc = document.querySelector('#scroll-content');
    const ih = window.__inputHandler;
    const vm = ih.viewportManager;
    const vs = ih.virtualScroll;
    const wasm = window.__wasm;
    const zoom = vm.getZoom();
    const cr = sc.getBoundingClientRect();
    // 마우스 핸들러와 동일한 식
    const cx = cp.clientX - cr.left;
    const cy = cp.clientY - cr.top;
    const pageIdx = vs.getPageAtY(cy);
    const pageOffset = vs.getPageOffset(pageIdx);
    const pageDisplayWidth = vs.getPageWidth(pageIdx);
    const pageLeft = (sc.clientWidth - pageDisplayWidth) / 2;
    const pageX = (cx - pageLeft) / zoom;
    const pageY = (cy - pageOffset) / zoom;

    // layout.controls 중 type=equation 만 추출 (개수만)
    let layoutSummary = null;
    try {
      const json = wasm.getPageControlLayout(pageIdx);
      const layout = JSON.parse(json);
      const eqs = (layout.controls || []).filter(c => c.type === 'equation');
      layoutSummary = {
        total: (layout.controls || []).length,
        equationCount: eqs.length,
      };
    } catch (e) {
      layoutSummary = { error: e?.message || String(e) };
    }

    // wasm.hitTest 결과 (mousedown 흐름 모사)
    let hitResult = null;
    try {
      hitResult = wasm.hitTest(pageIdx, pageX, pageY);
    } catch (e) {
      hitResult = { error: e?.message || String(e) };
    }

    // 머리말/꼬리말 hit
    let hfHit = null, fnHit = null, formHit = null;
    try { hfHit = wasm.hitTestHeaderFooter(pageIdx, pageX, pageY); } catch (e) {}
    try { fnHit = wasm.hitTestFootnote(pageIdx, pageX, pageY); } catch (e) {}
    try { formHit = wasm.getFormObjectAt(pageIdx, pageX, pageY); } catch (e) {}

    // findPictureAtClick 직접 호출
    let picHit = null;
    try {
      picHit = ih.findPictureAtClick(pageIdx, pageX, pageY);
    } catch (e) {
      picHit = { error: e?.message || String(e) };
    }
    return { cx, cy, pageIdx, pageOffset, pageDisplayWidth, pageLeft, pageX, pageY,
             picHit, layoutSummary, hitResult, hfHit, fnHit, formHit };
  }, clickPoint);

  // 4) 실제 클릭 이벤트 발생
  await page.mouse.click(clickPoint.clientX, clickPoint.clientY);
  await page.evaluate(() => new Promise(r => setTimeout(r, 300)));

  // 5) 클릭 결과 — cursor 상태 확인
  const afterClick = await page.evaluate(() => {
    const ih = window.__inputHandler;
    const cur = ih?.cursor;
    return {
      isInPictureObjectSelection: !!cur?.isInPictureObjectSelection?.(),
      isInTextBox: !!cur?.isInTextBox?.(),
      pos: cur?.getPosition?.() ? {
        sectionIndex: cur.getPosition().sectionIndex,
        paragraphIndex: cur.getPosition().paragraphIndex,
        charOffset: cur.getPosition().charOffset,
      } : null,
      selectedPictureRef: cur?.getSelectedPictureRef?.() ?? null,
    };
  });

  console.log(`  zoom=${beforeClick.zoom}  pageOffsets[0..3]=${beforeClick.pageOffsets.slice(0,4).map(v => v.toFixed(1)).join(', ')}`);
  console.log(`  pageHeight=${beforeClick.pageHeight.toFixed(1)}  pageWidth=${beforeClick.pageWidth.toFixed(1)}  pageLeftRaw=${beforeClick.pageLeftRaw}  pageLeftDOM=${beforeClick.pageLeftDOM.toFixed(1)}`);
  console.log(`  docX=${beforeClick.docX.toFixed(1)} docY=${beforeClick.docY.toFixed(1)} → scrollTop=${beforeClick.targetScrollY.toFixed(1)}`);
  console.log(`  clientX=${clickPoint.clientX.toFixed(1)} clientY=${clickPoint.clientY.toFixed(1)} cr=(${clickPoint.crLeft.toFixed(1)}, ${clickPoint.crTop.toFixed(1)}) inVp=${clickPoint.inViewport}`);
  console.log(`  [PROBE 좌표 역계산] cx=${probeResult.cx.toFixed(1)} cy=${probeResult.cy.toFixed(1)} → pageIdx=${probeResult.pageIdx} pageOffset=${probeResult.pageOffset.toFixed(1)} pageX=${probeResult.pageX.toFixed(1)} pageY=${probeResult.pageY.toFixed(1)}`);
  console.log(`  [PROBE layout] page ${probeResult.pageIdx} controls=${probeResult.layoutSummary?.total} equations=${probeResult.layoutSummary?.equationCount}`);
  console.log(`  [PROBE wasm.hitTest] ${JSON.stringify(probeResult.hitResult)}`);
  console.log(`  [PROBE hf/fn/form] hf=${JSON.stringify(probeResult.hfHit)} fn=${JSON.stringify(probeResult.fnHit)} form=${JSON.stringify(probeResult.formHit)}`);
  console.log(`  [PROBE findPictureAtClick] ${JSON.stringify(probeResult.picHit)}`);
  console.log(`  [AFTER CLICK] picSel=${afterClick.isInPictureObjectSelection} textBox=${afterClick.isInTextBox} pos=${JSON.stringify(afterClick.pos)} selRef=${JSON.stringify(afterClick.selectedPictureRef)}`);

  return { beforeClick, clickPoint, probeResult, afterClick };
}

runTest('Issue #595 — exam_math.hwp page 1 vs page 2 수식 hitTest 비교 (1365x1018 사용자 환경)', async ({ page }) => {
  // 사용자 환경: 1365x1018, zoom=100%
  await page.setViewport({ width: 1365, height: 1018 });
  await page.evaluate(() => new Promise(r => setTimeout(r, 200)));

  console.log('[1] exam_math.hwp 로드');
  const info = await loadHwpFile(page, 'exam_math.hwp');
  console.log(`  pageCount=${info.pageCount}`);

  // 환경 정보
  const env = await page.evaluate(() => {
    const ih = window.__inputHandler;
    const vs = ih?.virtualScroll;
    return {
      windowInner: { w: window.innerWidth, h: window.innerHeight },
      dpr: window.devicePixelRatio,
      zoom: ih?.viewportManager?.getZoom?.(),
      gridMode: vs?.isGridMode?.(),
      columns: vs?.getColumns?.(),
      pageCount: vs?.pageCount,
    };
  });
  console.log(`  env: window=${env.windowInner.w}x${env.windowInner.h} dpr=${env.dpr} zoom=${env.zoom} grid=${env.gridMode} cols=${env.columns} pageCount=${env.pageCount}`);
  await screenshot(page, 'issue595-01-loaded');

  // page 1 (0-based 0) — paraIdx=18 ci=0 수식 (x=130.8 y=818.3 w=108.0 h=17.5)
  const r1 = await probe(page, 'page 1 (0-based 0) — paraIdx=18 ci=0 수식 [zoom=1]', {
    targetPageIdx: 0,
    paraIdx: 18, ci: 0,
    pageX: 130.8 + 50, // bbox 중앙 근처
    pageY: 818.3 + 8,
  });
  await screenshot(page, 'issue595-02-page1-clicked');

  // page 2 (0-based 1) — paraIdx=65 ci=0 수식 (x=589.5 y=191.7 w=131.7 h=37.3) — 이슈 명세
  const r2 = await probe(page, 'page 2 (0-based 1) — paraIdx=65 ci=0 수식 [zoom=1]', {
    targetPageIdx: 1,
    paraIdx: 65, ci: 0,
    pageX: 589.5 + 65,
    pageY: 191.7 + 18,
  });
  await screenshot(page, 'issue595-03-page2-clicked');

  // ─── 추가 시나리오: 다양한 zoom 에서 page 2 수식 클릭 ───
  // 사용자 환경에서 발현 가능성: 모바일 fit-to-width (zoom < 1) / Hi-DPI 확대 (zoom > 1)
  for (const z of [0.5, 0.75, 1.5, 2.0]) {
    console.log(`\n>>> zoom=${z} 로 변경 후 page 2 수식 재시도`);
    await page.evaluate((zoom) => {
      window.__inputHandler.viewportManager.setZoom(zoom);
    }, z);
    await page.evaluate(() => new Promise(r => setTimeout(r, 600)));
    const rZ = await probe(page, `page 2 (0-based 1) — paraIdx=65 ci=0 수식 [zoom=${z}]`, {
      targetPageIdx: 1,
      paraIdx: 65, ci: 0,
      pageX: 589.5 + 65,
      pageY: 191.7 + 18,
    });
    await screenshot(page, `issue595-zoom-${z}`.replace('.', '_'));
  }

  // 원래 zoom 으로 복귀
  await page.evaluate(() => window.__inputHandler.viewportManager.setZoom(1.0));
  await page.evaluate(() => new Promise(r => setTimeout(r, 400)));

  // ─── 더블클릭 시뮬레이션 ───
  console.log('\n>>> page 2 수식 더블클릭 (실제 사용자 동작 모사)');
  const dblTarget = await page.evaluate(() => {
    const sc = document.querySelector('#scroll-content');
    const ih = window.__inputHandler;
    const vs = ih.virtualScroll;
    const vm = ih.viewportManager;
    const zoom = vm.getZoom();
    const pageIdx = 1;
    const pageW = vs.getPageWidth(pageIdx);
    const pageOffset = vs.getPageOffset(pageIdx);
    const pageLeftDOM = (sc.clientWidth - pageW) / 2;
    const pageX = 589.5 + 65, pageY = 191.7 + 18;
    const docX = pageLeftDOM + pageX * zoom;
    const docY = pageOffset + pageY * zoom;
    sc.parentElement.scrollTop = Math.max(0, docY - 200);
    return new Promise(r => requestAnimationFrame(() => {
      const cr = sc.getBoundingClientRect();
      r({ clientX: cr.left + docX, clientY: cr.top + docY });
    }));
  });
  await page.evaluate(() => new Promise(r => setTimeout(r, 400)));
  // 두 번 빠르게 클릭 (Puppeteer 더블클릭)
  await page.mouse.click(dblTarget.clientX, dblTarget.clientY, { clickCount: 2, delay: 50 });
  await page.evaluate(() => new Promise(r => setTimeout(r, 600)));
  const afterDbl = await page.evaluate(() => {
    const ih = window.__inputHandler;
    const cur = ih?.cursor;
    return {
      isInPictureObjectSelection: !!cur?.isInPictureObjectSelection?.(),
      selectedPictureRef: cur?.getSelectedPictureRef?.() ?? null,
    };
  });
  console.log(`  더블클릭 후: picSel=${afterDbl.isInPictureObjectSelection} selRef=${JSON.stringify(afterDbl.selectedPictureRef)}`);
  await screenshot(page, 'issue595-04-dblclick');

  // 비교 요약
  console.log('\n=== 비교 요약 ===');
  console.log(`page 1: picHit=${JSON.stringify(r1.probeResult.picHit)} → afterClick.picSel=${r1.afterClick.isInPictureObjectSelection}`);
  console.log(`page 2: picHit=${JSON.stringify(r2.probeResult.picHit)} → afterClick.picSel=${r2.afterClick.isInPictureObjectSelection}`);

  // 검증: page 1 정상, page 2 fail 가설 확인
  const page1Ok = r1.probeResult.picHit && !r1.probeResult.picHit.error;
  const page2Ok = r2.probeResult.picHit && !r2.probeResult.picHit.error;
  console.log(`\n[가설 검증] page1 picture hit OK=${!!page1Ok}  /  page2 picture hit OK=${!!page2Ok}`);
  if (page1Ok && !page2Ok) {
    console.log('  → 이슈 재현 ✓ (page 1 정상 / page 2 fail)');
  } else if (page1Ok && page2Ok) {
    console.log('  → 이슈 재현 실패 — 두 페이지 모두 정상 (좌표 모델 불일치 가능성)');
  } else {
    console.log(`  → 예상 외 결과: page1=${!!page1Ok}, page2=${!!page2Ok}`);
  }
});
