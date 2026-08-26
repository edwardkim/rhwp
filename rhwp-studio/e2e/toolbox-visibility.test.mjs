/**
 * E2E 테스트 — 기본 도구 상자 접기/펴기와 표시 상태 저장·복원
 *
 * 검증 항목:
 * 1. 저장값이 없으면 기본 도구 상자는 접힘, 서식 도구 상자는 보임
 * 2. 메뉴·우측 버튼·Ctrl+F1이 같은 커맨드와 상태를 공유한다
 * 3. 토글이 rhwp-settings 의 view.toolbarBasic / view.toolbarFormat 에 저장된다
 * 4. 숨기기로 저장한 뒤 리로드하면 첫 페인트부터 숨겨져 깜빡임이 없다
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
  const el = document.querySelector(`.md-item[data-cmd="${cmd}"]`);
  return {
    cmd,
    active: el?.classList.contains('active'),
    checked: el?.getAttribute('aria-checked'),
    disabled: el?.classList.contains('disabled'),
  };
});

const readToggleButton = () => {
  const el = document.getElementById('toolbox-basic-toggle');
  return {
    active: el?.classList.contains('active'),
    expanded: el?.getAttribute('aria-expanded'),
    label: el?.getAttribute('aria-label'),
    title: el?.getAttribute('title'),
  };
};

runTest('도구 상자 표시 상태 저장·복원', async ({ page }) => {
  // ── TC1: 저장값 없음 → 기본은 접힘, 서식은 보임 ─────────────
  await page.evaluate(() => {
    // 스킨 온보딩은 이미 마쳤지만 view.toolbar* 키는 전혀 없는 상태를 만든다.
    localStorage.setItem('rhwp-settings', JSON.stringify({
      theme: { mode: 'system', skin: 'default', skinChosen: true },
    }));
  });
  await loadApp(page);
  const first = await page.evaluate(readBars);
  assert(first.rootBasic === 'hidden' && first.rootFormat === 'shown',
    `TC1: 저장값이 없으면 기본은 접힘, 서식은 보임 (${first.rootBasic}/${first.rootFormat})`);
  assert(first.icon === 'none' && first.style !== 'none',
    `TC1: 기본 도구 모음만 숨겨짐 (${first.icon}/${first.style})`);

  // ── TC2: 메뉴와 우측 버튼이 같은 상태를 표시한다 ───────────
  const menuShown = await page.evaluate(readMenu);
  assert(menuShown.every(m => m.disabled === false),
    'TC2: 두 메뉴 항목이 비활성이 아님');
  assert(menuShown[0].active === false && menuShown[0].checked === 'false'
      && menuShown[1].active === true && menuShown[1].checked === 'true',
    `TC2: 각 표시 상태가 체크로 표시됨 (${JSON.stringify(menuShown)})`);
  const buttonCollapsed = await page.evaluate(readToggleButton);
  assert(buttonCollapsed.active === false && buttonCollapsed.expanded === 'false'
      && buttonCollapsed.label === '기본 도구 상자 펴기'
      && buttonCollapsed.title === '기본 도구 상자 펴기 (Ctrl+F1)',
    `TC2: 접힌 버튼 상태와 설명이 맞음 (${JSON.stringify(buttonCollapsed)})`);

  // ── TC3: 우측 버튼도 기존 커맨드를 실행하고 설정에 저장한다 ─
  await page.click('#toolbox-basic-toggle');
  const toggled = await page.evaluate(async () => {
    const a = window.rhwpStudio?.automation;
    await a.execute('view:toolbox-format');
    return {
      stored: JSON.parse(localStorage.getItem('rhwp-settings')).view,
      bars: {
        rootBasic: document.documentElement.dataset.toolboxBasic,
        icon: getComputedStyle(document.getElementById('icon-toolbar')).display,
        style: getComputedStyle(document.getElementById('style-bar')).display,
      },
      button: {
        expanded: document.getElementById('toolbox-basic-toggle').getAttribute('aria-expanded'),
        label: document.getElementById('toolbox-basic-toggle').getAttribute('aria-label'),
      },
    };
  });
  assert(toggled.stored.toolbarBasic === true && toggled.stored.toolbarFormat === false,
    `TC3: 버튼 토글이 rhwp-settings 에 저장됨 (${JSON.stringify(toggled.stored)})`);
  assert(toggled.bars.icon !== 'none' && toggled.bars.style === 'none',
    `TC3: 버튼으로 기본만 펴짐 (${JSON.stringify(toggled.bars)})`);
  assert(toggled.button.expanded === 'true' && toggled.button.label === '기본 도구 상자 접기',
    `TC3: 펼친 버튼 상태와 설명이 맞음 (${JSON.stringify(toggled.button)})`);

  // ── TC4: Ctrl+F1도 같은 커맨드로 다시 접는다 ───────────────
  await page.keyboard.down('Control');
  await page.keyboard.press('F1');
  await page.keyboard.up('Control');
  const shortcut = await page.evaluate(() => {
    const toggle = document.getElementById('toolbox-basic-toggle');
    return {
      stored: JSON.parse(localStorage.getItem('rhwp-settings')).view,
      bars: {
        icon: getComputedStyle(document.getElementById('icon-toolbar')).display,
        style: getComputedStyle(document.getElementById('style-bar')).display,
      },
      button: { expanded: toggle?.getAttribute('aria-expanded') },
    };
  });
  assert(shortcut.stored.toolbarBasic === false && shortcut.stored.toolbarFormat === false,
    `TC4: Ctrl+F1 토글이 같은 설정에 저장됨 (${JSON.stringify(shortcut.stored)})`);
  assert(shortcut.bars.icon === 'none' && shortcut.bars.style === 'none'
      && shortcut.button.expanded === 'false',
    `TC4: Ctrl+F1로 기본 도구 상자가 접힘 (${JSON.stringify(shortcut)})`);

  // ── TC5: 리로드 복원 + 첫 페인트부터 숨김(깜빡임 없음) ─────
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
    `TC5: 재시작 후 숨김이 복원됨 (${restored.rootBasic}/${restored.rootFormat})`);

  const samples = await page.evaluate(() => window.__toolboxSamples ?? []);
  const flashes = samples.filter(s => s.icon !== 'none' || s.style !== 'none');
  assert(samples.length > 0, `TC5: 프레임 샘플 확보 (${samples.length}개)`);
  assert(flashes.length === 0,
    `TC5: 숨긴 도구 모음이 보인 프레임 없음 (${flashes.length}/${samples.length})`);

  const menuHidden = await page.evaluate(readMenu);
  assert(menuHidden.every(m => m.active === false && m.checked === 'false'),
    `TC5: 숨김 상태가 체크 해제로 표시됨 (${JSON.stringify(menuHidden)})`);

  // 다음 실행에 영향을 주지 않도록 되돌린다.
  await page.evaluate(() => localStorage.removeItem('rhwp-settings'));
});
