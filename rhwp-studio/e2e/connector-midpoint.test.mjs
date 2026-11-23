/**
 * 관계형. 실제 마우스 드래그로 가운데 선분만 움직이고 양 끝점과 개체 크기를 보존한다.
 * 2026-11-24 KST. 본문·셀의 꺾인 선과 곡선 HWP/HWPX 8종 각 1쪽, 96 dpi 독립 한컴
 * Print 대비 Native·fresh WASM 전쪽 100.00%. mydocs/pr/assets/semanticist21-20261124-rework/pages.tsv.
 */
import assert from 'node:assert/strict';
import { loadHwpFile, runTest } from './helpers.mjs';

function readLine(nested) {
  const bridge = window.__wasm;
  const line = bridge.getPageControlLayout(0).controls.find(control => control.type === 'line' && Boolean(control.cellPath?.length) === nested);
  const props = line.cellPath?.length
    ? bridge.getCellShapePropertiesByPath(line.secIdx, line.paraIdx, line.cellPath, line.controlIdx)
    : bridge.getShapeProperties(line.secIdx, line.paraIdx, line.controlIdx);
  return {
    x: props.connectorMidX, y: props.connectorMidY, axis: props.connectorMidAxis,
    width: props.width, height: props.height, endpoints: [line.x1, line.y1, line.x2, line.y2],
  };
}

await runTest('연결선 가운데 손잡이 드래그·되돌리기·저장', async ({ page }) => {
  const samples = ['connector-arc', 'connector-stroke', 'nested-connector-arc', 'nested-connector-stroke',
    'two-vertical', 'two-horizontal-shift', 'one-elbow', 'no-elbow'];
  for (const sample of samples) {
    await loadHwpFile(page, `semanticist21-20261124-rework/${sample}.hwpx`);
    const nested = sample.startsWith('nested-');
    const before = await page.evaluate(readLine, nested);
    const sibling = nested ? await page.evaluate(readLine, false) : null;
    const endpoint = await page.evaluate(nested => {
      const input = window.__inputHandler;
      const line = window.__wasm.getPageControlLayout(0).controls.find(control => control.type === 'line' && Boolean(control.cellPath?.length) === nested);
      const content = document.querySelector('#scroll-content');
      const zoom = input.viewportManager.getZoom();
      const x = input.virtualScroll.getPageLeftResolved(0, content.clientWidth) + line.x1 * zoom;
      const y = input.virtualScroll.getPageOffset(0) + line.y1 * zoom;
      content.parentElement.scrollTop = Math.max(0, y - 220);
      const bounds = content.getBoundingClientRect();
      return { x: bounds.left + x, y: bounds.top + y, zoom };
    }, nested);
    await page.mouse.click(endpoint.x, endpoint.y);
    if (sample === 'one-elbow' || sample === 'no-elbow') {
      assert.equal(before.axis, undefined, sample);
      const selection = await page.evaluate(() => {
        const overlay = window.__inputHandler.pictureObjectRenderer;
        return { handles: overlay.handles.map(h => h.dir), guide: Boolean(overlay.layer.querySelector('svg line')) };
      });
      assert.deepEqual(selection, { handles: ['sw', 'ne'], guide: false }, sample);
      console.log(`${sample}: 두 끝점 선택과 가운데 손잡이·직선 안내선 부재 확인`);
      continue;
    }
    const handle = await page.evaluate(() => {
      const h = window.__inputHandler.pictureObjectRenderer.handles.find(h => h.dir === 'midpoint');
      if (!h) return null;
      const rect = h.el.getBoundingClientRect();
      return { x: rect.left + rect.width / 2, y: rect.top + rect.height / 2 };
    });
    assert.ok(handle, `${sample}: 선을 클릭하면 가운데 손잡이가 나타난다`);
    await page.mouse.move(handle.x, handle.y);
    await page.mouse.down();
    const dragAxis = await page.evaluate(() => window.__inputHandler.connectorMidpointDrag?.axis);
    assert.equal(dragAxis, before.axis, sample);
    await page.mouse.move(handle.x + 40, handle.y + 26, { steps: 5 });
    await page.mouse.up();
    const moved = await page.evaluate(readLine, nested);
    const expected = { ...before };
    assert.ok(before.axis === 'x' || before.axis === 'y', sample);
    expected[before.axis] += Math.round((before.axis === 'x' ? 40 : 26) * 75 / endpoint.zoom);
    assert.deepEqual(moved, expected, `${sample}: 이동 축·끝점·크기`);
    if (nested) assert.deepEqual(await page.evaluate(readLine, false), sibling, sample);
    await page.keyboard.down('Control');
    await page.keyboard.press('z');
    await page.keyboard.up('Control');
    assert.deepEqual(await page.evaluate(readLine, nested), before, `${sample}: 실행 취소`);
    await page.keyboard.down('Control');
    await page.keyboard.press('y');
    await page.keyboard.up('Control');
    assert.deepEqual(await page.evaluate(readLine, nested), moved, `${sample}: 다시 실행`);
    for (const format of ['Hwp', 'Hwpx']) {
      for (let pass = 0; pass < 2; pass++) {
        await page.evaluate(format => {
          const bridge = window.__wasm;
          bridge.loadDocument(bridge[`export${format}`](), `edited.${format.toLowerCase()}`);
        }, format);
        assert.deepEqual(await page.evaluate(readLine, nested), moved, `${sample}: ${format} 재열기 ${pass + 1}`);
      }
    }
    console.log(`${sample}: ${before.axis}축 이동, 실행 취소·다시 실행과 양 형식 저장 통과`);
  }
});
