/**
 * #4121 머리말/꼬리말 선택 Stage 2~3 E2E.
 *
 * 같은 HF 정의를 쓰는 화면 밖 페이지로 스크롤했을 때 선택 overlay가 새 visible page에
 * 투영되고, 다시 돌아왔을 때도 논리 범위가 유지되는지 검증한다.
 */
import { runTest, loadHwpFile, assert, screenshot } from './helpers.mjs';

process.env.VITE_URL = process.env.VITE_URL || 'http://localhost:7700';

const settle = (page, ms = 400) => page.evaluate(
  (delay) => new Promise(resolve => setTimeout(resolve, delay)),
  ms,
);

const headerFooterClientPoint = (page, position) => page.evaluate((pos) => {
  const handler = window.__inputHandler;
  const rect = handler.wasm.getCursorRectInHeaderFooter(
    pos.sectionIdx,
    pos.isHeader,
    pos.applyTo,
    pos.paraIdx,
    pos.charOffset,
    pos.pageNum,
  );
  const scrollContent = handler.container.querySelector('#scroll-content');
  const bounds = scrollContent.getBoundingClientRect();
  const zoom = handler.viewportManager.getZoom();
  const left = handler.virtualScroll.getPageLeftResolved(
    pos.pageNum,
    scrollContent.clientWidth,
  );
  const top = handler.virtualScroll.getPageOffset(pos.pageNum);
  return {
    x: bounds.left + left + rect.x * zoom,
    y: bounds.top + top - scrollContent.scrollTop
      + (rect.y + rect.height * 0.5) * zoom,
  };
}, position);

const visibleSelectionProjectsToPage = (page, pageNum) => page.evaluate((targetPage) => {
  const handler = window.__inputHandler;
  const top = handler.virtualScroll.getPageOffset(targetPage);
  const bottom = top + handler.virtualScroll.getPageHeight(targetPage);
  return Array.from(document.querySelectorAll('.selection-highlight'))
    .filter(el => el.style.display !== 'none')
    .some(el => {
      const value = Number.parseFloat(el.style.top);
      return value >= top && value <= bottom;
    });
}, pageNum);

runTest('#4121 HF 선택 반복 페이지 scroll-in 투영', async ({ page }) => {
  const { pageCount } = await loadHwpFile(page, 'biz_plan.hwp');
  assert(pageCount >= 2, `전제: 반복 HF 검증에 두 쪽 이상 필요 (actual=${pageCount})`);
  await page.evaluate(() => document.querySelector('.modal-overlay .dialog-btn-primary')?.click());

  const setup = await page.evaluate(() => {
    const handler = window.__inputHandler;
    const wasm = handler.wasm;
    const startPage = 0;
    const target = wasm.getHeaderFooterEditTarget(startPage, true);
    let repeatPage = -1;
    for (let pageNum = 1; pageNum < wasm.pageCount; pageNum++) {
      const candidate = wasm.getHeaderFooterEditTarget(pageNum, true);
      if (
        candidate.sectionIndex === target.sectionIndex
        && candidate.applyTo === target.applyTo
      ) {
        repeatPage = pageNum;
        break;
      }
    }
    if (repeatPage < 0) return { error: '같은 머리말 정의를 쓰는 반복 페이지 없음' };

    // 샘플의 기존 머리말에는 필드/인라인 컨트롤이 있을 수 있다. 이 E2E는 선택 투영만
    // 판정하므로 같은 target을 빈 정의로 재생성해 텍스트 offset 축을 결정적으로 만든다.
    const existing = JSON.parse(wasm.getHeaderFooter(target.sectionIndex, true, target.applyTo));
    if (existing.exists) wasm.deleteHeaderFooter(target.sectionIndex, true, target.applyTo);
    wasm.createHeaderFooter(target.sectionIndex, true, target.applyTo);
    handler.cursor.enterHeaderFooterMode(
      true,
      target.sectionIndex,
      target.applyTo,
      startPage,
    );
    handler.cursor.setHfCursorPosition(0, 0);
    handler.eventBus.emit('headerFooterModeChanged', {
      mode: 'header',
      sectionIdx: handler.cursor.hfSectionIdx,
      applyTo: handler.cursor.hfApplyTo,
      previewPage: handler.cursor.hfPreviewPage,
    });
    handler.viewportManager.setZoom(0.7);
    handler.focus();
    return { startPage, repeatPage, target };
  });
  assert(!setup.error, setup.error || 'HF setup');
  await settle(page);

  const initialLength = await page.evaluate(() => {
    const handler = window.__inputHandler;
    return JSON.parse(handler.wasm.getHeaderFooterParaInfo(
      handler.cursor.hfSectionIdx,
      true,
      handler.cursor.hfApplyTo,
      0,
    )).charCount;
  });
  if (initialLength === 0) {
    await page.keyboard.type('HEADER SELECT', { delay: 15 });
    await settle(page);
  }

  const dragPoints = await page.evaluate(() => {
    const handler = window.__inputHandler;
    const cursor = handler.cursor;
    const wasm = handler.wasm;
    const info = JSON.parse(wasm.getHeaderFooterParaInfo(
      cursor.hfSectionIdx,
      true,
      cursor.hfApplyTo,
      0,
    ));
    const pageNum = cursor.hfPreviewPage;
    const start = wasm.getCursorRectInHeaderFooter(
      cursor.hfSectionIdx, true, cursor.hfApplyTo, 0, 1, pageNum,
    );
    const end = wasm.getCursorRectInHeaderFooter(
      cursor.hfSectionIdx, true, cursor.hfApplyTo, 0, info.charCount - 1, pageNum,
    );
    const sc = handler.container.querySelector('#scroll-content');
    const bounds = sc.getBoundingClientRect();
    const zoom = handler.viewportManager.getZoom();
    const left = handler.virtualScroll.getPageLeftResolved(pageNum, sc.clientWidth);
    const top = handler.virtualScroll.getPageOffset(pageNum);
    const point = rect => ({
      x: bounds.left + left + rect.x * zoom,
      y: bounds.top + top + (rect.y + rect.height * 0.5) * zoom,
    });
    return { start: point(start), end: point(end), charCount: info.charCount };
  });
  assert(dragPoints.charCount >= 4, `전제: 드래그할 HF 텍스트가 충분함 (${dragPoints.charCount})`);
  await page.mouse.move(dragPoints.start.x, dragPoints.start.y);
  await page.mouse.down();
  await page.mouse.move(dragPoints.end.x, dragPoints.end.y, { steps: 8 });
  await page.mouse.up();
  await settle(page);
  const mouseSelection = await page.evaluate(() =>
    window.__inputHandler.cursor.getHeaderFooterSelectionOrdered());
  assert(
    mouseSelection !== null
      && (mouseSelection.start.paraIdx !== mouseSelection.end.paraIdx
        || mouseSelection.start.charOffset !== mouseSelection.end.charOffset),
    '실제 마우스 드래그가 HF 선택을 만든다',
  );

  await page.keyboard.press('Escape');
  await settle(page, 200);
  const afterEscape = await page.evaluate(() => ({
    inHeaderFooter: window.__inputHandler.cursor.isInHeaderFooter(),
    selection: window.__inputHandler.cursor.getHeaderFooterSelectionOrdered(),
  }));
  assert(afterEscape.inHeaderFooter, '선택이 있는 Esc는 HF 모드를 유지한다');
  assert(afterEscape.selection === null, '선택이 있는 Esc는 선택만 해제한다');

  await page.keyboard.down('Shift');
  await page.mouse.click(dragPoints.start.x, dragPoints.start.y);
  await page.keyboard.up('Shift');
  await settle(page, 200);
  const shiftClickSelection = await page.evaluate(() =>
    window.__inputHandler.cursor.getHeaderFooterSelectionOrdered());
  assert(shiftClickSelection !== null, '실제 Shift+클릭이 기존 HF 캐럿에서 선택을 확장한다');
  await page.keyboard.press('Escape');
  await settle(page, 100);

  await page.keyboard.press('Home');
  await page.keyboard.down('Shift');
  await page.keyboard.press('End');
  await page.keyboard.up('Shift');
  await settle(page);

  const selected = await page.evaluate(() => {
    const cursor = window.__inputHandler.cursor;
    return {
      selection: cursor.getHeaderFooterSelectionOrdered(),
      activeHighlights: Array.from(document.querySelectorAll('.selection-highlight'))
        .filter(el => el.style.display !== 'none').length,
    };
  });
  assert(selected.selection !== null, 'Shift+End가 HF 논리 선택을 만든다');
  assert(selected.activeHighlights > 0, '시작 페이지에 HF 선택 overlay가 보인다');

  const repeatProjection = await page.evaluate((repeatPage) => {
    const handler = window.__inputHandler;
    const vs = handler.virtualScroll;
    handler.viewportManager.setScrollTop(vs.getPageOffset(repeatPage));
    return { pageTop: vs.getPageOffset(repeatPage), pageHeight: vs.getPageHeight(repeatPage) };
  }, setup.repeatPage);
  await settle(page, 600);

  const afterScroll = await page.evaluate(({ pageTop, pageHeight }) => {
    const handler = window.__inputHandler;
    const tops = Array.from(document.querySelectorAll('.selection-highlight'))
      .filter(el => el.style.display !== 'none')
      .map(el => Number.parseFloat(el.style.top));
    return {
      selection: handler.cursor.getHeaderFooterSelectionOrdered(),
      projected: tops.some(top => top >= pageTop && top <= pageTop + pageHeight),
      tops,
    };
  }, repeatProjection);
  assert(afterScroll.selection !== null, 'scroll-in 뒤에도 HF 논리 선택이 유지된다');
  assert(afterScroll.projected, `새 visible 반복 페이지에 overlay가 투영된다 (${afterScroll.tops})`);

  await page.evaluate((startPage) => {
    const handler = window.__inputHandler;
    handler.viewportManager.setScrollTop(handler.virtualScroll.getPageOffset(startPage));
  }, setup.startPage);
  await settle(page, 600);
  const returned = await page.evaluate((startPage) => {
    const handler = window.__inputHandler;
    const top = handler.virtualScroll.getPageOffset(startPage);
    const bottom = top + handler.virtualScroll.getPageHeight(startPage);
    return Array.from(document.querySelectorAll('.selection-highlight'))
      .filter(el => el.style.display !== 'none')
      .some(el => {
        const value = Number.parseFloat(el.style.top);
        return value >= top && value <= bottom;
      });
  }, setup.startPage);
  assert(returned, '시작 페이지로 돌아오면 같은 선택 overlay가 다시 보인다');

  const copied = await page.evaluate(() => {
    const handler = window.__inputHandler;
    const data = new DataTransfer();
    handler.textarea.dispatchEvent(new ClipboardEvent('copy', {
      clipboardData: data, bubbles: true, cancelable: true,
    }));
    return { text: data.getData('text/plain'), html: data.getData('text/html') };
  });
  assert(copied.text === 'HEADER SELECT', `HF copy가 선택 평문을 제공한다 (${copied.text})`);
  assert(copied.html.includes('rhwp-studio-clipboard:'), 'HF copy가 fallback HTML marker를 제공한다');

  const afterCut = await page.evaluate(() => {
    const handler = window.__inputHandler;
    const data = new DataTransfer();
    handler.textarea.dispatchEvent(new ClipboardEvent('cut', {
      clipboardData: data, bubbles: true, cancelable: true,
    }));
    const info = JSON.parse(handler.wasm.getHeaderFooterParaInfo(
      handler.cursor.hfSectionIdx, true, handler.cursor.hfApplyTo, 0,
    ));
    return { copied: data.getData('text/plain'), length: info.charCount };
  });
  await settle(page, 200);
  assert(afterCut.copied === 'HEADER SELECT' && afterCut.length === 0,
    'HF cut은 복사 성공 뒤 선택을 한 번 삭제한다');

  await page.evaluate(() => window.__inputHandler.performUndo());
  await settle(page, 250);
  const cutUndo = await page.evaluate(() => {
    const handler = window.__inputHandler;
    const info = JSON.parse(handler.wasm.getHeaderFooterParaInfo(
      handler.cursor.hfSectionIdx, true, handler.cursor.hfApplyTo, 0,
    ));
    return {
      length: info.charCount,
      selection: handler.cursor.getHeaderFooterSelectionOrdered(),
      page: handler.cursor.hfPreviewPage,
    };
  });
  assert(cutUndo.length === 13 && cutUndo.selection !== null && cutUndo.page === setup.startPage,
    'cut Undo는 내용·HF 선택·previewPage를 복원한다');

  await page.evaluate(() => window.__inputHandler.performRedo());
  await settle(page, 250);
  const cutRedo = await page.evaluate(() => {
    const handler = window.__inputHandler;
    return {
      length: JSON.parse(handler.wasm.getHeaderFooterParaInfo(
        handler.cursor.hfSectionIdx, true, handler.cursor.hfApplyTo, 0,
      )).charCount,
      selection: handler.cursor.getHeaderFooterSelectionOrdered(),
    };
  });
  assert(cutRedo.length === 0 && cutRedo.selection === null,
    'cut Redo는 다시 삭제하고 선택을 접는다');
  await page.evaluate(() => window.__inputHandler.performUndo());
  await settle(page, 200);

  await page.keyboard.type('X');
  await settle(page, 200);
  const typed = await page.evaluate(() => {
    const handler = window.__inputHandler;
    return {
      length: JSON.parse(handler.wasm.getHeaderFooterParaInfo(
        handler.cursor.hfSectionIdx, true, handler.cursor.hfApplyTo, 0,
      )).charCount,
      selection: handler.cursor.getHeaderFooterSelectionOrdered(),
    };
  });
  assert(typed.length === 1 && typed.selection === null,
    'typing은 HF 선택을 한 번에 치환하고 선택을 접는다');
  await page.evaluate(() => window.__inputHandler.performUndo());
  await settle(page, 200);
  const typingUndo = await page.evaluate(() => ({
    length: JSON.parse(window.__inputHandler.wasm.getHeaderFooterParaInfo(
      window.__inputHandler.cursor.hfSectionIdx,
      true,
      window.__inputHandler.cursor.hfApplyTo,
      0,
    )).charCount,
    selection: window.__inputHandler.cursor.getHeaderFooterSelectionOrdered(),
  }));
  assert(typingUndo.length === 13 && typingUndo.selection === null,
    'typing Undo는 내용을 되돌리되 선택은 복원하지 않는다');

  await page.keyboard.press('Home');
  await page.keyboard.down('Shift');
  await page.keyboard.press('End');
  await page.keyboard.up('Shift');
  const pasted = await page.evaluate(() => {
    const handler = window.__inputHandler;
    const data = new DataTransfer();
    data.setData('text/plain', 'AA\nBB');
    handler.textarea.dispatchEvent(new ClipboardEvent('paste', {
      clipboardData: data, bubbles: true, cancelable: true,
    }));
    const first = JSON.parse(handler.wasm.getHeaderFooterParaInfo(
      handler.cursor.hfSectionIdx, true, handler.cursor.hfApplyTo, 0,
    ));
    const second = JSON.parse(handler.wasm.getHeaderFooterParaInfo(
      handler.cursor.hfSectionIdx, true, handler.cursor.hfApplyTo, 1,
    ));
    return {
      paraCount: first.paraCount,
      lengths: [first.charCount, second.charCount],
      cursor: [handler.cursor.hfParaIdx, handler.cursor.hfCharOffset],
    };
  });
  await settle(page, 200);
  assert(pasted.paraCount === 2 && pasted.lengths.join(',') === '2,2'
    && pasted.cursor.join(',') === '1,2',
  '평문 paste는 줄바꿈을 HF 문단 경계로 보존해 원자 치환한다');
  await page.evaluate(() => window.__inputHandler.performUndo());
  await settle(page, 200);

  await page.keyboard.press('Home');
  await page.keyboard.down('Shift');
  await page.keyboard.press('End');
  await page.keyboard.up('Shift');
  const beforeBold = await page.evaluate(() => {
    const handler = window.__inputHandler;
    const selection = handler.cursor.getHeaderFooterSelectionOrdered();
    return handler.wasm.getCharPropertiesInHeaderFooter(
      selection.start.sectionIdx, selection.start.isHeader, selection.start.applyTo,
      selection.start.paraIdx, selection.start.charOffset,
    ).bold;
  });
  await page.click('#btn-bold');
  await settle(page, 250);
  const afterBold = await page.evaluate(() => {
    const handler = window.__inputHandler;
    const selection = handler.cursor.getHeaderFooterSelectionOrdered();
    return {
      selected: selection !== null,
      bold: handler.wasm.getCharPropertiesInHeaderFooter(
        selection.start.sectionIdx, selection.start.isHeader, selection.start.applyTo,
        selection.start.paraIdx, selection.start.charOffset,
      ).bold,
    };
  });
  assert(afterBold.selected && afterBold.bold !== beforeBold,
    '부분 글자 서식은 HF 선택에 적용되고 선택을 유지한다');
  await page.evaluate(() => window.__inputHandler.performUndo());
  await settle(page, 250);
  const formatUndo = await page.evaluate(() => {
    const handler = window.__inputHandler;
    const selection = handler.cursor.getHeaderFooterSelectionOrdered();
    return {
      selected: selection !== null,
      bold: handler.wasm.getCharPropertiesInHeaderFooter(
        selection.start.sectionIdx, selection.start.isHeader, selection.start.applyTo,
        selection.start.paraIdx, selection.start.charOffset,
      ).bold,
    };
  });
  assert(formatUndo.selected && formatUndo.bold === beforeBold,
    '부분 서식 Undo는 내용과 같은 HF 선택을 복원한다');
  await page.evaluate(() => window.__inputHandler.performRedo());
  await settle(page, 250);
  const formatRedo = await page.evaluate(() => {
    const handler = window.__inputHandler;
    const selection = handler.cursor.getHeaderFooterSelectionOrdered();
    return {
      selected: selection !== null,
      bold: handler.wasm.getCharPropertiesInHeaderFooter(
        selection.start.sectionIdx, selection.start.isHeader, selection.start.applyTo,
        selection.start.paraIdx, selection.start.charOffset,
      ).bold,
    };
  });
  assert(formatRedo.selected && formatRedo.bold === afterBold.bold,
    '부분 서식 Redo도 같은 HF 선택을 유지한다');

  const imeReplaced = await page.evaluate(() => {
    const handler = window.__inputHandler;
    const textarea = handler.textarea;
    textarea.dispatchEvent(new CompositionEvent('compositionstart', { bubbles: true }));
    textarea.value = '한';
    textarea.dispatchEvent(new InputEvent('input', {
      bubbles: true,
      data: '한',
      inputType: 'insertCompositionText',
      isComposing: true,
    }));
    textarea.dispatchEvent(new CompositionEvent('compositionend', {
      bubbles: true,
      data: '한',
    }));
    return {
      length: JSON.parse(handler.wasm.getHeaderFooterParaInfo(
        handler.cursor.hfSectionIdx, true, handler.cursor.hfApplyTo, 0,
      )).charCount,
      selection: handler.cursor.getHeaderFooterSelectionOrdered(),
      cursor: [handler.cursor.hfParaIdx, handler.cursor.hfCharOffset],
    };
  });
  await settle(page, 250);
  assert(imeReplaced.length === 1 && imeReplaced.selection === null
    && imeReplaced.cursor.join(',') === '0,1',
  'IME 조합은 HF 선택 삭제와 최종 조합 문자열을 한 번에 치환한다');
  await page.evaluate(() => window.__inputHandler.performUndo());
  await settle(page, 250);
  const imeUndo = await page.evaluate(() => {
    const handler = window.__inputHandler;
    return {
      length: JSON.parse(handler.wasm.getHeaderFooterParaInfo(
        handler.cursor.hfSectionIdx, true, handler.cursor.hfApplyTo, 0,
      )).charCount,
      selection: handler.cursor.getHeaderFooterSelectionOrdered(),
    };
  });
  assert(imeUndo.length === 13 && imeUndo.selection === null,
    'IME 치환 Undo는 원문을 복원하되 선택은 복원하지 않는다');
  await page.evaluate(() => window.__inputHandler.performRedo());
  await settle(page, 250);
  const imeRedo = await page.evaluate(() => {
    const handler = window.__inputHandler;
    return {
      length: JSON.parse(handler.wasm.getHeaderFooterParaInfo(
        handler.cursor.hfSectionIdx, true, handler.cursor.hfApplyTo, 0,
      )).charCount,
      cursor: [handler.cursor.hfParaIdx, handler.cursor.hfCharOffset],
    };
  });
  assert(imeRedo.length === 1 && imeRedo.cursor.join(',') === '0,1',
    'IME 치환 Redo는 최종 조합 문자열과 HF 캐럿을 복원한다');

  // ── Stage 4: 4페이지 Both Header + Odd/Even Footer 통합 사용자 여정 ──
  const matrixDocument = await loadHwpFile(page, 'biz_plan.hwp');
  assert(matrixDocument.pageCount >= 4,
    `Stage 4 전제: 4페이지 이상 문서 (actual=${matrixDocument.pageCount})`);
  await page.evaluate(() => document.querySelector('.modal-overlay .dialog-btn-primary')?.click());

  const matrixSetup = await page.evaluate(() => {
    const handler = window.__inputHandler;
    const wasm = handler.wasm;
    // E2E helper의 direct loadDocument는 제품 파일 열기 수명주기를 거치지 않으므로, 첫
    // 여정의 snapshot id가 새 문서 history에 남지 않게 명시적으로 비운다.
    handler.history.clear(wasm);
    const pages = [0, 1, 2, 3];
    const sections = pages.map(pageNum =>
      wasm.getHeaderFooterEditTarget(pageNum, true).sectionIndex);
    if (new Set(sections).size !== 1) {
      return { error: `첫 4쪽이 한 구역이 아님: ${sections.join(',')}` };
    }
    const sectionIdx = sections[0];

    const resetDefinition = (isHeader, applyTo) => {
      const current = JSON.parse(wasm.getHeaderFooter(sectionIdx, isHeader, applyTo));
      if (current.exists) wasm.deleteHeaderFooter(sectionIdx, isHeader, applyTo);
    };
    for (const applyTo of [0, 1, 2]) {
      resetDefinition(true, applyTo);
      resetDefinition(false, applyTo);
    }

    wasm.createHeaderFooter(sectionIdx, true, 0);
    wasm.createHeaderFooter(sectionIdx, false, 1);
    wasm.createHeaderFooter(sectionIdx, false, 2);
    const bothHeader = wasm.replaceRangeInHeaderFooter(
      sectionIdx, true, 0, 0, 0, 0, 0, 'BOTH-H1\nBOTH-H2',
    );
    const evenFooter = wasm.replaceRangeInHeaderFooter(
      sectionIdx, false, 1, 0, 0, 0, 0, 'EVEN-F1\nEVEN-F2',
    );
    const oddFooter = wasm.replaceRangeInHeaderFooter(
      sectionIdx, false, 2, 0, 0, 0, 0, 'ODD-F1\nODD-F2',
    );
    if (!bothHeader.ok || !evenFooter.ok || !oddFooter.ok) {
      return { error: 'Stage 4 HF fixture 텍스트 구성 실패' };
    }

    const headerTargets = pages.map(pageNum => ({
      pageNum,
      ...wasm.getHeaderFooterEditTarget(pageNum, true),
    }));
    const footerTargets = pages.map(pageNum => ({
      pageNum,
      ...wasm.getHeaderFooterEditTarget(pageNum, false),
    }));
    handler.viewportManager.setZoom(0.55);
    handler.afterEdit();
    handler.cursor.enterHeaderFooterMode(true, sectionIdx, 0, pages[0]);
    handler.cursor.setHfCursorPosition(0, 0);
    handler.eventBus.emit('headerFooterModeChanged', {
      mode: 'header',
      sectionIdx: handler.cursor.hfSectionIdx,
      applyTo: handler.cursor.hfApplyTo,
      previewPage: handler.cursor.hfPreviewPage,
    });
    handler.updateCaret();
    handler.focus();
    return {
      pages,
      sectionIdx,
      headerTargets,
      footerTargets,
      mode: handler.cursor.headerFooterMode,
    };
  });
  assert(!matrixSetup.error, matrixSetup.error || 'Stage 4 HF fixture setup');
  assert(matrixSetup.mode === 'header', 'Stage 4 fixture가 Both 머리말 편집 모드로 진입한다');
  assert(matrixSetup.headerTargets.every(target => target.applyTo === 0),
    'Both 머리말이 첫 4쪽의 active target이다');
  assert(
    matrixSetup.footerTargets.filter(target => target.applyTo === 1).length === 2
      && matrixSetup.footerTargets.filter(target => target.applyTo === 2).length === 2,
    `꼬리말 active target이 홀짝 2쪽씩 분리된다 (${matrixSetup.footerTargets
      .map(target => `${target.pageNum}:${target.applyTo}`).join(',')})`,
  );
  await settle(page, 500);

  // Home → Shift+End → Shift+Down은 첫 문단 시작에서 둘째 문단 끝까지 확장한다.
  // Stage 2의 실제 mouse drag 여정과 함께 두 선택 입력 경로를 모두 고정한다.
  await page.keyboard.press('Home');
  await page.keyboard.down('Shift');
  await page.keyboard.press('End');
  await page.keyboard.press('ArrowDown');
  await page.keyboard.up('Shift');
  await settle(page, 300);
  const bothSelection = await page.evaluate(() =>
    window.__inputHandler.cursor.getHeaderFooterSelectionOrdered());
  assert(
    bothSelection?.start.paraIdx === 0
      && bothSelection.start.charOffset === 0
      && bothSelection.end.paraIdx === 1
      && bothSelection.end.charOffset === 7,
    `Shift 키가 Both 머리말 두 문단 전체를 선택한다 (${JSON.stringify(bothSelection)})`,
  );
  await screenshot(page, 'issue4121-stage4-both-header-multiline-selection');

  for (const pageNum of matrixSetup.pages) {
    await page.evaluate((targetPage) => {
      const handler = window.__inputHandler;
      handler.viewportManager.setScrollTop(handler.virtualScroll.getPageOffset(targetPage));
    }, pageNum);
    await settle(page, 500);
    assert(
      await visibleSelectionProjectsToPage(page, pageNum),
      `Both 머리말 선택이 ${pageNum + 1}쪽에 투영된다`,
    );
  }
  await page.evaluate(() => window.__inputHandler.viewportManager.setScrollTop(0));
  await settle(page, 400);

  const multiCopied = await page.evaluate(() => {
    const handler = window.__inputHandler;
    const data = new DataTransfer();
    handler.textarea.dispatchEvent(new ClipboardEvent('copy', {
      clipboardData: data, bubbles: true, cancelable: true,
    }));
    return data.getData('text/plain');
  });
  assert(multiCopied === 'BOTH-H1\nBOTH-H2',
    `다문단 HF copy가 문단 경계를 보존한다 (${JSON.stringify(multiCopied)})`);

  const multiCut = await page.evaluate(() => {
    const handler = window.__inputHandler;
    const data = new DataTransfer();
    handler.textarea.dispatchEvent(new ClipboardEvent('cut', {
      clipboardData: data, bubbles: true, cancelable: true,
    }));
    const info = JSON.parse(handler.wasm.getHeaderFooterParaInfo(
      handler.cursor.hfSectionIdx, true, handler.cursor.hfApplyTo, 0,
    ));
    return { copied: data.getData('text/plain'), paraCount: info.paraCount, length: info.charCount };
  });
  await settle(page, 250);
  assert(
    multiCut.copied === 'BOTH-H1\nBOTH-H2'
      && multiCut.paraCount === 1
      && multiCut.length === 0,
    '다문단 HF cut이 복사 성공 뒤 범위를 한 번 삭제한다',
  );
  await page.evaluate(() => window.__inputHandler.performUndo());
  await settle(page, 300);
  const multiCutUndo = await page.evaluate(() => {
    const handler = window.__inputHandler;
    const first = JSON.parse(handler.wasm.getHeaderFooterParaInfo(
      handler.cursor.hfSectionIdx, true, handler.cursor.hfApplyTo, 0,
    ));
    return {
      paraCount: first.paraCount,
      selection: handler.cursor.getHeaderFooterSelectionOrdered(),
    };
  });
  assert(
    multiCutUndo.paraCount === 2
      && multiCutUndo.selection?.start.paraIdx === 0
      && multiCutUndo.selection?.end.paraIdx === 1,
    '다문단 cut Undo가 두 문단과 원래 HF 선택을 복원한다',
  );

  const multiPaste = await page.evaluate(() => {
    const handler = window.__inputHandler;
    const data = new DataTransfer();
    data.setData('text/plain', 'PASTE-A\nPASTE-B\nPASTE-C');
    handler.textarea.dispatchEvent(new ClipboardEvent('paste', {
      clipboardData: data, bubbles: true, cancelable: true,
    }));
    const infos = [0, 1, 2].map(paraIdx => JSON.parse(
      handler.wasm.getHeaderFooterParaInfo(
        handler.cursor.hfSectionIdx, true, handler.cursor.hfApplyTo, paraIdx,
      ),
    ));
    return {
      paraCount: infos[0].paraCount,
      lengths: infos.map(info => info.charCount),
      cursor: [handler.cursor.hfParaIdx, handler.cursor.hfCharOffset],
    };
  });
  await settle(page, 250);
  assert(
    multiPaste.paraCount === 3
      && multiPaste.lengths.join(',') === '7,7,7'
      && multiPaste.cursor.join(',') === '2,7',
    `다문단 paste가 세 HF 문단으로 원자 치환된다 (${JSON.stringify(multiPaste)})`,
  );
  await page.evaluate(() => window.__inputHandler.performUndo());
  await settle(page, 300);
  const multiPasteUndo = await page.evaluate(() => ({
    paraCount: JSON.parse(window.__inputHandler.wasm.getHeaderFooterParaInfo(
      window.__inputHandler.cursor.hfSectionIdx,
      true,
      window.__inputHandler.cursor.hfApplyTo,
      0,
    )).paraCount,
    selection: window.__inputHandler.cursor.getHeaderFooterSelectionOrdered(),
  }));
  assert(multiPasteUndo.paraCount === 2 && multiPasteUndo.selection === null,
    '다문단 paste Undo는 원문을 복원하되 선택은 복원하지 않는다');

  // 서식 확인을 위해 두 문단을 실제 Shift 키로 다시 선택한다.
  await page.keyboard.press('ArrowUp');
  await page.keyboard.press('Home');
  await page.keyboard.down('Shift');
  await page.keyboard.press('End');
  await page.keyboard.press('ArrowDown');
  await page.keyboard.up('Shift');
  await settle(page, 200);
  const beforeMultiBold = await page.evaluate(() => {
    const handler = window.__inputHandler;
    return [0, 1].map(paraIdx => handler.wasm.getCharPropertiesInHeaderFooter(
      handler.cursor.hfSectionIdx, true, handler.cursor.hfApplyTo, paraIdx, 0,
    ).bold);
  });
  await page.click('#btn-bold');
  await settle(page, 300);
  const afterMultiBold = await page.evaluate(() => {
    const handler = window.__inputHandler;
    return {
      values: [0, 1].map(paraIdx => handler.wasm.getCharPropertiesInHeaderFooter(
        handler.cursor.hfSectionIdx, true, handler.cursor.hfApplyTo, paraIdx, 0,
      ).bold),
      selection: handler.cursor.getHeaderFooterSelectionOrdered(),
    };
  });
  assert(
    afterMultiBold.selection?.start.paraIdx === 0
      && afterMultiBold.selection?.end.paraIdx === 1
      && afterMultiBold.values.every((value, index) => value !== beforeMultiBold[index]),
    `다문단 부분 서식이 두 HF 문단에만 적용되고 선택을 유지한다 (${JSON.stringify({
      before: beforeMultiBold,
      after: afterMultiBold,
    })})`,
  );
  await page.evaluate(() => window.__inputHandler.performUndo());
  await settle(page, 250);
  const multiFormatUndo = await page.evaluate(() => ({
    values: [0, 1].map(paraIdx => window.__inputHandler.wasm
      .getCharPropertiesInHeaderFooter(
        window.__inputHandler.cursor.hfSectionIdx,
        true,
        window.__inputHandler.cursor.hfApplyTo,
        paraIdx,
        0,
      ).bold),
    selection: window.__inputHandler.cursor.getHeaderFooterSelectionOrdered(),
  }));
  assert(
    multiFormatUndo.selection?.end.paraIdx === 1
      && multiFormatUndo.values.every((value, index) => value === beforeMultiBold[index]),
    '다문단 부분 서식 Undo가 이전 서식과 같은 선택을 복원한다',
  );
  await page.evaluate(() => window.__inputHandler.performRedo());
  await settle(page, 250);
  const multiFormatRedo = await page.evaluate(() => ({
    values: [0, 1].map(paraIdx => window.__inputHandler.wasm
      .getCharPropertiesInHeaderFooter(
        window.__inputHandler.cursor.hfSectionIdx,
        true,
        window.__inputHandler.cursor.hfApplyTo,
        paraIdx,
        0,
      ).bold),
    selection: window.__inputHandler.cursor.getHeaderFooterSelectionOrdered(),
  }));
  assert(
    multiFormatRedo.selection?.end.paraIdx === 1
      && multiFormatRedo.values.join(',') === afterMultiBold.values.join(','),
    '다문단 부분 서식 Redo가 변경 서식과 같은 선택을 복원한다',
  );

  const evenTargets = matrixSetup.footerTargets.filter(target => target.applyTo === 1);
  const oddTargets = matrixSetup.footerTargets.filter(target => target.applyTo === 2);
  const activeParity = evenTargets;
  const otherParity = oddTargets;
  await page.evaluate(({ sectionIdx, startPage }) => {
    const handler = window.__inputHandler;
    handler.cursor.switchHeaderFooterTarget(false, sectionIdx, 1, startPage);
    handler.cursor.setHfCursorPosition(0, 0);
    handler.eventBus.emit('headerFooterModeChanged', {
      mode: 'footer',
      sectionIdx: handler.cursor.hfSectionIdx,
      applyTo: handler.cursor.hfApplyTo,
      previewPage: handler.cursor.hfPreviewPage,
    });
    handler.viewportManager.setScrollTop(
      handler.virtualScroll.getPageOffset(handler.cursor.hfPreviewPage),
    );
    handler.focus();
  }, { sectionIdx: matrixSetup.sectionIdx, startPage: activeParity[0].pageNum });
  await settle(page, 500);
  const representativeEditing = await page.evaluate(() => {
    const handler = window.__inputHandler;
    const layer = document.querySelector(
      `[data-rhwp-hf-edit-page="${handler.cursor.hfPreviewPage}"].is-representative`,
    );
    return {
      previewPage: handler.cursor.hfPreviewPage,
      expectedPreviewPage: handler.wasm.getHeaderFooterPreviewPage(handler.cursor.hfSectionIdx),
      label: document.querySelector('.tb-hf-label')?.textContent || '',
      badge: layer?.querySelector('.hf-edit-badge')?.textContent || '',
      hasPreviewCanvas: Boolean(layer?.querySelector('.hf-edit-preview-canvas')),
      hasStrongRegion: Boolean(layer?.querySelector('.hf-edit-region.is-representative')),
    };
  });
  assert(
    representativeEditing.previewPage === representativeEditing.expectedPreviewPage
      && representativeEditing.previewPage === matrixSetup.pages[0],
    `짝수 꼬리말도 구역 첫 페이지에서 대표 편집한다 (${JSON.stringify(representativeEditing)})`,
  );
  assert(
    representativeEditing.label.includes('꼬리말 · 짝수 쪽 편집 중')
      && representativeEditing.badge === '꼬리말(짝수 쪽)'
      && representativeEditing.hasPreviewCanvas
      && representativeEditing.hasStrongRegion,
    `대표 편집 타겟과 영역을 텍스트·강조로 표시한다 (${JSON.stringify(representativeEditing)})`,
  );
  await page.keyboard.press('Home');
  await page.keyboard.down('Shift');
  await page.keyboard.press('End');
  await page.keyboard.up('Shift');
  await settle(page, 250);
  const paritySelection = await page.evaluate(() =>
    window.__inputHandler.cursor.getHeaderFooterSelectionOrdered());
  assert(paritySelection?.start.applyTo === 1 && paritySelection.end.charOffset === 7,
    'Shift+End가 짝수 쪽 꼬리말 선택을 만든다');
  assert(
    await visibleSelectionProjectsToPage(page, representativeEditing.previewPage),
    '짝수 쪽 꼬리말 선택이 대표 편집 페이지에 투영된다',
  );

  for (const target of activeParity) {
    await page.evaluate((pageNum) => {
      const handler = window.__inputHandler;
      handler.viewportManager.setScrollTop(handler.virtualScroll.getPageOffset(pageNum));
    }, target.pageNum);
    await settle(page, 500);
    assert(await visibleSelectionProjectsToPage(page, target.pageNum),
      `짝수 쪽 꼬리말 선택이 같은 정의의 ${target.pageNum + 1}쪽에 투영된다`);
    const related = await page.evaluate((pageNum) => Boolean(document.querySelector(
      `[data-rhwp-hf-edit-page="${pageNum}"].is-related .hf-edit-region.is-related`,
    )), target.pageNum);
    assert(related, `짝수 쪽 ${target.pageNum + 1}쪽은 실제 적용 영역으로 연관 표시된다`);
  }
  for (const target of otherParity) {
    if (target.pageNum === representativeEditing.previewPage) continue;
    await page.evaluate((pageNum) => {
      const handler = window.__inputHandler;
      handler.viewportManager.setScrollTop(handler.virtualScroll.getPageOffset(pageNum));
    }, target.pageNum);
    await settle(page, 500);
    assert(!(await visibleSelectionProjectsToPage(page, target.pageNum)),
      `짝수 쪽 꼬리말 선택이 다른 홀수 정의의 ${target.pageNum + 1}쪽에는 투영되지 않는다`);
  }

  const switchTarget = otherParity.find(
    target => target.pageNum !== representativeEditing.previewPage,
  );
  assert(Boolean(switchTarget), '대표 페이지 밖에 홀수 꼬리말 적용 쪽이 있다');
  await page.evaluate((pageNum) => {
    const handler = window.__inputHandler;
    handler.viewportManager.setScrollTop(handler.virtualScroll.getPageOffset(pageNum));
  }, switchTarget.pageNum);
  await settle(page, 500);
  const oddFooterPoint = await headerFooterClientPoint(page, {
    sectionIdx: matrixSetup.sectionIdx,
    isHeader: false,
    applyTo: 2,
    paraIdx: 0,
    charOffset: 3,
    pageNum: switchTarget.pageNum,
  });
  await page.mouse.click(oddFooterPoint.x, oddFooterPoint.y);
  await settle(page, 350);
  const afterParitySwitch = await page.evaluate(() => {
    const handler = window.__inputHandler;
    return {
      mode: handler.cursor.headerFooterMode,
      applyTo: handler.cursor.hfApplyTo,
      page: handler.cursor.hfPreviewPage,
      previewPage: handler.wasm.getHeaderFooterPreviewPage(handler.cursor.hfSectionIdx),
      label: document.querySelector('.tb-hf-label')?.textContent || '',
      selection: handler.cursor.getHeaderFooterSelectionOrdered(),
    };
  });
  assert(
    afterParitySwitch.mode === 'footer'
      && afterParitySwitch.applyTo === 2
      && afterParitySwitch.page === afterParitySwitch.previewPage
      && afterParitySwitch.label.includes('홀수 쪽 편집 중')
      && afterParitySwitch.selection === null,
    `다른 홀짝 정의 클릭이 교차 선택 없이 target을 전환한다 (${JSON.stringify(afterParitySwitch)})`,
  );
  await screenshot(page, 'issue4121-stage4-odd-even-footer-switch');

  const actualPreviewTarget = await page.evaluate(() => {
    const handler = window.__inputHandler;
    return handler.wasm.getHeaderFooterEditTarget(handler.cursor.hfPreviewPage, false);
  });
  await page.click('[data-cmd="page:headerfooter-close"]');
  await settle(page, 250);
  const exited = await page.evaluate(() => {
    const handler = window.__inputHandler;
    return {
      inHeaderFooter: handler.cursor.isInHeaderFooter(),
      overlayCount: document.querySelectorAll('[data-rhwp-hf-edit-page]').length,
      toolbarHidden: document.querySelector('.tb-headerfooter-group')?.hidden,
      actualTarget: handler.wasm.getHeaderFooterEditTarget(0, false),
    };
  });
  assert(
    !exited.inHeaderFooter
      && exited.overlayCount === 0
      && exited.toolbarHidden === true
      && JSON.stringify(exited.actualTarget) === JSON.stringify(actualPreviewTarget),
    `편집 종료는 preview를 거두고 실제 페이지 target을 바꾸지 않는다 (${JSON.stringify(exited)})`,
  );
});
