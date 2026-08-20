/**
 * E2E 테스트 — 보기 > 도구 상자(기본/서식) 표시 상태 저장·복원 (#5738)
 *
 * 검증 항목:
 * 1. 저장값이 없으면 기본값은 보임
 * 2. 숨기기로 저장한 뒤 리로드하면 숨김이 복원되고, 첫 페인트부터 숨겨져 깜빡임이 없다
 * 3. 메뉴 항목이 활성이고 현재 켜짐/꺼짐을 active + aria-checked 로 표시한다
 * 4. 토글이 rhwp-settings 의 view.toolbarBasic / view.toolbarFormat 에 저장된다
 */

import { runTest, loadApp, assert } from './helpers.mjs';

process.env.VITE_URL = process.env.VITE_URL || 'http://localhost:7700';

const readBars = () => ({
  rootBasic: document.documentElement.dataset.toolboxBasic,
  rootFormat: document.documentElement.dataset.toolboxFormat,
  icon: getComputedStyle(document.getElementById('icon-toolbar')).display,
  style: getComputedStyle(document.getElementById('style-bar')).display,
});

const readMenu = () => ['view:toolbox-basic', 'view:toolbox-format'].map(cmd => {
  const el = document.querySelector(`[data-cmd="${cmd}"]`);
  return {
    cmd,
    active: el?.classList.contains('active'),
    checked: el?.getAttribute('aria-checked'),
    disabled: el?.classList.contains('disabled'),
  };
});

runTest('도구 상자 표시 상태 저장·복원', async ({ page }) => {
  // ── TC1: 저장값 없음 → 기본값 보임 ─────────────────────────
  await page.evaluate(() => localStorage.removeItem('rhwp-settings'));
  await loadApp(page);
  const first = await page.evaluate(readBars);
  assert(first.rootBasic === 'shown' && first.rootFormat === 'shown',
    `TC1: 저장값이 없으면 기본값은 보임 (${first.rootBasic}/${first.rootFormat})`);
  assert(first.icon !== 'none' && first.style !== 'none',
    `TC1: 두 도구 모음이 그려짐 (${first.icon}/${first.style})`);

  // ── TC2: 메뉴 항목은 활성이고 켜짐을 표시한다 ──────────────
  const menuShown = await page.evaluate(readMenu);
  assert(menuShown.every(m => m.disabled === false),
    'TC2: 두 메뉴 항목이 비활성이 아님');
  assert(menuShown.every(m => m.active === true && m.checked === 'true'),
    `TC2: 보임 상태가 체크로 표시됨 (${JSON.stringify(menuShown)})`);

  // ── TC3: 토글이 설정에 저장된다 ────────────────────────────
  const toggled = await page.evaluate(async () => {
    const a = window.rhwpStudio?.automation;
    await a.execute('view:toolbox-basic');
    await a.execute('view:toolbox-format');
    return {
      stored: JSON.parse(localStorage.getItem('rhwp-settings')).view,
      bars: {
        rootBasic: document.documentElement.dataset.toolboxBasic,
        icon: getComputedStyle(document.getElementById('icon-toolbar')).display,
        style: getComputedStyle(document.getElementById('style-bar')).display,
      },
    };
  });
  assert(toggled.stored.toolbarBasic === false && toggled.stored.toolbarFormat === false,
    `TC3: 숨김이 rhwp-settings 에 저장됨 (${JSON.stringify(toggled.stored)})`);
  assert(toggled.bars.icon === 'none' && toggled.bars.style === 'none',
    `TC3: 토글 즉시 두 도구 모음이 숨겨짐 (${JSON.stringify(toggled.bars)})`);

  // ── TC4: 리로드 복원 + 첫 페인트부터 숨김(깜빡임 없음) ─────
  await page.evaluateOnNewDocument(() => {
    window.__toolboxSamples = [];
    const sample = () => {
      const icon = document.getElementById('icon-toolbar');
      const style = document.getElementById('style-bar');
      if (icon && style) {
        window.__toolboxSamples.push({
          icon: getComputedStyle(icon).display,
          style: getComputedStyle(style).display,
        });
      }
      if (window.__toolboxSamples.length < 60) requestAnimationFrame(sample);
    };
    document.addEventListener('DOMContentLoaded', sample);
  });
  await loadApp(page);
  const restored = await page.evaluate(readBars);
  assert(restored.rootBasic === 'hidden' && restored.rootFormat === 'hidden',
    `TC4: 재시작 후 숨김이 복원됨 (${restored.rootBasic}/${restored.rootFormat})`);

  const samples = await page.evaluate(() => window.__toolboxSamples ?? []);
  const flashes = samples.filter(s => s.icon !== 'none' || s.style !== 'none');
  assert(samples.length > 0, `TC4: 프레임 샘플 확보 (${samples.length}개)`);
  assert(flashes.length === 0,
    `TC4: 숨긴 도구 모음이 보인 프레임 없음 (${flashes.length}/${samples.length})`);

  const menuHidden = await page.evaluate(readMenu);
  assert(menuHidden.every(m => m.active === false && m.checked === 'false'),
    `TC4: 숨김 상태가 체크 해제로 표시됨 (${JSON.stringify(menuHidden)})`);

  // 다음 실행에 영향을 주지 않도록 되돌린다.
  await page.evaluate(() => localStorage.removeItem('rhwp-settings'));
});
