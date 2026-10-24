/** #8103: 복원된 선택을 확정된 쪽 배치에 투영한다. */
import { runTest, loadApp, createNewDocument, assert, screenshot, setTestCase } from './helpers.mjs';
import { readFileSync } from 'node:fs';

const requestedRenderer = process.argv.find(arg => arg.startsWith('--renderer='))?.split('=')[1];
const renderers = requestedRenderer ? [requestedRenderer] : ['canvas2d', 'auto'];
const fontCache = process.argv.find(arg => arg.startsWith('--font-cache='))?.slice('--font-cache='.length);
const fontBytes = fontCache ? readFileSync(fontCache) : null;
if (renderers.some(renderer => !['canvas2d', 'auto', 'canvaskit'].includes(renderer))) {
  throw new Error(`지원하지 않는 renderer: ${requestedRenderer}`);
}

const frames = (page) => page.evaluate(() => new Promise(resolve =>
  requestAnimationFrame(() => requestAnimationFrame(resolve))));

async function key(page, name, modifier) {
  if (modifier) await page.keyboard.down(modifier);
  await page.keyboard.press(name);
  if (modifier) await page.keyboard.up(modifier);
}

async function layout(page, count) {
  await page.waitForFunction(n => window.__selectionLayoutCount > n, { timeout: 30_000 }, count);
  await frames(page);
}

// 직전 mutation의 quiet 예약이 실제로 남은 순간 새 mutation의 metrics 준비를 지연한다.
async function beginDelayedMutation(page) {
  await page.evaluate(() => new Promise(resolve => {
    const w = window.__wasm;
    window.__selectionGateEntered = false;
    const off = window.__eventBus.on('document-layout-refreshed', ({ source }) => {
      if (source !== 'mutation') return;
      off();
      const prepare = w.prepareCanvasMetrics.bind(w);
      w.prepareCanvasMetrics = async (...args) => {
        w.prepareCanvasMetrics = prepare;
        window.__selectionGateFinished = false;
        window.__selectionGateEntered = true;
        await new Promise(release => { window.__releaseSelectionLayout = release; });
        try { return await prepare(...args); }
        finally { window.__selectionGateFinished = true; }
      };
      window.__eventBus.emit('document-changed');
      resolve();
    });
    window.__eventBus.emit('document-changed');
  }));
  // CanvasKit은 resolve에서 실제 글꼴 준비를 await한다. 이전 gate의 release를
  // 호출하거나 준비 전 renderer를 사용하는 대신 이번 metrics 대기 진입을 확인한다.
  await page.waitForFunction(() => window.__selectionGateEntered === true);
}

async function settleRenderer(page, renderer) {
  if (renderer === 'auto') {
    await page.waitForFunction(() => window.__rendererSelection?.selectionReason !== 'autoRevisionPending',
      { timeout: 30_000 });
    await frames(page);
  }
}

async function state(page) {
  return page.evaluate(() => {
    const ih = window.__inputHandler;
    const content = document.getElementById('scroll-content');
    const container = document.getElementById('scroll-container');
    const selection = ih.cursor.getSelection();
    const ordered = ih.cursor.getSelectionOrdered();
    const rects = ordered ? window.__wasm.getSelectionRects(
      ordered.start.sectionIndex, ordered.start.paragraphIndex, ordered.start.charOffset,
      ordered.end.paragraphIndex, ordered.end.charOffset,
    ) : [];
    const highlights = [...content.querySelectorAll('.selection-highlight')]
      .filter(el => el.style.display !== 'none')
      .map(el => ['left', 'top', 'width', 'height'].map(k => parseFloat(el.style[k])));
    const zoom = ih.viewportManager.getZoom();
    const projected = rects.map(r => [
      ih.virtualScroll.getPageLeftResolved(r.pageIndex, content.clientWidth) + r.x * zoom,
      ih.virtualScroll.getPageOffset(r.pageIndex) + r.y * zoom,
      r.width * zoom, r.height * zoom,
    ]);
    const caretRect = ih.cursor.getRect();
    const caretElement = content.querySelector('.caret');
    const caret = caretRect && caretElement ? {
      actual: [parseFloat(caretElement.style.left), parseFloat(caretElement.style.top)],
      projected: [ih.virtualScroll.getPageLeftResolved(caretRect.pageIndex, content.clientWidth) + caretRect.x * zoom,
        ih.virtualScroll.getPageOffset(caretRect.pageIndex) + caretRect.y * zoom],
    } : null;
    return {
      selection, highlights, projected, caret, owners: rects.map(r => r.pageIndex),
      pages: window.__wasm.pageCount, virtualPages: ih.virtualScroll.pageCount,
      paragraphs: window.__wasm.getParagraphCount(0),
      texts: Array.from({ length: window.__wasm.getParagraphCount(0) }, (_, p) =>
        window.__wasm.getTextRange(0, p, 0, 100)),
      scroll: [container.scrollLeft, container.scrollTop], zoom,
      count: window.__selectionLayoutCount,
    };
  });
}

const same = (a, b) => JSON.stringify(a) === JSON.stringify(b);
const near = (a, b) => a.length === b.length && a.every((r, i) =>
  r.every((v, j) => Math.abs(v - b[i][j]) < 0.02));

function checkProjection(s, label) {
  assert(s.pages === s.virtualPages, `${label}: 엔진과 VirtualScroll 쪽 수 일치`);
  assert(near(s.highlights, s.projected), `${label}: 모든 음영이 확정 쪽 원점에 투영`);
  assert(s.caret && near([s.caret.actual], [s.caret.projected]), `${label}: focus 캐럿도 확정 쪽 원점에 투영`);
  for (const owner of [0, Math.floor(s.pages / 2), s.pages - 1]) {
    const indexes = s.owners.flatMap((p, i) => p === owner ? [i] : []);
    assert(indexes.length > 0 && indexes.every(i => s.highlights[i]
      && near([s.highlights[i]], [s.projected[i]])), `${label}: 쪽 ${owner + 1} 선택 좌표`);
  }
}

for (const renderer of renderers) {
await runTest(`선택 삭제 Undo의 확정 layout·스크롤·줌·stale fencing (${renderer})`, async ({ page }) => {
  const errors = [];
  const renderErrors = [];
  page.on('console', message => {
    if (message.type() === 'error' && /CanvasView|CanvasKit|렌더|font family/.test(message.text())) renderErrors.push(message.text());
  });
  let suppliedFonts = 0;
  page.on('pageerror', error => errors.push(String(error)));
  if (fontBytes) {
    // TLS 검증으로 내려받은 원본 CDN 바이트를 공급한다. renderer·metrics·선택 rect는 대역하지 않는다.
    await page.setRequestInterception(true);
    page.on('request', request => {
      if (request.url() === 'https://cdn.jsdelivr.net/gh/projectnoonnu/noonfonts_2104@1.0/HANBatang.woff') {
        suppliedFonts++;
        void request.respond({ status: 200, contentType: 'font/woff',
          headers: { 'access-control-allow-origin': '*' }, body: fontBytes });
      } else {
        void request.continue();
      }
    });
  }
  await loadApp(page, `?renderer=${renderer}&canvaskitSurface=software`);
  assert(await page.evaluate(expected => window.__renderBackendRequest.backend === expected, renderer),
    `${renderer}: 실제 요청 경로 확인`);
  await page.evaluate(() => {
    document.querySelector('.skin-onboarding-card')?.dispatchEvent(new MouseEvent('mousedown', { bubbles: true }));
    [...document.querySelectorAll('button.dialog-btn-primary')].find(el => el.offsetParent !== null)?.click();
  });
  const modifier = await page.evaluate(() => /Mac/i.test(navigator.platform) ? 'Meta' : 'Control');
  const text = Array.from({ length: 400 }, (_, i) => `P${String(i).padStart(3, '0')} selection sample`).join('\n');
  await createNewDocument(page);
  await page.evaluate(() => {
    window.__selectionLayoutCount = 0;
    window.__eventBus.on('document-layout-refreshed', ({ source }) => {
      if (source === 'mutation') window.__selectionLayoutCount++;
    });
    window.__eventBus.emit('page-view-settings-changed', {
      arrangement: { kind: 'single' }, pageMovement: { direction: 'vertical', wheelHorizontal: false },
    });
    window.__inputHandler.viewportManager.setZoom(1);
    window.__inputHandler.focus();
  });
  await page.evaluate(value => {
    const clipboardData = new DataTransfer();
    clipboardData.setData('text/plain', value);
    window.__inputHandler.textarea.dispatchEvent(new ClipboardEvent('paste', { clipboardData, bubbles: true, cancelable: true }));
  }, text);
  await layout(page, 0);
  await key(page, 'Home', modifier);
  await key(page, 'a', modifier);
  const original = await state(page);
  if (renderer === 'canvaskit') {
    assert(await page.evaluate(() => window.__canvasView.getRenderBackend() === 'canvaskit'),
      '준비된 문서의 CanvasKit 검사를 Canvas2D fallback으로 대체하지 않음');
  }
  assert(original.paragraphs === 400 && original.pages > 2, '400문단 여러 쪽 fixture');
  const expectedTexts = original.texts;

  for (const zoom of [1, 0.66]) {
    setTestCase(`zoom-${zoom}`);
    await page.evaluate(z => window.__inputHandler.viewportManager.setZoom(z), zoom);
    await frames(page);
    const before = await state(page);
    checkProjection(before, `${zoom} zoom`);
    await key(page, 'Backspace');
    await layout(page, before.count);
    const deleted = await state(page);
    assert(deleted.pages === 1 && deleted.paragraphs === 1 && deleted.texts[0] === '', '전체 삭제: 빈 한 쪽');
    await key(page, 'z', modifier);
    await layout(page, deleted.count);
    const restored = await state(page);
    console.log(JSON.stringify({ zoom, restoredPages: restored.pages,
      mismatchedHighlights: restored.projected.filter((r, i) =>
        !restored.highlights[i] || !near([r], [restored.highlights[i]])).length }));
    checkProjection(restored, `${zoom} Undo`);
    await settleRenderer(page, renderer);
    checkProjection(await state(page), `${zoom} settled renderer`);
    assert(same(restored.texts, expectedTexts), 'Undo: 원문 400문단 순서·내용 보존');
    assert(same(restored.selection, before.selection), 'Undo: anchor/focus 보존');

    for (const owner of [0, Math.floor(restored.pages / 2), restored.pages - 1]) {
      await page.evaluate(p => {
        const ih = window.__inputHandler;
        ih.viewportManager.setScrollTop(ih.virtualScroll.getPageOffset(p));
      }, owner);
      await frames(page);
      const scrolled = await state(page);
      checkProjection(scrolled, `${zoom} scroll p${owner + 1}`);
      assert(same(scrolled.selection, restored.selection), '스크롤: anchor/focus 유지');
      await screenshot(page, `issue8103-${renderer}-zoom${zoom}-page${owner + 1}`);
    }
    const scrolled = await state(page);
    await key(page, 'a', modifier);
    const fresh = await state(page);
    assert(near(scrolled.highlights, fresh.highlights), '같은 확정 layout의 새 선택 대조군과 전체 좌표 일치');
    assert(same(scrolled.scroll, fresh.scroll), '재투영은 스크롤을 이동하지 않음');

    // Redo는 선택을 해제하고 삭제하며, 다시 Undo하면 원래 선택을 되살린다.
    await page.keyboard.down(modifier);
    await page.keyboard.down('Shift');
    await page.keyboard.press('z');
    await page.keyboard.up('Shift');
    await page.keyboard.up(modifier);
    await layout(page, fresh.count);
    const redone = await state(page);
    assert(redone.pages === 1 && redone.texts[0] === '' && redone.selection === null
      && redone.highlights.length === 0, 'Redo: 실제 전체 삭제 범위·선택 해제 일치');
    await key(page, 'z', modifier);
    await layout(page, redone.count);
    const repeated = await state(page);
    checkProjection(repeated, `${zoom} repeat Undo`);
    // 새 선택 대조군을 만들지 않은 복원 선택에서, 마지막 쪽을 보며 실제로 삭제한다.
    await page.evaluate(() => {
      const ih = window.__inputHandler;
      ih.viewportManager.setScrollTop(ih.virtualScroll.getPageOffset(window.__wasm.pageCount - 1));
    });
    await frames(page);
    await key(page, 'Backspace');
    await layout(page, repeated.count);
    const removed = await state(page);
    assert(removed.pages === 1 && removed.paragraphs === 1 && removed.texts[0] === ''
      && removed.selection === null && removed.highlights.length === 0,
    '재선택 없는 복원 선택: 마지막 쪽에서 Backspace가 전체 음영 범위를 삭제');
    await key(page, 'z', modifier);
    await layout(page, removed.count);
    const recovered = await state(page);
    assert(same(recovered.texts, expectedTexts), '실제 Backspace의 Undo가 400문단 원문을 복원');
    checkProjection(recovered, `${zoom} Backspace Undo`);
  }

  setTestCase('zoom-dialog-transaction');
  const beforeDialog = await state(page);
  await page.click('#sb-zoom-display');
  await page.click('input[name="zoom-choice"][value="custom"]');
  const customZoom = await page.$('input[aria-describedby="zoom-dialog-custom-error"]');
  await customZoom.evaluate(input => { input.value = '80'; input.dispatchEvent(new Event('input', { bubbles: true })); });
  await page.click('.dialog-btn-primary');
  await frames(page);
  const afterDialog = await state(page);
  console.log(JSON.stringify({ dialogZoom: afterDialog.zoom, beforeZoom: beforeDialog.zoom }));
  assert(Math.abs(afterDialog.zoom - 0.8) < 0.001, '대화상자에서 요청한 80% 배율 적용');
  checkProjection(afterDialog, '실제 줌 대화상자의 최종 배치');
  assert(same(afterDialog.selection, beforeDialog.selection), '줌 대화상자는 복원 선택을 보존');

  setTestCase('arrangement-resize-smooth-zoom');
  const beforeViewChange = await state(page);
  await page.evaluate(() => {
    window.__eventBus.emit('page-view-settings-changed', {
      arrangement: { kind: 'double' }, pageMovement: { direction: 'vertical', wheelHorizontal: false },
    });
    window.__inputHandler.viewportManager.setZoom(0.66);
  });
  await page.setViewport({ width: 1100, height: 900 });
  await frames(page);
  const arranged = await state(page);
  checkProjection(arranged, '두 쪽 보기·viewport resize');
  assert(same(arranged.selection, beforeViewChange.selection), '배치·resize 후 anchor/focus 유지');
  await settleRenderer(page, renderer);
  await page.evaluate(() => {
    const ih = window.__inputHandler;
    const initialSelection = JSON.stringify(ih.cursor.getSelection());
    const probe = { samples: 0, coherent: true, preserved: true, maxDelta: 0 };
    const off = window.__eventBus.on('zoom-changed', () => {
      const selection = ih.cursor.getSelectionOrdered();
      const rects = window.__wasm.getSelectionRects(selection.start.sectionIndex,
        selection.start.paragraphIndex, selection.start.charOffset,
        selection.end.paragraphIndex, selection.end.charOffset);
      const content = document.getElementById('scroll-content');
      const zoom = ih.viewportManager.getZoom();
      const highlights = [...content.querySelectorAll('.selection-highlight')].filter(el => el.style.display !== 'none');
      probe.samples++;
      probe.coherent &&= rects.length === highlights.length;
      for (let i = 0; i < rects.length; i++) {
        const r = rects[i];
        const expected = [ih.virtualScroll.getPageLeftResolved(r.pageIndex, content.clientWidth) + r.x * zoom,
          ih.virtualScroll.getPageOffset(r.pageIndex) + r.y * zoom, r.width * zoom, r.height * zoom];
        const actual = ['left', 'top', 'width', 'height'].map(k => parseFloat(highlights[i]?.style[k]));
        const delta = Math.max(...actual.map((v, j) => Math.abs(v - expected[j])));
        probe.coherent &&= Number.isFinite(delta) && delta < 0.02;
        probe.maxDelta = Math.max(probe.maxDelta, delta);
      }
      probe.preserved &&= JSON.stringify(ih.cursor.getSelection()) === initialSelection;
      document.getElementById('scroll-container').scrollBy(0, 15);
    });
    window.__finishSelectionZoomProbe = () => { off(); return probe; };
    ih.viewportManager.setScrollTop(ih.virtualScroll.getPageOffset(3));
    ih.viewportManager.smoothZoomTo(0.9);
  });
  let samples = 0;
  let coherent = true;
  let preserved = true;
  for (let frame = 0; frame < 90; frame++) {
    await frames(page);
    const moving = await state(page);
    samples++;
    coherent &&= near(moving.highlights, moving.projected);
    preserved &&= same(moving.selection, arranged.selection);
    await page.evaluate(() => document.getElementById('scroll-container').scrollBy(0, 15));
    if (!await page.evaluate(() => window.__inputHandler.viewportManager.isZoomAnimating())) break;
  }
  const zoomProbe = await page.evaluate(() => window.__finishSelectionZoomProbe());
  console.log(JSON.stringify({ renderer, zoomProbe, nodeSamples: samples, nodeCoherent: coherent }));
  assert(zoomProbe.samples > 1 && zoomProbe.coherent && coherent, '실제 smooth zoom·scroll 중 각 관측 프레임의 선택 좌표 일치');
  assert(preserved && zoomProbe.preserved, 'smooth zoom·scroll 중 anchor/focus 유지');
  assert(Math.abs((await state(page)).zoom - 0.9) < 0.001, 'smooth zoom이 요청 배율에 도달');

  setTestCase('pending-layout-new-selection');
  await beginDelayedMutation(page);
  const pending = await state(page);
  const newer = await page.evaluate(() => {
    const ih = window.__inputHandler;
    // 방향이 역순인 새 선택을 layout 대기 중에 만든다.
    ih.cursor.selectRange({ sectionIndex: 0, paragraphIndex: 220, charOffset: 7 },
      { sectionIndex: 0, paragraphIndex: 200, charOffset: 2 });
    ih.updateCaret(true);
    ih.viewportManager.setScrollTop(ih.virtualScroll.getPageOffset(3));
    ih.viewportManager.setZoom(0.8);
    return ih.cursor.getSelection();
  });
  await frames(page);
  const waiting = await state(page);
  // 이전 quiet 타이머(300ms)가 발화할 시간을 실제 브라우저에서 지난다.
  await page.evaluate(() => new Promise(resolve => setTimeout(resolve, 450)));
  await page.evaluate(() => window.__releaseSelectionLayout());
  await layout(page, pending.count);
  const settled = await state(page);
  assert(same(settled.selection, newer), '지연 layout은 새 역방향 anchor/focus를 보존');
  assert(same(settled.scroll, waiting.scroll), '지연 layout 완료는 현재 스크롤을 보존');
  assert(near(settled.highlights, settled.projected), '지연 완료는 현재 선택·배율로 재투영');

  setTestCase('pending-layout-cleared-selection');
  await beginDelayedMutation(page);
  const clearing = await state(page);
  await page.evaluate(() => {
    const ih = window.__inputHandler;
    ih.cursor.clearSelection();
    ih.updateCaret(true);
    window.__releaseSelectionLayout();
  });
  await layout(page, clearing.count);
  const cleared = await state(page);
  assert(cleared.selection === null && cleared.highlights.length === 0, '지연 완료가 해제된 선택을 되살리지 않음');

  setTestCase('pending-layout-new-revision');
  await page.evaluate(() => {
    const ih = window.__inputHandler;
    ih.cursor.selectRange({ sectionIndex: 0, paragraphIndex: 41, charOffset: 4 },
      { sectionIndex: 0, paragraphIndex: 56, charOffset: 8 });
    ih.updateCaret(true);
  });
  await beginDelayedMutation(page);
  const obsolete = await state(page);
  await page.evaluate(() => window.__eventBus.emit('document-changed'));
  await layout(page, obsolete.count);
  await settleRenderer(page, renderer);
  const latest = await state(page);
  await page.evaluate(() => window.__releaseSelectionLayout());
  await page.waitForFunction(() => window.__selectionGateFinished);
  await frames(page);
  const afterObsolete = await state(page);
  assert(afterObsolete.count === latest.count, '역순 완료: 이전 revision은 mutation 완료 이벤트를 발행하지 않음');
  assert(latest.selection !== null && same(afterObsolete.selection, latest.selection) && near(afterObsolete.highlights, latest.highlights)
    && same(afterObsolete.scroll, latest.scroll), '역순 완료: 최신 선택·음영·스크롤 유지');

  setTestCase('pending-layout-document-switch');
  await page.evaluate(() => {
    const ih = window.__inputHandler;
    ih.cursor.selectRange({ sectionIndex: 0, paragraphIndex: 10, charOffset: 1 },
      { sectionIndex: 0, paragraphIndex: 20, charOffset: 3 });
    ih.updateCaret(true);
  });
  await beginDelayedMutation(page);
  await createNewDocument(page);
  const newDocument = await state(page);
  await page.evaluate(() => window.__releaseSelectionLayout());
  await page.waitForFunction(() => window.__selectionGateFinished);
  await frames(page);
  const switched = await state(page);
  assert(switched.pages === 1 && switched.paragraphs === 1 && switched.texts[0] === ''
    && switched.selection === null && switched.highlights.length === 0, '문서 교체: 이전 선택·음영·내용이 새 문서로 누출되지 않음');
  assert(switched.count === newDocument.count, '문서 교체 뒤 이전 mutation 완료 이벤트 폐기');
  assert(errors.length === 0, `pageerror 0건: ${JSON.stringify(errors)}`);
  assert(renderErrors.length === 0, `caught render error 0건: ${JSON.stringify(renderErrors)}`);
  if (fontBytes) assert(suppliedFonts > 0, '다운로드한 실제 원본 글꼴 바이트를 브라우저가 소비');
  console.log(JSON.stringify({ renderer, pages: original.pages, paragraphs: original.paragraphs,
    highlights: original.highlights.length, staleSelectionPreserved: same(settled.selection, newer) }));
}, { skipLoadApp: true });
}
