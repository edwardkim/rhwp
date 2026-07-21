/**
 * Task #2660 — 호스트 저장 완료 통지 (notifySaved) E2E
 *
 * TC-1: SDK export → notifySaved → dirty 해제 + draft 삭제, 재시작 시 복구 다이얼로그 없음
 * TC-2: notifySaved 생략 → 재시작 시 복구 다이얼로그 표시 (음성 대조)
 * TC-3: rhwp-connected가 notify-saved-v1 capability를 광고
 * TC-4: window.rhwpStudio.notifySaved (팝업/포크 통합 표면)
 *
 * 실행:
 *   CHROME_PATH="..." VITE_URL=http://localhost:7700 \
 *   node e2e/embed-save-ack.test.mjs --mode=headless
 */
import { resolve } from 'path';

import { runTest, assert, setTestCase } from './helpers.mjs';

// Windows 절대 경로(D:\...)도 Vite /@fs URL로 변환되도록 정규화한다
const EDITOR_MODULE_PATH = resolve(import.meta.dirname, '../../npm/editor/index.js').replace(/\\/g, '/');
const EDITOR_MODULE_URL = EDITOR_MODULE_PATH.startsWith('/')
  ? `/@fs${EDITOR_MODULE_PATH}`
  : `/@fs/${EDITOR_MODULE_PATH}`;
const VITE_URL = process.env.VITE_URL || 'http://localhost:7700';
const SAMPLE_URL = '/samples/footnote-01.hwp';

runTest('Task #2660 호스트 저장 완료 통지 (notifySaved)', async ({ page }) => {
  page.on('dialog', (dialog) => dialog.accept().catch(() => {}));

  setTestCase('Part A: SDK iframe — TC-1/2/3');
  await page.goto(`${VITE_URL}/@vite/client`, { waitUntil: 'domcontentloaded' });
  const partA = await page.evaluate(async ({ editorModuleUrl, sampleUrl }) => {
    const { createEditor } = await import(editorModuleUrl);
    const host = document.createElement('div');
    host.style.cssText = 'width: 100vw; height: 100vh';
    document.body.appendChild(host);

    const clearDb = () => new Promise((resolveClear) => {
      const req = indexedDB.deleteDatabase('rhwpStudioAutosave');
      req.onsuccess = req.onerror = req.onblocked = () => resolveClear();
    });
    const listDraftIds = async () => {
      const req = indexedDB.open('rhwpStudioAutosave', 1);
      const db = await new Promise((resolveDb, rejectDb) => {
        req.onupgradeneeded = () => {
          if (!req.result.objectStoreNames.contains('drafts')) {
            req.result.createObjectStore('drafts', { keyPath: 'id' });
          }
        };
        req.onerror = () => rejectDb(req.error);
        req.onsuccess = () => resolveDb(req.result);
      });
      const ids = await new Promise((resolveIds, rejectIds) => {
        const tx = db.transaction('drafts', 'readonly');
        const keysReq = tx.objectStore('drafts').getAllKeys();
        keysReq.onsuccess = () => resolveIds(keysReq.result);
        keysReq.onerror = () => rejectIds(keysReq.error);
      });
      db.close();
      return ids;
    };
    const waitRecoveryDialog = async (iframeEl, timeoutMs) => {
      const deadline = Date.now() + timeoutMs;
      while (Date.now() < deadline) {
        const dialog = iframeEl.contentDocument?.querySelector('.modal-overlay .dialog-wrap');
        if (dialog && (dialog.textContent || '').includes('문서 복구')) return true;
        await new Promise((delay) => setTimeout(delay, 200));
      }
      return false;
    };
    const newEditor = () => createEditor(host, {
      studioUrl: `${location.origin}/`,
      handshakeTimeoutMs: 10_000,
    });

    await clearDb();

    // TC-3: capability 광고
    const editor1 = await newEditor();
    const capabilityAdvertised = editor1._transport.supports('notify-saved-v1');

    // TC-1: export → notifySaved → dirty 해제 + draft 삭제 + fileName 갱신
    const sampleBuffer = await fetch(sampleUrl).then((response) => response.arrayBuffer());
    await editor1.loadFile(sampleBuffer, 'footnote-01.hwp', { suppressDialogs: true });
    const win1 = editor1.element.contentWindow;
    win1.__documentState.markDirty('e2e-embed-save');
    await win1.__autosaveManager.flushNow('e2e-embed-save');
    const draftsAfterFlush = (await listDraftIds()).length;
    const bytes = await editor1.exportHwp();
    const dirtyAfterExport = win1.__documentState.isDirty();
    const draftsAfterExport = (await listDraftIds()).length;
    const ack = await editor1.notifySaved('footnote-01-저장본.hwp');
    const dirtyAfterAck = win1.__documentState.isDirty();
    const fileNameAfterAck = win1.__wasm.fileName;
    const draftsAfterAck = (await listDraftIds()).length;
    editor1.destroy();

    // TC-1 재시작: 통지 후에는 복구 다이얼로그가 뜨지 않는다
    const editor2 = await newEditor();
    const dialogAfterAck = await waitRecoveryDialog(editor2.element, 2_500);

    // TC-2: notifySaved 생략 → 재시작 시 복구 다이얼로그 표시
    await editor2.loadFile(sampleBuffer, 'footnote-01.hwp', { suppressDialogs: true });
    const win2 = editor2.element.contentWindow;
    win2.__documentState.markDirty('e2e-no-ack');
    await win2.__autosaveManager.flushNow('e2e-no-ack');
    const draftsBeforeClose = (await listDraftIds()).length;
    editor2.destroy(); // 통지 없이 종료 — 팝업 강제 종료와 동일 조건

    const editor3 = await newEditor();
    const dialogWithoutAck = await waitRecoveryDialog(editor3.element, 10_000);
    editor3.destroy();
    await clearDb();
    host.remove();

    return {
      capabilityAdvertised,
      draftsAfterFlush,
      exportLength: bytes.byteLength,
      dirtyAfterExport,
      draftsAfterExport,
      ack,
      dirtyAfterAck,
      fileNameAfterAck,
      draftsAfterAck,
      dialogAfterAck,
      draftsBeforeClose,
      dialogWithoutAck,
    };
  }, { editorModuleUrl: EDITOR_MODULE_URL, sampleUrl: SAMPLE_URL });

  console.log(`  Part A result: ${JSON.stringify(partA)}`);
  assert(partA.capabilityAdvertised === true, 'TC-3: rhwp-connected가 notify-saved-v1을 광고한다');
  assert(partA.draftsAfterFlush >= 1, 'TC-1: dirty + flush 후 복구 draft 존재');
  assert(partA.exportLength > 0, 'TC-1: exportHwp bytes 취득');
  assert(partA.dirtyAfterExport === true, 'TC-1: export만으로는 dirty 유지 (자동 markClean 없음)');
  assert(partA.draftsAfterExport >= 1, 'TC-1: export만으로는 draft 보존 (업로드 실패 대비)');
  assert(partA.ack?.ok === true && partA.ack?.wasDirty === true,
    'TC-1: notifySaved 응답 { ok: true, wasDirty: true }');
  assert(partA.dirtyAfterAck === false, 'TC-1: notifySaved 후 dirty 해제');
  assert(partA.fileNameAfterAck === 'footnote-01-저장본.hwp', 'TC-1: fileName 파라미터 반영');
  assert(partA.draftsAfterAck === 0, 'TC-1: notifySaved 후 draft 삭제 완료');
  assert(partA.dialogAfterAck === false, 'TC-1: 통지 후 재시작 시 복구 다이얼로그 없음');
  assert(partA.draftsBeforeClose >= 1, 'TC-2: 통지 없이 종료 직전 draft 존재');
  assert(partA.dialogWithoutAck === true, 'TC-2: 통지 생략 시 재시작에서 복구 다이얼로그 표시');

  setTestCase('TC-4: window.rhwpStudio.notifySaved (팝업/포크 표면)');
  await page.goto(
    `${VITE_URL}/?url=${encodeURIComponent(SAMPLE_URL)}&filename=footnote-01.hwp`,
    { waitUntil: 'domcontentloaded', timeout: 30000 },
  );
  await page.waitForFunction(
    () => !!window.__wasm && !!window.__canvasView && !!window.rhwpStudio,
    { timeout: 60000 },
  );
  await page.evaluate(() => new Promise((delay) => setTimeout(delay, 500)));

  const partB = await page.evaluate(async () => {
    const listDraftIds = async () => {
      const req = indexedDB.open('rhwpStudioAutosave', 1);
      const db = await new Promise((resolveDb, rejectDb) => {
        req.onupgradeneeded = () => {
          if (!req.result.objectStoreNames.contains('drafts')) {
            req.result.createObjectStore('drafts', { keyPath: 'id' });
          }
        };
        req.onerror = () => rejectDb(req.error);
        req.onsuccess = () => resolveDb(req.result);
      });
      const ids = await new Promise((resolveIds, rejectIds) => {
        const tx = db.transaction('drafts', 'readonly');
        const keysReq = tx.objectStore('drafts').getAllKeys();
        keysReq.onsuccess = () => resolveIds(keysReq.result);
        keysReq.onerror = () => rejectIds(keysReq.error);
      });
      db.close();
      return ids;
    };

    window.__documentState.markDirty('e2e-window-api');
    await window.__autosaveManager.flushNow('e2e-window-api');
    const draftsAfterFlush = (await listDraftIds()).length;
    const ack = await window.rhwpStudio.notifySaved();
    const dirtyAfterAck = window.__documentState.isDirty();
    const draftsAfterAck = (await listDraftIds()).length;
    return { draftsAfterFlush, ack, dirtyAfterAck, draftsAfterAck };
  });

  console.log(`  TC-4 result: ${JSON.stringify(partB)}`);
  assert(partB.draftsAfterFlush >= 1, 'TC-4: dirty + flush 후 복구 draft 존재');
  assert(partB.ack?.ok === true && partB.ack?.wasDirty === true,
    'TC-4: window.rhwpStudio.notifySaved 응답 { ok: true, wasDirty: true }');
  assert(partB.dirtyAfterAck === false, 'TC-4: notifySaved 후 dirty 해제');
  assert(partB.draftsAfterAck === 0, 'TC-4: notifySaved 후 draft 삭제 완료');

  // TC-4 재시작: 통지 후에는 복구 다이얼로그가 뜨지 않는다
  await page.goto(`${VITE_URL}/`, { waitUntil: 'domcontentloaded', timeout: 30000 });
  await page.waitForFunction(() => !!window.__wasm, { timeout: 60000 });
  const dialogAfterWindowAck = await page.evaluate(async () => {
    const deadline = Date.now() + 2_500;
    while (Date.now() < deadline) {
      const dialog = document.querySelector('.modal-overlay .dialog-wrap');
      if (dialog && (dialog.textContent || '').includes('문서 복구')) return true;
      await new Promise((delay) => setTimeout(delay, 200));
    }
    return false;
  });
  assert(dialogAfterWindowAck === false, 'TC-4: 통지 후 재시작 시 복구 다이얼로그 없음');
}, { skipLoadApp: true });
