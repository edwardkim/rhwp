/**
 * E2E 테스트: 편집 undo 계약 실동작 검증 (Task #2301)
 *
 * tests/*.test.ts 의 소스 검사(문자열 매칭) 스위트 5종이 보장하지 못하는
 * 런타임 동작 — "실제 경로로 편집 → 문서 상태 변경 → undo → 원상 복원" —
 * 을 검증한다. 대응 관계:
 *
 *   케이스 1  find-replace-undo.test.ts      (#2037 모두 바꾸기)
 *   케이스 2  picture-props-undo.test.ts     (#2027 그림 속성 다이얼로그)
 *   케이스 3  equation-props-undo.test.ts    (#2077 수식 속성 다이얼로그)
 *   케이스 4  table-props-undo.test.ts       (#2053 표/셀 속성 다이얼로그)
 *   케이스 5  wrap-through-preserve.test.ts  (#2054 Through 배치 보존)
 *
 * 구동 원칙:
 *  - 다이얼로그 open(툴바 개체 속성 / Ctrl+F2), 확인 버튼, Ctrl+Z 는 실제
 *    DOM/키보드 경로로 구동한다.
 *  - 개체 삽입/선택은 기존 e2e 관례대로 window.__wasm / window.__inputHandler
 *    훅을 쓴다(좌표 클릭 의존 제거). raw wasm 호출은 undo 스택(스튜디오
 *    CommandHistory)에 기록되지 않으므로 순수 setup 으로 안전하다.
 *  - assert 는 IR 조회(searchAllText / get*Properties)만 사용한다 — 픽셀 비교 없음.
 *  - 세 속성 다이얼로그의 확인 버튼은 공통 클래스 `.dialog-btn-primary` 로 찾는다
 *    (그림/수식 "설정(D)", 표 "확인").
 */
import {
  runTest, setTestCase, createNewDocument, clickEditArea, typeText,
  screenshot, assert,
} from './helpers.mjs';

// ─── 공용 헬퍼 ──────────────────────────────────────────────

const sleep = (page, ms) => page.evaluate(t => new Promise(r => setTimeout(r, t)), ms);

/** 1x1 투명 PNG (그림 삽입용 최소 fixture) */
const TINY_PNG_BASE64 = 'iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8z8BQDwAEhQGAhKmMIQAAAABJRU5ErkJggg==';

/** undo 스택 깊이 */
async function undoDepth(page) {
  return await page.evaluate(() => window.__inputHandler?.history?.undoStack?.length ?? -1);
}

/** 편집 영역에 포커스를 두고 실제 Ctrl+Z (텍스트 편집 모드 전용) */
async function pressUndo(page) {
  await clickEditArea(page);
  await page.keyboard.down('Control');
  await page.keyboard.press('KeyZ');
  await page.keyboard.up('Control');
  await sleep(page, 450);
}

/**
 * 개체 선택 모드용 undo — edit:undo 커맨드 진입점(performUndo)을 직접 호출한다.
 * 키보드→undo 배선은 케이스 1(find-replace)의 실제 Ctrl+Z 로 이미 증명되므로,
 * 개체 케이스는 마우스 클릭 부작용 없이 스냅샷 복원만 검증한다.
 */
async function undoViaHandler(page) {
  await page.evaluate(() => {
    const ih = window.__inputHandler;
    ih.cursor.exitPictureObjectSelection?.();
    ih.cursor.exitTableObjectSelection?.();
    ih.performUndo();
  });
  await sleep(page, 450);
}

/** 모든 개체/표 선택 해제 (케이스 간 상태 격리) */
async function clearSelections(page) {
  await page.evaluate(() => {
    const ih = window.__inputHandler;
    ih?.cursor?.exitPictureObjectSelection?.();
    ih?.cursor?.exitTableObjectSelection?.();
  });
}

/** searchAllText 히트 수 (본문+셀) */
async function hitCount(page, query) {
  return await page.evaluate((q) => {
    const r = window.__wasm.searchAllText(q, false, true);
    const arr = typeof r === 'string' ? JSON.parse(r) : r;
    return Array.isArray(arr) ? arr.length : -1;
  }, query);
}

/** 문서 재렌더 트리거 (raw wasm 변형 후 뷰 동기화) */
async function refreshView(page) {
  await page.evaluate(() => window.__eventBus?.emit('document-changed'));
  await sleep(page, 350);
}

/** 툴바 개체 속성 버튼으로 속성 다이얼로그 열기 (실제 커맨드 경로) */
async function openObjectPropsDialog(page) {
  const dispatched = await page.evaluate(() => {
    const btn = document.querySelector('.tb-btn[data-cmd="format:object-properties"]');
    if (!btn) return false;
    btn.dispatchEvent(new MouseEvent('mousedown', { bubbles: true, cancelable: true }));
    return true;
  });
  if (!dispatched) throw new Error('개체 속성 툴바 버튼을 찾지 못함');
  await sleep(page, 500);
  const visible = await page.evaluate(() =>
    [...document.querySelectorAll('button.dialog-btn-primary')].some(b => b.offsetParent !== null));
  if (!visible) throw new Error('개체 속성 다이얼로그가 열리지 않음');
}

/** 다이얼로그 확인/설정 버튼 클릭 (.dialog-btn-primary 공통) */
async function clickPrimary(page) {
  const ok = await page.evaluate(() => {
    const btns = [...document.querySelectorAll('button.dialog-btn-primary')].filter(b => b.offsetParent !== null);
    if (!btns.length) return false;
    const b = btns[btns.length - 1];
    b.dispatchEvent(new MouseEvent('mousedown', { bubbles: true }));
    b.click();
    return true;
  });
  if (!ok) throw new Error('확인/설정 버튼(.dialog-btn-primary)을 찾지 못함');
  await sleep(page, 500);
}

/** 다이얼로그 내 라벨 포함 체크박스 토글 */
async function toggleDialogCheckbox(page, labelIncludes) {
  const r = await page.evaluate((lbl) => {
    const cbs = [...document.querySelectorAll('input[type=checkbox]')].filter(c => c.offsetParent !== null);
    const cb = cbs.find(c => {
      const row = c.closest('label') || c.closest('div');
      return row && row.textContent.includes(lbl);
    });
    if (!cb) return { ok: false };
    cb.click();
    return { ok: true, now: cb.checked };
  }, labelIncludes);
  if (!r.ok) throw new Error(`체크박스를 찾지 못함: "${labelIncludes}"`);
  await sleep(page, 150);
  return r;
}

/** 다이얼로그 탭 클릭 */
async function clickDialogTab(page, name) {
  const ok = await page.evaluate((n) => {
    const t = [...document.querySelectorAll('.dialog-tab')]
      .filter(x => x.offsetParent !== null && x.textContent.trim() === n).pop();
    if (!t) return false;
    t.click();
    return true;
  }, name);
  if (!ok) throw new Error(`다이얼로그 탭을 찾지 못함: "${name}"`);
  await sleep(page, 300);
}

/** 그림 조회 */
async function getPic(page, ci) {
  return await page.evaluate((c) => {
    const r = window.__wasm.getPictureProperties(0, 0, c);
    return typeof r === 'string' ? JSON.parse(r) : r;
  }, ci);
}

/** 그림 삽입(플로팅) 후 개체 선택. controlIdx 반환 */
async function insertFloatingPicture(page, extraProps = {}) {
  const ci = await page.evaluate(({ b64, extra }) => {
    const bin = atob(b64);
    const bytes = new Uint8Array(bin.length);
    for (let i = 0; i < bin.length; i++) bytes[i] = bin.charCodeAt(i);
    const ret = window.__wasm.insertPicture(0, 0, 0, '[]', bytes, 9000, 9000, 100, 100, 'png', '', null, null);
    const info = typeof ret === 'string' ? JSON.parse(ret) : ret;
    const c = info.controlIdx;
    // 플로팅 확정(+요청 속성). 브리지가 객체를 받아 내부에서 stringify 하므로
    // 객체를 그대로 넘긴다. raw wasm 이라 undo 스택에 기록되지 않음.
    window.__wasm.setPictureProperties(0, 0, c, { treatAsChar: false, ...extra });
    return c;
  }, { b64: TINY_PNG_BASE64, extra: extraProps });
  await refreshView(page);
  await page.evaluate((c) => {
    const ih = window.__inputHandler;
    ih.cursor.enterPictureObjectSelectionDirect(0, 0, c, 'image');
    ih.active = true;
    window.__eventBus?.emit('picture-object-selection-changed', true);
  }, ci);
  await sleep(page, 200);
  return ci;
}

// ─── 테스트 본문 ────────────────────────────────────────────

runTest('편집 undo 계약 실동작 (Task #2301)', async ({ page }) => {

  // ── 케이스 1: 모두 바꾸기 undo (#2037) ──────────────────
  setTestCase('find-replace: 모두 바꾸기 → Ctrl+Z 복원');
  console.log('\n[1] 모두 바꾸기 undo...');
  await createNewDocument(page);
  await clickEditArea(page);
  await typeText(page, 'alpha beta alpha');
  assert(await hitCount(page, 'alpha') === 2, '치환 전 alpha 2건');

  await page.keyboard.down('Control');
  await page.keyboard.press('F2');
  await page.keyboard.up('Control');
  await sleep(page, 400);
  assert(!!(await page.$('.find-dialog')), '찾아 바꾸기 다이얼로그 열림');

  await page.evaluate(() => {
    const inputs = [...document.querySelectorAll('.find-dialog-input')];
    inputs[0].value = 'alpha';
    inputs[1].value = 'gamma';
  });
  await page.evaluate(() => {
    const btn = [...document.querySelectorAll('.find-dialog button, .dialog-btn')]
      .find(b => b.offsetParent !== null && b.textContent.trim() === '모두 바꾸기');
    btn.dispatchEvent(new MouseEvent('mousedown', { bubbles: true }));
    btn.click();
  });
  await sleep(page, 400);

  assert(await hitCount(page, 'alpha') === 0, '모두 바꾸기 후 alpha 0건');
  assert(await hitCount(page, 'gamma') === 2, '모두 바꾸기 후 gamma 2건');
  assert(await undoDepth(page) >= 1, `undo 스택에 기록됨 (depth=${await undoDepth(page)})`);

  await page.keyboard.press('Escape');
  await sleep(page, 200);
  await pressUndo(page);
  assert(await hitCount(page, 'alpha') === 2, 'undo 후 alpha 2건 복원');
  assert(await hitCount(page, 'gamma') === 0, 'undo 후 gamma 0건');
  await screenshot(page, 'undo-01-find-replace');

  // ── 케이스 2: 그림 속성 다이얼로그 undo (#2027) ─────────
  setTestCase('picture-props: 쪽 영역 제한 토글 → Ctrl+Z 복원');
  await clearSelections(page);
  console.log('\n[2] 그림 속성 다이얼로그 undo...');
  await createNewDocument(page);
  const picCi = await insertFloatingPicture(page);

  const picBefore = await getPic(page, picCi);
  console.log(`  변경 전 restrictInPage=${picBefore.restrictInPage}`);
  const picDepth0 = await undoDepth(page);

  await openObjectPropsDialog(page);
  await toggleDialogCheckbox(page, '쪽 영역 안으로 제한');
  await clickPrimary(page);

  const picAfter = await getPic(page, picCi);
  assert(picAfter.restrictInPage !== picBefore.restrictInPage,
    `설정 후 restrictInPage 변경됨 (${picBefore.restrictInPage}→${picAfter.restrictInPage})`);
  assert(await undoDepth(page) === picDepth0 + 1, '다이얼로그 확인이 undo 스택에 1건 기록');

  await undoViaHandler(page);
  const picUndone = await getPic(page, picCi);
  assert(picUndone.restrictInPage === picBefore.restrictInPage,
    `undo 후 restrictInPage 복원 (${picUndone.restrictInPage})`);
  await screenshot(page, 'undo-02-picture-props');

  // ── 케이스 3: 수식 속성 다이얼로그 undo (#2077) ─────────
  setTestCase('equation-props: 글자 크기 변경 → Ctrl+Z 복원');
  await clearSelections(page);
  console.log('\n[3] 수식 속성 다이얼로그 undo...');
  await createNewDocument(page);
  const eqCi = await page.evaluate(() => {
    const ret = window.__wasm.insertEquation(0, 0, 0, 'a over b', 1000, 0);
    const info = typeof ret === 'string' ? JSON.parse(ret) : ret;
    return info.controlIdx;
  });
  await refreshView(page);
  await page.evaluate((c) => {
    const ih = window.__inputHandler;
    ih.cursor.enterPictureObjectSelectionDirect(0, 0, c, 'equation');
    ih.active = true;
    window.__eventBus?.emit('picture-object-selection-changed', true);
  }, eqCi);
  await sleep(page, 200);

  const getEq = () => page.evaluate((c) => {
    const r = window.__wasm.getEquationProperties(0, 0, c, -1, -1);
    return typeof r === 'string' ? JSON.parse(r) : r;
  }, eqCi);
  const eqBefore = await getEq();
  console.log(`  변경 전 fontSize=${eqBefore.fontSize}`);
  const eqDepth0 = await undoDepth(page);

  await openObjectPropsDialog(page);
  await clickDialogTab(page, '수식');
  await page.evaluate(() => {
    const inp = [...document.querySelectorAll('input[type=number]')]
      .filter(i => i.offsetParent !== null && i.max === '127').pop();
    if (!inp) throw new Error('수식 글자 크기 입력을 찾지 못함');
    inp.value = '20';
    inp.dispatchEvent(new Event('input', { bubbles: true }));
    inp.dispatchEvent(new Event('change', { bubbles: true }));
  });
  await clickPrimary(page);

  const eqAfter = await getEq();
  assert(eqAfter.fontSize !== eqBefore.fontSize,
    `설정 후 fontSize 변경됨 (${eqBefore.fontSize}→${eqAfter.fontSize})`);
  assert(await undoDepth(page) === eqDepth0 + 1, '다이얼로그 확인이 undo 스택에 1건 기록');

  await undoViaHandler(page);
  const eqUndone = await getEq();
  assert(eqUndone.fontSize === eqBefore.fontSize,
    `undo 후 fontSize 복원 (${eqUndone.fontSize})`);
  await screenshot(page, 'undo-03-equation-props');

  // ── 케이스 4: 표/셀 속성 다이얼로그 undo (#2053) ────────
  setTestCase('table-props: 쪽 영역 제한 토글 → Ctrl+Z 복원');
  await clearSelections(page);
  console.log('\n[4] 표/셀 속성 다이얼로그 undo...');
  await createNewDocument(page);
  // createTable 은 문단을 분할하므로 표의 실제 위치는 반환값의 paraIdx/controlIdx 를 쓴다.
  const tbl = await page.evaluate(() => {
    const ret = window.__wasm.createTable(0, 0, 0, 2, 2);
    const info = typeof ret === 'string' ? JSON.parse(ret) : ret;
    return { ppi: info.paraIdx, ci: info.controlIdx };
  });
  await refreshView(page);
  await page.evaluate((t) => {
    const ih = window.__inputHandler;
    ih.cursor.moveTo({
      sectionIndex: 0, paragraphIndex: 0, charOffset: 0,
      parentParaIndex: t.ppi, controlIndex: t.ci, cellIndex: 0,
    });
    ih.cursor.enterTableObjectSelectionDirect(0, t.ppi, t.ci);
    ih.active = true;
    window.__eventBus?.emit('table-object-selection-changed', true);
  }, tbl);
  await sleep(page, 200);

  const getTbl = () => page.evaluate((t) => {
    const r = window.__wasm.getTableProperties(0, t.ppi, t.ci);
    return typeof r === 'string' ? JSON.parse(r) : r;
  }, tbl);
  const tblBefore = await getTbl();
  console.log(`  변경 전 restrictInPage=${tblBefore.restrictInPage}`);
  const tblDepth0 = await undoDepth(page);

  await openObjectPropsDialog(page);
  await toggleDialogCheckbox(page, '쪽 영역 안으로 제한');
  await clickPrimary(page);

  const tblAfter = await getTbl();
  assert(tblAfter.restrictInPage !== tblBefore.restrictInPage,
    `확인 후 restrictInPage 변경됨 (${tblBefore.restrictInPage}→${tblAfter.restrictInPage})`);
  assert(await undoDepth(page) === tblDepth0 + 1, '다이얼로그 확인이 undo 스택에 1건 기록');

  await undoViaHandler(page);
  const tblUndone = await getTbl();
  assert(tblUndone.restrictInPage === tblBefore.restrictInPage,
    `undo 후 restrictInPage 복원 (${tblUndone.restrictInPage})`);
  await screenshot(page, 'undo-04-table-props');

  // ── 케이스 5: Through 배치 보존 (#2054) ─────────────────
  setTestCase('wrap-through: 속성창 확인만 → textWrap 보존');
  await clearSelections(page);
  console.log('\n[5] Through 배치 보존...');
  await createNewDocument(page);
  const thCi = await insertFloatingPicture(page, { textWrap: 'Through' });

  const thBefore = await getPic(page, thCi);
  assert(thBefore.textWrap === 'Through', `사전 상태 textWrap=Through (${thBefore.textWrap})`);

  await openObjectPropsDialog(page);
  await clickPrimary(page); // 아무것도 바꾸지 않고 확인만

  const thAfter = await getPic(page, thCi);
  assert(thAfter.textWrap === 'Through',
    `확인만 눌러도 Through 유지 (${thAfter.textWrap})`);
  await screenshot(page, 'undo-05-wrap-through');
});
