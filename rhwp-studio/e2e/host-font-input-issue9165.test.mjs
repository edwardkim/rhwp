import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { launchBrowser, createPage, closeBrowser, loadApp, createNewDocument } from './helpers.mjs';

const font = readFileSync(new URL('../../tests/fixtures/fonts/RHWPHostFixture-Regular.ttf', import.meta.url)).toString('base64');
const browser = await launchBrowser();
try {
  for (const catalog of ['none', 'empty', 'populated']) {
    const page = await createPage(browser);
    await loadApp(page, '?renderer=canvas2d');
    await createNewDocument(page);
    const result = await page.evaluate(async ({ catalog, font }) => {
      const w = window.__wasm, input = window.__inputHandler;
      w.insertText(0, 0, 0, 'A');
      w.insertPageBreak(0, 0, 1);
      const body = '가나다 ABC 0123 '.repeat(80);
      w.insertText(0, 1, 0, body);
      w.insertPageBreak(0, 1, body.length);
      w.insertText(0, 2, 0, body);
      const table = w.createTable(0, 0, 0, 1, 1);
      if (!table.ok || w.pageCount !== 3) throw Error('Invalid three-page fixture');
      w.insertTextInCell(0, table.paraIdx, table.controlIdx, 0, 0, 0, 'A');
      const family = 'RHWP Synthetic Input';
      w.applyCharFormatInCell(0, table.paraIdx, table.controlIdx, 0, 0, 0, 1,
        JSON.stringify({ fontId: w.findOrCreateFontId(family) }));
      const faceReads = [];
      if (catalog !== 'none') {
        await window.rhwpStudio.fonts.setProvider({
          getSnapshot: async () => ({ revision: '1', faces: catalog === 'empty' ? [] : [
            { id: 'fixture', family, fullName: family, postscriptName: 'RHWPHostFixture-Regular', style: 'Regular', weight: 400, slant: 'normal' },
            { id: 'alternate', family: 'RHWP Synthetic Alternate', fullName: 'RHWP Synthetic Alternate',
              postscriptName: 'RHWPAlternate-Regular', style: 'Regular', weight: 400, slant: 'normal' },
          ] }),
          subscribe: () => () => {},
          readFace: async id => { faceReads.push(id); return { bytes: Uint8Array.from(atob(font), c => c.charCodeAt(0)).buffer }; },
        }, {layout: false});
      }
      await window.__canvasView.loadDocument();
      input.cursor.moveTo({sectionIndex: 0, paragraphIndex: 0, charOffset: 1,
        parentParaIndex: table.paraIdx, controlIndex: table.controlIdx, cellIndex: 0, cellParaIndex: 0,
        cellPath: [{ controlIndex: table.controlIdx, cellIndex: 0, cellParaIndex: 0 }]});
      input.updateCaret();
      input.focus();
      const frame = () => new Promise(resolve => requestAnimationFrame(resolve));
      const insert = async () => {
        input.textarea.value = '가';
        input.textarea.dispatchEvent(new InputEvent('input', { bubbles: true, inputType: 'insertText', data: '가' }));
        await frame(); await frame();
      };
      await insert();
      const queriedPages = [];
      let prepares = 0, treeMs = 0;
      const tree = w.getPageLayerTreeObject.bind(w), prepare = w.prepareCanvasMetrics.bind(w);
      w.getPageLayerTreeObject = (...args) => {
        const start = performance.now();
        const result = tree(...args);
        treeMs += performance.now() - start;
        queriedPages.push(args[0]);
        return result;
      };
      w.prepareCanvasMetrics = (...args) => { prepares++; return prepare(...args); };
      const times = [], perInput = [];
      for (let i = 0; i < 20; i++) {
        const first = queriedPages.length;
        const start = performance.now();
        await insert();
        times.push(performance.now() - start);
        perInput.push(queriedPages.slice(first));
      }
      const text = w.getTextInCell(0, table.paraIdx, table.controlIdx, 0, 0, 0, 22);
      times.sort((a, b) => a - b);
      const timing = { catalog, trees: queriedPages.length, perInput, treeMs, prepares, reads: faceReads.length, medianMs: (times[9] + times[10]) / 2,
        text, cursor: input.cursor.getPosition().charOffset, pages: w.pageCount };
      const cell = () => w.getTextInCell(0, table.paraIdx, table.controlIdx, 0, 0, 0,
        w.getCellParagraphLength(0, table.paraIdx, table.controlIdx, 0, 0));
      const textarea = input.textarea;
      textarea.dispatchEvent(new CompositionEvent('compositionstart', { bubbles: true }));
      textarea.value = '한';
      textarea.dispatchEvent(new InputEvent('input', { bubbles: true, inputType: 'insertCompositionText',
        data: '한', isComposing: true }));
      textarea.dispatchEvent(new CompositionEvent('compositionend', { bubbles: true, data: '한' }));
      await frame(); await frame();
      const composed = cell();
      // Let the 300 ms contiguous-insert history group close before testing paste undo.
      await new Promise(resolve => setTimeout(resolve, 350));
      const clipboardData = new DataTransfer();
      clipboardData.setData('text/plain', 'Z\uE000');
      const paste = new ClipboardEvent('paste', { bubbles: true, cancelable: true, clipboardData });
      // Firefox supplies a separate DataTransfer for a constructed ClipboardEvent.
      Object.defineProperty(paste, 'clipboardData', { value: clipboardData });
      textarea.dispatchEvent(paste);
      await frame(); await frame();
      const pasted = cell();
      input.performUndo();
      await frame(); await frame();
      const undone = cell();
      input.performRedo();
      await frame(); await frame();
      const editing = { composed, pasted, undone, redone: cell(), finalPages: w.pageCount,
        finalCursor: input.cursor.getPosition().charOffset, finalPage: input.cursor.getRect().pageIndex };
      const geometry = { caret: input.cursor.getRect(), pages: [], cellStyle: w.getCellCharPropertiesAt(
        0, table.paraIdx, table.controlIdx, 0, 0, 1) };
      for (let pageIndex = 0; pageIndex < w.pageCount; pageIndex++) {
        const layer = tree(pageIndex);
        geometry.pages.push([layer.pageWidth, layer.pageHeight, layer.root]);
      }
      const layoutHash = Array.from(new Uint8Array(await crypto.subtle.digest('SHA-256',
        new TextEncoder().encode(JSON.stringify(geometry)))), b => b.toString(16).padStart(2, '0')).join('');
      if (catalog === 'populated') {
        w.applyCharFormat(0, 2, 0, 5, JSON.stringify({fontId: w.findOrCreateFontId('RHWP Synthetic Alternate')}));
        window.__eventBus.emit('document-changed');
        const deadline = performance.now() + 5000;
        while (!faceReads.includes('alternate') && performance.now() < deadline) await frame();
      }
      return { ...timing, ...editing, layoutHash, faceReads };
    }, {catalog, font});
    console.log(JSON.stringify(result));
    assert.equal(result.text, 'A' + '가'.repeat(21));
    assert.equal(result.composed, result.text + '한');
    assert.equal(result.pasted, result.composed + 'Z\uE000');
    assert.equal(result.undone, result.composed);
    assert.equal(result.redone, result.pasted);
    assert.equal(result.finalPages, 3);
    assert.equal(result.finalPage, 0);
    assert.equal(result.finalCursor, result.redone.length);
    assert.equal(result.cursor, 22);
    assert.equal(result.pages, 3);
    assert.equal(result.prepares, 20);
    if (catalog === 'populated') {
      assert.ok(result.reads > 0);
      assert.ok(result.faceReads.includes('alternate'), '새 스타일을 적용한 다른 페이지의 face 준비');
    }
    if (!process.argv.includes('--baseline')) {
      if (catalog === 'populated') {
        // A layout transition can require a full discovery pass before metrics settle.
        for (const pages of result.perInput.slice(-10)) assert.deepEqual(pages, [0]);
        assert.ok(result.trees < 60);
      }
      else assert.equal(result.trees, 0);
    }
    await page.close();
  }
} finally {
  await closeBrowser(browser);
}
