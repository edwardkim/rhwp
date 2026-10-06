/**
 * E2E (Issue #6202): 어울림 그림을 드래그로 옮기면 **화면도 다시 그려져야** 한다.
 *
 * 엔진은 PR #6709 로 배제 밴드를 다시 계산해 본문을 되감는다. studio 는 이동 기록을
 * `refresh: 'none'` 으로 실행해 그 결과를 화면에 반영하지 않았다 — 문서만 바뀌고
 * 화면은 옛 그림이 남는다.
 *
 * ⚠ 판별은 **puppeteer 스크린샷**으로 한다. studio 캔버스는 CanvasKit(WebGL) 이라
 * `getContext('2d').getImageData` 로 읽으면 렌더 백엔드와 무관한 상수만 나온다
 * (실측: 드래그 전후 잉크 891509 로 완전히 동일 — 판별력 0).
 * ⚠ 합성 마우스 이벤트는 `target` 을 `ih.container` 로 덮고 `onClickBound` /
 *   `onMouseMoveBound` / `onMouseUpBound` 로 넣어야 실제 드래그 경로를 탄다.
 */
import { runTest, loadHwpFile, assert } from './helpers.mjs';

const SAMPLE = '143E433F503322BD33.hwp';

await runTest('어울림 그림 이동 뒤 화면이 다시 그려진다 (#6202)', async ({ page }) => {
  await loadHwpFile(page, SAMPLE);

  // ── 대상 선택 (드래그 전 상태 확정)
  const setup = await page.evaluate(async () => {
    const wasm = window.__wasm;
    const ih = window.__inputHandler;
    const nextFrame = () =>
      new Promise((resolve) => requestAnimationFrame(() => requestAnimationFrame(resolve)));
    const layout = wasm.getPageControlLayout(0);
    const target = (layout.controls || []).find((c) => c.wrap === 'square');
    if (!target) return { error: '어울림(square) 그림을 찾지 못함' };
    ih.cursor.enterPictureObjectSelectionDirect(0, target.paraIdx, target.controlIdx, 'image');
    ih.renderPictureObjectSelection();
    await nextFrame();
    return { target };
  });
  if (setup.error) throw new Error(setup.error);

  // 그림이 **비켜간 뒤 글자가 흘러들어와야 하는 띠**만 잘라 본다. 전체 화면을 비교하면
  // 드래그 중 미리보기로 그림만 움직여도 달라져 판별력이 없다(음성 대조 통과).
  const band = await page.evaluate((target) => {
    const ih = window.__inputHandler;
    const sc = ih.container.querySelector('#scroll-content');
    const r = sc.getBoundingClientRect();
    return {
      x: Math.round(r.left + 60),
      y: Math.round(r.top + target.y + target.h + 80),
      width: 660,
      height: 110,
    };
  }, setup.target);

  const shot = async () => Buffer.from(await page.screenshot({ clip: band, encoding: 'binary' }));
  const before = await shot();

  // ── 이동 드래그
  const moved = await page.evaluate(async (target) => {
    const wasm = window.__wasm;
    const ih = window.__inputHandler;
    const nextFrame = () =>
      new Promise((resolve) => requestAnimationFrame(() => requestAnimationFrame(resolve)));
    const me = (type, x, y) => {
      const ev = new MouseEvent(type, { button: 0, clientX: x, clientY: y, bubbles: true });
      Object.defineProperty(ev, 'target', { value: ih.container, configurable: true });
      return ev;
    };
    const sc = ih.container.querySelector('#scroll-content');
    const rect = sc.getBoundingClientRect();
    const x = rect.left + target.x + target.w / 2;
    const y = rect.top + target.y + target.h / 2;

    ih.onClickBound(me('mousedown', x, y));
    for (let dy = 12; dy <= 72; dy += 12) {
      ih.onMouseMoveBound(me('mousemove', x, y + dy));
      await nextFrame();
    }
    ih.onMouseUpBound(me('mouseup', x, y + 72));
    await nextFrame();
    await nextFrame();

    const after = (wasm.getPageControlLayout(0).controls || [])
      .find((c) => c.wrap === 'square');
    return { beforeY: target.y, afterY: after ? after.y : null };
  }, setup.target);

  const after = await shot();

  const identical = before.length === after.length && before.equals(after);
  console.log(
    `[#6202] 그림 y ${moved.beforeY} → ${moved.afterY} · 글자 띠 ${JSON.stringify(band)} · ${before.length}B / ${after.length}B · 동일=${identical}`,
  );

  assert(
    moved.afterY !== null && Math.abs(moved.afterY - moved.beforeY) > 4,
    `드래그가 문서에 반영돼야 한다 (그림 y ${moved.beforeY} → ${moved.afterY})`,
  );
  assert(
    !identical,
    '그림이 내려온 자리의 글자 띠가 화면에서 다시 흘러야 한다 — #6202 회귀 (띠 스크린샷이 바이트까지 동일 = 본문이 안 되감김)',
  );
});
