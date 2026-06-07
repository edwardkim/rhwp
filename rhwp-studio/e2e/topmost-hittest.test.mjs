/**
 * E2E 테스트 (Issue #1280 v2): 겹침 클릭 = "최상단 개체" 선택
 *
 * 한컴 권위 샘플 samples/textbox-under-image.hwp 검증:
 *   글상자(사각형, 배치=글앞으로/InFrontOfText, plane 3) 위에 이미지(어울림/Square, plane 2)가
 *   겹쳐 있다. 한컴은 글상자가 plane 우위로 이미지 위에 보인다(이미지가 글상자 뒤).
 *
 * 1. getPageControlLayout 이 컨트롤별 plane/zOrder/stableIndex 노출 (Stage 1)
 *    — 글상자 shape.plane=3, 이미지 image.plane=2.
 * 2. findPictureAtClick 이 겹침 영역 클릭에서 "최상단"(plane 큰) 글상자를 반환 (Stage 3).
 *    수정 전: emit 순서 첫 적중(보통 글상자가 먼저라 우연히 맞을 수도 있으나) 규약 미보장.
 *    수정 후: (plane,zOrder,stableIndex) 최댓값 = 보이는 개체 선택(WYSIWYG).
 * 3. 겹치지 않는 이미지-단독 영역 클릭은 이미지를 반환 (단일 적중 시 회귀 0, 기하 가능 시).
 */
import { runTest, loadHwpFile, assert } from './helpers.mjs';

runTest('겹침 클릭 = 최상단 개체 선택 (#1280 v2)', async ({ page }) => {
  await loadHwpFile(page, 'textbox-under-image.hwp');

  const result = await page.evaluate(async () => {
    const wasm = window.__wasm;
    const ih = window.__inputHandler;
    const pageCount = wasm.pageCount;

    // 같은 페이지에서 겹치는 shape(글상자) + image(어울림) 한 쌍 탐색
    function overlaps(a, b) {
      return a.x < b.x + b.w && b.x < a.x + a.w && a.y < b.y + b.h && b.y < a.y + a.h;
    }
    for (let p = 0; p < pageCount; p++) {
      let layout;
      try { layout = wasm.getPageControlLayout(p); } catch { continue; }
      const controls = layout.controls || [];
      const shape = controls.find(c => c.type === 'shape');
      const image = controls.find(c => c.type === 'image');
      if (!shape || !image || !overlaps(shape, image)) continue;

      // 겹침(교집합) 중심 좌표
      const ix = Math.max(shape.x, image.x);
      const iy = Math.max(shape.y, image.y);
      const ix2 = Math.min(shape.x + shape.w, image.x + image.w);
      const iy2 = Math.min(shape.y + shape.h, image.y + image.h);
      const cx = (ix + ix2) / 2;
      const cy = (iy + iy2) / 2;
      const overlapHit = ih.findPictureAtClick(p, cx, cy);

      // 이미지-단독 영역(글상자 밖) 한 점 탐색 — 기하적으로 가능할 때만.
      let imageOnlyHit = undefined;
      const candidates = [
        [image.x + 2, cy],                       // 좌단
        [image.x + image.w - 2, cy],             // 우단
        [cx, image.y + 2],                       // 상단
        [cx, image.y + image.h - 2],             // 하단
      ];
      for (const [px, py] of candidates) {
        const inImage = px >= image.x && px <= image.x + image.w && py >= image.y && py <= image.y + image.h;
        const inShape = px >= shape.x && px <= shape.x + shape.w && py >= shape.y && py <= shape.y + shape.h;
        if (inImage && !inShape) {
          imageOnlyHit = ih.findPictureAtClick(p, px, py);
          break;
        }
      }

      return {
        page: p,
        shapePlane: shape.plane, imagePlane: image.plane,
        shapeZ: shape.zOrder, imageZ: image.zOrder,
        shapeStable: shape.stableIndex, imageStable: image.stableIndex,
        shapeRef: { sec: shape.secIdx, ppi: shape.paraIdx, ci: shape.controlIdx },
        imageRef: { sec: image.secIdx, ppi: image.paraIdx, ci: image.controlIdx },
        overlapHit: overlapHit ? { type: overlapHit.type, sec: overlapHit.sec, ppi: overlapHit.ppi, ci: overlapHit.ci } : null,
        imageOnlyHit: imageOnlyHit === undefined ? 'n/a' : (imageOnlyHit ? { type: imageOnlyHit.type, ci: imageOnlyHit.ci } : null),
      };
    }
    return { error: '겹치는 글상자(shape)+이미지(image) 쌍을 찾지 못함' };
  });

  assert(!result.error, `검증 실패: ${result.error}`);
  console.log('결과:', JSON.stringify(result, null, 2));

  // Stage 1: plane/zOrder/stableIndex 노출 (WASM 경유 end-to-end)
  assert(typeof result.shapePlane === 'number' && typeof result.imagePlane === 'number',
    `plane 미노출: shape=${result.shapePlane}, image=${result.imagePlane}`);
  assert(typeof result.shapeZ === 'number' && typeof result.shapeStable === 'number',
    `zOrder/stableIndex 미노출: z=${result.shapeZ}, stable=${result.shapeStable}`);
  assert(result.shapePlane === 3, `글상자 plane=3(InFrontOfText) 기대, 실제 ${result.shapePlane}`);
  assert(result.imagePlane === 2, `이미지 plane=2(Square) 기대, 실제 ${result.imagePlane}`);
  // 핵심 불변식: 글상자가 이미지보다 위(plane 큼) → 클릭 시 글상자 선택.
  assert(result.shapePlane > result.imagePlane, '글상자가 이미지보다 위(plane)여야 함');

  // Stage 3: 겹침 클릭 → 최상단(글상자, type=shape) 반환.
  assert(result.overlapHit, '겹침 영역 클릭에서 아무 개체도 hit 안 됨');
  assert(result.overlapHit.type === 'shape',
    `겹침 클릭이 최상단 글상자(shape) 아님: ${JSON.stringify(result.overlapHit)}`);
  assert(result.overlapHit.ci === result.shapeRef.ci && result.overlapHit.ppi === result.shapeRef.ppi,
    `겹침 클릭이 글상자 ref 와 불일치: ${JSON.stringify(result.overlapHit)} vs ${JSON.stringify(result.shapeRef)}`);

  // Stage 3 회귀: 이미지-단독 영역 클릭은 이미지 반환 (기하 가능 시).
  if (result.imageOnlyHit !== 'n/a') {
    assert(result.imageOnlyHit && result.imageOnlyHit.type === 'image',
      `이미지-단독 영역 클릭이 이미지 아님: ${JSON.stringify(result.imageOnlyHit)}`);
    console.log('  (이미지-단독 영역 클릭 → 이미지 반환 확인)');
  } else {
    console.log('  (이미지-단독 영역이 기하적으로 없어 단일-적중 회귀 검사는 생략)');
  }

  console.log('✅ #1280 v2: 겹침 클릭 = 최상단 개체(글상자) 선택 + plane 노출 통과');
});
