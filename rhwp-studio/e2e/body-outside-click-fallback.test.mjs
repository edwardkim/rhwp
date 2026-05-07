/**
 * 보류 ② 본문 외곽 클릭 fallback 결함 — 가설 (b) master page 글상자 hit 확정 e2e
 *
 * 본질: samples/hwpctl_Action_Table__v1.1.hwp (16p, landscape, margin_bottom=0)
 * 의 page 16 꼬리말 영역 (footer_area.height=0) 클릭 시:
 *   - 한컴: 문서 마지막 `}` 캐럿 배치 (본문 외곽 fallback)
 *   - RHWP: 페이지 2, 3 으로 뷰 점프 (본질 결함)
 *
 * 본 측정은 input-handler-mouse onMouseDown 내부에서 호출되는 wasm.hitTest
 * 결과를 직접 캡쳐하여 다음을 확인:
 *   (a) cursor.moveTo(invalid) fallback?
 *   (b) hit.isTextBox === true (master page 글상자 hit)?
 *   (c) 일반 paragraph hit 인데 cursor.getRect() 의 pageIndex 가 click 페이지와 다름?
 *
 * 측정 항목:
 *   - hit object 전체
 *   - cursor.getPosition() / getRect() (click 후)
 *   - scrollContainer.scrollTop (click 전후)
 *   - 한컴 정합 동작 (마지막 `}` 캐럿) 비교 가능 여부
 *
 * 실행:
 *   cd rhwp-studio
 *   npx vite --host 0.0.0.0 --port 7700 &
 *   node e2e/body-outside-click-fallback.test.mjs --mode=headless
 */
import { runTest, loadHwpFile, screenshot } from './helpers.mjs';

async function probeFooterClick(page, label, pageIdx, hwpX, hwpY) {
  console.log(`\n=== ${label} (page ${pageIdx}, hwpX=${hwpX}, hwpY=${hwpY}) ===`);

  // 사전: page 정보 + scroll 안정화
  const setup = await page.evaluate(({ pageIdx, hwpX, hwpY }) => {
    const sc = document.querySelector('#scroll-content');
    const ih = window.__inputHandler;
    const vs = ih.virtualScroll;
    const vm = ih.viewportManager;
    const wasm = window.__wasm;
    const zoom = vm.getZoom();
    const pw = vs.getPageWidth(pageIdx);
    const ph = vs.getPageHeight(pageIdx);
    const po = vs.getPageOffset(pageIdx);
    const pl = vs.getPageLeft(pageIdx);
    const plDOM = pl >= 0 ? pl : (sc.clientWidth - pw) / 2;
    const docX = plDOM + hwpX * zoom;
    const docY = po + hwpY * zoom;

    const scroller = sc.parentElement;
    const targetScrollY = Math.max(0, docY - 300);
    scroller.scrollTop = targetScrollY;

    return { zoom, pw, ph, po, pl, plDOM, docX, docY, targetScrollY };
  }, { pageIdx, hwpX, hwpY });

  await page.evaluate(() => new Promise(r => setTimeout(r, 500)));

  // click 직전 hit 결과 + scrollTop 측정
  const beforeClick = await page.evaluate(({ docX, docY, pageIdx }) => {
    const sc = document.querySelector('#scroll-content');
    const scroller = sc.parentElement;
    const cr = sc.getBoundingClientRect();
    const ih = window.__inputHandler;
    const vs = ih.virtualScroll;
    const vm = ih.viewportManager;
    const wasm = window.__wasm;
    const zoom = vm.getZoom();
    const cx = docX;
    const cy = docY;
    const pageDisplayWidth = vs.getPageWidth(pageIdx);
    const buggyLeft = (sc.clientWidth - pageDisplayWidth) / 2;
    const correctLeft = vs.getPageLeft(pageIdx);
    const correctLeftDOM = correctLeft >= 0 ? correctLeft : buggyLeft;
    const buggyPageX = (cx - buggyLeft) / zoom;
    const correctPageX = (cx - correctLeftDOM) / zoom;
    const pageY = (cy - vs.getPageOffset(pageIdx)) / zoom;

    // wasm hit 시리즈 (input-handler-mouse onMouseDown 흐름 모사 — buggy 좌표 사용)
    let hit = null, hfHit = null, fnHit = null;
    try { hit = wasm.hitTest(pageIdx, buggyPageX, pageY); } catch (e) { hit = { error: e?.message || String(e) }; }
    try { hfHit = wasm.hitTestHeaderFooter(pageIdx, buggyPageX, pageY); } catch (e) {}
    try { fnHit = wasm.hitTestFootnote(pageIdx, buggyPageX, pageY); } catch (e) {}

    // findPictureAtClick (buggy 좌표 기준)
    let picHit = null;
    try { picHit = ih.findPictureAtClick(pageIdx, buggyPageX, pageY); } catch (e) { picHit = { error: e?.message || String(e) }; }

    return {
      scrollTop: scroller.scrollTop,
      clientX: cr.left + docX,
      clientY: cr.top + docY,
      buggyPageX, correctPageX, pageY, buggyLeft, correctLeftDOM,
      hit, hfHit, fnHit, picHit,
    };
  }, { docX: setup.docX, docY: setup.docY, pageIdx });

  console.log(`  scroll.before = ${beforeClick.scrollTop.toFixed(1)}`);
  console.log(`  click @(${beforeClick.clientX.toFixed(1)}, ${beforeClick.clientY.toFixed(1)})`);
  console.log(`  buggyPageX=${beforeClick.buggyPageX.toFixed(1)} correctPageX=${beforeClick.correctPageX.toFixed(1)} pageY=${beforeClick.pageY.toFixed(1)}`);
  console.log(`  buggyLeft=${beforeClick.buggyLeft.toFixed(1)} correctLeftDOM=${beforeClick.correctLeftDOM.toFixed(1)}`);
  console.log(`  hit = ${JSON.stringify(beforeClick.hit)}`);
  console.log(`  hfHit = ${JSON.stringify(beforeClick.hfHit)}`);
  console.log(`  fnHit = ${JSON.stringify(beforeClick.fnHit)}`);
  console.log(`  picHit = ${JSON.stringify(beforeClick.picHit)}`);

  // 실제 click 발생
  await page.mouse.click(beforeClick.clientX, beforeClick.clientY);
  await page.evaluate(() => new Promise(r => setTimeout(r, 400)));

  // click 후 cursor + scroll 상태
  const afterClick = await page.evaluate(() => {
    const sc = document.querySelector('#scroll-content');
    const scroller = sc.parentElement;
    const ih = window.__inputHandler;
    const cur = ih?.cursor;
    const pos = cur?.getPosition?.();
    const rect = cur?.getRect?.();
    return {
      scrollTop: scroller.scrollTop,
      pos: pos ? { sec: pos.sectionIndex, para: pos.paragraphIndex, char: pos.charOffset, parentParaIndex: pos.parentParaIndex, controlIndex: pos.controlIndex } : null,
      rect: rect ? { pageIdx: rect.pageIndex, x: rect.x, y: rect.y, height: rect.height } : null,
      isInTextBox: !!cur?.isInTextBox?.(),
      isInPictureObjectSelection: !!cur?.isInPictureObjectSelection?.(),
      isInTableObjectSelection: !!cur?.isInTableObjectSelection?.(),
      isInHeaderFooter: !!cur?.isInHeaderFooter?.(),
    };
  });

  console.log(`  scroll.after = ${afterClick.scrollTop.toFixed(1)}  (delta = ${(afterClick.scrollTop - beforeClick.scrollTop).toFixed(1)})`);
  console.log(`  cursor.pos = ${JSON.stringify(afterClick.pos)}`);
  console.log(`  cursor.rect = ${JSON.stringify(afterClick.rect)}`);
  console.log(`  isInTextBox=${afterClick.isInTextBox} isInPictureObjectSelection=${afterClick.isInPictureObjectSelection} isInTableObjectSelection=${afterClick.isInTableObjectSelection} isInHeaderFooter=${afterClick.isInHeaderFooter}`);

  // 가설 (b) 확정 조건 점검
  const hypothesisB = beforeClick.hit?.isTextBox === true;
  const hypothesisA = !beforeClick.hit || beforeClick.hit.error;
  const scrollJumped = Math.abs(afterClick.scrollTop - beforeClick.scrollTop) > 50;
  const rectPageMismatch = afterClick.rect && afterClick.rect.pageIdx !== pageIdx;

  console.log(`\n  >>> 가설 (a) hit invalid: ${hypothesisA ? 'YES' : 'no'}`);
  console.log(`  >>> 가설 (b) isTextBox=true: ${hypothesisB ? 'YES (master page 글상자 hit)' : 'no'}`);
  console.log(`  >>> 가설 (c) rect.pageIdx mismatch: ${rectPageMismatch ? 'YES (click=' + pageIdx + ' rect=' + afterClick.rect?.pageIdx + ')' : 'no'}`);
  console.log(`  >>> scroll 점프 발생: ${scrollJumped ? 'YES' : 'no'}`);

  return { setup, beforeClick, afterClick, hypothesisA, hypothesisB, hypothesisC: rectPageMismatch, scrollJumped };
}

runTest('보류 ② 본문 외곽 fallback — hwpctl_Action_Table__v1.1.hwp 16p 꼬리말 click 가설 확정', async ({ page }) => {
  await page.setViewport({ width: 1600, height: 1000 });
  await page.evaluate(() => new Promise(r => setTimeout(r, 200)));

  console.log('[1] hwpctl_Action_Table__v1.1.hwp 로드');
  const info = await loadHwpFile(page, 'hwpctl_Action_Table__v1.1.hwp');
  console.log(`  pageCount=${info.pageCount}`);
  await screenshot(page, 'body-outside-01-loaded');

  // zoom=1.0 (단일 컬럼) — 그리드 모드 결함 배제
  await page.evaluate(() => window.__inputHandler.viewportManager.setZoom(1.0));
  await page.evaluate(() => new Promise(r => setTimeout(r, 500)));

  // page 정보 dump
  const pageDump = await page.evaluate(() => {
    const ih = window.__inputHandler;
    const vs = ih.virtualScroll;
    const wasm = window.__wasm;
    const list = [];
    for (let i = 0; i < vs.pageCount; i++) {
      list.push({ i, w: vs.getPageWidth(i), h: vs.getPageHeight(i), o: vs.getPageOffset(i) });
    }
    return list;
  });
  console.log(`\n페이지 정보 (총 ${pageDump.length} 페이지):`);
  for (const p of pageDump.slice(0, 4)) console.log(`  page ${p.i}: w=${p.w.toFixed(1)} h=${p.h.toFixed(1)} o=${p.o.toFixed(1)}`);
  if (pageDump.length > 4) console.log(`  ...`);
  for (const p of pageDump.slice(Math.max(0, pageDump.length - 3))) console.log(`  page ${p.i}: w=${p.w.toFixed(1)} h=${p.h.toFixed(1)} o=${p.o.toFixed(1)}`);

  // page 16 (0-based 15) 꼬리말 영역 click — pageHeight 의 95% 위치
  // landscape 297x210mm → pageHeight ≈ 793px (75 DPI 가정)
  const targetPageIdx = Math.min(15, pageDump.length - 1);
  const targetPage = pageDump[targetPageIdx];
  const footerY = targetPage.h * 0.96;  // 페이지 하단 4% (꼬리말 영역)
  const footerX = targetPage.w * 0.5;    // 페이지 가운데

  console.log(`\n[2] page ${targetPageIdx} 꼬리말 영역 click 시뮬레이션`);
  await probeFooterClick(page, `page ${targetPageIdx} 꼬리말 영역`, targetPageIdx, footerX, footerY);
  await screenshot(page, 'body-outside-02-page16-footer-click');

  // 비교 baseline: page 1 (0-based 0) 본문 영역 click — 정상 동작 확인
  console.log(`\n[3] page 0 본문 click (정상 baseline)`);
  await probeFooterClick(page, 'page 0 본문 영역', 0, pageDump[0].w * 0.5, pageDump[0].h * 0.5);
  await screenshot(page, 'body-outside-03-page0-body-click');

  // 비교: page 1 (0-based 1) 꼬리말 영역 click — 본 결함 재현 여부 (master page 영향 가설)
  if (pageDump.length > 1) {
    console.log(`\n[4] page 1 꼬리말 영역 click (master page 영향 비교)`);
    await probeFooterClick(page, 'page 1 꼬리말 영역', 1, pageDump[1].w * 0.5, pageDump[1].h * 0.96);
    await screenshot(page, 'body-outside-04-page1-footer-click');
  }
});
