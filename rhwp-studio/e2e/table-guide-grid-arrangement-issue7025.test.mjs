/**
 * E2E (#7025): 그리드 보기(두 쪽·맞쪽·여러 쪽)에서 표 리사이즈 안내선이 **그 쪽의 실제 X**
 * 위에 그려진다.
 *
 * 수정 전에는 오버레이가 단일 열 fallback 식 `(contentWidth - pageWidth)/2` 를 그대로 복사해
 * 써서, 그리드 모드에서 모든 쪽의 마커가 한 쪽 보기와 똑같은 자리에 찍혔다 — 3190263 2쪽
 * 실측: 쪽 2 는 x=813.7 인데 마커는 483.4(793.6px 이탈, 옆 쪽 위).
 *
 * `#685` 가 같은 단일 열 가정을 **입력(click) 축**에서 고쳤기 때문에 커서·드래그 판정은
 * 정상이었고 **그리기 축만** 어긋났다. 그래서 이 시험은 커서가 아니라 **마커 좌표**를 잰다.
 * 정본은 `VirtualScroll.getPageLeftResolved` 하나다.
 */
import { runTest, createNewDocument, clickEditArea, screenshot, assert } from './helpers.mjs';

const THICKNESS = 3;

await runTest('그리드 보기에서 표 안내선이 그 쪽의 실제 X 를 따른다 (#7025)', async ({ page }) => {
  await createNewDocument(page);
  await clickEditArea(page);
  await page.evaluate(() => new Promise(r => setTimeout(r, 400)));
  await page.evaluate(() => {
    const btn = [...document.querySelectorAll('button')].find(b => b.textContent?.includes('시작하기'));
    btn?.click();
  });
  await page.evaluate(() => new Promise(r => setTimeout(r, 300)));

  const out = await page.evaluate(async (thickness) => {
    const wasm = window.__wasm, ih = window.__inputHandler, cv = window.__canvasView;
    const nf = () => new Promise(r => requestAnimationFrame(() => requestAnimationFrame(r)));

    const created = wasm.createTable(0, 0, 0, 3, 3);
    if (!created?.ok) return { error: `createTable 실패: ${JSON.stringify(created)}` };
    cv?.loadDocument?.();
    await nf(); await nf();

    const vs = ih.virtualScroll;
    const sc = ih.container.querySelector('#scroll-content');
    const zoom = ih.viewportManager.getZoom();
    const bboxes = wasm.getTableCellBboxes(0, created.paraIdx, created.controlIdx, 0);
    const c00 = bboxes.find(b => b.row === 0 && b.col === 0);
    if (!c00) return { error: '(0,0) bbox 없음' };
    const borderPageX = c00.x + c00.w;   // 열 경계 1 의 쪽 좌표

    // 그리드 배치를 만들기 위한 합성 쪽 배열 (한 쪽짜리 새 문서라도 열 배치를 검사할 수 있다)
    const synth = Array.from({ length: 4 }, () => ({
      width: vs.getPageWidth(0) / zoom,
      height: vs.getPageHeight(0) / zoom,
    }));

    const markerLeft = () => {
      const mk = ih.tableResizeRenderer?.layer?.firstElementChild;
      return mk ? parseFloat(mk.style.left) : null;
    };

    const cases = [
      { label: '한 쪽', arrangement: { kind: 'single' }, pageIndex: 0 },
      { label: '한 쪽 · 뒤쪽', arrangement: { kind: 'single' }, pageIndex: 2 },
      { label: '두 쪽 · 오른쪽 열', arrangement: { kind: 'double' }, pageIndex: 1 },
      { label: '여러 쪽 3열 · 셋째 열', arrangement: { kind: 'multiple', columns: 3 }, pageIndex: 2 },
      { label: '맞쪽 · 첫 쪽', arrangement: { kind: 'facing' }, pageIndex: 0 },
    ];

    const rows = [];
    for (const c of cases) {
      vs.setPageDimensions(synth, zoom, sc.clientWidth, c.arrangement, 'vertical', sc.clientHeight);
      await nf();
      const cw = sc.clientWidth;
      const pageBboxes = bboxes.map(b => ({ ...b, pageIndex: c.pageIndex }));
      ih.tableResizeRenderer.showMarker(
        { type: 'col', index: 1, pageIndex: c.pageIndex }, pageBboxes, zoom,
      );
      await nf();
      rows.push({
        label: c.label,
        gridMode: vs.isGridMode(),
        actual: markerLeft(),
        expected: vs.getPageLeftResolved(c.pageIndex, cw) + borderPageX * zoom - thickness / 2,
        singleColumnFormula: (cw - vs.getPageWidth(c.pageIndex)) / 2 + borderPageX * zoom - thickness / 2,
      });
    }

    // 실제 hover 경로도 한 번 밟아 커서·마커가 함께 살아 있는지 확인 (두 쪽 보기)
    vs.setPageDimensions(synth, zoom, sc.clientWidth, { kind: 'double' }, 'vertical', sc.clientHeight);
    await nf();
    const rect = sc.getBoundingClientRect();
    const pl = vs.getPageLeftResolved(0, sc.clientWidth);
    const po = vs.getPageOffset(0);
    const me = (t, x, y) => {
      const e = new MouseEvent(t, { button: 0, buttons: 0, clientX: x, clientY: y, bubbles: true });
      Object.defineProperty(e, 'target', { value: ih.container, configurable: true });
      return e;
    };
    const cx = rect.left + pl - sc.scrollLeft + borderPageX * zoom;
    const cy = rect.top + po - sc.scrollTop + (c00.y + c00.h / 2) * zoom;
    ih.onMouseMoveBound(me('mousemove', cx, cy)); await nf();
    ih.onMouseMoveBound(me('mousemove', cx, cy)); await nf();
    const hover = {
      cursor: ih.container.style.cursor,
      actual: markerLeft(),
      expected: pl + borderPageX * zoom - thickness / 2,
    };
    return { rows, hover, zoom };
  }, THICKNESS);

  assert(!out.error, `시나리오 준비: ${out.error || 'ok'}`);

  for (const r of out.rows) {
    assert(
      r.actual !== null && Math.abs(r.actual - r.expected) < 0.5,
      `${r.label}: 마커 left ${r.actual?.toFixed(1)} 가 그 쪽의 실제 X 기준 ${r.expected.toFixed(1)} 와 같아야 한다`,
    );
  }

  // 그리드 배치에서는 단일 열 공식과 실제로 달라야 한다 — 같으면 결함이 되돌아온 것이다
  for (const r of out.rows.filter(v => v.gridMode)) {
    assert(
      Math.abs(r.expected - r.singleColumnFormula) > 1.0,
      `${r.label}: 이 배치는 단일 열 공식과 갈려야 시험이 의미를 갖는다 (전제 확인)`,
    );
    assert(
      Math.abs(r.actual - r.singleColumnFormula) > 1.0,
      `${r.label}: 단일 열 공식(${r.singleColumnFormula.toFixed(1)})으로 그리면 안 된다 (실제 ${r.actual.toFixed(1)})`,
    );
  }

  assert(out.hover.cursor === 'col-resize', `두 쪽 보기 hover 에서 col-resize 커서 (실제 '${out.hover.cursor}')`);
  assert(
    Math.abs(out.hover.actual - out.hover.expected) < 0.5,
    `두 쪽 보기 hover 마커 ${out.hover.actual?.toFixed(1)} = 정본 ${out.hover.expected.toFixed(1)}`,
  );

  await screenshot(page, 'table-guide-grid-arrangement-7025');
});
