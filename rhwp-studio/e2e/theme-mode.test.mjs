/**
 * E2E 테스트 — 보기 > 테마
 *
 * 검증 항목:
 * 1. system/light/dark 테마 명령이 동작한다.
 * 2. 선택 상태가 메뉴 active 및 localStorage에 반영된다.
 * 3. 앱 chrome은 dark로 바뀌어도 편집 용지는 흰색으로 유지된다.
 * 4. 새로고침 후 저장한 테마가 유지된다.
 */

import {
  runTest, loadApp, createNewDocument, assert, screenshot,
} from './helpers.mjs';

async function getThemeState(page) {
  return await page.evaluate(() => {
    const root = document.documentElement;
    const canvas = document.querySelector('#scroll-content canvas');
    const hRuler = document.getElementById('h-ruler');
    const activeModes = Array.from(document.querySelectorAll('[data-theme-mode-choice].active'))
      .map((element) => element.dataset.themeModeChoice)
      .filter(Boolean);
    const stored = JSON.parse(localStorage.getItem('rhwp-settings') || 'null');
    let hRulerAvg = null;
    if (hRuler instanceof HTMLCanvasElement) {
      const ctx = hRuler.getContext('2d');
      if (ctx) {
        const dpr = window.devicePixelRatio || 1;
        const rulerRect = hRuler.getBoundingClientRect();
        const canvasRect = canvas instanceof HTMLCanvasElement ? canvas.getBoundingClientRect() : null;
        const sampleCenterCssX = canvasRect
          ? (canvasRect.left + (canvasRect.width / 2) - rulerRect.left)
          : hRuler.clientWidth / 2;
        const sampleW = Math.max(1, Math.round(32 * dpr));
        const sampleH = Math.max(1, Math.round(4 * dpr));
        const sampleX = Math.max(0, Math.round(sampleCenterCssX * dpr) - Math.floor(sampleW / 2));
        const sampleY = Math.max(0, Math.round(12 * dpr));
        const clampedW = Math.min(sampleW, Math.max(0, hRuler.width - sampleX));
        const clampedH = Math.min(sampleH, Math.max(0, hRuler.height - sampleY));
        if (clampedW > 0 && clampedH > 0) {
          const data = ctx.getImageData(sampleX, sampleY, clampedW, clampedH).data;
          let r = 0;
          let g = 0;
          let b = 0;
          let count = 0;
          for (let i = 0; i < data.length; i += 4) {
            if (data[i + 3] === 0) continue;
            r += data[i];
            g += data[i + 1];
            b += data[i + 2];
            count += 1;
          }
          if (count > 0) {
            hRulerAvg = {
              r: Math.round(r / count),
              g: Math.round(g / count),
              b: Math.round(b / count),
            };
          }
        }
      }
    }
    return {
      mode: root.dataset.themeMode ?? '',
      effective: root.dataset.themeEffective ?? '',
      colorScheme: root.style.colorScheme,
      bodyBg: getComputedStyle(document.body).backgroundColor,
      canvasBg: canvas ? getComputedStyle(canvas).backgroundColor : '',
      hRulerAvg,
      themeColor: document.querySelector('meta[name="theme-color"]')?.getAttribute('content') ?? '',
      colorSchemeMeta: document.querySelector('meta[name="color-scheme"]')?.getAttribute('content') ?? '',
      activeModes,
      storedMode: stored?.theme?.mode ?? '',
    };
  });
}

async function selectTheme(page, mode) {
  await page.evaluate((selectedMode) => {
    const item = document.querySelector(`[data-cmd="view:theme-${selectedMode}"]`);
    if (!item) throw new Error(`테마 메뉴를 찾을 수 없습니다: ${selectedMode}`);
    item.dispatchEvent(new MouseEvent('click', { bubbles: true, cancelable: true }));
  }, mode);
  await page.evaluate(() => new Promise((resolve) => setTimeout(resolve, 200)));
}

runTest('보기 테마', async ({ page }) => {
  await loadApp(page);

  await page.evaluate(() => {
    localStorage.removeItem('rhwp-settings');
    window.__theme?.setThemeMode?.('system');
  });
  await page.evaluate(() => new Promise((resolve) => setTimeout(resolve, 200)));

  await createNewDocument(page);

  const initial = await getThemeState(page);
  assert(initial.mode === 'system', 'TC1: 기본 테마 모드는 system이다');
  assert(initial.activeModes.length === 1 && initial.activeModes[0] === 'system', 'TC1: 시스템 설정 메뉴만 active다');
  assert(initial.storedMode === 'system', 'TC1: localStorage에도 system이 저장된다');

  await selectTheme(page, 'dark');
  const dark = await getThemeState(page);
  await screenshot(page, 'theme-mode-01-dark');
  assert(dark.mode === 'dark', 'TC2: dark 명령 후 theme mode가 dark다');
  assert(dark.effective === 'dark', 'TC2: dark 명령 후 effective theme도 dark다');
  assert(dark.colorScheme === 'dark only', 'TC2: color-scheme도 dark only로 반영된다');
  assert(dark.colorSchemeMeta === 'only dark', 'TC2: color-scheme meta도 only dark로 반영된다');
  assert(dark.activeModes.length === 1 && dark.activeModes[0] === 'dark', 'TC2: 어둡게 메뉴만 active다');
  assert(dark.storedMode === 'dark', 'TC2: localStorage에 dark가 저장된다');
  assert(dark.themeColor === '#2b3037', 'TC2: 브라우저 theme-color가 dark token으로 갱신된다');
  assert(dark.canvasBg === 'rgb(255, 255, 255)', 'TC2: dark에서도 편집 용지는 흰색이다');
  assert(dark.bodyBg !== dark.canvasBg, 'TC2: 앱 chrome 배경과 편집 용지 색이 분리된다');
  assert(dark.hRulerAvg !== null, 'TC2: 눈금자 캔버스 샘플을 읽을 수 있다');
  assert(
    dark.hRulerAvg.r < 220 && dark.hRulerAvg.g < 220 && dark.hRulerAvg.b < 220,
    'TC2: dark에서 눈금자 본문은 흰 종이처럼 밝게 남지 않는다',
  );

  await loadApp(page);
  await createNewDocument(page);
  const persisted = await getThemeState(page);
  assert(persisted.mode === 'dark', 'TC3: 새로고침 후 dark 설정이 유지된다');
  assert(persisted.effective === 'dark', 'TC3: 새로고침 후 effective theme도 dark다');
  assert(persisted.colorScheme === 'dark only', 'TC3: 새로고침 후 color-scheme도 dark only다');
  assert(persisted.colorSchemeMeta === 'only dark', 'TC3: 새로고침 후 color-scheme meta도 only dark다');
  assert(persisted.activeModes.length === 1 && persisted.activeModes[0] === 'dark', 'TC3: 새로고침 후에도 dark 메뉴가 active다');
  assert(persisted.canvasBg === 'rgb(255, 255, 255)', 'TC3: 새로고침 후에도 편집 용지는 흰색이다');

  await selectTheme(page, 'light');
  const light = await getThemeState(page);
  await screenshot(page, 'theme-mode-02-light');
  assert(light.mode === 'light', 'TC4: light 명령 후 theme mode가 light다');
  assert(light.effective === 'light', 'TC4: light 명령 후 effective theme도 light다');
  assert(light.colorScheme === 'light only', 'TC4: color-scheme도 light only로 반영된다');
  assert(light.colorSchemeMeta === 'only light', 'TC4: color-scheme meta도 only light로 반영된다');
  assert(light.activeModes.length === 1 && light.activeModes[0] === 'light', 'TC4: 밝게 메뉴만 active다');
  assert(light.storedMode === 'light', 'TC4: localStorage에 light가 저장된다');
  assert(light.themeColor === '#f5f5f5', 'TC4: 브라우저 theme-color가 light token으로 갱신된다');
  assert(light.canvasBg === 'rgb(255, 255, 255)', 'TC4: light에서도 편집 용지는 흰색이다');
}, { skipLoadApp: true });
