// E2E: undo/redo 후 개체/표 선택 stale ref 해제 (Task #2303)
//
// 계약: undo/redo 는 문단 컨트롤 구성을 되돌릴 수 있으므로, 히스토리 점프 시
// 위치 기반 개체/표 선택 ref({sec, ppi, ci})를 유지하면 stale 이 된다.
// 수정 전에는 선택이 살아남아 개체 속성(format:object-properties)이
// "렌더링 오류: 지정된 컨트롤이 그림이 아닙니다" WASM 예외로 실패했다(#2303).
//
// 검증 흐름 (이슈 #2303 재현 시퀀스):
//   수식(ci=2)+부동 그림(ci=3) → 수식 삭제(그림 ci=2 시프트) → 그림 선택
//   → undo(수식 복원, 그림 ci=3 복귀) → 선택 해제 확인 → 개체 속성 무오류
//
// 실행: CHROME_PATH=... node e2e/undo-object-selection.test.mjs --mode=headless

import {
  runTest,
  createNewDocument,
  setTestCase,
  screenshot,
  assert,
} from './helpers.mjs';

const sleep = (page, ms) => page.evaluate((t) => new Promise((r) => setTimeout(r, t)), ms);

// 1×1 투명 PNG
const PNG_1PX = [
  137, 80, 78, 71, 13, 10, 26, 10, 0, 0, 0, 13, 73, 72, 68, 82, 0, 0, 0, 1, 0, 0, 0, 1,
  8, 6, 0, 0, 0, 31, 21, 196, 137, 0, 0, 0, 13, 73, 68, 65, 84, 120, 156, 99, 248, 255,
  255, 63, 0, 5, 254, 2, 254, 220, 204, 89, 231, 0, 0, 0, 0, 73, 69, 78, 68, 174, 66, 96, 130,
];

/** 콘솔 dispatcher 오류 수집기 — 커맨드 실행 실패 로그를 감시한다. */
function collectDispatcherErrors(page, store) {
  const listener = (msg) => {
    if (msg.type() === 'error' && msg.text().includes('커맨드 실행 실패')) {
      store.push(msg.text());
    }
  };
  page.on('console', listener);
  return () => page.off('console', listener);
}

/** 문단 0 의 컨트롤 배치를 probe 한다 (picture/equation 만). */
async function probeControls(page) {
  return page.evaluate(() => {
    const out = [];
    for (let ci = 0; ci < 8; ci++) {
      try { window.__wasm.getPictureProperties(0, 0, ci); out.push(`${ci}:picture`); } catch { /* skip */ }
      try { window.__wasm.getEquationProperties(0, 0, ci, -1, -1); out.push(`${ci}:equation`); } catch { /* skip */ }
    }
    return out;
  });
}

/** 도구 상자 '개체 속성' 버튼을 실제 DOM 경로로 누른다. */
async function clickObjectPropsButton(page) {
  await page.evaluate(() => {
    const btn = document.querySelector('.tb-btn[data-cmd="format:object-properties"]');
    btn.dispatchEvent(new MouseEvent('mousedown', { bubbles: true }));
    btn.click();
  });
  await sleep(page, 400);
}

runTest('undo/redo 후 개체 선택 stale ref 해제 (Task #2303)', async ({ page }) => {
  // ── 케이스 1: 그림 선택 + undo/redo ──────────────────────────
  setTestCase('그림 선택 중 undo → 선택 해제 + 개체 속성 무오류');
  console.log('[1] 그림 선택 undo/redo...');
  await createNewDocument(page);

  // 수식(ci=2) → 부동 그림(ci=3) — 실제 편집 라우터(snapshot)로 기록
  await page.evaluate((png) => {
    const ih = window.__inputHandler;
    ih.executeOperation({ kind: 'snapshot', operationType: 'insertEquation', operation: () => {
      window.__wasm.insertEquation(0, 0, 0, 'a = b', 1000, 0);
      return ih.getCursorPosition();
    } });
    ih.executeOperation({ kind: 'snapshot', operationType: 'insertPicture', operation: () => {
      const ret = window.__wasm.insertPicture(0, 0, 0, '[]', new Uint8Array(png), 9000, 9000, 100, 100, 'png', '', null, null);
      const info = typeof ret === 'string' ? JSON.parse(ret) : ret;
      window.__wasm.setPictureProperties(0, info.paraIdx, info.controlIdx, { treatAsChar: false });
      return ih.getCursorPosition();
    } });
  }, PNG_1PX);
  await sleep(page, 300);
  assert((await probeControls(page)).join(',') === '2:equation,3:picture', '초기 배치: 수식 ci=2, 그림 ci=3');

  // 수식 삭제 (개체 선택 Delete 키 핸들러와 동일 시퀀스) → 그림이 ci=2 로 시프트
  await page.evaluate(() => {
    const ih = window.__inputHandler;
    ih.cursor.enterPictureObjectSelectionDirect(0, 0, 2, 'equation');
    const ref = ih.cursor.getSelectedPictureRef();
    ih.cursor.moveOutOfSelectedPicture();
    ih.executeOperation({ kind: 'snapshot', operationType: 'deleteObject', operation: () => {
      window.__wasm.deleteEquationControl(ref.sec, ref.ppi, ref.ci);
      return ih.cursor.getPosition();
    } });
  });
  await sleep(page, 300);
  assert((await probeControls(page)).join(',') === '2:picture', '수식 삭제 후: 그림 ci=2 시프트');

  // 그림 개체 선택 (클릭 선택과 동일 상태)
  await page.evaluate(() => {
    const ih = window.__inputHandler;
    ih.cursor.enterPictureObjectSelectionDirect(0, 0, 2, 'image');
    ih.renderPictureObjectSelection?.();
  });
  assert(await page.evaluate(() => window.__inputHandler.isInPictureObjectSelection()), '그림 개체 선택됨');

  // undo (편집 메뉴 '되돌리기'와 동일 경로: edit:undo → performUndo)
  const errors = [];
  const stopCollect = collectDispatcherErrors(page, errors);
  await page.evaluate(() => window.__inputHandler.performUndo());
  await sleep(page, 500);

  assert((await probeControls(page)).join(',') === '2:equation,3:picture', 'undo 후: 수식 복원(ci=2)·그림 ci=3 복귀');
  assert(
    !(await page.evaluate(() => window.__inputHandler.isInPictureObjectSelection())),
    'undo 후 개체 선택이 해제된다 (stale ref 제거 — #2303 핵심 계약)',
  );
  assert(
    (await page.evaluate(() => window.__inputHandler.cursor.getSelectedPictureRef())) === null,
    'undo 후 선택 ref 가 null 이다',
  );

  // 개체 속성 실행 → 선택이 없으므로 no-op, dispatcher 오류가 없어야 한다
  await clickObjectPropsButton(page);
  assert(!(await page.evaluate(() => !!document.querySelector('.pp-dialog'))), '개체 속성 다이얼로그 미표시(no-op)');
  assert(errors.length === 0, `undo 후 개체 속성에서 커맨드 실행 실패 없음 (수집: ${errors.length}건)`);

  // redo 대칭: 그림 재선택(ci=3) → redo(수식 재삭제, 그림 ci=2 시프트) → 선택 해제
  await page.evaluate(() => {
    const ih = window.__inputHandler;
    ih.cursor.enterPictureObjectSelectionDirect(0, 0, 3, 'image');
    ih.renderPictureObjectSelection?.();
  });
  await page.evaluate(() => window.__inputHandler.performRedo());
  await sleep(page, 500);
  assert((await probeControls(page)).join(',') === '2:picture', 'redo 후: 수식 재삭제·그림 ci=2');
  assert(
    !(await page.evaluate(() => window.__inputHandler.isInPictureObjectSelection())),
    'redo 후에도 개체 선택이 해제된다',
  );
  await clickObjectPropsButton(page);
  assert(errors.length === 0, 'redo 후 개체 속성에서 커맨드 실행 실패 없음');
  stopCollect();
  await screenshot(page, 'undo-objsel-01-picture');

  // ── 케이스 2: 표 선택 + undo ────────────────────────────────
  setTestCase('표 선택 중 undo → 선택 해제');
  console.log('[2] 표 선택 undo...');
  await createNewDocument(page);

  const tbl = await page.evaluate(() => {
    const ih = window.__inputHandler;
    let info = null;
    ih.executeOperation({ kind: 'snapshot', operationType: 'createTable', operation: () => {
      const ret = window.__wasm.createTable(0, 0, 0, 2, 2);
      info = typeof ret === 'string' ? JSON.parse(ret) : ret;
      return ih.getCursorPosition();
    } });
    return { ppi: info.paraIdx, ci: info.controlIdx };
  });
  await sleep(page, 300);

  await page.evaluate((t) => {
    const ih = window.__inputHandler;
    ih.cursor.moveTo({
      sectionIndex: 0, paragraphIndex: 0, charOffset: 0,
      parentParaIndex: t.ppi, controlIndex: t.ci, cellIndex: 0,
    });
    ih.cursor.enterTableObjectSelectionDirect(0, t.ppi, t.ci);
  }, tbl);
  assert(await page.evaluate(() => window.__inputHandler.cursor.isInTableObjectSelection()), '표 개체 선택됨');

  const errors2 = [];
  const stopCollect2 = collectDispatcherErrors(page, errors2);
  await page.evaluate(() => window.__inputHandler.performUndo());
  await sleep(page, 500);

  assert(
    !(await page.evaluate(() => window.__inputHandler.cursor.isInTableObjectSelection())),
    'undo 후 표 개체 선택이 해제된다',
  );
  await clickObjectPropsButton(page);
  assert(errors2.length === 0, 'undo 후 개체 속성에서 커맨드 실행 실패 없음(표)');
  stopCollect2();
  await screenshot(page, 'undo-objsel-02-table');
});
