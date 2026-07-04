/**
 * E2E 테스트 — 초기 테마 bootstrap
 *
 * 검증 항목:
 * 1. 저장된 dark 테마가 앱 모듈 초기화 전에 documentElement에 반영된다.
 * 2. 첫 stylesheet보다 앞선 inline bootstrap이 color-scheme/meta/theme-color를 맞춘다.
 */

import {
  runTest, assert,
} from './helpers.mjs';

const VITE_URL = process.env.VITE_URL || 'http://localhost:7700';

runTest('초기 테마 bootstrap', async ({ page }) => {
  await page.evaluateOnNewDocument(() => {
    localStorage.setItem('rhwp-settings', JSON.stringify({
      version: 1,
      theme: { mode: 'dark' },
    }));
  });

  await page.goto(`${VITE_URL}?theme-bootstrap=${Date.now()}`, {
    waitUntil: 'domcontentloaded',
    timeout: 30000,
  });

  const bootstrapped = await page.evaluate(() => {
    const root = document.documentElement;
    return {
      mode: root.dataset.themeMode || '',
      effective: root.dataset.themeEffective || '',
      colorScheme: root.style.colorScheme,
      colorSchemeMeta: document.querySelector('meta[name="color-scheme"]')?.getAttribute('content') || '',
      themeColor: document.querySelector('meta[name="theme-color"]')?.getAttribute('content') || '',
      menuBarBg: getComputedStyle(document.getElementById('menu-bar')).backgroundColor,
    };
  });

  assert(bootstrapped.mode === 'dark', 'TC1: DOMContentLoaded 시점에 저장된 dark mode가 root dataset에 반영된다');
  assert(bootstrapped.effective === 'dark', 'TC1: DOMContentLoaded 시점에 effective theme도 dark다');
  assert(bootstrapped.colorScheme === 'dark only', 'TC1: DOMContentLoaded 시점에 root color-scheme이 dark only다');
  assert(bootstrapped.colorSchemeMeta === 'only dark', 'TC1: DOMContentLoaded 시점에 meta color-scheme이 only dark다');
  assert(bootstrapped.themeColor === '#2b3037', 'TC1: DOMContentLoaded 시점에 theme-color가 dark token이다');
  assert(bootstrapped.menuBarBg === 'rgb(43, 48, 55)', 'TC1: 초기 paint 대상 menu-bar가 dark token으로 계산된다');
}, { skipLoadApp: true });
