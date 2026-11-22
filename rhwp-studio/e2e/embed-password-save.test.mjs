import { randomUUID } from 'node:crypto';
import { resolve } from 'node:path';
import { runTest, assert } from './helpers.mjs';

const VITE_URL = process.env.VITE_URL || 'http://localhost:7700';
const file = resolve(import.meta.dirname, '../../npm/editor/index.js').replace(/\\/g, '/');
const EDITOR_URL = '/@fs' + (file.startsWith('/') ? file : '/' + file);

await runTest('Issue #5987 MessagePort 암호 내보내기와 호스트 저장 확인', async ({ page }) => {
  const password = randomUUID();
  let loggedSecret = false;
  let urlSecret = false;
  page.on('console', message => { loggedSecret ||= message.text().includes(password); });
  page.on('pageerror', error => { loggedSecret ||= error.message.includes(password); });
  page.on('request', request => { urlSecret ||= request.url().includes(password); });
  await page.goto(`${VITE_URL}/@vite/client`, { waitUntil: 'domcontentloaded' });

  const checks = await page.evaluate(async ({ editorUrl, password }) => {
    const { createEditor } = await import(editorUrl);
    const host = document.body.appendChild(document.createElement('main'));
    host.style.cssText = 'width: 100vw; height: 100vh';
    const editor = await createEditor(host, { studioUrl: location.origin, handshakeTimeoutMs: 10_000 });
    const studio = editor.element.contentWindow;
    const wasm = studio.__wasm;
    const store = await studio.eval('import("/src/recovery/autosave-store.ts")');
    const { HwpDocument, default: initWasm } = await import('/rhwp.js');
    await initWasm();
    const checks = {};
    const sample = await fetch('/samples/footnote-01.hwp').then(response => response.arrayBuffer());
    const load = () => editor.loadFile(sample, 'footnote-01.hwp', { skipUnsavedGuard: true });
    const readState = () => editor.getSaveProtectionState();
    const code = async attempt => {
      try { await attempt(); return null; } catch (error) { return error.code ?? 'other'; }
    };
    const exportRevision = async (format, expected) => {
      const state = expected ?? await readState();
      return editor.exportPasswordProtectedDocument({
        schemaVersion: 1, format, password,
        expectedDocumentEpoch: state.documentEpoch, expectedChangeSeq: state.changeSeq,
      });
    };
    const acknowledge = ({ schemaVersion, exportId, format, protection, documentEpoch, changeSeq }, fileName) =>
      editor.acknowledgeHostSave({ schemaVersion, exportId, format, protection, documentEpoch, changeSeq,
        ...(fileName ? { fileName } : {}) });
    const calls = { hwp: 0, hwpx: 0 };
    for (const [format, method] of [['hwp', 'exportHwpWithPasswordAndReport'], ['hwpx', 'exportHwpxWithPasswordAndReport']]) {
      const original = wasm[method].bind(wasm);
      wasm[method] = (...args) => { calls[format]++; return original(...args); };
    }
    const edit = text => studio.__inputHandler.executeOperation({
      kind: 'snapshot', operationType: 'password-save-fixture',
      operation(bridge) {
        bridge.insertText(0, 0, 0, text);
        return { sectionIndex: 0, paragraphIndex: 0, charOffset: text.length };
      },
    });
    const draftIds = async () => (await store.listAutosaveDrafts()).map(draft => draft.id).sort().join(',');
    const flush = () => studio.__autosaveManager.flushNow('password-save-fixture');
    const stateRetained = async (before) => studio.__documentState.isDirty()
      && Boolean(before) && await draftIds() === before
      && (await readState()).persistedProtection === 'plain';

    const hostDb = await new Promise((resolve, reject) => {
      const open = indexedDB.open('rhwp-5987-host-write', 1);
      open.onupgradeneeded = () => open.result.createObjectStore('documents');
      open.onsuccess = () => resolve(open.result);
      open.onerror = () => reject(open.error);
    });
    const write = (bytes, fail = false) => new Promise((resolve, reject) => {
      const tx = hostDb.transaction('documents', 'readwrite');
      tx.objectStore('documents').put(bytes, 'saved');
      tx.oncomplete = resolve;
      tx.onabort = () => reject(Object.assign(new Error('Host write aborted'), { code: 'write-failed' }));
      if (fail) tx.abort();
    });
    const readBytes = () => new Promise((resolve, reject) => {
      const request = hostDb.transaction('documents').objectStore('documents').get('saved');
      request.onsuccess = () => resolve(request.result);
      request.onerror = () => reject(request.error);
    });
    const openProtected = async (bytes, name, passphrase) => {
      const opening = code(() => editor.loadFile(bytes, name, { skipUnsavedGuard: true }));
      let input;
      for (let attempt = 0; attempt < 400 && !input; attempt++) {
        input = studio.document.querySelector('#hwp-password-input');
        if (!input) await new Promise(resolve => setTimeout(resolve, 25));
      }
      if (!input) throw new Error('Encrypted document password prompt did not open');
      input.value = passphrase;
      input.dispatchEvent(new studio.KeyboardEvent('keydown', { key: 'Enter', bubbles: true }));
      if (await opening) throw new Error('Encrypted document could not be opened');
    };
    const footnote = doc => {
      for (let page = 0; page < doc.pageCount(); page++) {
        if (!doc.pageHasFootnoteFootholds(page)) continue;
        const { sectionIdx, paraIdx, controlIdx, sourceType } = JSON.parse(doc.getPageFootnoteInfo(page, 0));
        const { number, texts } = JSON.parse(doc.getFootnoteInfo(sectionIdx, paraIdx, controlIdx));
        return JSON.stringify({ sectionIdx, paraIdx, sourceType, number, texts });
      }
      return null;
    };
    const original = new HwpDocument(new Uint8Array(sample));
    const originalFootnote = footnote(original);
    original.free();
    const refuses = create => {
      try { const doc = create(); doc.free(); return false; } catch { return true; }
    };
    const roundTrip = (bytes, format, text, checkFootnote = true) => {
      const rejectsPlain = refuses(() => new HwpDocument(bytes));
      const rejectsWrong = refuses(() => HwpDocument.openWithPassword(bytes, 'wrong-fixture-value'));
      const doc = HwpDocument.openWithPassword(bytes, password);
      try {
        return rejectsPlain && rejectsWrong && doc.getSourceFormat() === format
          && doc.getTextRange(0, 0, 0, text.length) === text
          && (!checkFootnote || (originalFootnote !== null && footnote(doc) === originalFootnote))
          && (format !== 'hwp' || JSON.parse(doc.getDocumentInfo()).version.startsWith('5.'));
      } finally { doc.free(); }
    };
    const exposesSecret = value => {
      const { bytes: _bytes, ...metadata } = value;
      return JSON.stringify(metadata).includes(password);
    };

    try {
      await load();
      const initial = await readState();
      checks.plainState = initial.loadedFormat === 'hwp5' && initial.persistedProtection === 'plain';
      const beforeEdit = await readState();
      edit('first-edit ');
      await flush();
      const draft = await draftIds();
      checks.staleBeforeSerialize = await code(() => exportRevision('hwp', beforeEdit)) === 'STALE_SAVE_REVISION'
        && calls.hwp === 0;

      const exportHook = wasm.onBeforeExport;
      wasm.onBeforeExport = () => {
        wasm.onBeforeExport = exportHook;
        exportHook?.();
        edit('during-export ');
      };
      checks.staleDuringSerialize = await code(() => exportRevision('hwp')) === 'STALE_SAVE_REVISION'
        && calls.hwp === 1 && await stateRetained(draft);

      const hwp = await exportRevision('hwp');
      checks.oneHwpArtifact = hwp.bytes instanceof Uint8Array && hwp.byteCount === hwp.bytes.byteLength
        && hwp.contentLoss.outputFormat === 'hwp' && calls.hwp === 2 && !exposesSecret(hwp);
      checks.omittedAck = await stateRetained(draft);
      checks.failedWrite = await code(() => write(hwp.bytes, true)) === 'write-failed'
        && await stateRetained(draft);
      checks.hwpRoundTrip = roundTrip(hwp.bytes, 'hwp', 'during-export first-edit ');

      await write(hwp.bytes);
      edit('during-write ');
      checks.staleAfterEdit = await code(() => acknowledge(hwp)) === 'STALE_SAVE_REVISION'
        && await stateRetained(draft);

      const previous = await exportRevision('hwp');
      await load();
      edit('replacement-document ');
      await flush();
      const nextDraft = await draftIds();
      checks.staleAfterReplace = nextDraft !== draft
        && await code(() => acknowledge(previous)) === 'STALE_SAVE_REVISION' && await stateRetained(nextDraft);

      const hwpx = await exportRevision('hwpx');
      checks.oneHwpxArtifact = hwpx.byteCount === hwpx.bytes.byteLength && hwpx.contentLoss.outputFormat === 'hwpx'
        && calls.hwpx === 1 && !exposesSecret(hwpx);
      await write(hwpx.bytes);
      const fromHost = await readBytes();
      checks.hwpxRoundTrip = roundTrip(fromHost, 'hwpx', 'replacement-document ');
      const ack = await acknowledge(hwpx, 'host-saved.hwpx');
      const protectedState = await readState();
      checks.ackCommitted = ack.ok && ack.wasDirty && !studio.__documentState.isDirty() && !await draftIds()
        && wasm.fileName === 'host-saved.hwpx' && protectedState.loadedFormat === 'hwp5'
        && protectedState.persistedProtection === 'password-protected' && protectedState.requiresPasswordForSave;
      checks.replayRejected = await code(() => acknowledge(hwpx)) === 'INVALID_SAVE_EXPORT';
      checks.passwordRequiredAgain = await code(() => editor.exportPasswordProtectedDocument({
        schemaVersion: 1, format: 'hwp', expectedDocumentEpoch: protectedState.documentEpoch,
        expectedChangeSeq: protectedState.changeSeq,
      })) === 'INVALID_REQUEST';

      for (const [bytes, inputFormat, text] of [
        [hwp.bytes, 'hwp', 'during-export first-edit '],
        [fromHost, 'hwpx', 'replacement-document '],
      ]) {
        await openProtected(bytes, 'repeat.' + inputFormat, password);
        edit('second-save ');
        for (const outputFormat of ['hwp', 'hwpx']) {
          const repeated = await exportRevision(outputFormat);
          await write(repeated.bytes);
          await acknowledge(repeated);
          checks[inputFormat + 'To' + outputFormat + 'Resave'] =
            roundTrip(await readBytes(), outputFormat, 'second-save ' + text);
        }
      }

      const legacy = await fetch('/samples/HWP3-password-123456.hwp').then(response => response.arrayBuffer());
      await openProtected(legacy, 'legacy.hwp', String.fromCharCode(49, 50, 51, 52, 53, 54));
      const hwp3 = await readState();
      const text = wasm.getTextRange(0, 0, 0, 8);
      const converted = await exportRevision('hwp', hwp3);
      checks.hwp3Conversion = hwp3.loadedFormat === 'hwp3' && hwp3.requiresPasswordForSave
        && converted.bytes[0] === 0xd0 && converted.bytes[1] === 0xcf
        && roundTrip(converted.bytes, 'hwp', text, false);

      const persisted = [
        studio.location.href, studio.document.documentElement.outerHTML,
        ...Array.from(studio.document.querySelectorAll('input'), input => input.value),
        JSON.stringify(studio.localStorage), JSON.stringify(studio.sessionStorage),
        JSON.stringify(await store.listAutosaveDrafts()),
      ];
      checks.noSecretInStudio = persisted.every(value => !value.includes(password));
      return checks;
    } finally {
      hostDb.close();
      indexedDB.deleteDatabase('rhwp-5987-host-write');
      editor.destroy();
    }
  }, { editorUrl: EDITOR_URL, password });

  for (const [name, passed] of Object.entries(checks)) assert(passed, name);
  assert(!loggedSecret, '콘솔과 처리되지 않은 오류에서 내보내기 암호가 노출되지 않는다');
  assert(!urlSecret, '암호를 포함한 브라우저 요청 URL이 없다');
}, { skipLoadApp: true });
