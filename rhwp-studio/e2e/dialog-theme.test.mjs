/**
 * E2E 테스트 — 다이얼로그 다크 테마 색상 정책
 *
 * 검증 항목:
 * 1. dark 테마에서 공통 입력 필드와 fieldset 계열이 UI dark token을 따른다.
 * 2. 수식/문단/쪽 테두리 preview의 문서 종이 배경은 흰색으로 유지된다.
 * 3. 쪽 테두리 preview 사방 버튼은 dark UI token을 따르고, 클릭 시 문서 preview 선이 추가된다.
 * 4. toolbar popup 계열도 dark UI token을 따른다.
 */

import {
  runTest, loadApp, createNewDocument, assert,
} from './helpers.mjs';

const delay = (ms = 200) => new Promise((resolve) => setTimeout(resolve, ms));

function colorParts(value) {
  const match = String(value || '').match(/rgba?\(([^)]+)\)/);
  if (!match) return null;
  const [r, g, b, a = '1'] = match[1].split(',').map((part) => Number.parseFloat(part.trim()));
  if (![r, g, b].every(Number.isFinite)) return null;
  return { r, g, b, a: Number.isFinite(a) ? a : 1 };
}

function isWhite(value) {
  const rgb = colorParts(value);
  return Boolean(rgb && rgb.r >= 245 && rgb.g >= 245 && rgb.b >= 245 && rgb.a > 0);
}

function isDarkSurface(value) {
  const rgb = colorParts(value);
  return Boolean(rgb && rgb.r <= 130 && rgb.g <= 130 && rgb.b <= 140 && rgb.a > 0);
}

function isVisibleLightText(value) {
  const rgb = colorParts(value);
  return Boolean(rgb && rgb.r >= 135 && rgb.g >= 135 && rgb.b >= 135 && rgb.a > 0);
}

async function setDarkTheme(page) {
  await page.evaluate(() => {
    localStorage.removeItem('rhwp-settings');
    window.__theme?.setThemeMode?.('dark');
  });
  await delay();
}

async function openMenuCommand(page, commandId) {
  await page.evaluate((cmd) => {
    const item = document.querySelector(`.md-item[data-cmd="${cmd}"]`);
    if (!item) throw new Error(`메뉴 항목을 찾을 수 없습니다: ${cmd}`);

    const menu = item.closest('.menu-item');
    const title = menu?.querySelector('.menu-title');
    if (title) {
      title.dispatchEvent(new MouseEvent('mousedown', { bubbles: true, cancelable: true }));
    }
    if (item.classList.contains('disabled')) {
      throw new Error(`메뉴 항목이 비활성 상태입니다: ${cmd}`);
    }
    item.dispatchEvent(new MouseEvent('click', { bubbles: true, cancelable: true }));
  }, commandId);
  await delay(300);
}

async function closeTopDialog(page) {
  await page.evaluate(() => {
    const closeButtons = Array.from(document.querySelectorAll('.dialog-close'));
    closeButtons.at(-1)?.dispatchEvent(new MouseEvent('click', { bubbles: true, cancelable: true }));
  });
  await delay(150);
}

async function getRootThemeState(page) {
  return await page.evaluate(() => ({
    mode: document.documentElement.dataset.themeMode || '',
    effective: document.documentElement.dataset.themeEffective || '',
    colorScheme: document.documentElement.style.colorScheme,
  }));
}

async function getGridDialogStyles(page) {
  await page.waitForSelector('.dialog-wrap .dialog-input[name="grid-horz"]', { timeout: 5000 });
  return await page.evaluate(() => {
    const input = document.querySelector('.dialog-input[name="grid-horz"]');
    const fieldset = input?.closest('fieldset');
    const legend = fieldset?.querySelector('legend');
    const button = document.querySelector('.dialog-wrap .dialog-btn:not(.dialog-btn-primary)');
    const inputStyle = input ? getComputedStyle(input) : null;
    const fieldsetStyle = fieldset ? getComputedStyle(fieldset) : null;
    const legendStyle = legend ? getComputedStyle(legend) : null;
    const buttonStyle = button ? getComputedStyle(button) : null;
    return {
      inputBg: inputStyle?.backgroundColor || '',
      inputColor: inputStyle?.color || '',
      inputBorder: inputStyle?.borderTopColor || '',
      fieldsetBorder: fieldsetStyle?.borderTopColor || '',
      legendColor: legendStyle?.color || '',
      buttonBg: buttonStyle?.backgroundColor || '',
      buttonColor: buttonStyle?.color || '',
    };
  });
}

async function getEquationDialogStyles(page) {
  await page.waitForSelector('.eq-preview', { timeout: 5000 });
  return await page.evaluate(() => {
    const preview = document.querySelector('.eq-preview');
    const script = document.querySelector('.eq-script');
    const input = document.querySelector('.eq-props-row .dialog-input');
    const previewStyle = preview ? getComputedStyle(preview) : null;
    const scriptStyle = script ? getComputedStyle(script) : null;
    const inputStyle = input ? getComputedStyle(input) : null;
    return {
      previewBg: previewStyle?.backgroundColor || '',
      previewBorder: previewStyle?.borderTopColor || '',
      scriptBg: scriptStyle?.backgroundColor || '',
      scriptColor: scriptStyle?.color || '',
      inputBg: inputStyle?.backgroundColor || '',
      inputColor: inputStyle?.color || '',
    };
  });
}

async function getPageBorderStyles(page) {
  await page.waitForSelector('.page-border-side-btn', { timeout: 5000 });
  return await page.evaluate(() => {
    const sideButton = document.querySelector('.page-border-side-btn');
    const fieldset = document.querySelector('.dialog-wrap fieldset');
    const legend = fieldset?.querySelector('legend');
    const svg = document.querySelector('.dialog-wrap svg');
    const rect = svg?.querySelector('rect');
    const sideButtonStyle = sideButton ? getComputedStyle(sideButton) : null;
    const fieldsetStyle = fieldset ? getComputedStyle(fieldset) : null;
    const legendStyle = legend ? getComputedStyle(legend) : null;
    const svgStyle = svg ? getComputedStyle(svg) : null;
    const rectStyle = rect ? getComputedStyle(rect) : null;
    return {
      sideButtonBg: sideButtonStyle?.backgroundColor || '',
      sideButtonColor: sideButtonStyle?.color || '',
      sideButtonBorder: sideButtonStyle?.borderTopColor || '',
      fieldsetBorder: fieldsetStyle?.borderTopColor || '',
      legendColor: legendStyle?.color || '',
      svgBg: svgStyle?.backgroundColor || '',
      rectFill: rectStyle?.fill || '',
      rectStroke: rectStyle?.stroke || '',
      lineCount: svg?.querySelectorAll('line').length ?? -1,
    };
  });
}

async function applyTopPageBorder(page) {
  await page.evaluate(() => {
    const topButton = Array.from(document.querySelectorAll('.page-border-side-btn'))
      .find((button) => button.getAttribute('title') === '위쪽');
    if (!topButton) throw new Error('쪽 테두리 위쪽 버튼을 찾을 수 없습니다');
    topButton.dispatchEvent(new MouseEvent('click', { bubbles: true, cancelable: true }));
  });
  await delay(150);
}

async function getParaShapeStyles(page) {
  await page.waitForSelector('.ps-preview', { timeout: 5000 });
  return await page.evaluate(() => {
    const preview = document.querySelector('.ps-preview');
    const input = document.querySelector('.ps-dialog .dialog-input');
    const activeAlign = document.querySelector('.ps-align-btn.active');
    const previewStyle = preview ? getComputedStyle(preview) : null;
    const inputStyle = input ? getComputedStyle(input) : null;
    const activeAlignStyle = activeAlign ? getComputedStyle(activeAlign) : null;
    return {
      previewBg: previewStyle?.backgroundColor || '',
      previewColor: previewStyle?.color || '',
      inputBg: inputStyle?.backgroundColor || '',
      inputColor: inputStyle?.color || '',
      activeAlignBg: activeAlignStyle?.backgroundColor || '',
      activeAlignColor: activeAlignStyle?.color || '',
    };
  });
}

async function openBulletPopup(page) {
  await page.evaluate(() => {
    const button = document.getElementById('tb-bullet');
    if (!button) throw new Error('글머리표 toolbar 버튼을 찾을 수 없습니다');
    button.dispatchEvent(new MouseEvent('mousedown', { bubbles: true, cancelable: true }));
  });
  await delay(150);
}

async function getBulletPopupStyles(page) {
  await page.waitForSelector('.bullet-popup button', { timeout: 5000 });
  return await page.evaluate(() => {
    const popup = document.querySelector('.bullet-popup');
    const button = popup?.querySelector('button');
    const popupStyle = popup ? getComputedStyle(popup) : null;
    const buttonStyle = button ? getComputedStyle(button) : null;
    return {
      popupBg: popupStyle?.backgroundColor || '',
      popupColor: popupStyle?.color || '',
      popupBorder: popupStyle?.borderTopColor || '',
      buttonBg: buttonStyle?.backgroundColor || '',
      buttonColor: buttonStyle?.color || '',
      buttonBorder: buttonStyle?.borderTopColor || '',
    };
  });
}

async function closeBulletPopup(page) {
  await page.evaluate(() => document.querySelector('.bullet-popup')?.remove());
  await delay(100);
}

async function openTableQuickGrid(page) {
  await page.evaluate(() => {
    const button = document.querySelector('button[data-cmd="table:create"]');
    if (!button) throw new Error('표 만들기 toolbar 버튼을 찾을 수 없습니다');
    button.dispatchEvent(new MouseEvent('mousedown', { bubbles: true, cancelable: true }));
  });
  await delay(150);
}

async function getTableQuickGridStyles(page) {
  await page.waitForFunction(() => {
    return Array.from(document.body.children).some((element) => {
      const style = getComputedStyle(element);
      return style.position === 'fixed'
        && style.zIndex === '9999'
        && element.textContent?.includes('표 만들기...');
    });
  }, { timeout: 5000 });
  return await page.evaluate(() => {
    const popup = Array.from(document.body.children).find((element) => {
      const style = getComputedStyle(element);
      return style.position === 'fixed'
        && style.zIndex === '9999'
        && element.textContent?.includes('표 만들기...');
    });
    const gridCell = popup?.querySelector('[data-row][data-col]');
    const cancelButton = popup?.querySelector('button');
    const popupStyle = popup ? getComputedStyle(popup) : null;
    const gridCellStyle = gridCell ? getComputedStyle(gridCell) : null;
    const cancelStyle = cancelButton ? getComputedStyle(cancelButton) : null;
    return {
      popupBg: popupStyle?.backgroundColor || '',
      popupColor: popupStyle?.color || '',
      popupBorder: popupStyle?.borderTopColor || '',
      gridCellBg: gridCellStyle?.backgroundColor || '',
      gridCellBorder: gridCellStyle?.borderTopColor || '',
      cancelBg: cancelStyle?.backgroundColor || '',
      cancelColor: cancelStyle?.color || '',
    };
  });
}

async function closeTableQuickGrid(page) {
  await page.evaluate(() => {
    Array.from(document.body.children).forEach((element) => {
      const style = getComputedStyle(element);
      if (
        style.position === 'fixed'
        && (style.zIndex === '9998' || style.zIndex === '9999')
        && (element.textContent?.includes('표 만들기...') || element.childElementCount === 0)
      ) {
        element.remove();
      }
    });
  });
  await delay(100);
}

runTest('다이얼로그 다크 테마 색상 정책', async ({ page }) => {
  await loadApp(page);
  await setDarkTheme(page);
  await createNewDocument(page);

  const root = await getRootThemeState(page);
  assert(root.mode === 'dark', 'TC1: 테스트 시작 전 명시적 dark 테마가 적용된다');
  assert(root.effective === 'dark', 'TC1: effective theme도 dark다');
  assert(root.colorScheme === 'dark only', 'TC1: color-scheme도 dark only다');

  await openMenuCommand(page, 'view:grid-settings');
  const grid = await getGridDialogStyles(page);
  assert(isDarkSurface(grid.inputBg), 'TC2: 격자 설정 dialog-input 배경은 dark UI surface다');
  assert(isVisibleLightText(grid.inputColor), 'TC2: 격자 설정 dialog-input 글자는 dark에서 읽힌다');
  assert(isDarkSurface(grid.inputBorder), 'TC2: 격자 설정 dialog-input 테두리는 dark UI token이다');
  assert(isDarkSurface(grid.fieldsetBorder), 'TC2: 격자 설정 fieldset 테두리는 dark UI token이다');
  assert(isVisibleLightText(grid.legendColor), 'TC2: 격자 설정 legend 글자는 dark에서 읽힌다');
  assert(isDarkSurface(grid.buttonBg), 'TC2: 격자 설정 보조 버튼 배경은 dark UI surface다');
  assert(isVisibleLightText(grid.buttonColor), 'TC2: 격자 설정 보조 버튼 글자는 dark에서 읽힌다');
  await closeTopDialog(page);

  await openMenuCommand(page, 'insert:equation');
  const equation = await getEquationDialogStyles(page);
  assert(isWhite(equation.previewBg), 'TC3: 수식 편집 미리보기 배경은 문서 종이 흰색으로 유지된다');
  assert(isDarkSurface(equation.previewBorder), 'TC3: 수식 편집 미리보기 테두리는 dark UI token이다');
  assert(isDarkSurface(equation.scriptBg), 'TC3: 수식 편집 스크립트 입력 배경은 dark UI surface다');
  assert(isVisibleLightText(equation.scriptColor), 'TC3: 수식 편집 스크립트 입력 글자는 dark에서 읽힌다');
  assert(isDarkSurface(equation.inputBg), 'TC3: 수식 편집 dialog-input 배경은 dark UI surface다');
  await closeTopDialog(page);

  await openMenuCommand(page, 'page:page-border');
  const pageBorderBefore = await getPageBorderStyles(page);
  assert(isDarkSurface(pageBorderBefore.sideButtonBg), 'TC4: 쪽 테두리 사방 버튼 배경은 dark UI surface다');
  assert(isVisibleLightText(pageBorderBefore.sideButtonColor), 'TC4: 쪽 테두리 사방 버튼 기호는 dark에서 읽힌다');
  assert(isDarkSurface(pageBorderBefore.sideButtonBorder), 'TC4: 쪽 테두리 사방 버튼 테두리는 dark UI token이다');
  assert(isDarkSurface(pageBorderBefore.fieldsetBorder), 'TC4: 쪽 테두리 fieldset 테두리는 dark UI token이다');
  assert(isVisibleLightText(pageBorderBefore.legendColor), 'TC4: 쪽 테두리 legend 글자는 dark에서 읽힌다');
  assert(isWhite(pageBorderBefore.svgBg), 'TC4: 쪽 테두리 중앙 SVG 배경은 문서 종이 흰색으로 유지된다');
  assert(isWhite(pageBorderBefore.rectFill), 'TC4: 쪽 테두리 중앙 rect fill은 문서 종이 흰색으로 유지된다');
  assert(pageBorderBefore.rectStroke === 'rgb(208, 208, 208)', 'TC4: 쪽 테두리 preview 가이드는 문서 preview 고정 회색이다');
  await applyTopPageBorder(page);
  const pageBorderAfter = await getPageBorderStyles(page);
  assert(pageBorderAfter.lineCount > pageBorderBefore.lineCount, 'TC4: dark에서도 사방 버튼 클릭 시 preview 선이 추가된다');
  await closeTopDialog(page);

  await openMenuCommand(page, 'format:para-shape');
  const para = await getParaShapeStyles(page);
  assert(isWhite(para.previewBg), 'TC5: 문단 모양 preview 배경은 문서 종이 흰색으로 유지된다');
  assert(para.previewColor === 'rgb(17, 17, 17)', 'TC5: 문단 모양 preview 텍스트는 문서 preview 검정 계열이다');
  assert(isDarkSurface(para.inputBg), 'TC5: 문단 모양 dialog-input 배경은 dark UI surface다');
  assert(isVisibleLightText(para.inputColor), 'TC5: 문단 모양 dialog-input 글자는 dark에서 읽힌다');
  assert(isDarkSurface(para.activeAlignBg), 'TC5: 문단 모양 정렬 버튼 active 배경은 dark UI token이다');
  assert(isVisibleLightText(para.activeAlignColor), 'TC5: 문단 모양 정렬 버튼 active 기호는 dark에서 읽힌다');
  await closeTopDialog(page);

  await openBulletPopup(page);
  const bullet = await getBulletPopupStyles(page);
  assert(isDarkSurface(bullet.popupBg), 'TC6: 글머리표 popup 배경은 dark UI surface다');
  assert(isVisibleLightText(bullet.popupColor), 'TC6: 글머리표 popup 글자는 dark에서 읽힌다');
  assert(isDarkSurface(bullet.popupBorder), 'TC6: 글머리표 popup 테두리는 dark UI token이다');
  assert(isDarkSurface(bullet.buttonBg), 'TC6: 글머리표 버튼 배경은 dark UI surface다');
  assert(isVisibleLightText(bullet.buttonColor), 'TC6: 글머리표 버튼 글자는 dark에서 읽힌다');
  assert(isDarkSurface(bullet.buttonBorder), 'TC6: 글머리표 버튼 테두리는 dark UI token이다');
  await closeBulletPopup(page);

  await openTableQuickGrid(page);
  const tableGrid = await getTableQuickGridStyles(page);
  assert(isDarkSurface(tableGrid.popupBg), 'TC7: 표 만들기 quick grid popup 배경은 dark UI surface다');
  assert(isVisibleLightText(tableGrid.popupColor), 'TC7: 표 만들기 quick grid popup 글자는 dark에서 읽힌다');
  assert(isDarkSurface(tableGrid.popupBorder), 'TC7: 표 만들기 quick grid popup 테두리는 dark UI token이다');
  assert(isDarkSurface(tableGrid.gridCellBg), 'TC7: 표 만들기 quick grid 셀 배경은 dark UI surface다');
  assert(isDarkSurface(tableGrid.gridCellBorder), 'TC7: 표 만들기 quick grid 셀 테두리는 dark UI token이다');
  assert(isDarkSurface(tableGrid.cancelBg), 'TC7: 표 만들기 quick grid 취소 버튼 배경은 dark UI surface다');
  assert(isVisibleLightText(tableGrid.cancelColor), 'TC7: 표 만들기 quick grid 취소 버튼 글자는 dark에서 읽힌다');
  await closeTableQuickGrid(page);
}, { skipLoadApp: true });
