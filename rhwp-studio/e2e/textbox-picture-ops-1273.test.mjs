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
      // 선택 → 핸들 dir 로 드래그 1회. onMid 는 mousemove 후 mouseup 전(라이브 드래그 중) 실행.
      const drag = async (stateName, dirPick, mdx, mdy, onMid) => {
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
        if (onMid) info.mid = onMid();
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

      // ───────── FLOATING 리사이즈 (글자처럼취급 해제 후 축소 → 글상자 이탈 회귀, Stage4) ─────────
      {
        const PX2HWP = 7200 / 96;
        select();
        ih.setObjectProperties(cursor.getSelectedPictureRef(), { treatAsChar: false });
        await nextFrame();
        const pB = getProps();
        const bbB = ih.findPictureBbox(cursor.getSelectedPictureRef());
        // onMid: 라이브 드래그 중(mouseup 전) 이미지 위치 캡처 — 예비박스 추적 확인
        const info = await drag('pictureResizeState', (x) => x.dir === 'nw', 30, 30,
          () => { const bb = ih.findPictureBbox(cursor.getSelectedPictureRef()); return bb ? Math.round(bb.y) : null; });
        const pA = getProps();
        const bbA = ih.findPictureBbox(cursor.getSelectedPictureRef());
        out.floating = {
          handleDir: info.handleDir, stateCellPath: info.stateCellPath,
          treatAsChar: pB.treatAsChar,
          vBefore: pB.vertOffset, vAfter: pA.vertOffset,
          hBefore: pB.horzOffset, hAfter: pA.horzOffset,
          pageAbsV: bbB ? Math.round(bbB.y * PX2HWP) : null,
          bbYBefore: bbB ? Math.round(bbB.y) : null,
          bbYMid: info.mid ?? null,
          bbYAfter: bbA ? Math.round(bbA.y) : null,
        };
      }

      // ───────── ROTATE (rotate 핸들) ─────────
      {
        const a0 = getProps().rotationAngle ?? 0;
        const info = await drag('pictureRotateState', (x) => x.dir === 'rotate', 60, 30);
        out.rotate = { ...info, a0, a1: getProps().rotationAngle ?? 0 };
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

  // FLOATING 리사이즈 — Stage 4: offset 이 페이지 절대값이 아니라 델타(컨테이너 상대)로 적용
  //  (글자처럼취급 해제 후 축소 시 글상자 밖으로 이탈하지 않아야 함)
  assert(result.floating?.handleDir === 'nw', 'floating 리사이즈 nw 핸들 실패');
  assert(result.floating.treatAsChar === false, 'floating 전환 실패 (treatAsChar≠false)');
  assert(Array.isArray(result.floating.stateCellPath) && result.floating.stateCellPath.length > 0,
    `floating 리사이즈 stateCellPath 누락: ${JSON.stringify(result.floating.stateCellPath)}`);
  assert(Math.abs(result.floating.vAfter - result.floating.vBefore) < 20000,
    `floating vertOffset 페이지절대값 점프(Stage4 회귀): ${result.floating.vBefore} → ${result.floating.vAfter} ` +
    `(page-abs≈${result.floating.pageAbsV})`);
  assert(Math.abs(result.floating.bbYAfter - result.floating.bbYBefore) < 300,
    `floating picture 가 글상자 밖으로 이탈(세로 점프): bbY ${result.floating.bbYBefore} → ${result.floating.bbYAfter}`);
  // 라이브 드래그 중 이미지가 예비박스를 추적해야 함(updatePictureResizeDrag offset 도 델타 기반)
  assert(result.floating.bbYMid != null && Math.abs(result.floating.bbYMid - result.floating.bbYBefore) < 300,
    `라이브 드래그 중 이미지가 예비박스에서 이탈(어긋남): bbY ${result.floating.bbYBefore} → mid ${result.floating.bbYMid}`);

  console.log('✅ #1273 글상자 picture: 리사이즈·회전·floating리사이즈 lifecycle + cellPath 보존 + undo 통과');
});
