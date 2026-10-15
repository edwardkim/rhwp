/**
 * E2E 테스트 (Issue #1280 v2): 겹침 최상단 선택 → 연산 lifecycle
 *
 * 메모리 룰 `audit-selection-ref-consumers`(PR #1254 교훈) 적용:
 *   겹침 클릭이 "다른 개체(최상단)"를 선택하게 되었으므로, 선택 후 삭제/오려두기 등
 *   실제 연산이 그 선택 ref 를 올바로 소비하는지(엉뚱한 개체를 건드리지 않는지) 검증한다.
 *
 * 시나리오: samples/textbox-under-image.hwp (글상자 plane3 위, 이미지 plane2 아래 — 겹침)
 *   1) 겹침 영역 클릭 → 최상단 글상자(shape) 선택 → performDelete → 글상자만 삭제, 이미지 잔존.
 *   2) (재로드) 동일 선택 → performCut → 글상자만 삭제(클립보드 이동), 이미지 잔존.
 */
import { runTest, loadHwpFile, assert } from './helpers.mjs';

/** 겹침 최상단 개체를 실제 hit-test 경로로 선택 후 op(delete|cut) 수행. (페이지 컨텍스트에서 실행) */
async function selectTopmostAndOperate(page, op) {
  return page.evaluate((operation) => {
    const wasm = window.__wasm;
    const ih = window.__inputHandler;
    if (!ih.isActive()) return { error: '편집기가 활성화되기 전에 개체 연산을 시작함' };
    const overlaps = (a, b) => a.x < b.x + b.w && b.x < a.x + a.w && a.y < b.y + b.h && b.y < a.y + a.h;

    let target = null;
    for (let p = 0; p < wasm.pageCount; p++) {
      let layout;
      try { layout = wasm.getPageControlLayout(p); } catch { continue; }
      const controls = layout.controls || [];
      const shapes = controls.filter((c) => c.type === 'shape');
      const images = controls.filter((c) => c.type === 'image');
      const shape = shapes[0], image = images[0];
      if (!shape || !image || !overlaps(shape, image)) continue;
      target = {
        p,
        cx: (Math.max(shape.x, image.x) + Math.min(shape.x + shape.w, image.x + image.w)) / 2,
        cy: (Math.max(shape.y, image.y) + Math.min(shape.y + shape.h, image.y + image.h)) / 2,
        beforeShapes: shapes.length, beforeImages: images.length,
        shapeRef: { sec: shape.secIdx, ppi: shape.paraIdx, ci: shape.controlIdx },
      };
      break;
    }
    if (!target) return { error: '겹치는 shape+image 쌍을 못 찾음' };

    // 실제 hit-test 경로로 최상단 개체 선택 → 연산이 그 선택 ref 를 소비
    const hit = ih.findPictureAtClick(target.p, target.cx, target.cy);
    if (!hit) return { error: '겹침 클릭에서 hit 없음' };
    ih.selectPictureObject(hit.sec, hit.ppi, hit.ci, hit.type);
    const sel = ih.getSelectedPictureRef();
    if (operation === 'delete') ih.performDelete();
    else if (operation === 'cut') ih.performCut();

    return {
      p: target.p, shapeRef: target.shapeRef,
      beforeShapes: target.beforeShapes, beforeImages: target.beforeImages,
      hitType: hit.type, hitCi: hit.ci, selType: sel?.type ?? null, selCi: sel?.ci ?? null,
    };
  }, op);
}

/** 특정 페이지의 type별 컨트롤 개수 */
async function countTypes(page, p) {
  return page.evaluate((pageIdx) => {
    const controls = window.__wasm.getPageControlLayout(pageIdx).controls || [];
    return {
      shapes: controls.filter((c) => c.type === 'shape').length,
      images: controls.filter((c) => c.type === 'image').length,
    };
  }, p);
}

async function runScenario(page, op, label) {
  await loadHwpFile(page, 'textbox-under-image.hwp');
  const r = await selectTopmostAndOperate(page, op);
  assert(!r.error, `${label} 시나리오 실패: ${r.error}`);
  console.log(`${label}:`, JSON.stringify(r));

  // 최상단 = 글상자(shape) 선택 확인
  assert(r.hitType === 'shape' && r.selType === 'shape',
    `${label} 최상단 선택이 글상자 아님: hit=${r.hitType}, sel=${r.selType}`);
  assert(r.selCi === r.shapeRef.ci, `${label} 선택 ref 가 글상자 ci 와 불일치: ${r.selCi} vs ${r.shapeRef.ci}`);

  await page.waitForFunction(({ p, beforeShapes }) => {
    const ih = window.__inputHandler;
    const controls = window.__wasm.getPageControlLayout(p).controls || [];
    return !ih.isInPictureObjectSelection()
      && controls.filter(control => control.type === 'shape').length === beforeShapes - 1;
  }, { timeout: 10000 }, { p: r.p, beforeShapes: r.beforeShapes });
  const after = await countTypes(page, r.p);
  console.log(`${label} 후 카운트:`, JSON.stringify(after), `(이전 shapes=${r.beforeShapes}, images=${r.beforeImages})`);

  // 글상자만 1개 줄고, 이미지는 그대로 (엉뚱한 개체 미처리)
  assert(after.shapes === r.beforeShapes - 1,
    `${label}: 글상자가 1개 줄지 않음: ${r.beforeShapes} → ${after.shapes}`);
  assert(after.images === r.beforeImages,
    `${label}: 이미지가 영향받음(엉뚱한 개체 처리): ${r.beforeImages} → ${after.images}`);
  if (op === 'cut') {
    const { copiedHtml, token } = await page.evaluate(async () => {
      const items = await navigator.clipboard.read();
      const item = items.find(value => value.types.includes('text/html'));
      return {
        copiedHtml: item ? await (await item.getType('text/html')).text() : '',
        token: window.__inputHandler.rhwpClipboardToken,
      };
    });
    assert(token && copiedHtml.includes(`<!--rhwp-studio-clipboard:${token}-->`),
      '이번 오려두기의 HTML 클립보드 기록 완료');
  }
}

runTest('겹침 최상단 선택 → 삭제/오려두기 lifecycle (#1280 v2)', async ({ browser, page }) => {
  // 임시 headless 프로필만 허용하며, 연결한 사용자 브라우저의 권한은 바꾸지 않는다.
  if (!browser._isRemote) {
    await browser.defaultBrowserContext().overridePermissions(new URL(page.url()).origin,
      ['clipboard-read', 'clipboard-write', 'clipboard-sanitized-write']);
  }
  await runScenario(page, 'delete', '삭제');
  await runScenario(page, 'cut', '오려두기');
  console.log('✅ #1280 v2: 겹침 최상단(글상자) 선택 → 삭제/오려두기가 글상자만 처리(이미지 잔존) 통과');
});
