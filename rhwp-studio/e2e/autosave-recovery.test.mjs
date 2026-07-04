/**
 * Task #1448 — 미저장 문서 자동 백업 복구 E2E
 *
 * 실행:
 *   CHROME_PATH="/Applications/Google Chrome.app/Contents/MacOS/Google Chrome" \
 *   VITE_URL=http://localhost:7700 \
 *   node e2e/autosave-recovery.test.mjs --mode=headless
 */
import {
  runTest,
  waitForCanvas,
  screenshot,
  assert,
  setTestCase,
} from './helpers.mjs';

const SAMPLE_HWP = '셀보호2.hwp';
const SAMPLE_HWPX = '셀보호2.hwpx';
const APP_URL = process.env.VITE_URL || 'http://localhost:7700';

function sampleUrl(filename) {
  return `/samples/${filename.split('/').map(encodeURIComponent).join('/')}`;
}

async function clearAutosaveDb(page) {
  await page.evaluate(async () => {
    const req = indexedDB.deleteDatabase('rhwpStudioAutosave');
    await new Promise((resolve) => {
      req.onsuccess = req.onerror = req.onblocked = () => resolve();
    });
  });
}

async function putDraft(page, draft) {
  await page.evaluate(async (input) => {
    const req = indexedDB.open('rhwpStudioAutosave', 1);
    const db = await new Promise((resolve, reject) => {
      req.onupgradeneeded = () => {
        const nextDb = req.result;
        if (!nextDb.objectStoreNames.contains('drafts')) {
          nextDb.createObjectStore('drafts', { keyPath: 'id' });
        }
      };
      req.onerror = () => reject(req.error);
      req.onsuccess = () => resolve(req.result);
    });
    await new Promise((resolve, reject) => {
      const tx = db.transaction('drafts', 'readwrite');
      tx.objectStore('drafts').put({
        ...input,
        data: new Uint8Array(input.data).buffer,
      });
      tx.oncomplete = () => resolve();
      tx.onerror = () => reject(tx.error);
    });
    db.close();
  }, draft);
}

async function draftExists(page, id) {
  return await page.evaluate(async (draftId) => {
    const req = indexedDB.open('rhwpStudioAutosave', 1);
    const db = await new Promise((resolve, reject) => {
      req.onupgradeneeded = () => {
        const nextDb = req.result;
        if (!nextDb.objectStoreNames.contains('drafts')) {
          nextDb.createObjectStore('drafts', { keyPath: 'id' });
        }
      };
      req.onerror = () => reject(req.error);
      req.onsuccess = () => resolve(req.result);
    });
    const found = await new Promise((resolve, reject) => {
      const tx = db.transaction('drafts', 'readonly');
      const getReq = tx.objectStore('drafts').get(draftId);
      getReq.onsuccess = () => resolve(Boolean(getReq.result));
      getReq.onerror = () => reject(getReq.error);
    });
    db.close();
    return found;
  }, id);
}

async function fetchSampleBytes(page, filename) {
  return await page.evaluate(async (url) => {
    const resp = await fetch(url);
    if (!resp.ok) throw new Error(`HTTP ${resp.status}`);
    return Array.from(new Uint8Array(await resp.arrayBuffer()));
  }, sampleUrl(filename));
}

async function navigateApp(page, search = '') {
  await page.goto(`${APP_URL}${search}`, { waitUntil: 'domcontentloaded', timeout: 30000 });
  await page.waitForFunction(() => !!window.__wasm && !!window.__canvasView, { timeout: 15000 });
  await page.evaluate(() => new Promise(r => setTimeout(r, 500)));
}

async function exportHwpFromSample(page, filename) {
  return await page.evaluate(async ({ fname, url }) => {
    const resp = await fetch(url);
    if (!resp.ok) throw new Error(`HTTP ${resp.status}`);
    const bytes = new Uint8Array(await resp.arrayBuffer());
    window.__wasm.loadDocument(bytes, fname);
    return Array.from(window.__wasm.exportHwp());
  }, { fname: filename, url: sampleUrl(filename) });
}

async function exportNewDocument(page) {
  return await page.evaluate(() => {
    window.__wasm.createNewDocument();
    return {
      fileName: window.__wasm.fileName || '새 문서.hwp',
      sourceFormat: window.__wasm.getSourceFormat(),
      data: Array.from(window.__wasm.exportHwp()),
    };
  });
}

async function openAndRestore(page, expectedFileNamePart) {
  await navigateApp(page);
  await page.waitForSelector('.modal-overlay .dialog-wrap', { timeout: 5000 });
  const dialogText = await page.$eval('.modal-overlay .dialog-wrap', el => el.textContent || '');
  assert(dialogText.includes('문서 복구'), '복구 대화상자 표시');
  assert(dialogText.includes('원본 파일을 자동으로 덮어쓰지 않습니다'), '원본 자동 덮어쓰기 아님 안내 표시');
  assert(dialogText.includes(expectedFileNamePart), `복구 후보 파일명 표시 (${expectedFileNamePart})`);

  await page.evaluate(() => {
    const button = Array.from(document.querySelectorAll('.modal-overlay .dialog-btn'))
      .find((btn) => (btn.textContent || '').trim() === '복구');
    if (!button) throw new Error('복구 버튼을 찾을 수 없습니다');
    button.click();
  });
  await waitForCanvas(page, 10000);
  await page.evaluate(() => new Promise(r => setTimeout(r, 800)));
}

runTest('Task #1448 자동 백업 복구', async ({ page }) => {
  page.on('dialog', async (dialog) => {
    await dialog.accept();
  });

  setTestCase('TC-0: 초기화와 기존 복구 DB 정리');
  await navigateApp(page, `?url=${encodeURIComponent(sampleUrl(SAMPLE_HWP))}&filename=${encodeURIComponent(SAMPLE_HWP)}`);
  await clearAutosaveDb(page);

  setTestCase('TC-1: 새 문서 draft 복구');
  const newDocument = await exportNewDocument(page);
  await putDraft(page, {
    id: 'e2e-new-draft',
    fileName: newDocument.fileName,
    sourceFormat: newDocument.sourceFormat,
    savedAt: Date.now(),
    byteLength: newDocument.data.length,
    data: newDocument.data,
    dirtyReason: 'e2e-new-document',
  });
  await openAndRestore(page, newDocument.fileName);
  const newDocState = await page.evaluate(() => ({
    fileName: window.__wasm.fileName,
    pageCount: window.__wasm.pageCount,
    isDirty: window.__documentState.isDirty(),
  }));
  assert(newDocState.pageCount >= 1, `새 문서 복구본 페이지 수 확인 (${newDocState.pageCount})`);
  assert(newDocState.fileName.includes('복구본') && newDocState.fileName.endsWith('.hwp'),
    `새 문서 복구본 파일명 확인 (${newDocState.fileName})`);
  assert(newDocState.isDirty === true, '새 문서 복구본은 저장 전 dirty 상태 유지');
  assert(!await draftExists(page, 'e2e-new-draft'), '복구 성공 후 원본 새 문서 draft 삭제');
  await screenshot(page, 'autosave-recovery-new-document');
  await page.evaluate(() => window.__documentState?.markClean?.('e2e-next-case'));

  setTestCase('TC-2: HWP draft 복구');
  await clearAutosaveDb(page);
  const hwpBytes = await fetchSampleBytes(page, SAMPLE_HWP);
  await putDraft(page, {
    id: 'e2e-hwp-draft',
    fileName: SAMPLE_HWP,
    sourceFormat: 'hwp',
    savedAt: Date.now(),
    byteLength: hwpBytes.length,
    data: hwpBytes,
    dirtyReason: 'e2e-hwp',
  });
  await openAndRestore(page, SAMPLE_HWP);
  const hwpState = await page.evaluate(() => ({
    fileName: window.__wasm.fileName,
    pageCount: window.__wasm.pageCount,
    isDirty: window.__documentState.isDirty(),
  }));
  assert(hwpState.pageCount >= 1, `HWP 복구본 페이지 수 확인 (${hwpState.pageCount})`);
  assert(hwpState.fileName.includes('복구본') && hwpState.fileName.endsWith('.hwp'),
    `HWP 복구본 파일명 확인 (${hwpState.fileName})`);
  assert(hwpState.isDirty === true, '복구본은 저장 전 dirty 상태 유지');
  assert(!await draftExists(page, 'e2e-hwp-draft'), '복구 성공 후 원본 HWP draft 삭제');
  await screenshot(page, 'autosave-recovery-hwp');
  await page.evaluate(() => window.__documentState?.markClean?.('e2e-next-case'));

  setTestCase('TC-3: HWPX 출처 draft는 HWP 복구본으로 열린다');
  await clearAutosaveDb(page);
  await navigateApp(page, `?url=${encodeURIComponent(sampleUrl(SAMPLE_HWPX))}&filename=${encodeURIComponent(SAMPLE_HWPX)}`);
  const hwpxAsHwpBytes = await exportHwpFromSample(page, SAMPLE_HWPX);
  await putDraft(page, {
    id: 'e2e-hwpx-draft',
    fileName: SAMPLE_HWPX,
    sourceFormat: 'hwpx',
    savedAt: Date.now(),
    byteLength: hwpxAsHwpBytes.length,
    data: hwpxAsHwpBytes,
    dirtyReason: 'e2e-hwpx',
  });
  await openAndRestore(page, SAMPLE_HWPX);
  const hwpxState = await page.evaluate(() => ({
    fileName: window.__wasm.fileName,
    sourceFormat: window.__wasm.getSourceFormat(),
    pageCount: window.__wasm.pageCount,
    isDirty: window.__documentState.isDirty(),
  }));
  assert(hwpxState.pageCount >= 1, `HWPX 출처 복구본 페이지 수 확인 (${hwpxState.pageCount})`);
  assert(hwpxState.fileName.includes('복구본') && hwpxState.fileName.endsWith('.hwp') && !hwpxState.fileName.endsWith('.hwpx'),
    `HWPX 출처 복구본은 .hwp 파일명 (${hwpxState.fileName})`);
  assert(hwpxState.sourceFormat === 'hwp', `복구 데이터는 HWP로 로드됨 (${hwpxState.sourceFormat})`);
  assert(hwpxState.isDirty === true, 'HWPX 출처 복구본도 저장 전 dirty 상태 유지');
  assert(!await draftExists(page, 'e2e-hwpx-draft'), '복구 성공 후 원본 HWPX draft 삭제');
  await screenshot(page, 'autosave-recovery-hwpx');
});
