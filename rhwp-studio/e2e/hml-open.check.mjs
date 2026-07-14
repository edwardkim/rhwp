/**
 * Standalone HML browser regression.
 *
 * Run with a Vite dev server on port 7700:
 *   CHROME_PATH="/path/to/chrome" node e2e/hml-open.check.mjs --mode=headless
 */
import { createHash } from 'node:crypto';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import {
  assert,
  runTest,
  screenshot,
  setTestCase,
  waitForCanvas,
} from './helpers.mjs';

const E2E_DIR = path.dirname(fileURLToPath(import.meta.url));
const REPO_ROOT = path.resolve(E2E_DIR, '..', '..');
const HML_FIXTURE = path.join(REPO_ROOT, 'samples', 'hml', 'formatting_table.hml');
const HWP_FIXTURE = path.join(REPO_ROOT, 'samples', 'field-01.hwp');

function sha256(filePath) {
  return createHash('sha256').update(readFileSync(filePath)).digest('hex');
}

async function uploadDocument(page, filePath, expectedFormat) {
  const input = await page.$('#file-input');
  if (!input) throw new Error('file-input not found');
  await input.uploadFile(filePath);
  await page.waitForFunction(
    (format) => window.__wasm?.getSourceFormat?.() === format
      && (window.__wasm?.pageCount ?? 0) > 0,
    { timeout: 30000 },
    expectedFormat,
  );
  await waitForCanvas(page, 30000);
}

runTest('HML open, semantic HML save, reopen, and HWP regression', async ({ page }) => {
  const sourceHashBefore = sha256(HML_FIXTURE);

  setTestCase('HML file input open and render');
  await uploadDocument(page, HML_FIXTURE, 'hml');

  const renderState = await page.evaluate(() => {
    const canvas = document.querySelector('#scroll-container canvas');
    const rect = canvas?.getBoundingClientRect();
    const svg = window.__wasm?.renderPageSvg?.(0) ?? '';
    const paragraphCount = window.__wasm?.getParagraphCount?.(0) ?? 0;
    const bodyText = Array.from({ length: paragraphCount }, (_, paragraphIndex) => {
      try {
        return window.__wasm?.getTextRange?.(0, paragraphIndex, 0, 1000) ?? '';
      } catch {
        return '';
      }
    }).join('\n');
    return {
      pageCount: window.__wasm?.pageCount ?? 0,
      sourceFormat: window.__wasm?.getSourceFormat?.() ?? '',
      fileName: window.__wasm?.fileName ?? '',
      canvasVisible: !!rect && rect.width > 0 && rect.height > 0,
      renderHas123: svg.includes('123') || bodyText.includes('123'),
      renderHasAbc: svg.includes('abc') || bodyText.includes('abc'),
      renderHasEfg: svg.includes('efg') || bodyText.includes('efg'),
    };
  });

  assert(renderState.sourceFormat === 'hml', `sourceFormat=hml (${renderState.sourceFormat})`);
  assert(renderState.pageCount >= 1, `HML page count >= 1 (${renderState.pageCount})`);
  assert(renderState.canvasVisible, 'first HML page canvas is visible');
  assert(
    renderState.renderHas123 && renderState.renderHasAbc && renderState.renderHasEfg,
    'visible page and shared document text preserve core fixture text: 123, abc, efg',
  );

  setTestCase('HML import warning');
  await page.waitForFunction(
    () => document.getElementById('rhwp-toast-container')?.textContent?.includes('HML'),
    { timeout: 10000 },
  );
  const warningText = await page.$eval(
    '#rhwp-toast-container',
    (element) => element.textContent ?? '',
  );
  assert(warningText.includes('HML 2.91 문서를 열었습니다'), `import warning shown (${warningText})`);
  assert(
    warningText.includes('의미를 보존해 저장') && warningText.includes('원본 바이트와 동일하지는 않습니다'),
    'import warning explains semantic save without byte identity',
  );
  await screenshot(page, 'hml-open-warning');

  await page.evaluate(() => {
    const buttons = [...document.querySelectorAll('#rhwp-toast-container button')];
    const confirm = buttons.find((button) => button.textContent?.trim() === '확인');
    confirm?.click();
  });

  setTestCase('edit, Ctrl+S, and HML-default semantic save');
  await page.evaluate(() => {
    window.__wasm.insertText(0, 0, 0, 'HML_EDIT_');
    window.__hmlSaveProbe = { pickerCalls: 0, writes: 0, blob: null, options: null };
    window.showSaveFilePicker = async (options) => {
      window.__hmlSaveProbe.pickerCalls += 1;
      window.__hmlSaveProbe.options = options;
      return {
        kind: 'file',
        name: 'saved.hml',
        async getFile() { return new File([], 'saved.hml'); },
        async createWritable() {
          return {
            async write(blob) {
              window.__hmlSaveProbe.writes += 1;
              window.__hmlSaveProbe.blob = blob;
            },
            async close() {},
          };
        },
      };
    };
  });

  await page.keyboard.down('Control');
  await page.keyboard.press('KeyS');
  await page.keyboard.up('Control');
  await page.waitForFunction(
    () => [...document.querySelectorAll('.dialog-title')]
      .some((element) => element.textContent?.includes('HML 문서 저장')),
    { timeout: 10000 },
  );

  const saveDialog = await page.evaluate(() => {
    const overlay = [...document.querySelectorAll('.modal-overlay')]
      .find((element) => element.querySelector('.dialog-title')?.textContent?.includes('HML 문서 저장'));
    const buttons = [...(overlay?.querySelectorAll('button') ?? [])]
      .map((button) => button.textContent?.trim() ?? '');
    return {
      text: overlay?.textContent ?? '',
      buttons,
      probe: window.__hmlSaveProbe,
    };
  });

  assert(saveDialog.text.includes('의미를 보존해 저장'), 'dialog explains semantic HML save');
  assert(saveDialog.buttons.includes('HML로 저장'), 'HML is the primary save option');
  assert(saveDialog.buttons.includes('HWP로 저장'), 'HWP Save As option is present');
  assert(saveDialog.buttons.includes('HWPX로 저장'), 'HWPX Save As option is present');
  assert(saveDialog.probe.pickerCalls === 0, 'save picker is not opened before format selection');
  assert(saveDialog.probe.writes === 0, 'no file write occurs before format selection');
  assert(sha256(HML_FIXTURE) === sourceHashBefore, 'uploaded HML fixture remains byte-for-byte unchanged');
  await screenshot(page, 'hml-save-format-prompt');

  await page.evaluate(() => {
    const overlay = [...document.querySelectorAll('.modal-overlay')]
      .find((element) => element.querySelector('.dialog-title')?.textContent?.includes('HML 문서 저장'));
    const saveHml = [...(overlay?.querySelectorAll('button') ?? [])]
      .find((button) => button.textContent?.trim() === 'HML로 저장');
    saveHml?.click();
  });
  await page.waitForFunction(() => window.__hmlSaveProbe?.blob !== null, { timeout: 10000 });

  const savedHml = await page.evaluate(async () => {
    const blob = window.__hmlSaveProbe.blob;
    const bytes = new Uint8Array(await blob.arrayBuffer());
    const xml = new TextDecoder().decode(bytes);
    const info = window.__wasm.loadDocument(bytes, 'saved.hml');
    const metadata = window.__wasm.getHmlOpenMetadata();
    const text = window.__wasm.getTextRange(0, 0, 0, 1000);
    return {
      type: blob.type,
      pickerCalls: window.__hmlSaveProbe.pickerCalls,
      writes: window.__hmlSaveProbe.writes,
      pickerTypes: window.__hmlSaveProbe.options?.types,
      sourceFormat: window.__wasm.getSourceFormat(),
      pageCount: info?.pageCount ?? 0,
      hasEdit: text.includes('HML_EDIT_'),
      hasScriptCode: xml.includes('<SCRIPTCODE'),
      warningPaths: metadata?.warnings?.map((warning) => warning.xmlPath) ?? [],
    };
  });

  assert(savedHml.type === 'application/xml', `HML MIME is application/xml (${savedHml.type})`);
  assert(savedHml.pickerCalls === 1 && savedHml.writes === 1, 'HML save uses one picker and one write');
  assert(savedHml.pickerTypes?.[0]?.accept?.['application/xml']?.includes('.hml'), 'picker requests .hml');
  assert(savedHml.sourceFormat === 'hml' && savedHml.pageCount >= 1, 'saved bytes reopen as HML');
  assert(savedHml.hasEdit, 'saved HML retains the applied edit');
  assert(savedHml.hasScriptCode, 'saved HML retains SCRIPTCODE');
  assert(savedHml.warningPaths.includes('/HWPML/TAIL/SCRIPTCODE'), 'reopened HML retains warning path');
  assert(sha256(HML_FIXTURE) === sourceHashBefore, 'uploaded HML fixture remains byte-for-byte unchanged');

  setTestCase('existing HWP file-input regression');
  await uploadDocument(page, HWP_FIXTURE, 'hwp');
  await page.waitForFunction(
    () => document.getElementById('sb-message')?.textContent?.includes('field-01.hwp'),
    { timeout: 30000 },
  );
  const hwpState = await page.evaluate(() => ({
    sourceFormat: window.__wasm?.getSourceFormat?.() ?? '',
    pageCount: window.__wasm?.pageCount ?? 0,
    statusText: document.getElementById('sb-message')?.textContent ?? '',
  }));
  assert(hwpState.sourceFormat === 'hwp', `existing HWP still opens (${hwpState.sourceFormat})`);
  assert(hwpState.pageCount >= 1, `existing HWP page count >= 1 (${hwpState.pageCount})`);
  assert(hwpState.statusText.includes('field-01.hwp'), `status names opened HWP (${hwpState.statusText})`);
});
