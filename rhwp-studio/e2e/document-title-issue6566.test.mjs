/** #6566: 실제 문서 열기/실패/새 문서/저장과 보조 bridge의 창 제목 소유권. */
import { runTest, assert } from './helpers.mjs';

async function expectTitle(page, expected) {
  try {
    await page.waitForFunction(value => document.title === value, { timeout: 10000 }, expected);
  } catch { throw new Error(`title expected=${expected}, actual=${await page.title()}`); }
  assert(await page.title() === expected, `창 제목: ${expected}`);
}

async function fileCommand(page, command) {
  await page.click('#menu-bar .menu-item[data-menu="file"] .menu-title');
  await page.waitForSelector('#menu-bar .menu-item[data-menu="file"].open');
  await page.click(`.md-item[data-cmd="${command}"]`);
}

runTest('문서 파일명과 창 제목 (#6566)', async ({ page }) => {
  await page.evaluate(() => localStorage.setItem('rhwp-settings', JSON.stringify({
    version: 1, theme: { mode: 'system', skin: 'default', skinChosen: true },
  })));
  const appUrl = new URL(page.url());
  await page.goto(`${appUrl.origin}/?chrome=embed`, { waitUntil: 'domcontentloaded' });
  await page.waitForFunction(() => !!window.__canvasView);
  await expectTitle(page, 'rhwp-studio');

  const failedInitial = await page.evaluate(() => {
    try { window.__wasm.loadDocument(new Uint8Array([1, 2, 3]), '손상.hwp'); }
    catch { return true; }
    return false;
  });
  assert(failedInitial, '손상 파일 로드 실패를 실제 WASM에서 확인');
  await expectTitle(page, 'rhwp-studio');

  await page.goto(appUrl.origin, { waitUntil: 'domcontentloaded' });
  await page.waitForFunction(() => !!window.__canvasView);
  await expectTitle(page, '새 문서.hwp - rhwp-studio');

  const opened = await page.evaluate(async () => {
    const response = await fetch('/samples/para-001.hwp');
    if (!response.ok) throw new Error(`fixture HTTP ${response.status}`);
    const bytes = new Uint8Array(await response.arrayBuffer());
    const requestId = 'title-6566';
    const done = new Promise(resolve => {
      const off = window.__eventBus.on('open-document-bytes:done', payload => {
        if (payload.requestId !== requestId) return;
        off(); resolve(payload);
      });
    });
    window.__eventBus.emit('open-document-bytes', {
      bytes, fileName: '검토 <원본> & 001.hwp', fileHandle: null,
      skipUnsavedGuard: true, requestId,
    });
    return done;
  });
  assert(opened.ok, '실제 열기 경로 성공');
  await expectTitle(page, '검토 <원본> & 001.hwp - rhwp-studio');

  const retained = await page.evaluate(() => {
    const generation = window.__wasm.documentGeneration;
    try { window.__wasm.loadDocument(new Uint8Array([1, 2, 3]), '실패.hwp'); }
    catch { return window.__wasm.documentGeneration === generation; }
    return false;
  });
  assert(retained, '두 번째 파일 로드 실패 시 기존 문서 보존');
  await expectTitle(page, '검토 <원본> & 001.hwp - rhwp-studio');

  await page.evaluate(async () => {
    const { WasmBridge } = await import('/src/core/wasm-bridge.ts');
    const auxiliary = new WasmBridge();
    await auxiliary.initialize();
    auxiliary.createNewDocument();
    auxiliary.fileName = '비교 전용.hwp';
    auxiliary.releaseDocument();
  });
  await expectTitle(page, '검토 <원본> & 001.hwp - rhwp-studio');

  await page.evaluate(() => {
    window.__titleSaveWritten = false;
    window.showSaveFilePicker = async () => ({
      name: '다른 이름.hwpx',
      createWritable: async () => ({
        write: async blob => { window.__titleSaveWritten = blob.size > 0; },
        close: async () => {},
      }),
    });
  });
  await fileCommand(page, 'file:save-as-hwpx');
  await page.waitForSelector('.dialog-body input[type="text"]');
  await page.click('.dialog-footer .dialog-btn-primary');
  await expectTitle(page, '다른 이름.hwpx - rhwp-studio');
  assert(await page.evaluate(() => window.__titleSaveWritten), '다른 이름 저장이 실제 바이트를 씀');
  await page.evaluate(() => window.rhwpStudio.notifySaved('호스트 저장.hwp'));
  await expectTitle(page, '호스트 저장.hwp - rhwp-studio');

  await fileCommand(page, 'file:new-doc');
  await expectTitle(page, '새 문서.hwp - rhwp-studio');
});
