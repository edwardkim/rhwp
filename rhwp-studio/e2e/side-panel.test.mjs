/**
 * E2E 테스트 — 오른쪽 패널 자리와 내장 AI 도우미 플러그인 (Stage 1)
 *
 * 검증 항목:
 * 1. 최상위 창에서는 ai-assistant 가 자동으로 올라오고 도구 메뉴에 항목을 심는다
 * 2. 패널을 열면 편집 영역이 좁아지고 눈금자·편집 영역 배치는 유지된다
 * 3. 닫기 단추·Esc 로 닫으면 편집 영역 폭이 원래대로 돌아오고 포커스가 편집 영역으로 간다
 * 4. 끌어서 바꾼 폭을 다시 열 때 기억한다
 * 5. getSelectionText 는 선택 영역 평문을 돌려주고 내부 클립보드를 건드리지 않는다
 *
 * 계획: mydocs/plans/rhwp_studio_ai_chat_p0_impl.md §3.1
 */

import {
  runTest, createNewDocument, clickEditArea, typeText, screenshot, assert,
} from './helpers.mjs';

process.env.VITE_URL = process.env.VITE_URL || 'http://localhost:7700';

const geometry = (page) => page.evaluate(() => {
  const rect = (id) => {
    const el = document.getElementById(id);
    if (!el || el.hidden) return null;
    const r = el.getBoundingClientRect();
    return { left: r.left, top: r.top, width: r.width, height: r.height };
  };
  return {
    scroll: rect('scroll-container'),
    hRuler: rect('h-ruler'),
    vRuler: rect('v-ruler'),
    corner: rect('ruler-corner'),
    panel: rect('side-panel-host'),
  };
});

runTest('오른쪽 패널 자리', async ({ page }) => {
  await createNewDocument(page);
  await page.waitForFunction(
    () => window.rhwpStudio?.plugins?.isLoaded?.('ai-assistant'),
    { timeout: 10000 },
  );

  // ── TC1: 자동 로드와 메뉴 항목 ─────────────────────────
  const loaded = await page.evaluate(() => ({
    list: window.rhwpStudio.plugins.list().map((p) => p.id),
    inMenu: !!document.querySelector('.md-item[data-cmd="ext:ai.toggle-panel"]'),
    open: window.rhwpStudio.plugins.invoke('ai-assistant', 'isPanelOpen', []),
  }));
  assert(loaded.list.includes('ai-assistant'), `TC1: ai-assistant 자동 로드 (${loaded.list.join(',')})`);
  assert(loaded.inMenu, 'TC1: 도구 메뉴에 AI 도우미 항목');
  assert(loaded.open === false, 'TC1: 처음에는 닫혀 있음');

  const before = await geometry(page);
  assert(before.panel === null, 'TC2: 닫힌 패널은 자리를 차지하지 않음');

  // ── TC2: 열기 — 편집 영역이 좁아지고 배치는 유지 ─────────
  await page.evaluate(() => window.rhwpStudio.automation.execute('ext:ai.toggle-panel'));
  await page.evaluate(() => new Promise((r) => setTimeout(r, 300)));
  const opened = await geometry(page);
  assert(opened.panel && opened.panel.width >= 260, `TC2: 패널 표시 (${opened.panel?.width}px)`);
  assert(Math.abs(opened.scroll.width + opened.panel.width - before.scroll.width) <= 2,
    `TC2: 편집 영역이 패널 폭만큼 좁아짐 (${before.scroll.width} → ${opened.scroll.width})`);
  assert(opened.panel.left >= opened.scroll.left + opened.scroll.width - 1, 'TC2: 패널은 편집 영역 오른쪽');
  assert(opened.hRuler.left === opened.scroll.left && opened.hRuler.top < opened.scroll.top,
    'TC2: 가로 눈금자는 편집 영역 위');
  assert(opened.vRuler.top === opened.scroll.top && opened.vRuler.left < opened.scroll.left,
    'TC2: 세로 눈금자는 편집 영역 왼쪽');
  assert(opened.panel.top === opened.hRuler.top, 'TC2: 패널은 눈금자 줄부터 세로 전체');
  const notice = await page.$eval('#side-panel-host .ai-panel-notice', (el) => el.textContent);
  assert(notice.length > 0, 'TC2: AI 패널 본문 마운트');
  await screenshot(page, 'side-panel-open');

  // ── TC3: 닫기 단추 / Esc ──────────────────────────────
  await page.click('#side-panel-host .spl-close');
  await page.evaluate(() => new Promise((r) => setTimeout(r, 300)));
  const closed = await geometry(page);
  assert(closed.panel === null, 'TC3: 닫기 단추로 닫힘');
  assert(Math.abs(closed.scroll.width - before.scroll.width) <= 1, 'TC3: 편집 영역 폭 복원');

  await page.evaluate(() => window.rhwpStudio.plugins.invoke('ai-assistant', 'togglePanel', [true]));
  await page.focus('#side-panel-host .spl-close');
  await page.keyboard.press('Escape');
  await page.evaluate(() => new Promise((r) => setTimeout(r, 200)));
  const afterEsc = await page.evaluate(() => ({
    open: window.rhwpStudio.plugins.invoke('ai-assistant', 'isPanelOpen', []),
    focus: document.activeElement?.id ?? '',
  }));
  assert(afterEsc.open === false, 'TC3: Esc 로 닫힘');
  assert(afterEsc.focus === 'scroll-container', `TC3: 포커스가 편집 영역으로 (${afterEsc.focus})`);

  // ── TC4: 끈 폭을 기억 ─────────────────────────────────
  await page.evaluate(() => window.rhwpStudio.plugins.invoke('ai-assistant', 'togglePanel', [true]));
  await page.evaluate(() => new Promise((r) => setTimeout(r, 200)));
  const resizer = await page.$eval('#side-panel-host .spl-resizer', (el) => {
    const r = el.getBoundingClientRect();
    return { x: r.left + r.width / 2, y: r.top + 100 };
  });
  const widthBefore = (await geometry(page)).panel.width;
  await page.mouse.move(resizer.x, resizer.y);
  await page.mouse.down();
  await page.mouse.move(resizer.x - 80, resizer.y, { steps: 5 });
  await page.mouse.up();
  await page.evaluate(() => window.rhwpStudio.plugins.invoke('ai-assistant', 'togglePanel', [false]));
  await page.evaluate(() => window.rhwpStudio.plugins.invoke('ai-assistant', 'togglePanel', [true]));
  await page.evaluate(() => new Promise((r) => setTimeout(r, 200)));
  const widthAfter = (await geometry(page)).panel.width;
  assert(Math.abs(widthAfter - (widthBefore + 80)) <= 2,
    `TC4: 끈 폭을 다시 열 때 유지 (${widthBefore} → ${widthAfter})`);
  await page.evaluate(() => window.rhwpStudio.plugins.invoke('ai-assistant', 'togglePanel', [false]));

  // ── TC5: 선택 평문 ────────────────────────────────────
  await clickEditArea(page);
  await typeText(page, '가나다라마');
  await page.evaluate(() => new Promise((r) => setTimeout(r, 300)));
  await page.keyboard.down('Shift');
  for (let i = 0; i < 3; i += 1) await page.keyboard.press('ArrowLeft');
  await page.keyboard.up('Shift');
  const clipboardBefore = await page.evaluate(() => window.__wasm.getClipboardText?.() ?? null);
  const text = await page.evaluate(() =>
    window.rhwpStudio.plugins.invoke('ai-assistant', 'selectionText', []));
  const clipboardAfter = await page.evaluate(() => window.__wasm.getClipboardText?.() ?? null);
  assert(text?.text === '다라마' && text.truncated === false, `TC5: 선택 평문 (${JSON.stringify(text)})`);
  assert(clipboardAfter === clipboardBefore, 'TC5: 내부 클립보드 그대로');
});
