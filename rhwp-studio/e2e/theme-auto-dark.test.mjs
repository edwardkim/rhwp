/**
 * E2E 테스트 — Chrome Auto Dark Mode 대응
 *
 * 실행 예:
 *   CHROME_EXTRA_ARGS=--enable-features=WebContentsForceDark \
 *     node e2e/theme-auto-dark.test.mjs --mode=headless
 */

import { PNG } from 'pngjs';
import {
  runTest, loadApp, createNewDocument, assert,
} from './helpers.mjs';

const delay = (ms = 200) => new Promise((resolve) => setTimeout(resolve, ms));

async function setTheme(page, mode) {
  await page.evaluate((selectedMode) => window.__theme?.setThemeMode?.(selectedMode), mode);
  await delay();
}

async function getThemeState(page) {
  return await page.evaluate(() => {
    const root = document.documentElement;
    return {
      mode: root.dataset.themeMode || '',
      effective: root.dataset.themeEffective || '',
      colorScheme: root.style.colorScheme,
      colorSchemeMeta: document.querySelector('meta[name="color-scheme"]')?.getAttribute('content') || '',
      themeColor: document.querySelector('meta[name="theme-color"]')?.getAttribute('content') || '',
    };
  });
}

async function detectAutoDark(page) {
  return await page.evaluate(() => {
    const detection = document.createElement('div');
    detection.style.display = 'none';
    detection.style.backgroundColor = 'canvas';
    detection.style.colorScheme = 'light';
    document.body.appendChild(detection);
    const bg = getComputedStyle(detection).backgroundColor;
    detection.remove();
    return bg !== 'rgb(255, 255, 255)';
  });
}

async function sampleElementPixel(page, selector) {
  const point = await page.evaluate((targetSelector) => {
    const element = document.querySelector(targetSelector);
    if (!element) throw new Error(`픽셀 샘플 대상 없음: ${targetSelector}`);
    const rect = element.getBoundingClientRect();
    return {
      x: Math.max(0, Math.floor(rect.right - 20)),
      y: Math.max(0, Math.floor(rect.top + rect.height / 2)),
    };
  }, selector);
  const buffer = await page.screenshot({ encoding: 'binary', fullPage: false });
  const png = PNG.sync.read(buffer);
  const x = Math.min(png.width - 1, point.x);
  const y = Math.min(png.height - 1, point.y);
  const i = (y * png.width + x) * 4;
  return {
    r: png.data[i],
    g: png.data[i + 1],
    b: png.data[i + 2],
  };
}

function isLightPixel(pixel) {
  return pixel.r >= 210 && pixel.g >= 210 && pixel.b >= 210;
}

function isDarkPixel(pixel) {
  return pixel.r <= 100 && pixel.g <= 110 && pixel.b <= 125;
}

runTest('Chrome Auto Dark Mode 대응', async ({ page }) => {
  await loadApp(page);
  await page.evaluate(() => localStorage.removeItem('rhwp-settings'));
  await setTheme(page, 'light');
  await createNewDocument(page);

  const autoDark = await detectAutoDark(page);
  assert(autoDark, 'TC1: 테스트 Chrome에서 Auto Dark Mode가 활성화되어 있다');

  const light = await getThemeState(page);
  assert(light.mode === 'light', 'TC2: 명시적 밝게 테마가 적용된다');
  assert(light.effective === 'light', 'TC2: effective theme도 light다');
  assert(light.colorScheme === 'light only', 'TC2: root color-scheme이 light only다');
  assert(light.colorSchemeMeta === 'only light', 'TC2: meta color-scheme이 only light다');
  assert(light.themeColor === '#f5f5f5', 'TC2: theme-color는 light token이다');

  const lightMenuPixel = await sampleElementPixel(page, '#menu-bar');
  assert(
    isLightPixel(lightMenuPixel),
    `TC2: Auto Dark Mode에서도 밝게 테마 menu-bar가 밝게 페인트된다 (${lightMenuPixel.r},${lightMenuPixel.g},${lightMenuPixel.b})`,
  );

  await setTheme(page, 'dark');
  const dark = await getThemeState(page);
  assert(dark.mode === 'dark', 'TC3: 명시적 어둡게 테마가 적용된다');
  assert(dark.effective === 'dark', 'TC3: effective theme도 dark다');
  assert(dark.colorScheme === 'dark only', 'TC3: root color-scheme이 dark only다');
  assert(dark.colorSchemeMeta === 'only dark', 'TC3: meta color-scheme이 only dark다');
  assert(dark.themeColor === '#2b3037', 'TC3: theme-color는 dark token이다');

  const darkMenuPixel = await sampleElementPixel(page, '#menu-bar');
  assert(
    isDarkPixel(darkMenuPixel),
    `TC3: 어둡게 테마는 Chrome 강제 변환이 아니라 앱 dark token으로 페인트된다 (${darkMenuPixel.r},${darkMenuPixel.g},${darkMenuPixel.b})`,
  );
}, { skipLoadApp: true });
