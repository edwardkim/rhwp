/**
 * E2E 테스트 (Issue #1280 v2): 삽입 글상자 = floating + 글앞으로(InFrontOfText)
 *
 * #1280 본편은 삽입 글상자를 인라인(treat_as_char=true)으로 만들었으나, v2에서 한컴 정답값
 * floating(treat_as_char=false) + InFrontOfText 로 되돌린다. 그래야 글상자 위 어울림 이미지가
 * 글상자 뒤로 가고(plane 3>2), 로드된 기존 글상자와 정합한다.
 *
 * 실제 삽입 경로(enterTextboxPlacementMode + 마우스 드래그)로 글상자를 만든 뒤,
 * getPageControlLayout 에서 그 글상자(shape)가 plane=3(InFrontOfText) / wrap=inFrontOfText 임을 확인.
 */
import { runTest, createNewDocument, assert } from './helpers.mjs';

runTest('#1280 v2: 삽입 글상자가 floating+InFrontOfText(plane 3)', async ({ page }) => {
  await createNewDocument(page);

  // 실제 삽입 경로: 배치 모드 + 마우스 드래그 → finishTextboxPlacement
  const placementType = await page.evaluate(() => {
    const ih = window.__inputHandler;
    ih.enterTextboxPlacementMode();
    return ih.shapePlacementType;
  });
  assert(placementType === 'textbox', `shapePlacementType='textbox' 기대, 실제 '${placementType}'`);

  const canvas = await page.$('#scroll-container canvas');
  const box = await canvas.boundingBox();
  if (!box) throw new Error('캔버스 boundingBox가 null입니다');
  const x1 = box.x + 150, y1 = box.y + 120, x2 = box.x + 420, y2 = box.y + 300;
  await page.mouse.move(x1, y1);
  await page.mouse.down();
  await page.mouse.move((x1 + x2) / 2, (y1 + y2) / 2);
  await page.mouse.move(x2, y2);
  await page.mouse.up();
  await page.evaluate(() => new Promise((r) => setTimeout(r, 400)));

  // 생성된 글상자 ref + 레이아웃의 plane/wrap 확인
  const result = await page.evaluate(() => {
    const ih = window.__inputHandler;
    const ref = ih.cursor.getSelectedPictureRef?.();
    if (!ref || ref.type !== 'shape') return { error: `글상자 미생성/미선택: ${JSON.stringify(ref)}` };
    for (let p = 0; p < window.__wasm.pageCount; p++) {
      let layout;
      try { layout = window.__wasm.getPageControlLayout(p); } catch { continue; }
      const c = (layout.controls || []).find(
        (x) => x.type === 'shape' && x.secIdx === ref.sec && x.paraIdx === ref.ppi && x.controlIdx === ref.ci,
      );
      if (c) return { plane: c.plane, wrap: c.wrap, ci: c.controlIdx };
    }
    return { error: '레이아웃에서 삽입 글상자를 못 찾음' };
  });

  assert(!result.error, `검증 실패: ${result.error}`);
  console.log('삽입 글상자:', JSON.stringify(result));
  // floating + 글앞으로(InFrontOfText) = plane 3, wrap 'inFrontOfText'
  assert(result.plane === 3, `삽입 글상자 plane=3(InFrontOfText) 기대, 실제 ${result.plane}`);
  assert(result.wrap === 'inFrontOfText', `삽입 글상자 wrap='inFrontOfText' 기대, 실제 '${result.wrap}'`);

  console.log('✅ #1280 v2: 삽입 글상자가 floating+InFrontOfText(plane 3)로 생성됨');
});
