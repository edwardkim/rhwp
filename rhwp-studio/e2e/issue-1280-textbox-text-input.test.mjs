/**
 * E2E 회귀: #1280 — rhwp-studio가 삽입한 글상자가 text_box 없는 Rectangle로 생성되어
 * 커서 진입·타이핑·붙여넣기가 모두 실패하던 결함.
 *
 * 핵심: 반드시 `enterTextboxPlacementMode()` + 마우스 드래그(=실제 프런트 삽입 경로)로
 * 글상자를 만든다. WASM `createShapeControl`을 직접 호출하면 프런트 버그(shapePlacementType)를
 * 우회하므로 이 회귀를 잡지 못한다.
 *
 * 검증 단계:
 *   1. enterTextboxPlacementMode() 직후 shapePlacementType === 'textbox' (정확한 회귀 가드)
 *   2. 마우스 드래그 → finishTextboxPlacement → 글상자 도형 생성·선택
 *   3. 생성된 글상자가 text_box를 가짐 (getShapeText.ok === true)
 *   4. 글상자 안에 텍스트 입력·보존 (수정 전엔 "글상자 없음"으로 실패)
 *
 * 실행:
 *   cd rhwp-studio
 *   npx vite --host 0.0.0.0 --port 7700 &
 *   # Windows: CHROME_PATH 지정 후 headless
 *   CHROME_PATH="C:\Program Files\Google\Chrome\Application\chrome.exe" \
 *     node e2e/issue-1280-textbox-text-input.test.mjs --mode=headless
 */
import { runTest, createNewDocument, screenshot, assert } from './helpers.mjs';

runTest('#1280: 삽입한 글상자에 커서 진입·텍스트 입력', async ({ page }) => {
  await createNewDocument(page);

  // (1) 글상자 배치 모드 진입 직후 shapePlacementType 검증.
  //     수정 전: 'rectangle' (버그) → 이 단언에서 즉시 실패. 수정 후: 'textbox'.
  const placementType = await page.evaluate(() => {
    const ih = window.__inputHandler;
    ih.enterTextboxPlacementMode();
    return ih.shapePlacementType;
  });
  assert(
    placementType === 'textbox',
    `글상자 배치 모드의 shapePlacementType === 'textbox' (실제 '${placementType}') — #1280 핵심 회귀`,
  );

  // (2) 편집 영역 캔버스에서 마우스 드래그 → 실제 finishTextboxPlacement 경로 구동.
  const canvas = await page.$('#scroll-container canvas');
  const box = await canvas.boundingBox();
  if (!box) throw new Error('캔버스 boundingBox가 null입니다');
  const x1 = box.x + 150;
  const y1 = box.y + 120;
  const x2 = box.x + 420;
  const y2 = box.y + 300;
  await page.mouse.move(x1, y1);
  await page.mouse.down();
  await page.mouse.move((x1 + x2) / 2, (y1 + y2) / 2);
  await page.mouse.move(x2, y2);
  await page.mouse.up();
  await page.evaluate(() => new Promise((r) => setTimeout(r, 400)));
  await screenshot(page, 'issue-1280-01-textbox-created');

  // (3) 생성된 글상자 위치 확보 — finishTextboxPlacement가 선택 상태로 진입시킨다.
  const ref = await page.evaluate(() => {
    const r = window.__inputHandler.cursor.getSelectedPictureRef?.();
    return r ? { sec: r.sec, para: r.ppi, ctrl: r.ci, type: r.type } : null;
  });
  assert(
    !!ref && ref.type === 'shape',
    `드래그로 글상자 도형이 생성·선택됨 (ref=${JSON.stringify(ref)})`,
  );

  // (4) 글상자 안에 텍스트 입력 → 수정 전엔 "지정된 Shape 컨트롤에 텍스트 박스가 없습니다"로 throw.
  //     입력이 성공한다는 것 자체가 글상자에 text_box가 생성됐다는 결정적 증거다(#1280).
  //     이어서 getTextInCellByPath로 내부 문단 텍스트 보존을 확인한다(글상자=단일 셀 경로).
  const typed = await page.evaluate(
    ({ sec, para, ctrl }) => {
      try {
        window.__wasm.insertTextInCell(sec, para, ctrl, 0, 0, 0, '글상자 텍스트');
        window.__eventBus?.emit('document-changed');
        const path = JSON.stringify([{ controlIndex: ctrl, cellIndex: 0, cellParaIndex: 0 }]);
        const readBack = window.__wasm.getTextInCellByPath(sec, para, path, 0, 50);
        return { ok: true, readBack };
      } catch (e) {
        return { ok: false, error: e.message || String(e) };
      }
    },
    ref,
  );
  assert(
    typed.ok,
    `글상자에 텍스트 입력 성공 (#1280 — 수정 전엔 "텍스트 박스가 없습니다"로 실패)${typed.error ? ` err=${typed.error}` : ''}`,
  );
  assert(
    typeof typed.readBack === 'string' && typed.readBack.includes('글상자 텍스트'),
    `글상자 내부 첫 문단 텍스트 보존 (readBack="${typed.readBack}")`,
  );
  await screenshot(page, 'issue-1280-02-text-inserted');
});
