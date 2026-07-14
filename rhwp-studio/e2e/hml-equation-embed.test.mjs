import { resolve } from 'path';
import { readFileSync } from 'fs';

import { runTest, assert, screenshot } from './helpers.mjs';

const EDITOR_MODULE_URL = `/@fs${resolve(import.meta.dirname, '../../npm/editor/index.js')}`;
const FIXTURE_BYTES = Array.from(readFileSync(resolve(
  import.meta.dirname,
  '../../tests/fixtures/hml/exambank_math_equations_min.hml',
)));
const VITE_URL = process.env.VITE_URL || 'http://localhost:7700';

async function waitForFrame(page) {
  for (let attempt = 0; attempt < 100; attempt += 1) {
    const frame = page.frames().find((candidate) => candidate.parentFrame() === page.mainFrame());
    if (frame) return frame;
    await new Promise((delay) => setTimeout(delay, 50));
  }
  throw new Error('editor iframe was not attached');
}

async function waitForScript(page, expected) {
  let actual = null;
  for (let attempt = 0; attempt < 100; attempt += 1) {
    actual = await page.evaluate(() => {
      const frameWindow = window.__hmlEquationE2e.editor.element.contentWindow;
      return frameWindow.__wasm.getEquationProperties(0, 2, 0).script;
    });
    if (actual === expected) return;
    await new Promise((delay) => setTimeout(delay, 50));
  }
  throw new Error(`equation script did not become ${expected}; actual=${actual}`);
}

runTest('PR #2219 HML equation canvas edit/undo/export/reload', async ({ page }) => {
  const browserErrors = [];
  page.on('pageerror', (error) => browserErrors.push(String(error)));
  page.on('console', (message) => {
    if (message.type() === 'error') {
      const location = message.location();
      browserErrors.push(`${message.text()} @ ${location.url}:${location.lineNumber}`);
    }
  });

  await page.goto(`${VITE_URL}/e2e/embed-harness.html`, { waitUntil: 'domcontentloaded' });
  const before = await page.evaluate(async ({ editorModuleUrl, fixtureBytes }) => {
    const { createEditor } = await import(editorModuleUrl);
    const settleLoad = async (editor, promise) => {
      let settled = false;
      promise.finally(() => { settled = true; });
      for (let attempt = 0; attempt < 600 && !settled; attempt += 1) {
        const fallback = Array.from(
          editor.element.contentDocument?.querySelectorAll('button') ?? [],
        ).find((button) => button.textContent?.includes('대체 글꼴로 보기'));
        fallback?.click();
        await new Promise((delay) => setTimeout(delay, 100));
      }
      return promise;
    };
    const host = document.createElement('div');
    host.style.cssText = 'width: 100vw; height: 100vh';
    document.body.replaceChildren(host);
    const editor = await createEditor(host, {
      studioUrl: `${location.origin}/`,
      handshakeTimeoutMs: 10_000,
    });
    await settleLoad(editor, editor.loadFile(new Uint8Array(fixtureBytes), 'equations.hml'));
    window.__hmlEquationE2e = { editor, host, createEditor, settleLoad };
    const frameWindow = editor.element.contentWindow;
    return {
      scripts: [[2, 0], [3, 0], [12, 0], [13, 0]].map(([paragraph, control]) => (
        frameWindow.__wasm.getEquationProperties(0, paragraph, control).script
      )),
      saveState: await editor.getHmlSaveState(),
    };
  }, { editorModuleUrl: EDITOR_MODULE_URL, fixtureBytes: FIXTURE_BYTES });

  const clickPoint = await page.evaluate(() => {
    const editor = window.__hmlEquationE2e.editor;
    const frame = editor.element;
    const frameWindow = frame.contentWindow;
    const frameDocument = frame.contentDocument;
    const layoutRaw = frameWindow.__wasm.getPageControlLayout(0);
    const layout = typeof layoutRaw === 'string' ? JSON.parse(layoutRaw) : layoutRaw;
    const equation = layout.controls.find((control) => (
      control.type === 'equation' && control.paraIdx === 2 && control.controlIdx === 0
    ));
    if (!equation || equation.w <= 0 || equation.h <= 0) {
      throw new Error('first visible equation layout region was not found');
    }
    const scrollContent = frameDocument.querySelector('#scroll-content');
    const scroller = scrollContent.parentElement;
    const inputHandler = frameWindow.__inputHandler;
    const pageOffset = inputHandler.virtualScroll.getPageOffset(0);
    const pageLeft = inputHandler.virtualScroll.getPageLeftResolved(0, scrollContent.clientWidth);
    const zoom = inputHandler.viewportManager.getZoom();
    const docY = pageOffset + (equation.y + equation.h / 2) * zoom;
    scroller.scrollTop = Math.max(0, docY - scroller.clientHeight / 2);
    return new Promise((resolvePoint) => requestAnimationFrame(() => {
      const frameRect = frame.getBoundingClientRect();
      const scrollRect = scrollContent.getBoundingClientRect();
      resolvePoint({
        x: frameRect.left + scrollRect.left + pageLeft + (equation.x + equation.w / 2) * zoom,
        y: frameRect.top + scrollRect.top + pageOffset
          + (equation.y + equation.h / 2) * zoom,
      });
    }));
  });

  await new Promise((delay) => setTimeout(delay, 300));
  await page.mouse.click(clickPoint.x, clickPoint.y);
  const frame = await waitForFrame(page);
  await page.mouse.click(clickPoint.x, clickPoint.y, { button: 'right' });
  await frame.waitForSelector(
    '.context-menu [data-cmd="insert:equation-edit"]',
    { visible: true },
  );
  await frame.click('.context-menu [data-cmd="insert:equation-edit"]');
  await frame.waitForSelector('.eq-dialog .eq-script', { visible: true });
  const selected = await page.evaluate(() => {
    const selectedRef = window.__hmlEquationE2e.editor.element.contentWindow
      .__inputHandler.getSelectedPictureRef();
    return selectedRef && {
      type: selectedRef.type,
      paragraph: selectedRef.ppi,
      control: selectedRef.ci,
    };
  });
  assert(
    selected?.type === 'equation' && selected.paragraph === 2 && selected.control === 0,
    '실제 canvas click으로 첫 수식을 선택한다',
  );

  const scriptArea = await frame.$('.eq-dialog .eq-script');
  await scriptArea.click();
  await scriptArea.evaluate((element) => element.select());
  await scriptArea.type('x^2 + 2');
  const okButton = await frame.$('.eq-dialog .dialog-btn-primary');
  await okButton.click();
  await waitForScript(page, 'x^2 + 2');
  const editedState = await page.evaluate(() => window.__hmlEquationE2e.editor.getHmlSaveState());

  await frame.click('.menu-item[data-menu="edit"] .menu-title');
  await frame.click('.menu-item[data-menu="edit"] [data-cmd="edit:undo"]');
  await waitForScript(page, 'x^2 +1');
  const undoState = await page.evaluate(() => window.__hmlEquationE2e.editor.getHmlSaveState());

  await frame.click('.menu-item[data-menu="edit"] .menu-title');
  await frame.click('.menu-item[data-menu="edit"] [data-cmd="edit:redo"]');
  await waitForScript(page, 'x^2 + 2');
  const redoState = await page.evaluate(() => window.__hmlEquationE2e.editor.getHmlSaveState());

  const roundtrip = await page.evaluate(async () => {
    const state = window.__hmlEquationE2e;
    const exported = await state.editor.exportHml();
    const firstEditor = state.editor;
    const firstHost = state.host;
    const secondHost = document.createElement('div');
    secondHost.style.cssText = 'width: 100vw; height: 100vh';
    document.body.appendChild(secondHost);
    const editor = await state.createEditor(secondHost, {
      studioUrl: `${location.origin}/`,
      handshakeTimeoutMs: 10_000,
    });
    await state.settleLoad(editor, editor.loadFile(exported, 'roundtrip.hml'));
    state.previous = { editor: firstEditor, host: firstHost };
    state.editor = editor;
    state.host = secondHost;
    await new Promise((delay) => setTimeout(delay, 300));

    const frameWindow = editor.element.contentWindow;
    const frameDocument = editor.element.contentDocument;
    const wasm = frameWindow.__wasm;
    const inputHandler = frameWindow.__inputHandler;
    const scrollContent = frameDocument.querySelector('#scroll-content');
    const layoutRaw = wasm.getPageControlLayout(0);
    const layout = typeof layoutRaw === 'string' ? JSON.parse(layoutRaw) : layoutRaw;
    const regions = layout.controls
      .filter((control) => control.type === 'equation')
      .map((control) => ({
        x: control.x,
        y: control.y,
        width: control.w,
        height: control.h,
        paragraph: control.paraIdx,
        control: control.controlIdx,
      }));
    const nonOverlapping = regions.every((region, index) => regions.slice(index + 1).every((other) => (
      region.x + region.width <= other.x
        || other.x + other.width <= region.x
        || region.y + region.height <= other.y
        || other.y + other.height <= region.y
    )));
    const pageOffset = inputHandler.virtualScroll.getPageOffset(0);
    const pageLeft = inputHandler.virtualScroll.getPageLeftResolved(0, scrollContent.clientWidth);
    const zoom = inputHandler.viewportManager.getZoom();
    const canvases = Array.from(scrollContent.querySelectorAll('canvas'));
    const scrollRect = scrollContent.getBoundingClientRect();
    const inkCounts = regions.map((region) => {
      const left = pageLeft + region.x * zoom;
      const top = pageOffset + region.y * zoom;
      const right = left + region.width * zoom;
      const bottom = top + region.height * zoom;
      let ink = 0;
      for (const canvas of canvases) {
        const canvasRect = canvas.getBoundingClientRect();
        const canvasLeft = canvasRect.left - scrollRect.left;
        const canvasTop = canvasRect.top - scrollRect.top;
        const canvasRight = canvasLeft + canvasRect.width;
        const canvasBottom = canvasTop + canvasRect.height;
        const x1 = Math.max(left, canvasLeft);
        const y1 = Math.max(top, canvasTop);
        const x2 = Math.min(right, canvasRight);
        const y2 = Math.min(bottom, canvasBottom);
        if (x2 <= x1 || y2 <= y1) continue;
        const context = canvas.getContext('2d');
        if (!context) continue;
        const scaleX = canvas.width / canvasRect.width;
        const scaleY = canvas.height / canvasRect.height;
        const image = context.getImageData(
          Math.floor((x1 - canvasLeft) * scaleX),
          Math.floor((y1 - canvasTop) * scaleY),
          Math.max(1, Math.ceil((x2 - x1) * scaleX)),
          Math.max(1, Math.ceil((y2 - y1) * scaleY)),
        );
        for (let offset = 0; offset < image.data.length; offset += 4) {
          const red = image.data[offset];
          const green = image.data[offset + 1];
          const blue = image.data[offset + 2];
          const alpha = image.data[offset + 3];
          if (alpha > 0 && red + green + blue < 720) ink += 1;
        }
      }
      return ink;
    });
    return {
      scripts: regions.map((region) => (
        wasm.getEquationProperties(0, region.paragraph, region.control).script
      )),
      regions,
      inkCounts,
      nonOverlapping,
      saveState: await editor.getHmlSaveState(),
      exportedLength: exported.byteLength,
      documentLang: frameDocument.documentElement.lang,
      documentTitle: frameDocument.title,
    };
  });

  await screenshot(page, 'pr2219-hml-equation-roundtrip');
  const runtimeErrors = [...browserErrors];
  await page.evaluate(() => {
    window.__hmlEquationE2e.previous?.editor.destroy();
    window.__hmlEquationE2e.previous?.host.remove();
    window.__hmlEquationE2e.editor.destroy();
    window.__hmlEquationE2e.host.remove();
    delete window.__hmlEquationE2e;
  });

  assert(before.scripts.join('|') === 'x^2 +1|x^2 +1|3|3', 'repo fixture의 네 SCRIPT를 읽는다');
  assert(roundtrip.scripts.join('|') === 'x^2 + 2|x^2 +1|3|3', '편집과 미수정 SCRIPT가 public RPC 재로드 후 유지된다');
  assert(
    [before.saveState, editedState, undoState, redoState, roundtrip.saveState].every((state) => (
      state.sourceFormat === 'hml' && state.hmlSavable && state.blockers.length === 0
    )),
    'edit/undo/redo/export/reload 전체에서 canonical save-state가 유지된다',
  );
  assert(roundtrip.regions.length === 4, '재로드 후 네 수식 영역이 있다');
  assert(
    roundtrip.regions.every((region) => region.width > 0 && region.height > 0)
      && roundtrip.nonOverlapping,
    '네 수식 영역은 non-empty이고 서로 겹치지 않는다',
  );
  assert(
    roundtrip.inkCounts.every((count) => count > 0),
    `네 수식 영역에 실제 canvas ink가 있다: ${roundtrip.inkCounts.join(',')}`,
  );
  assert(roundtrip.exportedLength > 0, '부모 public exportHml RPC가 bytes를 반환한다');
  assert(roundtrip.documentLang === 'ko' && roundtrip.documentTitle, '임베드 문서 접근성 메타데이터가 있다');
  assert(runtimeErrors.length === 0, `런타임 브라우저 오류가 없다: ${runtimeErrors.join(' | ')}`);
}, { skipLoadApp: true });
