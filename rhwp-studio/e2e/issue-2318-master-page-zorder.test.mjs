// Issue #2318: 바탕쪽 개체가 본문 텍스트를 가림 — studio 다층 canvas 합성 검증.
//
// shortcut.hwp 의 바탕쪽 글상자(쪽번호 "1", wrap=글 앞으로)는 한컴에서 본문
// 텍스트 뒤에 깔린다. 정정 전에는 front overlay canvas 로 승격되어 본문
// "Ctrl+Y"/"Alt+Y" 를 덮었다. 정정 후 계약:
//   1) 바탕쪽 op 는 behindText plane → behind overlay canvas 존재
//   2) front overlay canvas 부재 (이 문서에 본문 기준 front 개체 없음)
//   3) 본문 flow canvas 가 behind overlay 보다 위 (z-index)
//
// 실행: CHROME_CDP=http://localhost:19222 node e2e/issue-2318-master-page-zorder.test.mjs --mode=host

import { runTest, loadHwpFile, screenshot, assert } from './helpers.mjs';

runTest('#2318 바탕쪽 z-order — 본문 텍스트가 바탕쪽 위에 표시', async ({ page }) => {
  const { pageCount } = await loadHwpFile(page, 'basic/shortcut.hwp');
  assert(pageCount >= 1, `shortcut.hwp 로드 (${pageCount}페이지)`);

  // 렌더 안정화 (overlay layer 생성 + 비동기 재렌더 대기)
  await page.evaluate(() => new Promise((r) => setTimeout(r, 1500)));

  const layers = await page.evaluate(() => {
    const behind = document.querySelector(
      '[data-rhwp-overlay-page="0"][data-rhwp-layer-kind="behind"]',
    );
    const front = document.querySelector(
      '[data-rhwp-overlay-page="0"][data-rhwp-layer-kind="front"]',
    );
    const flowCanvas = behind?.parentElement?.querySelector('canvas:not([data-rhwp-overlay-page])')
      ?? document.querySelector('#scroll-container canvas');
    return {
      hasBehind: !!behind,
      hasFront: !!front,
      behindZ: behind ? Number(behind.style.zIndex || 0) : null,
      flowZ: flowCanvas ? Number(flowCanvas.style.zIndex || 0) : null,
    };
  });

  assert(layers.hasBehind, '바탕쪽 → behind overlay canvas 존재 (plane cap 적용)');
  assert(!layers.hasFront, 'front overlay canvas 부재 (바탕쪽이 front 로 승격되지 않음)');
  assert(
    layers.behindZ !== null && layers.flowZ !== null && layers.flowZ > layers.behindZ,
    `본문 flow canvas(z=${layers.flowZ})가 behind overlay(z=${layers.behindZ})보다 위`,
  );

  await screenshot(page, 'issue2318-01-master-page-behind');

  // 바탕쪽 쪽번호 "1" 과 본문 "Ctrl+Y"/"Alt+Y" 가 겹치는 페이지 1 하단으로 스크롤
  // (정정 전에는 "1" 이 텍스트를 덮었던 영역 — 시각 판정용 기록)
  await page.evaluate(() => {
    const sc = document.querySelector('#scroll-container');
    if (sc) sc.scrollTop = sc.scrollHeight * 0.09;
  });
  await page.evaluate(() => new Promise((r) => setTimeout(r, 800)));
  await screenshot(page, 'issue2318-02-page1-bottom-overlap');
});
