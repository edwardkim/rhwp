/**
 * E2E 테스트: 양식 컨트롤 — 셀 커서 진입(#111) + 체크박스 클릭 토글(#112)
 *
 * 시나리오:
 * 1. form-002.hwpx 로드
 * 2. 렌더 트리에서 FormObject(체크박스) bbox 추출
 * 3. hitTest()로 체크박스 셀 클릭 시뮬레이션 → parentParaIndex 확인 (#111)
 * 4. getFormObjectAt()으로 체크박스 감지 확인 (#112)
 * 5. setFormValue()로 체크 토글 후 값 변화 확인 (#112)
 */
import { runTest, loadHwpFile, screenshot, assert, setTestCase } from './helpers.mjs';

runTest('양식 컨트롤 셀 커서 진입 및 체크박스 토글', async ({ page }) => {

  // ── TC-1: form-002.hwpx 로드 ──────────────────────────────
  setTestCase('TC-1: 문서 로드');
  const { pageCount } = await loadHwpFile(page, 'hwpx/form-002.hwpx');
  assert(pageCount >= 1, `form-002.hwpx 로드 성공 (${pageCount}페이지)`);
  await screenshot(page, 'form-01-loaded');

  // ── TC-2: 렌더 트리에서 첫 번째 FormObject bbox 추출 ──────
  setTestCase('TC-2: FormObject bbox 추출');
  const formBbox = await page.evaluate(() => {
    const w = window.__wasm;
    if (!w?.doc?.getPageRenderTree) return null;

    function findFormNode(node) {
      if (node.type === 'Form') return node;
      for (const child of (node.children || [])) {
        const found = findFormNode(child);
        if (found) return found;
      }
      return null;
    }

    try {
      const tree = JSON.parse(w.doc.getPageRenderTree(0));
      const formNode = findFormNode(tree);
      if (!formNode) return null;
      return {
        x: formNode.bbox.x,
        y: formNode.bbox.y,
        w: formNode.bbox.w,
        h: formNode.bbox.h,
      };
    } catch (e) {
      return { error: e.message };
    }
  });

  assert(formBbox !== null && !formBbox?.error, `FormObject bbox 추출 성공: ${JSON.stringify(formBbox)}`);

  if (!formBbox || formBbox.error) {
    console.error('FormObject bbox 추출 실패:', formBbox);
    return;
  }

  // 체크박스 중앙 좌표
  const cx = formBbox.x + formBbox.w / 2;
  const cy = formBbox.y + formBbox.h / 2;
  console.log(`FormObject 중앙 좌표: (${cx.toFixed(1)}, ${cy.toFixed(1)})`);

  // ── TC-3: hitTest()로 셀 커서 진입 확인 (#111) ───────────
  setTestCase('TC-3: hitTest 셀 진입 (#111)');
  const hitResult = await page.evaluate((x, y) => {
    const w = window.__wasm;
    if (!w?.doc?.hitTest) return null;
    try {
      return JSON.parse(w.doc.hitTest(0, x, y));
    } catch (e) {
      return { error: e.message };
    }
  }, cx, cy);

  console.log('hitTest 결과:', JSON.stringify(hitResult));
  assert(hitResult !== null && !hitResult?.error, `hitTest 호출 성공`);
  assert(hitResult?.parentParaIndex !== undefined, `셀 진입 확인: parentParaIndex=${hitResult?.parentParaIndex}`);
  assert(hitResult?.controlIndex !== undefined, `controlIndex 존재: ${hitResult?.controlIndex}`);
  assert(hitResult?.cellIndex !== undefined, `cellIndex 존재: ${hitResult?.cellIndex}`);
  await screenshot(page, 'form-02-hittest');

  // ── TC-4: getFormObjectAt()으로 체크박스 감지 (#112) ─────
  setTestCase('TC-4: getFormObjectAt 체크박스 감지 (#112)');
  const formHit = await page.evaluate((x, y) => {
    const w = window.__wasm;
    if (!w?.getFormObjectAt) return null;
    try {
      return w.getFormObjectAt(0, x, y);
    } catch (e) {
      return { found: false, error: e.message };
    }
  }, cx, cy);

  console.log('getFormObjectAt 결과:', JSON.stringify(formHit));
  assert(formHit?.found === true, `체크박스 감지: found=${formHit?.found}`);
  assert(formHit?.formType === 'CheckBox', `formType=CheckBox 확인: ${formHit?.formType}`);
  await screenshot(page, 'form-03-form-hit');

  // ── TC-5: 체크박스 토글 (#112) ───────────────────────────
  setTestCase('TC-5: 체크박스 값 토글 (#112)');
  if (!formHit?.found) {
    console.warn('TC-5 건너뜀: 체크박스 감지 실패');
    return;
  }

  const initialValue = formHit.value;
  const expectedValue = initialValue === 0 ? 1 : 0;
  console.log(`초기 value=${initialValue}, 토글 후 기대값=${expectedValue}`);
  console.log(`formHit 경로:`, JSON.stringify(formHit));

  // setFormValue / setFormValueInCell로 토글
  const toggleResult = await page.evaluate((hit, newVal) => {
    const w = window.__wasm;
    const valueJson = JSON.stringify({ value: newVal });
    try {
      let r;
      if (hit.inCell && hit.tablePara !== undefined && hit.tableCi !== undefined
          && hit.cellIdx !== undefined && hit.cellPara !== undefined) {
        // 셀 내부 폼
        if (!w?.doc?.setFormValueInCell) return { ok: false, error: 'setFormValueInCell 없음' };
        r = JSON.parse(w.doc.setFormValueInCell(hit.sec, hit.tablePara, hit.tableCi,
          hit.cellIdx, hit.cellPara, hit.ci, valueJson));
      } else {
        if (!w?.doc?.setFormValue) return { ok: false, error: 'setFormValue 없음' };
        r = JSON.parse(w.doc.setFormValue(hit.sec, hit.para, hit.ci, valueJson));
      }
      window.__eventBus?.emit('document-changed');
      return r;
    } catch (e) {
      return { ok: false, error: e.message };
    }
  }, formHit, expectedValue);

  console.log('setFormValue 결과:', JSON.stringify(toggleResult));
  assert(toggleResult?.ok === true, `setFormValue 성공: ok=${toggleResult?.ok}`);

  await page.evaluate(() => new Promise(r => setTimeout(r, 500)));
  await screenshot(page, 'form-04-toggled');

  // 토글 후 값 확인: getFormObjectAt으로 재조회
  const afterHit = await page.evaluate((x, y) => {
    const w = window.__wasm;
    if (!w?.getFormObjectAt) return null;
    try { return w.getFormObjectAt(0, x, y); } catch { return null; }
  }, cx, cy);

  console.log('토글 후 getFormObjectAt:', JSON.stringify(afterHit));
  assert(afterHit?.value === expectedValue, `토글 결과 확인: value=${afterHit?.value} (기대=${expectedValue})`);

  // 원래 값으로 복원
  await page.evaluate((hit, origVal) => {
    const w = window.__wasm;
    const valueJson = JSON.stringify({ value: origVal });
    if (hit.inCell && hit.tablePara !== undefined) {
      w?.doc?.setFormValueInCell?.(hit.sec, hit.tablePara, hit.tableCi,
        hit.cellIdx, hit.cellPara, hit.ci, valueJson);
    } else {
      w?.doc?.setFormValue?.(hit.sec, hit.para, hit.ci, valueJson);
    }
    window.__eventBus?.emit('document-changed');
  }, formHit, initialValue);

  await page.evaluate(() => new Promise(r => setTimeout(r, 300)));
  await screenshot(page, 'form-05-restored');
});
