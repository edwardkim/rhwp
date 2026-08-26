/**
 * E2E 테스트: 반응형 레이아웃과 #6118 서식 바 콘텐츠 경계 검증
 */
import { launchBrowser, loadApp, screenshot, closeBrowser, closePage, createPage } from './helpers.mjs';
import { TestReporter } from './report-generator.mjs';

const VIEWPORTS = [
  { name: 'wide-desktop', width: 1920, height: 1080, styleMode: 'full' },
  { name: 'desktop', width: 1280, height: 900, styleMode: 'full' },
  { name: 'narrow-desktop', width: 1024, height: 768, styleMode: 'full' },
  { name: 'full-boundary', width: 976, height: 900, styleMode: 'full' },
  { name: 'two-row-boundary', width: 975, height: 900, styleMode: 'inline' },
  { name: 'compact-desktop', width: 883, height: 900, styleMode: 'inline' },
  { name: 'tablet', width: 768, height: 1024, styleMode: 'inline' },
  { name: 'command-inline-boundary', width: 460, height: 900, styleMode: 'inline' },
  { name: 'command-overflow-boundary', width: 459, height: 900, styleMode: 'overflow' },
  { name: 'mobile-wide', width: 412, height: 915, styleMode: 'overflow' },
  { name: 'mobile-medium', width: 390, height: 844, styleMode: 'overflow' },
  { name: 'mobile', width: 375, height: 812, styleMode: 'overflow' },
];

const THEME_LAYOUTS = [
  { name: 'full', width: 976, height: 900, styleMode: 'full' },
  { name: 'inline', width: 460, height: 900, styleMode: 'inline' },
  { name: 'overflow', width: 375, height: 812, styleMode: 'overflow' },
];

const THEME_CASES = ['default', 'flat', 'oldschool'].flatMap(skin =>
  ['light', 'dark'].flatMap(theme =>
    THEME_LAYOUTS.map(layout => ({ ...layout, skin, theme })),
  ),
);

async function primeTheme(page, skin = 'default', mode = 'light') {
  await page.evaluateOnNewDocument(({ selectedSkin, selectedMode }) => {
    localStorage.setItem('rhwp-settings', JSON.stringify({
      theme: { mode: selectedMode, skin: selectedSkin, skinChosen: true },
    }));
  }, { selectedSkin: skin, selectedMode: mode });
}

async function run() {
  console.log('=== E2E: 반응형 레이아웃 테스트 ===\n');

  const browser = await launchBrowser();
  const reporter = new TestReporter('반응형 레이아웃 테스트');
  let passed = 0;
  let failed = 0;

  const check = (tc, cond, msg) => {
    if (cond) {
      passed++;
      console.log(`  PASS: ${msg}`);
      reporter.pass(tc, msg);
    } else {
      failed++;
      console.error(`  FAIL: ${msg}`);
      reporter.fail(tc, msg);
    }
  };

  for (const vp of VIEWPORTS) {
    const tc = `${vp.name} (${vp.width}x${vp.height})`;
    console.log(`\n[${vp.name}] ${vp.width}x${vp.height}...`);

    const page = await createPage(browser, vp.width, vp.height);

    try {
      await primeTheme(page);
      await loadApp(page);
      await page.evaluate(() => window.__eventBus?.emit('create-new-document'));
      await page.evaluate(() => new Promise(resolve => setTimeout(resolve, 1000)));

      const result = await page.evaluate(() => {
        const canvas = document.querySelector('canvas');
        const menuBar = document.getElementById('menu-bar');
        const toolbar = document.getElementById('icon-toolbar');
        const styleBar = document.getElementById('style-bar');
        const statusBar = document.getElementById('status-bar');
        const editor = document.getElementById('editor-area');
        const field = styleBar?.querySelector('.sb-field-ribbon-group');
        const command = styleBar?.querySelector('.sb-command-track');
        const paragraph = styleBar?.querySelector('.sb-paragraph-ribbon-group');
        const overflowButton = document.getElementById('btn-style-overflow');
        const overflowPanel = document.getElementById('style-overflow-panel');

        const isVisible = (element) => {
          if (!element) return false;
          const style = getComputedStyle(element);
          return style.display !== 'none' && style.visibility !== 'hidden'
            && element.getClientRects().length > 0;
        };
        const top = element => Math.round(element?.getBoundingClientRect().top ?? -1);
        const styleRows = new Set([top(field), top(command)]).size;

        return {
          hasCanvas: !!canvas,
          menuBarVisible: isVisible(menuBar),
          menuBarHeight: menuBar?.offsetHeight ?? 0,
          toolbarVisible: isVisible(toolbar),
          toolbarHeight: toolbar?.offsetHeight ?? 0,
          styleBarVisible: isVisible(styleBar),
          styleBarHeight: styleBar?.offsetHeight ?? 0,
          styleRows,
          paragraphVisible: isVisible(paragraph),
          overflowButtonVisible: isVisible(overflowButton),
          overflowExpanded: overflowButton?.getAttribute('aria-expanded'),
          overflowPanelHidden: overflowPanel?.hidden ?? null,
          statusBarVisible: isVisible(statusBar),
          editorVisible: isVisible(editor),
          pageCount: window.__wasm?.pageCount ?? 0,
          rootClientWidth: document.documentElement.clientWidth,
          rootScrollWidth: document.documentElement.scrollWidth,
          styleClientWidth: styleBar?.clientWidth ?? 0,
          styleScrollWidth: styleBar?.scrollWidth ?? 0,
        };
      });

      check(tc, result.hasCanvas, '캔버스 존재');
      check(tc, result.editorVisible, '편집 영역 표시');
      check(tc, result.pageCount >= 1, `페이지 수: ${result.pageCount}`);
      check(tc, result.menuBarVisible, '메뉴바 표시');
      check(tc, result.styleBarVisible, '서식 도구 표시');
      check(tc, result.statusBarVisible, '상태 표시줄 표시');
      check(tc, result.styleRows <= 2, `서식 바 최대 2행 (rows=${result.styleRows})`);
      check(
        tc,
        result.rootScrollWidth <= result.rootClientWidth,
        `page 가로 overflow 없음 (${result.rootScrollWidth}/${result.rootClientWidth})`,
      );
      check(
        tc,
        result.styleScrollWidth <= result.styleClientWidth,
        `서식 바 가로 overflow 없음 (${result.styleScrollWidth}/${result.styleClientWidth})`,
      );

      if (vp.styleMode === 'full') {
        check(tc, result.styleRows === 1, `전체 압축 1행 (rows=${result.styleRows})`);
        check(tc, result.styleBarHeight <= 36, `전체 압축 높이 36px 이하 (h=${result.styleBarHeight})`);
        check(tc, result.paragraphVisible, '문단 명령 inline 표시');
        check(tc, !result.overflowButtonVisible, '더보기 숨김');
      } else if (vp.styleMode === 'inline') {
        check(tc, result.styleRows === 2, `필드+명령 2행 (rows=${result.styleRows})`);
        check(tc, result.paragraphVisible, '문단 명령 inline 표시');
        check(tc, !result.overflowButtonVisible, '더보기 숨김');
      } else {
        check(tc, result.styleRows === 2, `필드+명령 2행 (rows=${result.styleRows})`);
        check(tc, !result.paragraphVisible, '닫힌 panel의 문단 명령 숨김');
        check(tc, result.overflowButtonVisible, '문단 더보기 표시');
        check(tc, result.overflowExpanded === 'false', '더보기 초기 접힘');
        check(tc, result.overflowPanelHidden === true, '닫힌 panel 접근성 트리 제외');

        const interaction = await page.evaluate(async () => {
          const trigger = document.getElementById('btn-style-overflow');
          const panel = document.getElementById('style-overflow-panel');
          const command = document.getElementById('btn-align-left');
          const selectionCommand = document.getElementById('btn-align-center');
          const paragraphButtons = Array.from(panel?.querySelectorAll('.sb-btn') ?? []);
          const nextFrame = () => new Promise(resolve => requestAnimationFrame(resolve));

          trigger?.focus();
          trigger?.dispatchEvent(new KeyboardEvent('keydown', {
            key: 'ArrowDown', bubbles: true, cancelable: true,
          }));
          await nextFrame();
          const openedByKeyboard = trigger?.getAttribute('aria-expanded') === 'true'
            && panel?.hidden === false
            && document.activeElement === command;
          window.dispatchEvent(new KeyboardEvent('keydown', {
            key: 'Escape', bubbles: true, cancelable: true,
          }));
          const closedByEscape = trigger?.getAttribute('aria-expanded') === 'false'
            && panel?.hidden === true
            && document.activeElement === trigger;

          trigger?.click();
          await nextFrame();
          document.body.dispatchEvent(new PointerEvent('pointerdown', {
            bubbles: true, cancelable: true,
          }));
          const closedByOutside = trigger?.getAttribute('aria-expanded') === 'false'
            && panel?.hidden === true;

          trigger?.click();
          await new Promise(resolve => requestAnimationFrame(resolve));
          const openedByClick = trigger?.getAttribute('aria-expanded') === 'true'
            && panel?.hidden === false
            && document.activeElement === command;
          selectionCommand?.dispatchEvent(new MouseEvent('mousedown', { bubbles: true, cancelable: true }));
          selectionCommand?.click();
          await nextFrame();
          const focusReturned = document.activeElement === trigger;

          const triggerIcon = document.getElementById('style-overflow-current-icon');
          const activeMirrored = selectionCommand?.classList.contains('active') === true
            && trigger?.classList.contains('active') === true
            && trigger?.getAttribute('aria-label')?.includes('현재 가운데 정렬') === true
            && triggerIcon?.classList.contains('sb-al-center') === true;
          for (const button of paragraphButtons) button.disabled = true;
          await new Promise(resolve => setTimeout(resolve, 0));
          const disabledMirrored = trigger?.disabled === true;
          for (const button of paragraphButtons) button.disabled = false;

          return {
            openedByKeyboard,
            closedByEscape,
            closedByOutside,
            openedByClick,
            closedByCommand: trigger?.getAttribute('aria-expanded') === 'false' && panel?.hidden === true,
            focusReturned,
            activeMirrored,
            disabledMirrored,
          };
        });
        check(tc, interaction.openedByKeyboard, 'ArrowDown 열기와 첫 명령 focus');
        check(tc, interaction.closedByEscape, 'Escape 닫기와 trigger focus 복귀');
        check(tc, interaction.closedByOutside, '외부 pointer로 panel 닫힘');
        check(tc, interaction.openedByClick, 'click 열기와 첫 명령 focus');
        check(tc, interaction.closedByCommand, '명령 실행 뒤 panel 닫힘');
        check(tc, interaction.focusReturned, '명령 실행 뒤 trigger focus 복귀');
        check(tc, interaction.activeMirrored, 'paragraph active 상태를 trigger에 표시');
        check(tc, interaction.disabledMirrored, 'paragraph disabled 상태를 trigger에 표시');
      }

      if (vp.name === 'full-boundary') {
        const formatting = await page.evaluate(async () => {
          const fontSize = document.getElementById('font-size');
          const charfxButton = document.getElementById('btn-charfx');
          const charfxDropdown = document.getElementById('charfx-dropdown');
          const charfxItem = document.querySelector('#charfx-menu .sb-dropdown-item');
          const highlightButton = document.getElementById('btn-highlight');
          const highlightDropdown = document.getElementById('highlight-dropdown');
          const highlightSwatch = document.querySelector('#highlight-palette .sb-hl-swatch');
          const highlightBar = document.getElementById('highlight-bar');
          const colorPicker = document.getElementById('text-color-picker');
          const colorBar = document.getElementById('color-bar');

          fontSize.value = '12';
          fontSize.dispatchEvent(new KeyboardEvent('keydown', {
            key: 'Enter', bubbles: true, cancelable: true,
          }));

          charfxButton.dispatchEvent(new MouseEvent('mousedown', { bubbles: true, cancelable: true }));
          const charfxOpened = charfxDropdown.classList.contains('open');
          charfxItem.dispatchEvent(new MouseEvent('mousedown', { bubbles: true, cancelable: true }));

          highlightButton.dispatchEvent(new MouseEvent('mousedown', { bubbles: true, cancelable: true }));
          const highlightOpened = highlightDropdown.classList.contains('open');
          highlightSwatch.dispatchEvent(new MouseEvent('mousedown', { bubbles: true, cancelable: true }));

          colorPicker.value = '#123456';
          colorPicker.dispatchEvent(new Event('input', { bubbles: true }));
          await new Promise(resolve => setTimeout(resolve, 0));

          return {
            fontSizeApplied: fontSize.value === '12',
            charfxOpened,
            charfxClosed: !charfxDropdown.classList.contains('open'),
            highlightOpened,
            highlightClosed: !highlightDropdown.classList.contains('open'),
            highlightChanged: getComputedStyle(highlightBar).backgroundColor !== 'rgb(255, 240, 0)',
            colorChanged: getComputedStyle(colorBar).backgroundColor === 'rgb(18, 52, 86)',
          };
        });
        for (const [label, value] of Object.entries(formatting)) {
          check(tc, value, `서식 control 상호작용: ${label}`);
        }
      }

      if (vp.name === 'desktop') check(tc, result.toolbarVisible, '도구 상자 표시');

      console.log(
        `  Layout: menu=${result.menuBarHeight}px toolbar=${result.toolbarHeight}px style=${result.styleBarHeight}px rows=${result.styleRows}`,
      );

      await screenshot(page, `responsive-${vp.name}`);
      const tcResults = reporter.results.filter(entry => entry.tc === tc);
      if (tcResults.length > 0) {
        tcResults[tcResults.length - 1].screenshot = `responsive-${vp.name}.png`;
      }
    } catch (err) {
      console.error(`  ERROR: ${err.message}`);
      reporter.fail(tc, err.message);
      failed++;
    } finally {
      await closePage(page);
    }
  }

  for (const themeCase of THEME_CASES) {
    const { skin, theme, name, width, height, styleMode } = themeCase;
    const tc = `theme ${skin}/${theme}/${name} (${width}x${height})`;
    console.log(`\n[theme] ${skin}/${theme}/${name} ${width}x${height}...`);
    const page = await createPage(browser, width, height);

    try {
      await primeTheme(page, skin, theme);
      await loadApp(page);
      await page.evaluate(() => window.__eventBus?.emit('create-new-document'));
      await page.evaluate(() => new Promise(resolve => setTimeout(resolve, 400)));

      const result = await page.evaluate(async (expectedStyleMode) => {
        const root = document.documentElement;
        const styleBar = document.getElementById('style-bar');
        const field = styleBar.querySelector('.sb-field-ribbon-group');
        const command = styleBar.querySelector('.sb-command-track');
        const trigger = document.getElementById('btn-style-overflow');
        const panel = document.getElementById('style-overflow-panel');
        const firstParagraph = document.getElementById('btn-align-left');
        const button = document.getElementById('btn-bold');
        const top = element => Math.round(element.getBoundingClientRect().top);
        const rows = new Set([top(field), top(command)]).size;
        const parseRgb = value => (value.match(/[\d.]+/g) ?? []).slice(0, 3).map(Number);
        const luminance = ([r, g, b]) => {
          const linear = [r, g, b].map(channel => {
            const normalized = channel / 255;
            return normalized <= 0.03928
              ? normalized / 12.92
              : ((normalized + 0.055) / 1.055) ** 2.4;
          });
          return 0.2126 * linear[0] + 0.7152 * linear[1] + 0.0722 * linear[2];
        };
        const contrast = (foreground, background) => {
          const a = luminance(parseRgb(foreground));
          const b = luminance(parseRgb(background));
          return (Math.max(a, b) + 0.05) / (Math.min(a, b) + 0.05);
        };
        const barStyle = getComputedStyle(styleBar);
        const buttonStyle = getComputedStyle(button);
        let panelState = null;
        if (expectedStyleMode === 'overflow') {
          trigger.click();
          await new Promise(resolve => requestAnimationFrame(resolve));
          const panelStyle = getComputedStyle(panel);
          panelState = {
            visible: panel.hidden === false && panel.getClientRects().length > 0,
            focused: document.activeElement === firstParagraph,
            background: panelStyle.backgroundColor,
            borderWidth: parseFloat(panelStyle.borderTopWidth),
            textContrast: contrast(getComputedStyle(firstParagraph).color, panelStyle.backgroundColor),
          };
        }
        return {
          themeMode: root.dataset.themeMode,
          effectiveTheme: root.dataset.themeEffective,
          skin: root.dataset.themeSkin ?? 'default',
          rows,
          barHeight: styleBar.offsetHeight,
          rootOverflow: root.scrollWidth - root.clientWidth,
          styleOverflow: styleBar.scrollWidth - styleBar.clientWidth,
          barBackground: barStyle.backgroundColor,
          barBorderWidth: parseFloat(barStyle.borderBottomWidth),
          iconContrast: contrast(buttonStyle.color, barStyle.backgroundColor),
          panelState,
        };
      }, styleMode);

      check(tc, result.themeMode === theme, `theme mode=${result.themeMode}`);
      check(tc, result.effectiveTheme === theme, `effective theme=${result.effectiveTheme}`);
      check(tc, result.skin === skin, `skin=${result.skin}`);
      check(tc, result.rows === (styleMode === 'full' ? 1 : 2), `행 수=${result.rows}`);
      check(tc, styleMode !== 'full' || result.barHeight <= 36, `전체 압축 높이=${result.barHeight}px`);
      check(tc, result.rootOverflow <= 0 && result.styleOverflow <= 0, '가로 overflow 없음');
      check(tc, result.barBackground !== 'rgba(0, 0, 0, 0)', `bar 배경=${result.barBackground}`);
      check(tc, result.barBorderWidth >= 1, `bar 경계=${result.barBorderWidth}px`);
      check(tc, result.iconContrast >= 3, `icon contrast=${result.iconContrast.toFixed(2)}`);
      if (styleMode === 'overflow') {
        check(tc, result.panelState?.visible, '더보기 panel 표시');
        check(tc, result.panelState?.focused, '더보기 첫 명령 focus');
        check(tc, result.panelState?.background !== 'rgba(0, 0, 0, 0)', `panel 배경=${result.panelState?.background}`);
        check(tc, result.panelState?.borderWidth >= 1, `panel 경계=${result.panelState?.borderWidth}px`);
        check(tc, result.panelState?.textContrast >= 3, `panel contrast=${result.panelState?.textContrast.toFixed(2)}`);
      }

      await screenshot(page, `responsive-theme-${skin}-${theme}-${name}`);
    } catch (err) {
      console.error(`  ERROR: ${err.message}`);
      reporter.fail(tc, err.message);
      failed++;
    } finally {
      await closePage(page);
    }
  }

  console.log(`\n=== 결과: ${passed} passed, ${failed} failed ===`);
  if (failed > 0) process.exitCode = 1;

  reporter.generate('../output/e2e/responsive-report.html');
  await closeBrowser(browser);
}

run();
