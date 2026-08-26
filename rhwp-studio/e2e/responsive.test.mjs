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
        const toolbarViewport = document.getElementById('icon-toolbar-viewport');
        const toolbarTrack = toolbar?.querySelector('.tb-scroll-track');
        const toolbarPrevious = document.getElementById('icon-toolbar-prev');
        const toolbarNext = document.getElementById('icon-toolbar-next');
        const toolbarGroups = Array.from(toolbarTrack?.querySelectorAll(':scope > .tb-group') ?? [])
          .filter(group => getComputedStyle(group).display !== 'none');
        const toolbarLabels = Array.from(toolbarTrack?.querySelectorAll('.tb-label') ?? []);
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
        const toolbarRows = new Set(toolbarGroups.map(top)).size;

        return {
          hasCanvas: !!canvas,
          menuBarVisible: isVisible(menuBar),
          menuBarHeight: menuBar?.offsetHeight ?? 0,
          toolbarVisible: isVisible(toolbar),
          toolbarHeight: toolbar?.offsetHeight ?? 0,
          toolbarRows,
          toolbarLabelsVisible: toolbarLabels.some(isVisible),
          toolbarPreviousVisible: isVisible(toolbarPrevious),
          toolbarNextVisible: isVisible(toolbarNext),
          toolbarPreviousDisabled: toolbarPrevious?.disabled ?? null,
          toolbarNextDisabled: toolbarNext?.disabled ?? null,
          toolbarPreviousAriaHidden: toolbarPrevious?.getAttribute('aria-hidden'),
          toolbarNextAriaHidden: toolbarNext?.getAttribute('aria-hidden'),
          toolbarClientWidth: toolbar?.clientWidth ?? 0,
          toolbarScrollWidth: toolbar?.scrollWidth ?? 0,
          toolbarViewportClientWidth: toolbarViewport?.clientWidth ?? 0,
          toolbarViewportScrollWidth: toolbarViewport?.scrollWidth ?? 0,
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
      check(tc, result.toolbarVisible, '기본 도구 상자 표시');
      check(tc, result.toolbarHeight === 56, `기본 도구 상자 한 줄 높이 (h=${result.toolbarHeight})`);
      check(tc, result.toolbarRows === 1, `기본 도구 그룹 한 줄 (rows=${result.toolbarRows})`);
      check(tc, result.toolbarLabelsVisible, '기본 도구 label 밀도 유지');
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
      check(
        tc,
        result.toolbarScrollWidth <= result.toolbarClientWidth,
        `기본 도구 외부 overflow 없음 (${result.toolbarScrollWidth}/${result.toolbarClientWidth})`,
      );

      const toolbarOverflowExpected = vp.width <= 1024;
      if (toolbarOverflowExpected) {
        check(tc, !result.toolbarPreviousVisible && result.toolbarNextVisible, '시작 위치는 다음 이동 버튼만 표시');
        check(
          tc,
          result.toolbarPreviousDisabled === true && result.toolbarPreviousAriaHidden === 'true',
          '시작 위치 이전 버튼은 slot만 남기고 접근성 트리에서 숨김',
        );
        check(
          tc,
          result.toolbarNextDisabled === false && result.toolbarNextAriaHidden === 'false',
          '시작 위치 다음 버튼 enabled',
        );
        check(
          tc,
          result.toolbarViewportScrollWidth > result.toolbarViewportClientWidth,
          `기본 도구 내부 scroll (${result.toolbarViewportScrollWidth}/${result.toolbarViewportClientWidth})`,
        );
      } else {
        check(tc, !result.toolbarPreviousVisible && !result.toolbarNextVisible, '내용이 맞으면 이동 버튼 숨김');
        check(
          tc,
          result.toolbarViewportScrollWidth <= result.toolbarViewportClientWidth,
          '내용이 맞으면 내부 overflow 없음',
        );
      }

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
          const currentAlignmentMirrored = selectionCommand?.classList.contains('active') === true
            && trigger?.classList.contains('active') === false
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
            currentAlignmentMirrored,
            disabledMirrored,
          };
        });
        check(tc, interaction.openedByKeyboard, 'ArrowDown 열기와 첫 명령 focus');
        check(tc, interaction.closedByEscape, 'Escape 닫기와 trigger focus 복귀');
        check(tc, interaction.closedByOutside, '외부 pointer로 panel 닫힘');
        check(tc, interaction.openedByClick, 'click 열기와 첫 명령 focus');
        check(tc, interaction.closedByCommand, '명령 실행 뒤 panel 닫힘');
        check(tc, interaction.focusReturned, '명령 실행 뒤 trigger focus 복귀');
        check(tc, interaction.currentAlignmentMirrored, '현재 paragraph 정렬은 표시하되 닫힌 trigger는 중립 유지');
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

      if (vp.name === 'mobile') {
        const toolbarInteraction = await page.evaluate(async () => {
          const root = document.getElementById('icon-toolbar');
          const viewport = document.getElementById('icon-toolbar-viewport');
          const track = root.querySelector('.tb-scroll-track');
          const previous = document.getElementById('icon-toolbar-prev');
          const next = document.getElementById('icon-toolbar-next');
          const visibleGroups = () => Array.from(track.querySelectorAll(':scope > .tb-group'))
            .filter(group => getComputedStyle(group).display !== 'none' && group.offsetWidth > 0);
          const settle = () => new Promise(resolve => {
            let previousLeft = viewport.scrollLeft;
            let stableFrames = 0;
            let frames = 0;
            const tick = () => {
              const current = viewport.scrollLeft;
              stableFrames = Math.abs(current - previousLeft) < 0.5 ? stableFrames + 1 : 0;
              previousLeft = current;
              frames++;
              if (stableFrames >= 4 || frames >= 90) resolve();
              else requestAnimationFrame(tick);
            };
            requestAnimationFrame(tick);
          });

          next.click();
          await settle();
          const firstTarget = viewport.scrollLeft;
          const viewportLeft = viewport.getBoundingClientRect().left;
          const alignedToGroup = firstTarget > 1 && visibleGroups().some(group => (
            Math.abs(group.getBoundingClientRect().left - viewportLeft) <= 1
          ));
          const middleNavigationVisible = getComputedStyle(previous).visibility !== 'hidden'
            && getComputedStyle(next).visibility !== 'hidden';

          viewport.focus();
          viewport.dispatchEvent(new KeyboardEvent('keydown', {
            key: 'End', bubbles: true, cancelable: true,
          }));
          await settle();
          const maximum = viewport.scrollWidth - viewport.clientWidth;
          const endedByKeyboard = Math.abs(viewport.scrollLeft - maximum) <= 1
            && next.disabled === true && previous.disabled === false
            && next.getAttribute('aria-hidden') === 'true'
            && getComputedStyle(next).visibility === 'hidden'
            && getComputedStyle(previous).visibility !== 'hidden';

          const splitArrow = track.querySelector('.tb-split-arrow');
          splitArrow?.focus();
          await settle();
          splitArrow?.dispatchEvent(new MouseEvent('mousedown', { bubbles: true, cancelable: true }));
          await new Promise(resolve => requestAnimationFrame(resolve));
          const splitMenu = track.querySelector('.tb-split-menu');
          const scrollViewportRect = viewport.getBoundingClientRect();
          const splitMenuRect = splitMenu?.getBoundingClientRect();
          const splitMenuVisible = !!splitMenuRect
            && getComputedStyle(splitMenu).display === 'block'
            && splitMenuRect.top >= scrollViewportRect.bottom
            && splitMenuRect.left >= 0
            && splitMenuRect.right <= window.innerWidth
            && splitMenuRect.bottom <= window.innerHeight;
          document.body.dispatchEvent(new MouseEvent('mousedown', { bubbles: true }));

          viewport.dispatchEvent(new KeyboardEvent('keydown', {
            key: 'Home', bubbles: true, cancelable: true,
          }));
          await settle();
          const startedByKeyboard = viewport.scrollLeft <= 1
            && previous.disabled === true && next.disabled === false
            && previous.getAttribute('aria-hidden') === 'true'
            && getComputedStyle(previous).visibility === 'hidden'
            && getComputedStyle(next).visibility !== 'hidden';

          return { alignedToGroup, middleNavigationVisible, endedByKeyboard, splitMenuVisible, startedByKeyboard };
        });
        check(tc, toolbarInteraction.alignedToGroup, '다음 버튼은 track 기준 group 경계로 이동');
        check(tc, toolbarInteraction.middleNavigationVisible, '중간 위치는 양쪽 이동 버튼 표시');
        check(tc, toolbarInteraction.endedByKeyboard, 'End로 마지막 group 도달·다음 버튼 숨김');
        check(tc, toolbarInteraction.splitMenuVisible, '가로 viewport 밖에도 split menu가 잘리지 않음');
        check(tc, toolbarInteraction.startedByKeyboard, 'Home으로 첫 group 복귀·이전 버튼 숨김');

        const toolbarViewport = await page.$('#icon-toolbar-viewport');
        const viewportBox = await toolbarViewport?.boundingBox();
        if (viewportBox) {
          await page.mouse.move(viewportBox.x + viewportBox.width / 2, viewportBox.y + viewportBox.height / 2);
          await page.mouse.wheel({ deltaX: 180 });
          await new Promise(resolve => setTimeout(resolve, 150));
        }
        const nativeWheelMoved = await page.evaluate(
          () => document.getElementById('icon-toolbar-viewport')?.scrollLeft > 1,
        );
        check(tc, nativeWheelMoved, 'trackpad 수평 wheel로 내부 이동');

        const toolbarModeAndVisibility = await page.evaluate(async () => {
          const root = document.getElementById('icon-toolbar');
          const viewport = document.getElementById('icon-toolbar-viewport');
          const track = root.querySelector('.tb-scroll-track');
          const previous = document.getElementById('icon-toolbar-prev');
          const next = document.getElementById('icon-toolbar-next');
          const waitFrames = async (count = 4) => {
            for (let index = 0; index < count; index++) {
              await new Promise(resolve => requestAnimationFrame(resolve));
            }
          };

          const lastGroup = Array.from(track.querySelectorAll('.tb-group'))
            .filter(group => getComputedStyle(group).display !== 'none')
            .at(-1);
          const lastCommand = Array.from(lastGroup?.querySelectorAll('.tb-btn') ?? []).at(-1);
          lastCommand?.focus();
          await waitFrames();
          const viewportRect = viewport.getBoundingClientRect();
          const commandRect = lastCommand?.getBoundingClientRect();
          const focusedCommandVisible = !!commandRect
            && commandRect.left >= viewportRect.left - 1
            && commandRect.right <= viewportRect.right + 1;

          viewport.scrollLeft = viewport.scrollWidth;
          window.__eventBus?.emit('headerFooterModeChanged', 'header');
          await waitFrames();
          const headerFooter = track.querySelector('.tb-headerfooter-group');
          const defaultGroup = track.querySelector('.tb-group:not(.tb-headerfooter-group):not(.tb-note-group):not(.tb-rotate-group)');
          const modeReset = getComputedStyle(headerFooter).display !== 'none'
            && getComputedStyle(defaultGroup).display === 'none'
            && viewport.scrollLeft <= 1;

          window.__eventBus?.emit('headerFooterModeChanged', 'none');
          await waitFrames();
          const defaultRestored = getComputedStyle(defaultGroup).display !== 'none'
            && viewport.scrollLeft <= 1
            && !next.hidden && previous.disabled && !next.disabled
            && getComputedStyle(previous).visibility === 'hidden'
            && getComputedStyle(next).visibility !== 'hidden';

          document.documentElement.dataset.toolboxBasic = 'hidden';
          const hiddenWithShell = getComputedStyle(root).display === 'none';
          document.documentElement.dataset.toolboxBasic = 'shown';
          await waitFrames();
          const restoredWithNavigation = getComputedStyle(root).display === 'flex'
            && root.offsetHeight === 56 && !next.hidden;

          return { focusedCommandVisible, modeReset, defaultRestored, hiddenWithShell, restoredWithNavigation };
        });
        check(tc, toolbarModeAndVisibility.focusedCommandVisible, 'offscreen command focus를 viewport 안에 표시');
        check(tc, toolbarModeAndVisibility.modeReset, '머리말 mode 전환 뒤 시작 위치 재계산');
        check(tc, toolbarModeAndVisibility.defaultRestored, '기본 mode 복귀 뒤 이동 상태 재계산');
        check(tc, toolbarModeAndVisibility.hiddenWithShell, '기본 도구 상자 숨김은 이동 버튼까지 포함');
        check(tc, toolbarModeAndVisibility.restoredWithNavigation, '기본 도구 상자 복귀 뒤 overflow 재계산');
      }

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
        const toolbar = document.getElementById('icon-toolbar');
        const toolbarTrack = toolbar.querySelector('.tb-scroll-track');
        const toolbarViewport = document.getElementById('icon-toolbar-viewport');
        const toolbarNext = document.getElementById('icon-toolbar-next');
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
        const resolveCssColor = (value) => {
          const probe = document.createElement('span');
          probe.style.color = value;
          document.body.appendChild(probe);
          const resolved = getComputedStyle(probe).color;
          probe.remove();
          return resolved;
        };
        const barStyle = getComputedStyle(styleBar);
        const buttonStyle = getComputedStyle(button);
        const toolbarStyle = getComputedStyle(toolbar);
        const toolbarNextStyle = getComputedStyle(toolbarNext);
        const toolbarBackground = toolbarStyle.backgroundColor !== 'rgba(0, 0, 0, 0)'
          ? toolbarStyle.backgroundColor
          : resolveCssColor(getComputedStyle(root).getPropertyValue('--ui-toolbar-bg-start'));
        const toolbarGroups = Array.from(toolbarTrack.querySelectorAll(':scope > .tb-group'))
          .filter(group => getComputedStyle(group).display !== 'none');
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
          toolbarHeight: toolbar.offsetHeight,
          toolbarRows: new Set(toolbarGroups.map(top)).size,
          toolbarOverflow: toolbar.scrollWidth - toolbar.clientWidth,
          toolbarViewportOverflow: toolbarViewport.scrollWidth - toolbarViewport.clientWidth,
          toolbarNavigationVisible: toolbarNext.hidden === false,
          toolbarBackground,
          toolbarPainted: toolbarStyle.backgroundColor !== 'rgba(0, 0, 0, 0)'
            || toolbarStyle.backgroundImage !== 'none',
          toolbarBorderWidth: parseFloat(toolbarStyle.borderBottomWidth),
          toolbarNavigationContrast: contrast(toolbarNextStyle.color, toolbarBackground),
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
      check(tc, result.toolbarHeight === 56 && result.toolbarRows === 1, `기본 도구 한 줄=${result.toolbarHeight}px`);
      check(tc, result.toolbarOverflow <= 0 && result.toolbarViewportOverflow > 0, '기본 도구 내부 overflow 격리');
      check(tc, result.toolbarNavigationVisible, '기본 도구 이동 버튼 표시');
      check(tc, result.toolbarPainted, `toolbar 배경=${result.toolbarBackground}`);
      check(tc, result.toolbarBorderWidth >= 1, `toolbar 경계=${result.toolbarBorderWidth}px`);
      check(tc, result.toolbarNavigationContrast >= 3, `toolbar nav contrast=${result.toolbarNavigationContrast.toFixed(2)}`);
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
