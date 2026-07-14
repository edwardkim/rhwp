async page => {
  const fixtureUrl = 'http://127.0.0.1:7701/tests/fixtures/hml/exambank_math_equations_min.hml';
  const editorModuleUrl = '/@fs/Users/chaeseong-gug/Documents/PARA/Resource/rhwp/.codex-worktrees/pr2219-hml-equation-export/npm/editor/index.js';
  const evidenceRoot = '/Users/chaeseong-gug/Documents/PARA/Resource/rhwp/.codex-worktrees/pr2219-hml-equation-export/docs/changelog/2026-07/13-pr-2219-hml-equation-export-implementation/qa';
  const errors = [];
  page.on('pageerror', error => errors.push(`pageerror:${String(error)}`));
  page.on('console', message => {
    if (message.type() === 'error') errors.push(`console:${message.text()}`);
  });

  const waitForScript = async expected => {
    await page.waitForFunction(value => {
      const frameWindow = window.__qa.editor.element.contentWindow;
      return frameWindow.__wasm.getEquationProperties(0, 2, 0).script === value;
    }, expected, { timeout: 10000 });
  };

  const fixtureResponse = await page.request.get(fixtureUrl);
  if (!fixtureResponse.ok()) throw new Error(`fixture HTTP ${fixtureResponse.status()}`);
  const fixtureBytes = Array.from(await fixtureResponse.body());
  await page.goto('http://127.0.0.1:7700/e2e/embed-harness.html', {
    waitUntil: 'domcontentloaded',
  });
  await page.evaluate(async ({ fixtureBytes, editorModuleUrl }) => {
    const fixture = new Uint8Array(fixtureBytes);
    const { createEditor } = await import(editorModuleUrl);
    const settleLoad = async (editor, promise) => {
      let settled = false;
      promise.finally(() => { settled = true; });
      for (let attempt = 0; attempt < 600 && !settled; attempt += 1) {
        const fallback = Array.from(
          editor.element.contentDocument?.querySelectorAll('button') ?? [],
        ).find(button => button.textContent?.includes('대체 글꼴로 보기'));
        fallback?.click();
        await new Promise(resolve => setTimeout(resolve, 100));
      }
      return promise;
    };
    const host = document.createElement('div');
    host.style.cssText = 'width:100vw;height:100vh';
    document.body.replaceChildren(host);
    const editor = await createEditor(host, {
      studioUrl: `${location.origin}/`,
      handshakeTimeoutMs: 10000,
    });
    await settleLoad(editor, editor.loadFile(fixture, 'equations.hml'));
    window.__qa = { editor, host, createEditor, settleLoad };
  }, { fixtureBytes, editorModuleUrl });

  const before = await page.evaluate(async () => {
    const frameWindow = window.__qa.editor.element.contentWindow;
    return {
      scripts: [[2, 0], [3, 0], [12, 0], [13, 0]].map(([paragraph, control]) =>
        frameWindow.__wasm.getEquationProperties(0, paragraph, control).script),
      saveState: await window.__qa.editor.getHmlSaveState(),
    };
  });
  if (before.scripts.join('|') !== 'x^2 +1|x^2 +1|3|3') {
    throw new Error(`initial scripts mismatch: ${before.scripts.join('|')}`);
  }
  await page.screenshot({
    path: `${evidenceRoot}/to-be-hml-equations.png`,
    fullPage: true,
  });

  const clickPoint = await page.evaluate(() => {
    const editor = window.__qa.editor;
    const frame = editor.element;
    const frameWindow = frame.contentWindow;
    const frameDocument = frame.contentDocument;
    const layoutRaw = frameWindow.__wasm.getPageControlLayout(0);
    const layout = typeof layoutRaw === 'string' ? JSON.parse(layoutRaw) : layoutRaw;
    const equation = layout.controls.find(control =>
      control.type === 'equation' && control.paraIdx === 2 && control.controlIdx === 0);
    if (!equation || equation.w <= 0 || equation.h <= 0) {
      throw new Error('first equation has no visible layout region');
    }
    const scrollContent = frameDocument.querySelector('#scroll-content');
    const scroller = scrollContent.parentElement;
    const inputHandler = frameWindow.__inputHandler;
    const pageOffset = inputHandler.virtualScroll.getPageOffset(0);
    const pageLeft = inputHandler.virtualScroll.getPageLeftResolved(0, scrollContent.clientWidth);
    const zoom = inputHandler.viewportManager.getZoom();
    const docY = pageOffset + (equation.y + equation.h / 2) * zoom;
    scroller.scrollTop = Math.max(0, docY - scroller.clientHeight / 2);
    return new Promise(resolve => requestAnimationFrame(() => {
      const frameRect = frame.getBoundingClientRect();
      const scrollRect = scrollContent.getBoundingClientRect();
      resolve({
        x: frameRect.left + scrollRect.left + pageLeft + (equation.x + equation.w / 2) * zoom,
        y: frameRect.top + scrollRect.top + pageOffset + (equation.y + equation.h / 2) * zoom,
      });
    }));
  });
  await page.waitForTimeout(300);
  await page.mouse.click(clickPoint.x, clickPoint.y);
  await page.mouse.click(clickPoint.x, clickPoint.y, { button: 'right' });
  const frame = page.frames().find(candidate => candidate.parentFrame() === page.mainFrame());
  if (!frame) throw new Error('editor iframe missing');
  await frame.locator('.context-menu [data-cmd="insert:equation-edit"]').click();
  await frame.locator('.eq-dialog .eq-script').fill('x^3 +2');
  await frame.locator('.eq-dialog .dialog-btn-primary').click();
  await waitForScript('x^3 +2');
  const selected = await page.evaluate(() => {
    const ref = window.__qa.editor.element.contentWindow.__inputHandler.getSelectedPictureRef();
    return ref && { type: ref.type, paragraph: ref.ppi, control: ref.ci };
  });
  if (selected?.type !== 'equation' || selected.paragraph !== 2 || selected.control !== 0) {
    throw new Error(`canvas selection mismatch: ${JSON.stringify(selected)}`);
  }
  const editedState = await page.evaluate(() => window.__qa.editor.getHmlSaveState());

  await frame.locator('.menu-item[data-menu="edit"] .menu-title').click();
  await frame.locator('.menu-item[data-menu="edit"] [data-cmd="edit:undo"]').click();
  await waitForScript('x^2 +1');
  const undoState = await page.evaluate(() => window.__qa.editor.getHmlSaveState());
  await frame.locator('.menu-item[data-menu="edit"] .menu-title').click();
  await frame.locator('.menu-item[data-menu="edit"] [data-cmd="edit:redo"]').click();
  await waitForScript('x^3 +2');
  const redoState = await page.evaluate(() => window.__qa.editor.getHmlSaveState());

  const roundtrip = await page.evaluate(async () => {
    const state = window.__qa;
    const exported = await state.editor.exportHml();
    const secondHost = document.createElement('div');
    secondHost.style.cssText = 'width:100vw;height:100vh';
    document.body.appendChild(secondHost);
    const second = await state.createEditor(secondHost, {
      studioUrl: `${location.origin}/`,
      handshakeTimeoutMs: 10000,
    });
    await state.settleLoad(second, second.loadFile(exported, 'roundtrip.hml'));
    state.previous = { editor: state.editor, host: state.host };
    state.host.style.display = 'none';
    state.editor = second;
    state.host = secondHost;
    await new Promise(resolve => setTimeout(resolve, 300));

    const frameWindow = second.element.contentWindow;
    const frameDocument = second.element.contentDocument;
    const wasm = frameWindow.__wasm;
    const inputHandler = frameWindow.__inputHandler;
    const scrollContent = frameDocument.querySelector('#scroll-content');
    const layoutRaw = wasm.getPageControlLayout(0);
    const layout = typeof layoutRaw === 'string' ? JSON.parse(layoutRaw) : layoutRaw;
    const regions = layout.controls.filter(control => control.type === 'equation').map(control => ({
      x: control.x, y: control.y, width: control.w, height: control.h,
      paragraph: control.paraIdx, control: control.controlIdx,
    }));
    const nonOverlapping = regions.every((region, index) => regions.slice(index + 1).every(other =>
      region.x + region.width <= other.x || other.x + other.width <= region.x ||
      region.y + region.height <= other.y || other.y + other.height <= region.y));
    const pageOffset = inputHandler.virtualScroll.getPageOffset(0);
    const pageLeft = inputHandler.virtualScroll.getPageLeftResolved(0, scrollContent.clientWidth);
    const zoom = inputHandler.viewportManager.getZoom();
    const canvases = Array.from(scrollContent.querySelectorAll('canvas'));
    const scrollRect = scrollContent.getBoundingClientRect();
    const inkCounts = regions.map(region => {
      const left = pageLeft + region.x * zoom;
      const top = pageOffset + region.y * zoom;
      const right = left + region.width * zoom;
      const bottom = top + region.height * zoom;
      let ink = 0;
      for (const canvas of canvases) {
        const rect = canvas.getBoundingClientRect();
        const canvasLeft = rect.left - scrollRect.left;
        const canvasTop = rect.top - scrollRect.top;
        const x1 = Math.max(left, canvasLeft);
        const y1 = Math.max(top, canvasTop);
        const x2 = Math.min(right, canvasLeft + rect.width);
        const y2 = Math.min(bottom, canvasTop + rect.height);
        if (x2 <= x1 || y2 <= y1) continue;
        const context = canvas.getContext('2d');
        if (!context) continue;
        const scaleX = canvas.width / rect.width;
        const scaleY = canvas.height / rect.height;
        const image = context.getImageData(
          Math.floor((x1 - canvasLeft) * scaleX), Math.floor((y1 - canvasTop) * scaleY),
          Math.max(1, Math.ceil((x2 - x1) * scaleX)),
          Math.max(1, Math.ceil((y2 - y1) * scaleY)),
        );
        for (let offset = 0; offset < image.data.length; offset += 4) {
          if (image.data[offset + 3] > 0 &&
              image.data[offset] + image.data[offset + 1] + image.data[offset + 2] < 720) ink += 1;
        }
      }
      return ink;
    });
    return {
      scripts: regions.map(region => wasm.getEquationProperties(
        0, region.paragraph, region.control).script),
      regions,
      inkCounts,
      nonOverlapping,
      saveState: await second.getHmlSaveState(),
      exportedLength: exported.byteLength,
      lang: frameDocument.documentElement.lang,
      title: frameDocument.title,
    };
  });
  await page.screenshot({
    path: `${evidenceRoot}/to-be-hml-roundtrip.png`,
    fullPage: true,
  });

  const states = [before.saveState, editedState, undoState, redoState, roundtrip.saveState];
  if (!states.every(state => state.sourceFormat === 'hml' && state.hmlSavable && state.blockers.length === 0)) {
    throw new Error(`save-state drift: ${JSON.stringify(states)}`);
  }
  if (roundtrip.scripts.join('|') !== 'x^3 +2|x^2 +1|3|3') {
    throw new Error(`roundtrip scripts mismatch: ${roundtrip.scripts.join('|')}`);
  }
  if (roundtrip.regions.length !== 4 ||
      !roundtrip.regions.every(region => region.width > 0 && region.height > 0) ||
      !roundtrip.nonOverlapping || !roundtrip.inkCounts.every(count => count > 0)) {
    throw new Error(`visual equation assertion failed: ${JSON.stringify(roundtrip)}`);
  }
  if (roundtrip.exportedLength <= 0 || roundtrip.lang !== 'ko' || !roundtrip.title || errors.length) {
    throw new Error(`runtime/accessibility assertion failed: ${JSON.stringify({ roundtrip, errors })}`);
  }
  return {
    verdict: 'PASS',
    beforeScripts: before.scripts,
    roundtripScripts: roundtrip.scripts,
    saveStates: states,
    regions: roundtrip.regions,
    inkCounts: roundtrip.inkCounts,
    exportedLength: roundtrip.exportedLength,
    accessibility: { lang: roundtrip.lang, title: roundtrip.title },
    runtimeErrors: errors,
    actionCount: 16,
  };
}
