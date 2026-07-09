/**
 * E2E: 한셀 OLE 미리보기는 표처럼 보이더라도 셀 내부 편집으로 진입하지 않는다.
 */
import {
  runTest,
  createNewDocument,
  loadHwpFile,
  screenshot,
  assert,
} from './helpers.mjs';

const MINIMAL_PNG = [
  0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48,
  0x44, 0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00,
  0x00, 0x1F, 0x15, 0xC4, 0x89, 0x00, 0x00, 0x00, 0x0A, 0x49, 0x44, 0x41, 0x54, 0x78,
  0x9C, 0x63, 0x00, 0x01, 0x00, 0x00, 0x05, 0x00, 0x01, 0x0D, 0x0A, 0x00, 0x00, 0x00,
  0x00, 0x49, 0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
];

async function showParagraphMarks(page) {
  await page.evaluate(() => {
    window.__wasm.setShowParagraphMarks(true);
    window.__canvasView.loadDocument();
  });
}

async function clickOleRightCaret(page) {
  await page.evaluate(() => {
    window.__inputHandler.exitPictureObjectSelectionAndAfterEdit?.();
    window.__inputHandler.moveCursorTo?.({ sectionIndex: 0, paragraphIndex: 0, charOffset: 0 });
  });
  await page.evaluate(() => new Promise(resolve => setTimeout(resolve, 100)));

  const clickPoint = await page.evaluate(() => {
    const layout = window.__wasm.getPageControlLayout(0);
    const ole = layout.controls.find(c => c.type === 'ole');
    if (!ole) throw new Error('OLE layout not found');

    const scrollContent = document.querySelector('#scroll-content');
    const rect = scrollContent.getBoundingClientRect();
    const zoom = window.__inputHandler.viewportManager.getZoom();
    const pageIdx = 0;
    const pageOffset = window.__inputHandler.virtualScroll.getPageOffset(pageIdx);
    const pageLeft = window.__inputHandler.virtualScroll.getPageLeftResolved(
      pageIdx,
      scrollContent.clientWidth,
    );

    return {
      x: rect.left + pageLeft + (ole.x + ole.w + 3) * zoom,
      y: rect.top + pageOffset + (ole.y + 5) * zoom,
    };
  });
  await page.mouse.click(clickPoint.x, clickPoint.y);
  await page.evaluate(() => new Promise(resolve => setTimeout(resolve, 150)));
}

async function clickOlePreview(page) {
  const clickPoint = await page.evaluate(() => {
    const layout = window.__wasm.getPageControlLayout(0);
    const ole = layout.controls.find(c => c.type === 'ole');
    if (!ole) throw new Error('OLE layout not found');

    const scrollContent = document.querySelector('#scroll-content');
    const rect = scrollContent.getBoundingClientRect();
    const zoom = window.__inputHandler.viewportManager.getZoom();
    const pageIdx = 0;
    const pageOffset = window.__inputHandler.virtualScroll.getPageOffset(pageIdx);
    const pageLeft = window.__inputHandler.virtualScroll.getPageLeftResolved(
      pageIdx,
      scrollContent.clientWidth,
    );

    return {
      x: rect.left + pageLeft + (ole.x + 10) * zoom,
      y: rect.top + pageOffset + (ole.y + 10) * zoom,
      ole,
    };
  });
  await page.mouse.click(clickPoint.x, clickPoint.y);
  await page.evaluate(() => new Promise(resolve => setTimeout(resolve, 300)));
  return clickPoint;
}

async function readOleKeyboardState(page) {
  return await page.evaluate(() => {
    const cursor = window.__inputHandler.cursor;
    const ole = window.__wasm.getPageControlLayout(0).controls.find(c => c.type === 'ole');
    const pos = cursor.getPosition?.() ?? null;
    const activeRect = pos
      ? window.__wasm.getCursorRect(pos.sectionIndex, pos.paragraphIndex, pos.charOffset)
      : null;
    return {
      ole,
      pos,
      paraCount: window.__wasm.getParagraphCount(0),
      activeRect,
      isCell: cursor.isInCell?.() ?? false,
      isPictureSelection: cursor.isInPictureObjectSelection?.() ?? false,
      selected: cursor.getSelectedPictureRef?.() ?? null,
    };
  });
}

async function exerciseEnterBackspaceReenter(page) {
  await clickOleRightCaret(page);
  await page.keyboard.press('Enter');
  await page.evaluate(() => new Promise(resolve => setTimeout(resolve, 150)));
  await page.keyboard.press('Backspace');
  await page.evaluate(() => new Promise(resolve => setTimeout(resolve, 150)));
  await page.keyboard.press('Enter');
  await page.evaluate(() => new Promise(resolve => setTimeout(resolve, 300)));
  return await readOleKeyboardState(page);
}

async function setOleCaptionAndReadBack(page) {
  await clickOlePreview(page);
  const selection = await page.evaluate(() => {
    const cursor = window.__inputHandler.cursor;
    const selected = cursor.getSelectedPictureRef?.() ?? null;
    const isPictureSelection = cursor.isInPictureObjectSelection?.() ?? false;
    if (!isPictureSelection || selected?.type !== 'ole') {
      return { selected, isPictureSelection, opened: false };
    }
    window.__inputHandler.dispatcher.dispatch('format:object-properties');
    return { selected, isPictureSelection, opened: true };
  });

  await page.waitForSelector('.dialog-wrap', { timeout: 5000 });
  const dialogState = await page.evaluate(() => {
    const marginTab = [...document.querySelectorAll('.dialog-tab')]
      .find(el => el.textContent?.trim() === '여백/캡션');
    if (!marginTab) throw new Error('여백/캡션 tab not found');
    marginTab.click();

    const captionBtns = [...document.querySelectorAll('.pp-caption-btn')];
    if (captionBtns.length !== 9) {
      throw new Error(`caption button count mismatch: ${captionBtns.length}`);
    }
    captionBtns[8].click();

    const expandLabel = [...document.querySelectorAll('label')]
      .find(el => el.textContent?.includes('여백 부분까지 너비 확대'));
    if (expandLabel) expandLabel.click();

    const ok = [...document.querySelectorAll('button')]
      .find(el => el.textContent?.trim() === '설정(D)');
    if (!ok) throw new Error('설정(D) button not found');
    const state = {
      tabTexts: [...document.querySelectorAll('.dialog-tab')].map(el => el.textContent?.trim()),
      captionCount: captionBtns.length,
      captionDisabled: captionBtns.map(el => el.disabled),
      values: [...document.querySelectorAll('.pp-caption-attrs input')]
        .map(el => ({ type: el.type, value: el.value, checked: el.checked, disabled: el.disabled })),
    };
    ok.click();
    return state;
  });

  await page.waitForSelector('.dialog-wrap', { hidden: true, timeout: 5000 }).catch(() => {});
  const after = await page.evaluate(() => {
    const props = window.__wasm.getShapeProperties(0, 0, 2);
    return {
      hasCaption: props.hasCaption,
      captionDirection: props.captionDirection,
      captionVertAlign: props.captionVertAlign,
      captionWidth: props.captionWidth,
      captionSpacing: props.captionSpacing,
      captionIncludeMargin: props.captionIncludeMargin,
    };
  });
  return { selection, dialogState, after };
}

async function removeOleCaptionAndReadBack(page) {
  await clickOlePreview(page);
  const selection = await page.evaluate(() => {
    const cursor = window.__inputHandler.cursor;
    const selected = cursor.getSelectedPictureRef?.() ?? null;
    const isPictureSelection = cursor.isInPictureObjectSelection?.() ?? false;
    if (!isPictureSelection || selected?.type !== 'ole') {
      return { selected, isPictureSelection, opened: false };
    }
    window.__inputHandler.dispatcher.dispatch('format:object-properties');
    return { selected, isPictureSelection, opened: true };
  });

  await page.waitForSelector('.dialog-wrap', { timeout: 5000 });
  const dialogState = await page.evaluate(() => {
    const marginTab = [...document.querySelectorAll('.dialog-tab')]
      .find(el => el.textContent?.trim() === '여백/캡션');
    if (!marginTab) throw new Error('여백/캡션 tab not found');
    marginTab.click();

    const captionBtns = [...document.querySelectorAll('.pp-caption-btn')];
    if (captionBtns.length !== 9) {
      throw new Error(`caption button count mismatch: ${captionBtns.length}`);
    }
    captionBtns[4].click();

    const ok = [...document.querySelectorAll('button')]
      .find(el => el.textContent?.trim() === '설정(D)');
    if (!ok) throw new Error('설정(D) button not found');
    const state = {
      captionCount: captionBtns.length,
      activeIndex: captionBtns.findIndex(el => el.classList.contains('active')),
      captionDisabled: captionBtns.map(el => el.disabled),
    };
    ok.click();
    return state;
  });

  await page.waitForSelector('.dialog-wrap', { hidden: true, timeout: 5000 }).catch(() => {});
  const after = await page.evaluate(() => {
    const props = window.__wasm.getShapeProperties(0, 0, 2);
    return {
      hasCaption: props.hasCaption,
    };
  });
  return { selection, dialogState, after };
}

async function exercisePictureCopyPasteAfterCaretMove(page) {
  return await page.evaluate((bytes) => {
    window.__wasm.insertText(0, 0, 0, '붙여넣기 위치');
    const targetOffset = window.__wasm.getParagraphLength(0, 0);
    const inserted = window.__wasm.insertPicture(
      0,
      0,
      targetOffset,
      '[]',
      new Uint8Array(bytes),
      24000,
      16000,
      1,
      1,
      'png',
      'copy-paste-after-caret-move',
    );
    const beforeImages = window.__wasm
      .getPageControlLayout(0)
      .controls
      .filter(c => c.type === 'image')
      .length;

    window.__inputHandler.cursor.enterPictureObjectSelectionDirect(0, inserted.paraIdx, inserted.controlIdx, 'image');
    window.__inputHandler.performCopy();
    const copiedText = window.__wasm.getClipboardText();

    window.__inputHandler.cursor.exitPictureObjectSelection();
    window.__inputHandler.cursor.moveTo({
      sectionIndex: 0,
      paragraphIndex: 0,
      charOffset: targetOffset,
    });
    window.__inputHandler.active = true;

    const pasteEvent = new Event('paste', { bubbles: true, cancelable: true });
    Object.defineProperty(pasteEvent, 'clipboardData', {
      value: {
        getData(type) {
          return type === 'text/plain' ? copiedText : '';
        },
        items: [],
      },
    });
    window.__inputHandler.onPaste(pasteEvent);
    window.__canvasView.loadDocument();

    const afterImages = window.__wasm
      .getPageControlLayout(0)
      .controls
      .filter(c => c.type === 'image')
      .length;
    return {
      inserted,
      copiedText,
      beforeImages,
      afterImages,
      pos: window.__inputHandler.cursor.getPosition?.() ?? null,
    };
  }, MINIMAL_PNG);
}

runTest('Issue #2069 OLE 개체 선택', async ({ page }) => {
  await createNewDocument(page);
  const picturePasteState = await exercisePictureCopyPasteAfterCaretMove(page);
  assert(picturePasteState.copiedText === '[그림]', `그림 복사 내부 클립보드 텍스트 확인: ${JSON.stringify(picturePasteState)}`);
  assert(
    picturePasteState.afterImages === picturePasteState.beforeImages + 1,
    `그림 복사 후 다른 캐럿 위치 붙여넣기는 그림 컨트롤을 하나 더 만들어야 함: ${JSON.stringify(picturePasteState)}`,
  );

  await loadHwpFile(page, '한셀OLE.hwp');
  await showParagraphMarks(page);

  const captionState = await setOleCaptionAndReadBack(page);
  assert(captionState.selection.opened === true, `OLE 개체 속성 대화상자를 열 수 있어야 함: ${JSON.stringify(captionState.selection)}`);
  assert(captionState.dialogState.captionCount === 9, `OLE 캡션 위치 버튼 9개 확인: ${JSON.stringify(captionState.dialogState)}`);
  assert(captionState.dialogState.captionDisabled.every(v => v === false), `OLE 캡션 컨트롤은 활성화되어야 함: ${JSON.stringify(captionState.dialogState)}`);
  assert(captionState.after.hasCaption === true, `OLE 캡션 설정이 반영되어야 함: ${JSON.stringify(captionState.after)}`);
  assert(captionState.after.captionDirection === 'Right', `OLE 캡션 방향 확인: ${JSON.stringify(captionState.after)}`);
  assert(captionState.after.captionVertAlign === 'Bottom', `OLE 캡션 세로 위치 확인: ${JSON.stringify(captionState.after)}`);
  assert(captionState.after.captionWidth === Math.round(30 * 283.46), `OLE 캡션 크기 확인: ${JSON.stringify(captionState.after)}`);
  assert(captionState.after.captionSpacing === Math.round(3 * 283.46), `OLE 캡션 간격 확인: ${JSON.stringify(captionState.after)}`);
  assert(captionState.after.captionIncludeMargin === true, `OLE 캡션 여백 확장 확인: ${JSON.stringify(captionState.after)}`);
  await screenshot(page, 'ole-caption-right-bottom');

  const removedCaptionState = await removeOleCaptionAndReadBack(page);
  assert(removedCaptionState.selection.opened === true, `OLE 개체 속성 대화상자를 다시 열 수 있어야 함: ${JSON.stringify(removedCaptionState.selection)}`);
  assert(removedCaptionState.dialogState.activeIndex === 4, `중앙 캡션 없음 버튼이 선택되어야 함: ${JSON.stringify(removedCaptionState.dialogState)}`);
  assert(removedCaptionState.after.hasCaption === false, `중앙 캡션 없음 설정은 OLE 캡션을 제거해야 함: ${JSON.stringify(removedCaptionState.after)}`);
  await screenshot(page, 'ole-caption-removed');

  await loadHwpFile(page, '한셀OLE.hwp');
  await showParagraphMarks(page);

  const reenterState = await exerciseEnterBackspaceReenter(page);
  assert(reenterState.paraCount === 2, `Enter→Backspace→Enter 후 문단 수 확인: ${reenterState.paraCount}`);
  assert(reenterState.ole?.paraIdx === 0, `Backspace 후 재진입 Enter에서도 OLE는 원래 문단에 남아야 함: ${JSON.stringify(reenterState.ole)}`);
  assert(reenterState.pos?.paragraphIndex === 1, `Backspace 후 재진입 Enter 커서는 뒤 문단으로 이동해야 함: ${JSON.stringify(reenterState.pos)}`);
  assert(reenterState.isCell === false, 'Backspace 후 재진입 Enter가 OLE 내부 표 셀 편집으로 들어가면 안 됨');
  assert(
    Math.abs(reenterState.activeRect.x - (reenterState.ole.x + reenterState.ole.w)) <= 1.0,
    `Backspace 후 재진입 Enter caret은 OLE 오른쪽 wrap-zone에 있어야 함: rect=${JSON.stringify(reenterState.activeRect)}, ole=${JSON.stringify(reenterState.ole)}`,
  );
  await screenshot(page, 'ole-enter-backspace-reenter');

  await loadHwpFile(page, '한셀OLE.hwp');
  await showParagraphMarks(page);

  await clickOlePreview(page);

  const state = await page.evaluate(() => {
    const cursor = window.__inputHandler.cursor;
    const ole = window.__wasm.getPageControlLayout(0).controls.find(c => c.type === 'ole');
    const oleCaretRect = window.__wasm.getCursorRect(0, 0, 0);
    return {
      ole,
      selected: cursor.getSelectedPictureRef?.() ?? null,
      rect: cursor.getRect?.() ?? null,
      oleCaretRect,
      isCell: cursor.isInCell?.() ?? false,
      isPictureSelection: cursor.isInPictureObjectSelection?.() ?? false,
    };
  });

  assert(state.isPictureSelection === true, 'OLE 내부 클릭은 개체 선택 상태로 진입해야 함');
  assert(state.selected?.type === 'ole', `선택 개체 타입 확인: ${JSON.stringify(state.selected)}`);
  assert(state.isCell === false, 'OLE 미리보기 내부를 표 셀처럼 편집하면 안 됨');
  assert(
    Math.abs(state.oleCaretRect.x - (state.ole.x + state.ole.w)) <= 1.0,
    `OLE 선택 caret은 OLE 오른쪽에 있어야 함: rect=${JSON.stringify(state.oleCaretRect)}, ole=${JSON.stringify(state.ole)}`,
  );

  await screenshot(page, 'ole-click-object-selection');
});
