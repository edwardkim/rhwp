/**
 * E2E 테스트 (Issue #1273): 사각형 글상자(Shape text_box) 안 picture 의
 * 마우스 드래그 조작(리사이즈·회전·이동) lifecycle.
 *
 * #1171 의 hit-test/속성 round-trip 테스트(textbox-picture-1171)는 WASM by-path API 를
 * **직접** 호출하여, 드래그 상태(pictureResizeState 등) 구성이 cellPath 를 떨어뜨리는
 * 결함을 우회했다. 본 테스트는 그 공백을 메운다 — InputHandler 의 실제 드래그 경로
 * (onClick → mousemove → mouseup, 실제 핸들 좌표)를 구동하여:
 *   1) 드래그 상태 ref 에 cellPath 가 보존되는지 (Stage 1, 핵심 회귀 검증)
 *   2) 리사이즈가 by-path API 로 실제 반영되고 undo 로 원복되는지 (+ 콘솔 오류 0건)
 *   3) 회전이 by-path 로 반영되는지
 *
 * 대상: samples/tac-img-02.hwp 섹션0 문단25 글상자 안 picture (cellPath sentinel, 페이지5).
 * 대상 picture 는 treat_as_char=true(글상자 내 인라인)이므로 이동 드래그는 N/A —
 * 이동 by-path(Stage 2)는 리사이즈 undo(동일 setCell*PropertiesByPath 경로)로 간접 커버.
 */
import { runTest, loadHwpFile, assert } from './helpers.mjs';

runTest('글상자 안 picture 마우스 드래그 조작 lifecycle (#1273)', async ({ page }) => {
  await loadHwpFile(page, 'tac-img-02.hwp');

  const result = await page.evaluate(async () => {
    const wasm = window.__wasm;
    const ih = window.__inputHandler;
    const cursor = ih.cursor;

    // 조작 중 '실패/범위 초과/그림이 아님' 콘솔 오류 감지
    const warnings = [];
    const origWarn = console.warn;
    console.warn = (...a) => { warnings.push(a.map(String).join(' ')); origWarn.apply(console, a); };

    const nextFrame = () => new Promise((r) => requestAnimationFrame(() => requestAnimationFrame(r)));
    const out = { warnings };

    try {
      // 1) 글상자 picture(paraIdx=25, cellPath) 탐색
      let found = null;
      for (let p = 0; p < wasm.pageCount; p++) {
        let layout; try { layout = wasm.getPageControlLayout(p); } catch { continue; }
        for (const c of layout.controls || []) {
          if (c.type === 'image' && c.paraIdx === 25 && c.controlIdx === 0 && c.cellPath) { found = c; break; }
        }
        if (found) break;
      }
      if (!found) { out.error = 'paraIdx=25 글상자 picture 를 찾지 못함'; return out; }
      const cellPath = found.cellPath;
      out.cellPath = cellPath;

      const getProps = () => wasm.getCellPicturePropertiesByPath(0, 25, cellPath, 0);
      const sc = ih.container.querySelector('#scroll-content');
      const select = () => {
        cursor.enterPictureObjectSelectionDirect(0, 25, 0, 'image', undefined, undefined, undefined, undefined, cellPath);
        ih.renderPictureObjectSelection();
      };
      // target 명시: 직접 핸들러 호출 시 e.target.closest 가드 통과용 (container 는 툴바 밖)
      const me = (type, x, y) => {
        const ev = new MouseEvent(type, { button: 0, clientX: x, clientY: y, bubbles: true });
        Object.defineProperty(ev, 'target', { value: ih.container, configurable: true });
        return ev;
      };
      // 핸들(content 좌표)을 뷰포트 안으로 스크롤 — onClick 의 스크롤바영역 가드 통과
      const ensureVisible = async (contentY) => {
        ih.container.scrollTop = Math.max(0, contentY - ih.container.clientHeight / 2);
        await nextFrame();
        select(); // 스크롤 후 핸들 재배치
        await nextFrame();
      };
      // 선택 → 핸들 dir 로 드래그 1회. 반환: { stateCellPath, dragging }
      const drag = async (stateName, dirPick, mdx, mdy) => {
        select();
        let h = (ih.pictureObjectRenderer.handles || []).find(dirPick);
        if (!h) return { handleDir: null };
        await ensureVisible(h.cy);
        h = (ih.pictureObjectRenderer.handles || []).find(dirPick);
        if (!h) return { handleDir: null };
        const r = sc.getBoundingClientRect();
        const dx = r.left + h.cx, dy = r.top + h.cy;
        ih.onClickBound(me('mousedown', dx, dy));
        const st = ih[stateName];
        const info = { handleDir: h.dir, stateCellPath: st?.ref?.cellPath ?? null, dragging: !!st };
        ih.onMouseMoveBound(me('mousemove', dx + mdx, dy + mdy));
        await nextFrame();
        ih.onMouseUpBound(me('mouseup', dx + mdx, dy + mdy));
        return info;
      };

      // ───────── RESIZE (se 핸들 +40,+40) + undo ─────────
      {
        const w0 = getProps().width;
        // se(우하단) 핸들 + (+40,+40) → 가로/세로 확대 (방향 결정적)
        const info = await drag('pictureResizeState', (x) => x.dir === 'se', 40, 40);
        out.resize = { ...info, w0, w1: getProps().width };
        ih.handleUndo();
        out.resize.wUndo = getProps().width;
      }

      // ───────── ROTATE (rotate 핸들) ─────────
      {
        const a0 = getProps().rotationAngle ?? 0;
        const info = await drag('pictureRotateState', (x) => x.dir === 'rotate', 60, 30);
        out.rotate = { ...info, a0, a1: getProps().rotationAngle ?? 0 };
      }

      // ───────── MOVE (treat_as_char 이면 N/A) ─────────
      {
        const p0 = getProps();
        out.move = { treatAsChar: !!p0.treatAsChar };
      }
    } finally {
      console.warn = origWarn;
    }
    return out;
  });

  assert(!result.error, `검증 실패: ${result.error}`);
  console.log('결과:', JSON.stringify(result, null, 2));

  // 공통: 조작 중 오류 경고 0건
  const fails = result.warnings.filter((w) => /실패|범위 초과|그림이 아닙니다/.test(w));
  assert(fails.length === 0, `조작 중 오류 경고 발생: ${JSON.stringify(fails)}`);

  // RESIZE — Stage 1 핵심 회귀: 드래그 상태 ref 에 cellPath 보존 + by-path 반영 + undo 원복
  assert(result.resize?.handleDir, '리사이즈 핸들을 찾지 못함 (선택/렌더/스크롤 실패)');
  assert(Array.isArray(result.resize.stateCellPath) && result.resize.stateCellPath.length > 0,
    `pictureResizeState.ref.cellPath 누락 — Stage1 회귀: ${JSON.stringify(result.resize.stateCellPath)}`);
  assert(result.resize.w1 > result.resize.w0,
    `리사이즈 미반영: ${result.resize.w0} → ${result.resize.w1}`);
  assert(result.resize.wUndo === result.resize.w0,
    `리사이즈 undo 원복 실패: ${result.resize.w0} → ${result.resize.w1} → ${result.resize.wUndo}`);

  // ROTATE — Stage 1: 회전 드래그 상태 ref 에 cellPath 보존 + 각도 반영
  assert(result.rotate?.handleDir === 'rotate', '회전 핸들을 찾지 못함');
  assert(Array.isArray(result.rotate.stateCellPath) && result.rotate.stateCellPath.length > 0,
    `pictureRotateState.ref.cellPath 누락 — Stage1 회귀: ${JSON.stringify(result.rotate.stateCellPath)}`);
  assert(result.rotate.a1 !== result.rotate.a0,
    `회전 각도 미반영: ${result.rotate.a0} → ${result.rotate.a1}`);

  console.log(`ℹ️ MOVE: treat_as_char=${result.move.treatAsChar} (인라인 picture → 이동 드래그 N/A; ` +
    `by-path 이동은 리사이즈 undo 의 setCellPicturePropertiesByPath 로 간접 커버)`);
  console.log('✅ #1273 글상자 picture: 리사이즈·회전 lifecycle + cellPath 보존 + undo 통과');
});
