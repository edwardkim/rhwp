/** #9133: 실제 Studio 대화창의 제목·조작은 화면 안에, 넘친 내용은 본문 안에 둔다. UI 회귀이며 문서 조판 검사가 아니다. */
import { mkdirSync, writeFileSync } from 'node:fs';
import path from 'node:path';
import { runTest, assert, createNewDocument } from './helpers.mjs';

const OUTPUT = process.env.RHWP_DIALOG_EVIDENCE_DIR || '../output/issue9133/browser';
const DIALOG = '.modal-overlay .dialog-wrap';
const VIEWPORTS = [[800, 360], [768, 320], [767, 320], [320, 240], [360, 640], [1280, 900]];

async function open(page, kind) {
  await page.evaluate(async (kind) => {
    const wasm = window.__wasm, bus = window.__eventBus;
    window.__viewportChoice9133 = null;
    let dialog;
    if (kind === 'page') {
      const { PageSetupDialog } = await import('/src/ui/page-setup-dialog.ts');
      dialog = new PageSetupDialog(wasm, bus, 0);
    } else if (kind === 'options') {
      const { OptionsDialog } = await import('/src/ui/options-dialog.ts');
      dialog = new OptionsDialog(bus);
    } else if (kind === 'about') {
      const { AboutDialog } = await import('/src/ui/about-dialog.ts');
      dialog = new AboutDialog();
    } else if (kind === 'char') {
      const { CharShapeDialog } = await import('/src/ui/char-shape-dialog.ts');
      dialog = new CharShapeDialog(wasm, bus);
      dialog.show(window.__inputHandler.getCharProperties());
    } else if (kind === 'para') {
      const { ParaShapeDialog } = await import('/src/ui/para-shape-dialog.ts');
      dialog = new ParaShapeDialog(wasm, bus);
      dialog.show(window.__inputHandler.getParaProperties());
    } else if (kind === 'fonts') {
      const { FontSetDialog } = await import('/src/ui/font-set-dialog.ts');
      dialog = new FontSetDialog();
    } else if (kind === 'style') {
      const { StyleDialog } = await import('/src/ui/style-dialog.ts');
      dialog = new StyleDialog(wasm, bus);
    } else if (kind === 'table') {
      const { TableCellPropsDialog } = await import('/src/ui/table-cell-props-dialog.ts');
      wasm.createNewDocument();
      const table = wasm.createTable(0, 0, 0, 2, 2);
      if (!table.ok) throw new Error('검증용 표 생성 실패');
      dialog = new TableCellPropsDialog(wasm, bus, { sec: 0, ppi: table.paraIdx, ci: table.controlIdx }, 0, 'table');
    } else if (kind === 'picture') {
      const { PicturePropsDialog } = await import('/src/ui/picture-props-dialog.ts');
      wasm.createNewDocument();
      const canvas = document.createElement('canvas');
      canvas.width = canvas.height = 8;
      canvas.getContext('2d').fillRect(0, 0, 8, 8);
      const bytes = new Uint8Array(await (await fetch(canvas.toDataURL())).arrayBuffer());
      const picture = wasm.insertPicture(0, 0, 0, '[]', bytes, 7200, 7200, 8, 8, 'png');
      if (!picture.ok) throw new Error('검증용 그림 생성 실패');
      dialog = new PicturePropsDialog(wasm, bus);
      dialog.open(0, picture.paraIdx, picture.controlIdx);
    } else if (kind === 'recovery') {
      const { showAutosaveRecoveryDialog } = await import('/src/recovery/recovery-ui.ts');
      const drafts = Array.from({ length: 12 }, (_, i) => ({ id: `viewport-${i}`, fileName: `검증 문서 ${i}.hwpx`,
        sourceFormat: 'hwpx', savedAt: Date.now(), byteLength: 128, data: new ArrayBuffer(128) }));
      void showAutosaveRecoveryDialog(drafts).then(r => { window.__viewportChoice9133 = r; });
    } else if (kind === 'pdf') {
      const { showPdfImportDialog } = await import('/src/ui/pdf-import-dialog.ts');
      window.__viewportReads9133 = 0;
      void showPdfImportDialog({ byteLength: 128, read: async () => {
        window.__viewportReads9133++; throw new Error('선택 대기 중 PDF를 읽으면 안 됩니다');
      } }, 'viewport.pdf', options => wasm.getPdfImportPolicy(options))
        .then(r => { window.__viewportChoice9133 = r; }, e => { window.__viewportChoice9133 = { code: e.code }; });
    }
    window.__viewportDialog9133 = dialog;
    if (dialog && !['char', 'para', 'picture'].includes(kind)) dialog.show();
  }, kind);
  await page.waitForSelector(DIALOG);
  await page.evaluate(() => new Promise(r => requestAnimationFrame(() => requestAnimationFrame(r))));
}

async function measure(page) {
  return page.$eval(DIALOG, wrap => {
    const box = el => {
      const r = el.getBoundingClientRect();
      return { top: r.top, bottom: r.bottom, left: r.left, right: r.right };
    };
    return { viewport: { width: innerWidth, height: innerHeight }, frame: box(wrap),
      close: box(wrap.querySelector('.dialog-close')),
      actions: [...wrap.querySelectorAll('.dialog-footer .dialog-btn, .cs-right-col .dialog-btn')]
        .filter(b => b.getClientRects().length).map(b => ({ text: b.textContent.trim(), ...box(b) })),
      wrapScroll: wrap.scrollTop };
  });
}
const inside = (r, vp) => r.top >= -0.5 && r.left >= -0.5 && r.bottom <= vp.height + 0.5 && r.right <= vp.width + 0.5;

await runTest('Issue #9133 공통 대화창 viewport·본문 스크롤·조작 보존', async ({ page }) => {
  mkdirSync(OUTPUT, { recursive: true });
  await createNewDocument(page);
  const failures = [], results = [];
  const check = (ok, label) => { if (!ok) failures.push(label); };
  const cases = [
    ...['page', 'options', 'about', 'char', 'para', 'fonts', 'style', 'recovery', 'pdf', 'table', 'picture']
      .flatMap(kind => VIEWPORTS.map(vp => ({ kind, vp }))),
  ];
  for (const { kind, vp: [width, height] } of cases) {
    const name = `${kind}-${width}x${height}`;
    console.log(`  case: ${name}`);
    await page.setViewport({ width, height });
    await open(page, kind);
    const before = await measure(page);
    check(inside(before.frame, before.viewport), `${name}: frame`);
    check(inside(before.close, before.viewport), `${name}: title close`);
    check(before.actions.length > 0 && before.actions.every(r => inside(r, before.viewport)), `${name}: actions`);
    const scroll = await page.$eval(DIALOG, wrap => {
      const outer = wrap.querySelector('.dialog-body');
      outer.scrollTop = outer.scrollHeight;
      const body = wrap.querySelector('.recovery-draft-list')
        || [...wrap.querySelectorAll('.dialog-tab-panel')].find(p => p.getClientRects().length && ['auto', 'scroll'].includes(getComputedStyle(p).overflowY))
        || wrap.querySelector('.dialog-body');
      const max = body.scrollHeight - body.clientHeight;
      body.scrollTop = body.scrollHeight;
      const rect = body.getBoundingClientRect(), frame = wrap.getBoundingClientRect();
      return { max, moved: body.scrollTop, remaining: body.scrollHeight - body.clientHeight - body.scrollTop,
        clientHeight: body.clientHeight, top: rect.top, bottom: rect.bottom,
        inFrame: Math.min(rect.bottom, frame.bottom) > Math.max(rect.top, frame.top),
        overflow: getComputedStyle(body).overflowY };
    });
    if (scroll.max > 1) check(['auto', 'scroll'].includes(scroll.overflow) && scroll.moved > 0 && scroll.remaining <= 1, `${name}: body end`);
    check(scroll.clientHeight > 0 && scroll.inFrame, `${name}: visible scroll viewport`);
    await page.evaluate(() => new Promise(r => requestAnimationFrame(() => requestAnimationFrame(r))));
    const after = await measure(page);
    check(after.wrapScroll === 0 && inside(after.close, after.viewport) && after.actions.every(r => inside(r, after.viewport)), `${name}: fixed actions after scroll`);
    await page.screenshot({ path: path.join(OUTPUT, `${name}.png`) });
    results.push({ name, before, scroll, after });
    if (kind === 'recovery') {
      const radio = await page.$eval(`${DIALOG} .recovery-draft-list label:last-child input`, el => {
        const r = el.getBoundingClientRect();
        return { x: r.x + r.width / 2, y: r.y + r.height / 2, hit: document.elementFromPoint(r.x + r.width / 2, r.y + r.height / 2) === el };
      });
      check(radio.hit, `${name}: last draft can be selected`);
      if (radio.hit) {
        await page.mouse.click(radio.x, radio.y);
        await page.click(`${DIALOG} .dialog-btn-primary`);
        await page.waitForSelector(DIALOG, { hidden: true });
        check(await page.evaluate(() => window.__viewportChoice9133?.action === 'restore' && window.__viewportChoice9133.draftId === 'viewport-11'), `${name}: restore selected last draft`);
        continue;
      }
    }
    const cancel = after.actions.find(b => /취소|닫기|나중에/.test(b.text));
    if (cancel && inside(cancel, after.viewport)) {
      await page.mouse.click((cancel.left + cancel.right) / 2, (cancel.top + cancel.bottom) / 2);
    } else {
      // RED 실행에서도 나머지 대조군을 수집한다. 이 분기는 정상 조작 증거로 세지 않는다.
      await page.$eval(`${DIALOG} .dialog-close`, b => b.click());
    }
    try {
      await page.waitForSelector(DIALOG, { hidden: true, timeout: 1000 });
    } catch {
      check(false, `${name}: cancel hit target`);
      await page.$eval(`${DIALOG} .dialog-close`, b => b.click());
      await page.waitForSelector(DIALOG, { hidden: true, timeout: 1000 });
    }
    if (kind === 'pdf') check(await page.evaluate(() => window.__viewportReads9133 === 0 && window.__viewportChoice9133?.code === 'PDF_IMPORT_CANCELLED'), `${name}: cancel without reading PDF`);
    if (kind === 'recovery') check(await page.evaluate(() => window.__viewportChoice9133?.action === 'later'), `${name}: later result`);
  }

  if (failures.length) {
    writeFileSync(path.join(OUTPUT, 'measurements.json'), JSON.stringify({ results, failures }, null, 2));
    assert(false, `${cases.length}개 창·화면 조합: ${failures.join(', ')}`);
    return;
  }

  await page.setViewport({ width: 1280, height: 900 });
  await open(page, 'page');
  await page.$eval(`${DIALOG} input[type=number]`, el => { el.value = '211'; el.dataset.viewportMarker = 'yes'; });
  for (const [width, height] of [[800, 360], [360, 640], [768, 320]]) {
    await page.setViewport({ width, height });
    const m = await measure(page);
    check(inside(m.frame, m.viewport) && m.actions.every(r => inside(r, m.viewport)), `live resize ${width}x${height}`);
    check(await page.$eval('[data-viewport-marker]', el => el.value === '211'), 'resize preserves input');
  }
  await page.keyboard.press('Escape');
  await page.waitForSelector(DIALOG, { hidden: true });
  await open(page, 'recovery');
  await page.$eval(`${DIALOG} .dialog-footer .dialog-btn:last-child`, button => button.focus());
  await page.keyboard.press('Enter');
  await page.waitForSelector(DIALOG, { hidden: true });
  check(await page.evaluate(() => window.__viewportChoice9133?.action === 'later'), 'Enter on recovery later never restores');
  await open(page, 'options');
  await page.mouse.click(2, 2);
  check(await page.$(DIALOG) !== null, 'input dialog ignores overlay click');
  await page.keyboard.press('Escape');
  await page.waitForSelector(DIALOG, { hidden: true });
  // 실제 F7 명령은 services/편집 라우터를 포함한다. 화면 보완 뒤에도 확인·Undo가 살아 있다.
  await createNewDocument(page);
  await page.setViewport({ width: 800, height: 360 });
  const originalGutter = await page.evaluate(() => window.__wasm.getPageDef(0).marginGutter);
  await page.keyboard.press('F7');
  await page.waitForSelector(DIALOG);
  await page.$eval(DIALOG, wrap => {
    const inputs = [...wrap.querySelectorAll('input[type=number]')];
    const gutter = inputs.at(-1);
    gutter.value = '1.5';
    gutter.dispatchEvent(new Event('change', { bubbles: true }));
  });
  await page.click(`${DIALOG} .dialog-btn-primary`);
  await page.waitForSelector(DIALOG, { hidden: true });
  check(await page.evaluate(() => window.__wasm.getPageDef(0).marginGutter === 425), 'F7 confirm applies 1.5mm gutter');
  await page.keyboard.down('Control'); await page.keyboard.press('z'); await page.keyboard.up('Control');
  await page.waitForFunction(value => window.__wasm.getPageDef(0).marginGutter === value, {}, originalGutter);
  check(true, 'actual undo restores gutter');
  await page.keyboard.down('Control'); await page.keyboard.down('Shift'); await page.keyboard.press('z');
  await page.keyboard.up('Shift'); await page.keyboard.up('Control');
  await page.waitForFunction(() => window.__wasm.getPageDef(0).marginGutter === 425);
  check(true, 'actual redo restores applied gutter');
  await page.keyboard.press('F7');
  await page.waitForSelector(DIALOG);
  await page.$eval(`${DIALOG} .dialog-btn:not(.dialog-btn-primary)`, button => button.focus());
  await page.keyboard.press('Enter');
  await page.waitForSelector(DIALOG, { hidden: true });
  check(await page.evaluate(() => window.__wasm.getPageDef(0).marginGutter === 425), 'Enter on cancel preserves document');
  // CDP로 터치만 활성화한다. setViewport(hasTouch)는 앱을 reload하므로 resize 검증과 분리한다.
  await page.setViewport({ width: 360, height: 640 });
  const touchSession = await page.createCDPSession();
  await touchSession.send('Emulation.setTouchEmulationEnabled', { enabled: true, maxTouchPoints: 1 });
  await open(page, 'options');
  const touch = (await measure(page)).actions.find(b => b.text === '취소');
  await page.touchscreen.tap((touch.left + touch.right) / 2, (touch.top + touch.bottom) / 2);
  await page.waitForSelector(DIALOG, { hidden: true });
  check(true, 'touch cancel closes portrait sheet');
  await touchSession.detach();
  writeFileSync(path.join(OUTPUT, 'measurements.json'), JSON.stringify({ results, failures }, null, 2));
  assert(failures.length === 0, `${cases.length}개 실제 창·화면 조합과 resize/취소 계약: ${failures.join(', ')}`);
});
