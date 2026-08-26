/**
 * E2E 테스트: 반응형 레이아웃과 #6118 서식 바 콘텐츠 경계 검증
 */
import { launchBrowser, loadApp, screenshot, closeBrowser, closePage, createPage } from './helpers.mjs';
import { TestReporter } from './report-generator.mjs';

const VIEWPORTS = [
  { name: 'desktop', width: 1280, height: 900, styleMode: 'full' },
  { name: 'full-boundary', width: 976, height: 900, styleMode: 'full' },
  { name: 'two-row-boundary', width: 975, height: 900, styleMode: 'inline' },
  { name: 'tablet', width: 768, height: 1024, styleMode: 'inline' },
  { name: 'command-inline-boundary', width: 460, height: 900, styleMode: 'inline' },
  { name: 'command-overflow-boundary', width: 459, height: 900, styleMode: 'overflow' },
  { name: 'mobile', width: 375, height: 812, styleMode: 'overflow' },
];

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
          trigger?.click();
          await new Promise(resolve => requestAnimationFrame(resolve));
          const opened = trigger?.getAttribute('aria-expanded') === 'true'
            && panel?.hidden === false
            && document.activeElement === command;
          command?.dispatchEvent(new MouseEvent('mousedown', { bubbles: true, cancelable: true }));
          command?.click();
          return {
            opened,
            closed: trigger?.getAttribute('aria-expanded') === 'false' && panel?.hidden === true,
            focusReturned: document.activeElement === trigger,
          };
        });
        check(tc, interaction.opened, '더보기 열기와 첫 명령 focus');
        check(tc, interaction.closed, '명령 실행 뒤 panel 닫힘');
        check(tc, interaction.focusReturned, '명령 실행 뒤 trigger focus 복귀');
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

  console.log(`\n=== 결과: ${passed} passed, ${failed} failed ===`);
  if (failed > 0) process.exitCode = 1;

  reporter.generate('../output/e2e/responsive-report.html');
  await closeBrowser(browser);
}

run();
