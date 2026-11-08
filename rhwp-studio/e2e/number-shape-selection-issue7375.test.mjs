// 관계형: 표 셀 번호 도형의 실제 선택·이동·삭제와 Undo의 소유·표시 경계 복원을 검사한다.
// 주석 갱신: 2026-11-05 18:52 KST. 원본 control 주소를 쓰며 실물 px 기대값은 고정하지 않는다.
// 원본 27쪽 Native/fresh WASM 99.99897%/99.99691%, 28쪽 99.99893%/99.99678%(96dpi·compat2022).
import assert from 'node:assert/strict';
import { mkdirSync, writeFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { createHash } from 'node:crypto';
import { loadApp, runTest } from './helpers.mjs';

const output = new URL('../../output/e2e/issue7375/', import.meta.url);
mkdirSync(output, { recursive: true });
const settle = page => page.evaluate(() => new Promise(resolve => requestAnimationFrame(() => requestAnimationFrame(resolve))));

async function pointer(page, pageIndex, x, y) {
  await page.evaluate(({ pageIndex, y }) => {
    const input = window.__inputHandler;
    input.viewportManager.setScrollTop(Math.max(0, input.virtualScroll.getPageOffset(pageIndex)
      + y * input.viewportManager.getZoom() - 300));
  }, { pageIndex, y });
  await settle(page);
  return page.evaluate(({ pageIndex, x, y }) => {
    const input = window.__inputHandler, content = document.querySelector('#scroll-content');
    const bounds = content.getBoundingClientRect(), zoom = input.viewportManager.getZoom();
    return {
      x: bounds.left + input.virtualScroll.getPageLeftResolved(pageIndex, content.clientWidth) + x * zoom,
      y: bounds.top + input.virtualScroll.getPageOffset(pageIndex) + y * zoom,
    };
  }, { pageIndex, x, y });
}

async function offsets(page, ref) {
  return page.evaluate(ref => {
    const p = window.__inputHandler.getObjectProperties(ref);
    return { horzOffset: p.horzOffset, vertOffset: p.vertOffset };
  }, ref);
}

async function undo(page, modifier) {
  await page.keyboard.down(modifier);
  try { await page.keyboard.press('KeyZ'); } finally { await page.keyboard.up(modifier); }
  await settle(page);
}

runTest('#7375 27·28쪽 번호 도형의 독립 선택·드래그·삭제·Undo', async ({ page }) => {
  const initializationJs = [];
  const responseTasks = [];
  page.on('response', response => {
    if (new URL(response.url()).pathname.endsWith('/rhwp.js')) {
      responseTasks.push((async () => {
        const body = await response.buffer();
        initializationJs.push({ url: response.url(), status: response.status(),
          sha256: createHash('sha256').update(body).digest('hex'),
          hasCellShapeDelete: body.includes(Buffer.from('deleteCellShapeControlByPath')) });
      })());
    }
  });
  await loadApp(page);
  await Promise.all(responseTasks);
  assert.ok(initializationJs.length > 0);
  assert.ok(initializationJs.every(response => response.status === 200 && response.hasCellShapeDelete),
    '초기화 응답의 실제 JS가 셀 도형 삭제 바인딩을 제공한다');
  const errors = [];
  page.on('console', m => {
    if (/셀 문단.*실패|getSelectionRects 실패|커맨드 실행 실패/.test(m.text())) errors.push(m.text());
  });
  const fileInput = await page.$('#file-input');
  assert.ok(fileInput);
  await fileInput.uploadFile(fileURLToPath(new URL('../../samples/issue7333/aaaaaa.hwp', import.meta.url)));
  await page.waitForFunction(() => document.querySelector('#file-input').value === ''
    || [...document.querySelectorAll('button')].some(b => b.textContent.trim() === '대체 글꼴로 보기'), { timeout: 180000 });
  for (const button of await page.$$('button')) {
    if (await button.evaluate(n => n.textContent.trim() === '대체 글꼴로 보기')) { await button.click(); break; }
  }
  await page.waitForFunction(() => window.__wasm?.pageCount === 50 && document.querySelector('#file-input').value === '', { timeout: 180000 });
  await settle(page);
  const results = [];
  for (const { pageIndex, paraIndex } of [{ pageIndex: 26, paraIndex: 383 }, { pageIndex: 27, paraIndex: 392 }]) {
    console.log(`  ${pageIndex + 1}쪽 번호 도형 선택·이동·삭제 검사`);
    const target = await page.evaluate(({ pageIndex, paraIndex }) => {
      const input = window.__inputHandler;
      const control = window.__wasm.getPageControlLayout(pageIndex).controls.find(c =>
        c.type === 'shape' && c.paraIdx === paraIndex && c.controlIdx === 0 && c.cellPath?.length === 1);
      if (!control) throw new Error('원본 번호 도형 control이 없다');
      const ref = input.findPictureAtClick(pageIndex, control.x + control.w * 0.12, control.y + control.h / 2);
      const before = input.getObjectProperties(ref);
      const probe = { calls: 0, original: input.executeOperation };
      input.__issue7375Probe = probe;
      input.executeOperation = function (...args) { probe.calls++; return probe.original.apply(this, args); };
      return { control, ref, before };
    }, { pageIndex, paraIndex });
    const point = await pointer(page, pageIndex, target.control.x + target.control.w * 0.12, target.control.y + target.control.h / 2);
    await page.mouse.click(point.x, point.y);
    await settle(page);
    const selected = await page.evaluate(() => {
      const input = window.__inputHandler, probe = input.__issue7375Probe;
      input.executeOperation = probe.original; delete input.__issue7375Probe;
      return { ref: input.cursor.getSelectedPictureRef(), object: input.cursor.isInPictureObjectSelection(), operations: probe.calls };
    });
    assert.equal(selected.object, true, `${pageIndex + 1}쪽은 실제 개체 선택 모드다`);
    assert.equal(selected.ref?.type, 'shape');
    assert.equal(selected.ref?.ppi, paraIndex);
    assert.equal(selected.ref?.ci, 0, '뒤 그림이나 옆 사각형 대신 번호 도형을 선택한다');
    assert.equal(selected.ref?.pageIndex, pageIndex);
    assert.deepEqual(selected.ref?.cellPath, target.control.cellPath);
    assert.equal(selected.operations, 0, '선택만으로 문서·z-order·Undo를 바꾸지 않는다');
    const before = await offsets(page, selected.ref);
    const moves = [];
    for (const modifier of ['Control', 'Meta']) {
      const box = await page.evaluate(ref => window.__inputHandler.findPictureBbox(ref), selected.ref);
      assert.ok(box);
      const edge = await pointer(page, pageIndex, box.x + box.w * 0.12, box.y + box.h / 2);
      await page.mouse.click(edge.x, edge.y);
      await settle(page);
      const drag = await pointer(page, pageIndex, box.x + box.w / 2, box.y + box.h / 2);
      await page.mouse.move(drag.x, drag.y);
      await page.mouse.down();
      await page.mouse.move(drag.x + 24, drag.y + 18, { steps: 4 });
      await page.mouse.up();
      await settle(page);
      const moved = await offsets(page, selected.ref);
      assert.notDeepEqual(moved, before, `실제 드래그가 해당 도형의 저장 위치를 바꾼다: ${JSON.stringify({ before, moved })}`);
      await undo(page, modifier);
      const restored = await offsets(page, selected.ref);
      assert.deepEqual(restored, before, `${modifier}+Z가 원래 위치를 복원한다`);
      const restoredBox = await page.evaluate(ref => window.__inputHandler.findPictureBbox(ref), selected.ref);
      assert.deepEqual(restoredBox, box, `${modifier}+Z가 저장 좌표와 실제 표시 경계를 함께 복원한다`);
      moves.push({ modifier, moved, restored, box, restoredBox });
    }
    const box = await page.evaluate(ref => window.__inputHandler.findPictureBbox(ref), selected.ref);
    const deletePoint = await pointer(page, pageIndex, box.x + box.w * 0.12, box.y + box.h / 2);
    await page.mouse.click(deletePoint.x, deletePoint.y);
    await settle(page);
    const controls = await page.evaluate(pageIndex => window.__wasm.getPageControlLayout(pageIndex).controls, pageIndex);
    await page.keyboard.press('Delete');
    await settle(page);
    const deleted = await page.evaluate(({ pageIndex, paraIndex }) => {
      const all = window.__wasm.getPageControlLayout(pageIndex).controls;
      return all.filter(c => c.paraIdx === paraIndex && c.cellPath?.length === 1);
    }, { pageIndex, paraIndex });
    const originalChildren = controls.filter(c => c.paraIdx === paraIndex && c.cellPath?.length === 1);
    assert.equal(deleted.length, originalChildren.length - 1, '실제 Delete는 외부 표 대신 선택한 셀 도형 하나만 지운다');
    await undo(page, pageIndex === 26 ? 'Control' : 'Meta');
    const restoredControls = await page.evaluate(pageIndex => window.__wasm.getPageControlLayout(pageIndex).controls, pageIndex);
    assert.deepEqual(restoredControls, controls, '삭제 Undo가 번호·형제 그림·표의 실제 표시와 주소를 복원한다');
    await page.screenshot({ path: fileURLToPath(new URL(`page${pageIndex + 1}-number-shape.png`, output)) });
    results.push({ page: pageIndex + 1, selected, before, moves, deletion: { before: originalChildren.length, after: deleted.length, restored: true } });
    await page.keyboard.press('Escape');
    await settle(page);
  }
  assert.deepEqual(errors, [], '선택·드래그 중 잘못된 셀 참조나 명령 실패가 없다');
  assert.equal(await page.evaluate(() => window.__wasm.pageCount), 50);
  const backend = await page.evaluate(() => window.__canvasView.getRenderBackend());
  writeFileSync(new URL('browser-selection-result.json', output), JSON.stringify({ initializationJs, backend, results, errors }, null, 2) + '\n');
}, { skipLoadApp: true });
