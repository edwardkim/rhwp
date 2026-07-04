/**
 * E2E 테스트 (Issue #1282): 회전된 표 셀 내부 picture 리사이즈.
 *
 * Rust by-path API 직접 호출만으로는 rhwp-studio 의 실제 드래그 상태(cellPath 보존,
 * 회전 bbox 기준 리사이즈, Undo 기록)까지 검증하지 못한다. 이 테스트는
 * samples/ta-pic-001-r.hwp 의 표 셀 내부 회전 그림을 선택하고, 실제 InputHandler
 * 마우스 드래그 경로를 호출해 셀 높이와 표시 bbox가 안정적으로 갱신되는지 확인한다.
 */
import { runTest, loadHwpFile, assert } from './helpers.mjs';

await runTest('회전 표 셀 picture 리사이즈 드래그 안정성 (#1282)', async ({ page }) => {
  await loadHwpFile(page, 'ta-pic-001-r.hwp');

  const result = await page.evaluate(async () => {
    const wasm = window.__wasm;
    const ih = window.__inputHandler;
    const cursor = ih.cursor;
    const nextFrame = () => new Promise((resolve) => requestAnimationFrame(() => requestAnimationFrame(resolve)));
    const PX2HWP = 7200 / 96;

    const findTarget = () => {
      for (let pageIndex = 0; pageIndex < wasm.pageCount; pageIndex += 1) {
        const layout = wasm.getPageControlLayout(pageIndex);
        for (const ctrl of layout.controls || []) {
          const path = ctrl.cellPath;
          const first = Array.isArray(path) ? path[0] : null;
          if (
            ctrl.type === 'image'
            && ctrl.paraIdx === 0
            && ctrl.controlIdx === 0
            && first
            && (first.controlIndex ?? first.controlIdx) === 2
            && (first.cellIndex ?? first.cellIdx) === 2
            && (first.cellParaIndex ?? first.cellParaIdx) === 0
          ) {
            return { pageIndex, ctrl };
          }
        }
      }
      return null;
    };

    const target = findTarget();
    if (!target) return { error: 'ta-pic-001-r.hwp 대상 회전 셀 picture를 찾지 못함' };

    const cellPath = target.ctrl.cellPath;
    const getProps = () => wasm.getCellPicturePropertiesByPath(0, 0, cellPath, 0);
    const getCellProps = () => wasm.getCellProperties(0, 0, 2, 2);
    const getBbox = () => {
      const found = findTarget();
      return found?.ctrl ?? null;
    };
    const getOwnerCellBbox = () => {
      const found = findTarget();
      if (!found) return null;
      const cells = wasm.getTableCellBboxes(0, 0, 2, found.pageIndex);
      return (cells || []).find((cell) => cell.cellIdx === 2) ?? null;
    };
    const centerOf = (bbox) => bbox ? { x: bbox.x + bbox.w / 2, y: bbox.y + bbox.h / 2 } : null;
    const ratioOf = (props) => props?.height ? props.width / props.height : null;
    const signed32 = (value) => {
      const n = Number(value ?? 0);
      return n > 0x7fffffff ? n - 0x100000000 : n;
    };
    const requiredCellHeight = (cell, pic) =>
      Math.max(0, signed32(pic.vertOffset))
      + pic.height
      + cell.paddingTop
      + cell.paddingBottom;
    const mmToHwp = (mm) => Math.round(mm * 7200 / 25.4);
    const sampleRenderedRightLeak = (cellBbox, pictureBbox) => {
      if (!cellBbox || !pictureBbox) return { error: 'cell/picture bbox 없음' };
      const pageInfo = wasm.getPageInfo(0);
      const pageCanvas = Array.from(document.querySelectorAll('#scroll-container canvas'))
        .find((canvas) => !canvas.dataset.rhwpOverlayPage
          && !canvas.dataset.rhwpGridPage
          && canvas.width > 0
          && canvas.height > 0);
      if (!pageCanvas) return { error: 'page canvas 없음' };
      const ctx = pageCanvas.getContext('2d');
      if (!ctx) return { error: '2d context 없음' };
      const scale = pageCanvas.width / pageInfo.width;
      const clamp = (value, min, max) => Math.min(max, Math.max(min, value));
      const x0 = clamp(Math.floor((cellBbox.x + cellBbox.w + 1.5) * scale), 0, pageCanvas.width - 1);
      const x1 = clamp(Math.ceil((cellBbox.x + cellBbox.w + 8.0) * scale), x0 + 1, pageCanvas.width);
      const yStartPage = Math.max(cellBbox.y + 8, pictureBbox.y + pictureBbox.h * 0.58);
      const yEndPage = Math.min(cellBbox.y + cellBbox.h - 8, pictureBbox.y + pictureBbox.h - 8);
      const y0 = clamp(Math.floor(yStartPage * scale), 0, pageCanvas.height - 1);
      const y1 = clamp(Math.ceil(yEndPage * scale), y0 + 1, pageCanvas.height);
      const width = x1 - x0;
      const height = y1 - y0;
      if (width <= 0 || height <= 0) {
        return { error: `샘플 영역 없음: ${JSON.stringify({ x0, x1, y0, y1 })}` };
      }
      let data;
      try {
        data = ctx.getImageData(x0, y0, width, height).data;
      } catch (e) {
        return { error: e?.message || String(e) };
      }
      let nonWhite = 0;
      let dark = 0;
      for (let i = 0; i < data.length; i += 4) {
        const r = data[i];
        const g = data[i + 1];
        const b = data[i + 2];
        const a = data[i + 3];
        if (a > 12 && (r < 245 || g < 245 || b < 245)) {
          nonWhite += 1;
          if (r < 170 || g < 170 || b < 170) dark += 1;
        }
      }
      const total = width * height;
      return {
        rect: {
          x: x0 / scale,
          y: y0 / scale,
          w: width / scale,
          h: height / scale,
        },
        nonWhite,
        dark,
        total,
        nonWhiteRatio: total > 0 ? nonWhite / total : 1,
        scale,
      };
    };

    const select = () => {
      cursor.enterPictureObjectSelectionDirect(
        0,
        0,
        0,
        'image',
        target.ctrl.cellIdx,
        target.ctrl.cellParaIdx,
        undefined,
        target.ctrl.outerTableControlIdx,
        cellPath,
      );
      ih.renderPictureObjectSelection();
    };

    const me = (type, x, y) => {
      const ev = new MouseEvent(type, { button: 0, clientX: x, clientY: y, bubbles: true });
      Object.defineProperty(ev, 'target', { value: ih.container, configurable: true });
      return ev;
    };

    const ensureHandleVisible = async (contentY) => {
      ih.container.scrollTop = Math.max(0, contentY - ih.container.clientHeight / 2);
      await nextFrame();
      select();
      await nextFrame();
    };

    const dragResize = async () => {
      select();
      let handle = (ih.pictureObjectRenderer.handles || []).find((h) => h.dir === 'se');
      if (!handle) return { error: 'se 리사이즈 핸들을 찾지 못함' };
      await ensureHandleVisible(handle.cy);
      handle = (ih.pictureObjectRenderer.handles || []).find((h) => h.dir === 'se');
      if (!handle) return { error: '스크롤 후 se 리사이즈 핸들을 찾지 못함' };

      const sc = ih.container.querySelector('#scroll-content');
      const rect = sc.getBoundingClientRect();
      const x = rect.left + handle.cx;
      const y = rect.top + handle.cy;
      ih.onClickBound(me('mousedown', x, y));
      const stateCellPath = ih.pictureResizeState?.ref?.cellPath ?? null;
      ih.onMouseMoveBound(me('mousemove', x + 52, y + 44));
      await nextFrame();
      const midBbox = getBbox();
      ih.onMouseUpBound(me('mouseup', x + 52, y + 44));
      await nextFrame();
      return { stateCellPath, midBbox };
    };

    const beforeProps = getProps();
    const beforeCell = getCellProps();
    const beforeBbox = getBbox();
    const beforeCellBbox = getOwnerCellBbox();
    const beforeCenter = centerOf(beforeBbox);

    const drag = await dragResize();
    if (drag.error) return { error: drag.error };

    const afterProps = getProps();
    const afterCell = getCellProps();
    const afterBbox = getBbox();
    const afterCellBbox = getOwnerCellBbox();
    const afterCenter = centerOf(afterBbox);
    const midCenter = centerOf(drag.midBbox);
    const rotationInputValue = await new Promise((resolve) => {
      ih.dispatcher?.dispatch?.('insert:picture-props');
      requestAnimationFrame(() => {
        const rows = Array.from(document.querySelectorAll('.pp-dialog .dialog-row'));
        const rotationRow = rows.find((row) => row.textContent?.includes('회전각'));
        const rotation = rotationRow?.querySelector('input[type="number"]');
        const value = rotation?.value ?? null;
        document.querySelector('.pp-dialog .dialog-close')?.click();
        resolve(value);
      });
    });

    ih.handleUndo();
    await nextFrame();
    const undoProps = getProps();
    const undoCell = getCellProps();

    wasm.setCellPicturePropertiesByPath(0, 0, cellPath, 0, {
      width: Math.round(beforeProps.width * 3),
      height: Math.round(beforeProps.height * 3),
      rotationAngle: beforeProps.rotationAngle,
    });
    window.__canvasView?.loadDocument?.();
    await nextFrame();
    select();
    await nextFrame();
    const oversizedProps = getProps();
    const oversizedCell = getCellProps();
    const oversizedBbox = getBbox();
    const oversizedCellBbox = getOwnerCellBbox();

    wasm.setCellPicturePropertiesByPath(0, 0, cellPath, 0, {
      width: afterProps.width,
      height: afterProps.height,
      rotationAngle: afterProps.rotationAngle,
    });
    window.__canvasView?.loadDocument?.();
    await nextFrame();
    select();
    await nextFrame();
    const directGrownCell = getCellProps();
    const directGrownBbox = getBbox();
    const directGrownCellBbox = getOwnerCellBbox();
    const directGrownCenter = centerOf(directGrownBbox);

    wasm.setCellPicturePropertiesByPath(0, 0, cellPath, 0, {
      rotationAngle: 0,
    });
    window.__canvasView?.loadDocument?.();
    await nextFrame();
    select();
    await nextFrame();
    const rotationOnlyProps = getProps();
    const rotationOnlyCell = getCellProps();
    const rotationOnlyBbox = getBbox();
    const rotationOnlyCellBbox = getOwnerCellBbox();
    const rotationOnlyCenter = centerOf(rotationOnlyBbox);

    const shrinkHeight = Math.max(200, Math.round(beforeProps.height * 0.66));
    wasm.setCellPicturePropertiesByPath(0, 0, cellPath, 0, {
      width: beforeProps.width,
      height: shrinkHeight,
      rotationAngle: 0,
    });
    window.__canvasView?.loadDocument?.();
    await nextFrame();
    select();
    await nextFrame();
    const shrinkProps = getProps();
    const shrinkCell = getCellProps();
    const shrinkBbox = getBbox();
    const shrinkCellBbox = getOwnerCellBbox();

    const hancomExpected = {
      width: mmToHwp(97.45),
      height: mmToHwp(115.07),
      vertOffset: mmToHwp(36.82),
      horzOffset: 0,
      rotationAngle: 0,
      textWrap: 'TopAndBottom',
    };
    wasm.setCellPicturePropertiesByPath(0, 0, cellPath, 0, hancomExpected);
    window.__canvasView?.loadDocument?.();
    await nextFrame();
    const hancomProps = getProps();
    const hancomCell = getCellProps();
    const hancomBbox = getBbox();
    const hancomCellBbox = getOwnerCellBbox();
    const hancomRightLeak = sampleRenderedRightLeak(hancomCellBbox, hancomBbox);
    const negativeMove = -mmToHwp(20);
    const clampMaxHorz = Math.max(
      0,
      hancomCell.width - hancomCell.paddingLeft - hancomCell.paddingRight - hancomProps.width,
    );

    wasm.setCellPicturePropertiesByPath(0, 0, cellPath, 0, {
      horzOffset: negativeMove,
      vertOffset: negativeMove,
    });
    window.__canvasView?.loadDocument?.();
    await nextFrame();
    const clampStartProps = getProps();

    wasm.setCellPicturePropertiesByPath(0, 0, cellPath, 0, {
      horzOffset: hancomCell.width * 2,
    });
    window.__canvasView?.loadDocument?.();
    await nextFrame();
    const clampEndProps = getProps();

    select();
    if (typeof ih.moveSelectedPicture !== 'function') {
      return { error: 'InputHandler.moveSelectedPicture 함수를 찾지 못함' };
    }
    ih.moveSelectedPicture('ArrowLeft');
    window.__canvasView?.loadDocument?.();
    await nextFrame();
    const keyboardClampProps = getProps();

    wasm.setCellPicturePropertiesByPath(0, 0, cellPath, 0, {
      ...hancomExpected,
      restrictInPage: false,
    });
    window.__canvasView?.loadDocument?.();
    await nextFrame();
    const unrestrictedToggleProps = getProps();
    const unrestrictedToggleCell = getCellProps();
    const unrestrictedToggleBbox = getBbox();
    const unrestrictedToggleCellBbox = getOwnerCellBbox();

    return {
      stateCellPath: drag.stateCellPath,
      beforeProps,
      afterProps,
      undoProps,
      oversizedProps,
      rotationOnlyProps,
      shrinkProps,
      hancomExpected,
      hancomProps,
      clampStartProps,
      clampEndProps,
      keyboardClampProps,
      clampMaxHorz,
      unrestrictedToggleProps,
      unrestrictedToggleCell,
      unrestrictedToggleBbox,
      unrestrictedToggleCellBbox,
      beforeCell,
      afterCell,
      undoCell,
      oversizedCell,
      directGrownCell,
      rotationOnlyCell,
      shrinkCell,
      hancomCell,
      requiredAfter: requiredCellHeight(afterCell, afterProps),
      requiredUndo: requiredCellHeight(undoCell, undoProps),
      requiredOversized: requiredCellHeight(oversizedCell, oversizedProps),
      requiredRotationOnly: requiredCellHeight(rotationOnlyCell, rotationOnlyProps),
      requiredShrink: requiredCellHeight(shrinkCell, shrinkProps),
      requiredHancom: requiredCellHeight(hancomCell, hancomProps),
      rotationInputValue,
      beforeRatio: ratioOf(beforeProps),
      afterRatio: ratioOf(afterProps),
      beforeBbox,
      beforeCellBbox,
      afterBbox,
      afterCellBbox,
      oversizedBbox,
      oversizedCellBbox,
      directGrownBbox,
      directGrownCellBbox,
      rotationOnlyBbox,
      rotationOnlyCellBbox,
      hancomBbox,
      hancomCellBbox,
      hancomRightLeak,
      beforeCenter,
      midCenter,
      afterCenter,
      directGrownCenter,
      rotationOnlyCenter,
      shrinkBbox,
      shrinkCellBbox,
      centerJumpAfter: beforeCenter && afterCenter
        ? Math.hypot(afterCenter.x - beforeCenter.x, afterCenter.y - beforeCenter.y)
        : null,
      centerJumpMid: beforeCenter && midCenter
        ? Math.hypot(midCenter.x - beforeCenter.x, midCenter.y - beforeCenter.y)
        : null,
      centerJumpRotationOnly: directGrownCenter && rotationOnlyCenter
        ? Math.hypot(rotationOnlyCenter.x - directGrownCenter.x, rotationOnlyCenter.y - directGrownCenter.y)
        : null,
      pxToHwp: PX2HWP,
    };
  });

  assert(!result.error, `검증 실패: ${result.error}`);
  console.log('결과:', JSON.stringify(result, null, 2));

  assert(Array.isArray(result.stateCellPath) && result.stateCellPath.length === 1,
    `드래그 상태 cellPath 보존 실패: ${JSON.stringify(result.stateCellPath)}`);
  assert(result.afterProps.height > result.beforeProps.height,
    `picture height 증가 실패: ${result.beforeProps.height} → ${result.afterProps.height}`);
  assert(result.afterProps.rotationAngle === result.beforeProps.rotationAngle,
    `리사이즈 후 회전각 보존 실패: ${result.beforeProps.rotationAngle} → ${result.afterProps.rotationAngle}`);
  assert(String(result.rotationInputValue) === String(result.afterProps.rotationAngle),
    `속성창 회전각 표시 불일치: dialog=${result.rotationInputValue}, props=${result.afterProps.rotationAngle}`);
  assert(result.beforeRatio != null && result.afterRatio != null && Math.abs(result.afterRatio - result.beforeRatio) < 0.02,
    `회전 picture 코너 리사이즈 비율 보존 실패: ${result.beforeRatio} → ${result.afterRatio}`);
  assert(result.afterCell.height > result.beforeCell.height,
    `owner cell height 증가 실패: ${result.beforeCell.height} → ${result.afterCell.height}`);
  assert(result.afterCell.height >= result.requiredAfter,
    `owner cell height 부족: cell=${result.afterCell.height}, required=${result.requiredAfter}`);
  assert(result.afterBbox && result.afterBbox.h > result.beforeBbox.h,
    `표시 bbox 높이 증가 실패: ${result.beforeBbox?.h} → ${result.afterBbox?.h}`);
  assert(result.centerJumpMid != null && result.centerJumpMid < 60,
    `라이브 드래그 중 bbox 중심 과도 이동: ${result.centerJumpMid}`);
  assert(result.centerJumpAfter != null && result.centerJumpAfter < 60,
    `확정 후 bbox 중심 과도 이동: ${result.centerJumpAfter}`);
  assert(result.undoProps.height === result.beforeProps.height && result.undoProps.width === result.beforeProps.width,
    `Undo picture size 복구 실패: before=${result.beforeProps.width}x${result.beforeProps.height}, undo=${result.undoProps.width}x${result.undoProps.height}`);
  assert(result.undoCell.height >= result.requiredUndo,
    `Undo 후 owner cell height 부족: cell=${result.undoCell.height}, required=${result.requiredUndo}`);
  assert(result.oversizedCell.height >= result.requiredOversized,
    `과대 리사이즈 후 owner cell height 부족: cell=${result.oversizedCell.height}, required=${result.requiredOversized}`);
  assert(result.oversizedProps.width > result.beforeCell.width,
    `과대 리사이즈 picture width가 기존 셀 폭보다 커지지 않음: pic=${result.oversizedProps.width}, beforeCell=${result.beforeCell.width}`);
  assert(result.oversizedCell.width === result.beforeCell.width,
    `과대 리사이즈 후 owner cell width가 바뀜: before=${result.beforeCell.width}, current=${result.oversizedCell.width}`);
  assert(result.oversizedCellBbox && result.beforeCellBbox && Math.abs(result.oversizedCellBbox.w - result.beforeCellBbox.w) < 1.5,
    `과대 리사이즈 후 owner cell 화면 폭이 바뀜: before=${JSON.stringify(result.beforeCellBbox)}, current=${JSON.stringify(result.oversizedCellBbox)}`);
  assert(result.directGrownCell.height > result.undoCell.height,
    `직접 재확대 owner cell height 증가 실패: undo=${result.undoCell.height}, grown=${result.directGrownCell.height}`);
  assert(result.rotationOnlyProps.rotationAngle === 0,
    `회전각 단독 변경 반영 실패: rotationAngle=${result.rotationOnlyProps.rotationAngle}`);
  assert(result.rotationOnlyProps.width > 0 && result.rotationOnlyProps.height > 0,
    `회전각 0도 단독 변경 후 picture size 비정상: ${result.rotationOnlyProps.width}x${result.rotationOnlyProps.height}`);
  assert(result.centerJumpRotationOnly != null && result.centerJumpRotationOnly < 60,
    `회전각 0도 단독 변경 후 bbox 중심 과도 이동: ${result.centerJumpRotationOnly}`);
  assert(result.rotationOnlyCell.height < result.directGrownCell.height,
    `회전각 0도 단독 변경 후 owner cell height 감소 실패: grown=${result.directGrownCell.height}, rotationOnly=${result.rotationOnlyCell.height}`);
  assert(result.rotationOnlyCell.height >= result.requiredRotationOnly,
    `회전각 0도 단독 변경 후 owner cell height 부족: cell=${result.rotationOnlyCell.height}, required=${result.requiredRotationOnly}`);
  assert(result.shrinkProps.rotationAngle === 0,
    `축소/회전 0도 반영 실패: rotationAngle=${result.shrinkProps.rotationAngle}`);
  assert(result.shrinkCell.height < result.directGrownCell.height,
    `축소/회전 0도 후 owner cell height 감소 실패: grown=${result.directGrownCell.height}, shrink=${result.shrinkCell.height}`);
  assert(result.shrinkCell.height >= result.requiredShrink,
    `축소/회전 0도 후 owner cell height 부족: cell=${result.shrinkCell.height}, required=${result.requiredShrink}`);
  assert(result.shrinkBbox && result.shrinkBbox.h < result.afterBbox.h,
    `축소/회전 0도 후 표시 bbox 높이 감소 실패: grown=${result.afterBbox?.h}, shrink=${result.shrinkBbox?.h}`);
  assert(Math.abs(result.hancomProps.width - result.hancomExpected.width) <= 1
    && Math.abs(result.hancomProps.height - result.hancomExpected.height) <= 1
    && Math.abs(result.hancomProps.vertOffset - result.hancomExpected.vertOffset) <= 1
    && result.hancomProps.rotationAngle === 0,
    `한컴 비교 속성값 반영 실패: expected=${JSON.stringify(result.hancomExpected)}, current=${JSON.stringify(result.hancomProps)}`);
  assert(result.hancomCell.width === result.beforeCell.width,
    `한컴 비교 상태에서 owner cell width가 바뀜: before=${result.beforeCell.width}, current=${result.hancomCell.width}`);
  assert(result.hancomCell.height >= result.requiredHancom,
    `한컴 비교 상태에서 owner cell height 부족: cell=${result.hancomCell.height}, required=${result.requiredHancom}`);
  assert(result.hancomCellBbox && result.beforeCellBbox && Math.abs(result.hancomCellBbox.w - result.beforeCellBbox.w) < 1.5,
    `한컴 비교 상태에서 owner cell 화면 폭이 바뀜: before=${JSON.stringify(result.beforeCellBbox)}, current=${JSON.stringify(result.hancomCellBbox)}`);
  assert(result.hancomRightLeak && !result.hancomRightLeak.error,
    `한컴 비교 상태 렌더 픽셀 샘플 실패: ${JSON.stringify(result.hancomRightLeak)}`);
  assert(result.hancomRightLeak.nonWhiteRatio < 0.002 && result.hancomRightLeak.dark <= 2,
    `한컴 비교 상태에서 오른쪽 셀로 렌더 픽셀이 침범: ${JSON.stringify(result.hancomRightLeak)}`);
  assert(result.clampStartProps && result.clampStartProps.horzOffset === 0,
    `쪽 영역 제한 on 그림의 왼쪽 이동 클램프 실패: ${JSON.stringify(result.clampStartProps)}`);
  assert(result.clampStartProps && result.clampStartProps.vertOffset === 0,
    `쪽 영역 제한 on 그림의 위쪽 이동 클램프 실패: ${JSON.stringify(result.clampStartProps)}`);
  assert(result.clampEndProps && result.clampEndProps.horzOffset === result.clampMaxHorz,
    `쪽 영역 제한 on 그림의 오른쪽 이동 클램프 실패: max=${result.clampMaxHorz}, props=${JSON.stringify(result.clampEndProps)}`);
  assert(result.keyboardClampProps && result.keyboardClampProps.horzOffset === result.clampMaxHorz,
    `쪽 영역 제한 on 그림의 방향키 이동 클램프 실패: max=${result.clampMaxHorz}, props=${JSON.stringify(result.keyboardClampProps)}`);
  assert(result.unrestrictedToggleProps && result.unrestrictedToggleProps.restrictInPage === false,
    `쪽 영역 제한 off 토글 반영 실패: ${JSON.stringify(result.unrestrictedToggleProps)}`);
  assert(result.unrestrictedToggleBbox && result.unrestrictedToggleCellBbox
    && result.unrestrictedToggleBbox.y < result.unrestrictedToggleCellBbox.y,
    `쪽 영역 제한 off 토글 후 그림이 표 셀 흐름에서 분리되지 않음: picture=${JSON.stringify(result.unrestrictedToggleBbox)}, cell=${JSON.stringify(result.unrestrictedToggleCellBbox)}`);
  assert(result.unrestrictedToggleBbox && result.unrestrictedToggleCellBbox
    && result.unrestrictedToggleBbox.y + result.unrestrictedToggleBbox.h <= result.unrestrictedToggleCellBbox.y + 18,
    `쪽 영역 제한 off 토글 후 그림이 소유 셀 내부에 남아 있음: picture=${JSON.stringify(result.unrestrictedToggleBbox)}, cell=${JSON.stringify(result.unrestrictedToggleCellBbox)}`);

  console.log('✅ #1282 회전 표 셀 picture: 드래그 리사이즈/셀폭 유지/렌더 클립 안정성 통과');
});

await runTest('쪽 영역 제한 off 전환은 no 샘플 렌더와 일치 (#1282)', async ({ page }) => {
  const nextFrame = () => page.evaluate(
    () => new Promise((resolve) => requestAnimationFrame(() => requestAnimationFrame(resolve))),
  );
  const captureTarget = async () => page.evaluate(() => {
    const wasm = window.__wasm;
    let target = null;
    for (let pageIndex = 0; pageIndex < wasm.pageCount; pageIndex += 1) {
      const layout = wasm.getPageControlLayout(pageIndex);
      for (const ctrl of layout.controls || []) {
        const first = Array.isArray(ctrl.cellPath) ? ctrl.cellPath[0] : null;
        if (
          ctrl.type === 'image'
          && first
          && (first.controlIndex ?? first.controlIdx) === 2
          && (first.cellIndex ?? first.cellIdx) === 2
          && (first.cellParaIndex ?? first.cellParaIdx) === 0
        ) {
          target = { pageIndex, ctrl };
          break;
        }
      }
      if (target) break;
    }
    if (!target) return { error: 'target picture not found' };
    const layout = wasm.getPageControlLayout(target.pageIndex);
    const table = layout.controls.find((ctrl) => ctrl.type === 'table');
    const cells = wasm.getTableCellBboxes(0, 0, 2, target.pageIndex);
    const ownerCell = (cells || []).find((cell) => cell.cellIdx === target.ctrl.cellIdx) ?? null;
    return {
      pageIndex: target.pageIndex,
      picture: target.ctrl,
      table,
      ownerCell,
      props: wasm.getCellPicturePropertiesByPath(0, 0, target.ctrl.cellPath, 0),
      cell: wasm.getCellProperties(0, 0, 2, 2),
    };
  });
  const nearlySame = (a, b, tolerance = 0.25) => Math.abs(a - b) <= tolerance;

  await loadHwpFile(page, 'ta-pic-001-r-쪽영역안제한.hwp');
  await page.evaluate(() => {
    const wasm = window.__wasm;
    const layout = wasm.getPageControlLayout(0);
    const target = layout.controls.find((ctrl) => (
      ctrl.type === 'image'
      && ctrl.cellPath?.[0]?.controlIndex === 2
      && ctrl.cellPath?.[0]?.cellIndex === 2
    ));
    if (!target) throw new Error('target picture not found');
    wasm.setCellPicturePropertiesByPath(0, 0, target.cellPath, 0, { restrictInPage: false });
    window.__canvasView?.loadDocument?.();
  });
  await nextFrame();
  const toggled = await captureTarget();

  await loadHwpFile(page, 'ta-pic-001-r-쪽영역안제한no.hwp');
  const oracle = await captureTarget();

  assert(!toggled.error && !oracle.error,
    `쪽 영역 제한 off 비교 대상 조회 실패: toggled=${JSON.stringify(toggled)}, oracle=${JSON.stringify(oracle)}`);
  assert(toggled.props.restrictInPage === false,
    `쪽 영역 제한 off 토글 반영 실패: ${JSON.stringify(toggled.props)}`);
  assert(toggled.cell.height === oracle.cell.height,
    `쪽 영역 제한 off 토글 후 셀 높이가 no 샘플과 다름: toggled=${toggled.cell.height}, oracle=${oracle.cell.height}`);
  assert(nearlySame(toggled.picture.y, oracle.picture.y)
    && nearlySame(toggled.picture.h, oracle.picture.h)
    && nearlySame(toggled.table.y, oracle.table.y)
    && nearlySame(toggled.table.h, oracle.table.h)
    && nearlySame(toggled.ownerCell.y, oracle.ownerCell.y),
    `쪽 영역 제한 off 토글 렌더가 no 샘플과 다름: toggled=${JSON.stringify(toggled)}, oracle=${JSON.stringify(oracle)}`);

  await loadHwpFile(page, 'ta-pic-001-r-쪽영역안제한.hwp');
  await page.evaluate(() => {
    const wasm = window.__wasm;
    const layout = wasm.getPageControlLayout(0);
    const target = layout.controls.find((ctrl) => (
      ctrl.type === 'image'
      && ctrl.cellPath?.[0]?.controlIndex === 2
      && ctrl.cellPath?.[0]?.cellIndex === 2
    ));
    if (!target) throw new Error('target picture not found');
    wasm.setCellPicturePropertiesByPath(0, 0, target.cellPath, 0, { restrictInPage: false });
    wasm.setCellPicturePropertiesByPath(0, 0, target.cellPath, 0, {
      horzOffset: 2030,
      vertOffset: -5890,
    });
    window.__canvasView?.loadDocument?.();
  });
  await nextFrame();
  const moved = await captureTarget();
  assert(!moved.error && moved.props.vertOffset === -5890 && Math.abs(moved.props.vertOffset) < 100000,
    `제한 off 상단 이동 후 세로 offset 표시가 비정상임: ${JSON.stringify(moved.props)}`);
  assert(moved.picture.y < toggled.picture.y - 50,
    `제한 off 상단 이동 후 그림이 위로 이동하지 않음: before=${JSON.stringify(toggled.picture)}, after=${JSON.stringify(moved.picture)}`);
  assert(moved.table.y < toggled.table.y - 100,
    `제한 off 자리차지 그림 이동 후 아래 표가 당겨지지 않음: before=${JSON.stringify(toggled.table)}, after=${JSON.stringify(moved.table)}`);

  console.log('✅ #1282 쪽 영역 제한 off 전환: 저장 no 샘플 렌더와 일치');
});
