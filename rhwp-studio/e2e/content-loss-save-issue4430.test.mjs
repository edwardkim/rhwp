/**
 * Issue #4430 — serializer content-loss artifact -> explicit Studio save E2E.
 *
 * A tracked public HWPX sample is copied and patched only in browser memory: the
 * two ZIP entry-name occurrences are changed without changing the compressed
 * manifest. Five picture owners still reference one now-unreadable resource, so
 * each real reported export must produce one exact resource-level loss.
 * This covers the explicit HWP/HWPX serializer-save slice only; auxiliary
 * byte-only consumers and unrelated warning surfaces remain out of scope.
 *
 * Run: npm run e2e:issue-4430-content-loss
 */
import {
  assert,
  loadApp,
  runTest,
  setTestCase,
  waitForCanvas,
} from './helpers.mjs';

const SAMPLE_URL = '/samples/test-image.hwpx';
const SAMPLE_NAME = 'test-image.hwpx';
const SAMPLE_SIZE = 22_602;
const ORIGINAL_ENTRY = 'BinData/image1.bmp';
const MISSING_ENTRY = 'BinData/ghost1.bmp';
const ORIGINAL_ENTRY_OFFSETS = [4_255, 22_037];
const NOTICE_PREFIX = '파일은 저장되었지만 일부 내용을 보존하지 못했습니다.';
const PERSISTENCE_PROBE_MS = 8_500;

const REPORTED_METHODS = [
  'exportHwpWithReport',
  'exportHwpWithPasswordAndReport',
  'exportHwpxWithReport',
  'exportHwpxWithPasswordAndReport',
];
const LEGACY_BYTE_ONLY_METHODS = [
  'exportHwp',
  'exportHwpWithPassword',
  'exportHwpx',
  'exportHwpxWithPassword',
];

const EXPECTED = {
  hwpx: {
    command: 'file:save-as-hwpx',
    extension: 'hwpx',
    mimeType: 'application/hwp+zip',
    magic: [0x50, 0x4b, 0x03, 0x04],
    noticeFormat: 'HWPX',
    report: {
      schemaVersion: 1,
      outputFormat: 'hwpx',
      count: 1,
      losses: [{
        code: 'binaryContentEmptied',
        subject: 'binaryData',
        path: 'BinData/image1.bmp',
        reason: 'resourceReadFailedOrLimitExceeded',
        resourceId: 1,
      }],
    },
  },
  hwp: {
    command: 'file:save-as-hwp',
    extension: 'hwp',
    mimeType: 'application/x-hwp',
    magic: [0xd0, 0xcf, 0x11, 0xe0],
    noticeFormat: 'HWP',
    report: {
      schemaVersion: 1,
      outputFormat: 'hwp',
      count: 1,
      losses: [{
        code: 'binaryContentEmptied',
        subject: 'binaryData',
        path: '/BinData/BIN0001.bmp',
        reason: 'rawPassthroughUnavailable',
        resourceId: 1,
      }],
    },
  },
};

function requireCondition(condition, message) {
  assert(condition, message);
  if (!condition) throw new Error(message);
}

function expectedPath(format) {
  return EXPECTED[format].report.losses[0].path;
}

function expectedReportedMethod(format, passwordProtected = false) {
  if (format === 'hwp') {
    return passwordProtected ? 'exportHwpWithPasswordAndReport' : 'exportHwpWithReport';
  }
  return passwordProtected ? 'exportHwpxWithPasswordAndReport' : 'exportHwpxWithReport';
}

async function loadMemoryPatchedSample(page) {
  await loadApp(page);
  const loaded = await page.evaluate(async ({
    sampleUrl,
    sampleName,
    originalEntry,
    missingEntry,
  }) => {
    const response = await fetch(sampleUrl, { cache: 'no-store' });
    if (!response.ok) throw new Error(`sample fetch failed: HTTP ${response.status}`);

    const bytes = new Uint8Array(await response.arrayBuffer());
    const original = new TextEncoder().encode(originalEntry);
    const missing = new TextEncoder().encode(missingEntry);
    if (original.length !== missing.length) {
      throw new Error('entry-name replacement must preserve ZIP offsets');
    }

    const positionsOf = (haystack, needle) => {
      const positions = [];
      for (let offset = 0; offset <= haystack.length - needle.length; offset += 1) {
        let matches = true;
        for (let i = 0; i < needle.length; i += 1) {
          if (haystack[offset + i] !== needle[i]) {
            matches = false;
            break;
          }
        }
        if (matches) positions.push(offset);
      }
      return positions;
    };

    const originalPositions = positionsOf(bytes, original);
    if (originalPositions.length !== 2) {
      throw new Error(`expected exactly two entry names, got ${originalPositions.length}`);
    }
    for (const offset of originalPositions) bytes.set(missing, offset);

    const patchedOriginalPositions = positionsOf(bytes, original);
    const patchedMissingPositions = positionsOf(bytes, missing);
    if (patchedOriginalPositions.length !== 0 || patchedMissingPositions.length !== 2) {
      throw new Error('browser-memory entry-name replacement was not exactly two-for-two');
    }

    const info = window.__wasm?.loadDocument(bytes, sampleName);
    if (!info) throw new Error('loadDocument returned no document info');
    await window.__canvasView?.loadDocument?.();

    // A second fetch produces a new ArrayBuffer and proves the served fixture was
    // not changed by the browser-memory Uint8Array mutation above.
    const pristineResponse = await fetch(sampleUrl, { cache: 'no-store' });
    if (!pristineResponse.ok) {
      throw new Error(`pristine sample fetch failed: HTTP ${pristineResponse.status}`);
    }
    const pristine = new Uint8Array(await pristineResponse.arrayBuffer());
    return {
      byteLength: bytes.length,
      originalPositions,
      patchedOriginalCount: patchedOriginalPositions.length,
      patchedMissingPositions,
      pristineOriginalPositions: positionsOf(pristine, original),
      pristineMissingCount: positionsOf(pristine, missing).length,
      pageCount: info.pageCount,
      sourceFormat: window.__wasm?.getSourceFormat?.(),
    };
  }, {
    sampleUrl: SAMPLE_URL,
    sampleName: SAMPLE_NAME,
    originalEntry: ORIGINAL_ENTRY,
    missingEntry: MISSING_ENTRY,
  });

  requireCondition(loaded.byteLength === SAMPLE_SIZE,
    `tracked sample size is stable (${loaded.byteLength})`);
  requireCondition(JSON.stringify(loaded.originalPositions) === JSON.stringify(ORIGINAL_ENTRY_OFFSETS),
    `exactly two source entry names occur at ${ORIGINAL_ENTRY_OFFSETS.join(', ')}`);
  requireCondition(loaded.patchedOriginalCount === 0
      && JSON.stringify(loaded.patchedMissingPositions) === JSON.stringify(ORIGINAL_ENTRY_OFFSETS),
  'browser-memory copy replaces exactly two equal-length entry names');
  requireCondition(JSON.stringify(loaded.pristineOriginalPositions) === JSON.stringify(ORIGINAL_ENTRY_OFFSETS)
      && loaded.pristineMissingCount === 0,
  'a fresh fetch remains pristine after the in-memory mutation');
  requireCondition(loaded.sourceFormat === 'hwpx' && loaded.pageCount >= 1,
    `patched sample loads as HWPX (${loaded.pageCount} pages)`);
  await waitForCanvas(page, 30_000);
}

async function installPersistenceHarness(page, options = {}) {
  const config = {
    picker: options.picker ?? 'success',
    anchor: options.anchor ?? 'success',
    handleName: options.handleName ?? 'issue4430.hwpx',
  };
  await page.evaluate(({
    config,
    noticePrefix,
    reportedMethods,
    legacyByteOnlyMethods,
  }) => {
    const state = {
      events: [],
      alerts: [],
      observedNoticeTexts: [],
      reportedCalls: [],
      legacyCalls: [],
      artifacts: [],
      savedBlob: null,
      downloadName: '',
    };
    window.__issue4430 = state;

    const wasm = window.__wasm;
    if (!wasm) throw new Error('WasmBridge is unavailable');
    for (const method of reportedMethods) {
      const original = wasm[method];
      if (typeof original !== 'function') {
        throw new Error(`reported WasmBridge method is unavailable: ${method}`);
      }
      wasm[method] = function observeReportedExport(...args) {
        state.reportedCalls.push({
          method,
          argumentCount: args.length,
          hasNonEmptyPasswordArgument: method.includes('Password')
            && args.length === 1
            && typeof args[0] === 'string'
            && args[0].length > 0,
        });
        state.events.push(`reported:${method}:call`);
        const artifact = original.apply(this, args);
        const contentLoss = JSON.parse(JSON.stringify(artifact.contentLoss));
        state.events.push(`reported:${method}:report`);
        const bytes = artifact.bytes.slice();
        state.events.push(`reported:${method}:bytes`);
        state.artifacts.push({ method, contentLoss, bytes });
        return artifact;
      };
    }
    for (const method of legacyByteOnlyMethods) {
      const original = wasm[method];
      if (typeof original !== 'function') {
        throw new Error(`legacy WasmBridge method is unavailable: ${method}`);
      }
      wasm[method] = function observeLegacyExport(...args) {
        state.legacyCalls.push({ method, argumentCount: args.length });
        state.events.push(`legacy:${method}`);
        return original.apply(this, args);
      };
    }

    window.alert = (message) => {
      state.events.push('alert');
      state.alerts.push(String(message));
    };

    URL.createObjectURL = (blob) => {
      state.events.push('objectURL');
      state.savedBlob = blob;
      return 'blob:issue4430-captured';
    };
    URL.revokeObjectURL = () => {
      state.events.push('revokeObjectURL');
    };

    HTMLAnchorElement.prototype.click = function clickIssue4430Anchor() {
      state.downloadName = this.download;
      if (config.anchor === 'error') {
        state.events.push('anchor:throw');
        throw new Error('issue4430 deterministic anchor failure');
      }
      state.events.push('anchor:click');
    };

    window.showSaveFilePicker = async (pickerOptions) => {
      if (config.picker === 'abort') {
        state.events.push('picker:abort');
        throw new DOMException('issue4430 picker cancelled', 'AbortError');
      }
      if (config.picker === 'error') {
        state.events.push('picker:error');
        throw new Error('issue4430 deterministic picker failure');
      }

      state.events.push('picker:success');
      const handle = {
        name: config.handleName || pickerOptions?.suggestedName || 'issue4430.hwpx',
        async createWritable() {
          state.events.push('writable:create');
          return {
            async write(blob) {
              state.events.push('write');
              state.savedBlob = blob;
            },
            async close() {
              state.events.push('close');
            },
          };
        },
      };
      return handle;
    };

    const seen = new WeakSet();
    const recordNotices = () => {
      for (const node of document.querySelectorAll('#rhwp-toast-container [role="status"]')) {
        const text = node.textContent || '';
        if (!text.includes(noticePrefix) || seen.has(node)) continue;
        seen.add(node);
        state.events.push('notice');
        state.observedNoticeTexts.push(text);
      }
    };
    new MutationObserver(recordNotices).observe(document.body, { childList: true, subtree: true });
    recordNotices();
  }, {
    config,
    noticePrefix: NOTICE_PREFIX,
    reportedMethods: REPORTED_METHODS,
    legacyByteOnlyMethods: LEGACY_BYTE_ONLY_METHODS,
  });
}

async function resetHarnessObservations(page) {
  await page.evaluate(() => {
    const state = window.__issue4430;
    state.events.length = 0;
    state.alerts.length = 0;
    state.observedNoticeTexts.length = 0;
    state.reportedCalls.length = 0;
    state.legacyCalls.length = 0;
    state.artifacts.length = 0;
    state.savedBlob = null;
    state.downloadName = '';
  });
}

async function harnessSnapshot(page) {
  return page.evaluate(async ({ noticePrefix }) => {
    const state = window.__issue4430;
    const blob = state.savedBlob;
    let blobBytes = null;
    let blobInfo = null;
    if (blob) {
      blobBytes = new Uint8Array(await blob.arrayBuffer());
      blobInfo = {
        type: blob.type,
        size: blobBytes.length,
        head: Array.from(blobBytes.slice(0, 8)),
      };
    }
    const artifacts = state.artifacts.map((artifact) => ({
      method: artifact.method,
      contentLoss: artifact.contentLoss,
      byteLength: artifact.bytes.length,
      persistedBlobMatches: blobBytes === null ? null : artifact.bytes.length === blobBytes.length
        && artifact.bytes.every((byte, index) => byte === blobBytes[index]),
    }));
    const currentNoticeTexts = [...document.querySelectorAll('#rhwp-toast-container [role="status"]')]
      .map((node) => node.textContent || '')
      .filter((text) => text.includes(noticePrefix));
    return {
      events: [...state.events],
      alerts: [...state.alerts],
      observedNoticeTexts: [...state.observedNoticeTexts],
      currentNoticeTexts,
      reportedCalls: state.reportedCalls.map((call) => ({ ...call })),
      legacyCalls: state.legacyCalls.map((call) => ({ ...call })),
      artifacts,
      blobInfo,
      downloadName: state.downloadName,
      dirty: window.__documentState?.isDirty?.() ?? null,
    };
  }, { noticePrefix: NOTICE_PREFIX });
}

async function markDocumentDirty(page) {
  await page.evaluate(() => window.__eventBus?.emit('document-changed', 'issue4430-e2e'));
  await page.waitForFunction(() => window.__documentState?.isDirty?.() === true);
}

async function markDocumentCleanForNavigation(page) {
  await page.evaluate(() => window.__documentState?.markClean('issue4430-e2e-navigation'));
  await page.waitForFunction(() => window.__documentState?.isDirty?.() === false);
}

async function settleUi(page) {
  await page.evaluate(() => new Promise((resolve) => {
    requestAnimationFrame(() => requestAnimationFrame(() => setTimeout(resolve, 25)));
  }));
}

async function waitForHarnessEvent(page, event) {
  await page.waitForFunction(
    (expectedEvent) => window.__issue4430?.events?.includes(expectedEvent),
    { timeout: 30_000 },
    event,
  );
}

async function waitForNotice(page, path) {
  await page.waitForFunction(({ noticePrefix, expectedPath }) => {
    const notices = [...document.querySelectorAll('#rhwp-toast-container [role="status"]')]
      .filter((node) => (node.textContent || '').includes(noticePrefix)
        && (node.textContent || '').includes(expectedPath));
    return notices.length === 1;
  }, { timeout: 30_000 }, { noticePrefix: NOTICE_PREFIX, expectedPath: path });
}

async function clickFileCommand(page, command) {
  const title = await page.$('#menu-bar .menu-item[data-menu="file"] .menu-title');
  requireCondition(title !== null, 'File menu title exists');
  await title.click();
  await title.dispose();
  await page.waitForSelector('#menu-bar .menu-item[data-menu="file"].open');

  const item = await page.$(`.md-item[data-cmd="${command}"]`);
  requireCondition(item !== null, `${command} menu item exists`);
  const disabled = await item.evaluate((element) => element.classList.contains('disabled'));
  requireCondition(!disabled, `${command} menu item is enabled`);
  await item.click();
  await item.dispose();
}

async function waitForDialog(page, title, hidden = false) {
  await page.waitForFunction(({ expectedTitle, shouldBeHidden }) => {
    const exists = [...document.querySelectorAll('.modal-overlay .dialog-wrap')]
      .some((dialog) => {
        const titleNode = dialog.querySelector('.dialog-title')?.firstChild;
        return titleNode?.textContent?.trim() === expectedTitle;
      });
    return shouldBeHidden ? !exists : exists;
  }, { timeout: 10_000 }, { expectedTitle: title, shouldBeHidden: hidden });
}

async function findDialog(page, title) {
  const dialogs = await page.$$('.modal-overlay .dialog-wrap');
  for (const dialog of dialogs) {
    const dialogTitle = await dialog.$eval(
      '.dialog-title',
      (element) => element.firstChild?.textContent?.trim() || '',
    ).catch(() => '');
    if (dialogTitle === title) return dialog;
    await dialog.dispose();
  }
  return null;
}

async function clickDialogButton(page, title, label) {
  await waitForDialog(page, title);
  const dialog = await findDialog(page, title);
  requireCondition(dialog !== null, `${title} dialog exists`);
  const buttons = await dialog.$$('button');
  let target = null;
  for (const button of buttons) {
    const text = await button.evaluate((element) => element.textContent?.trim() || '');
    if (text === label) {
      target = button;
      break;
    }
    await button.dispose();
  }
  requireCondition(target !== null, `${title} dialog has ${label} button`);
  await target.click();
  await target.dispose();
  await dialog.dispose();
}

async function enterSaveAsName(page, fileName, action = 'confirm') {
  const title = '다른 이름으로 저장';
  await waitForDialog(page, title);
  const dialog = await findDialog(page, title);
  requireCondition(dialog !== null, 'Save As dialog exists');
  const input = await dialog.$('.dialog-body input[type="text"]');
  requireCondition(input !== null, 'Save As filename input exists');
  await input.click();
  await input.focus();
  await page.keyboard.press('Home');
  await page.keyboard.down('Shift');
  try {
    await page.keyboard.press('End');
  } finally {
    await page.keyboard.up('Shift');
  }
  const selection = await input.evaluate((element) => ({
    start: element.selectionStart,
    end: element.selectionEnd,
    length: element.value.length,
  }));
  requireCondition(selection.start === 0 && selection.end === selection.length,
    `Save As keyboard selection spans the full filename (${selection.start}..${selection.end}/${selection.length})`);
  await page.keyboard.press('Backspace');
  await page.keyboard.type(fileName);
  await input.dispose();
  await dialog.dispose();
  await clickDialogButton(page, title, action === 'password' ? '암호 설정...' : '확인');
}

async function enterPasswordAndConfirm(page, password) {
  const title = '문서 암호 설정';
  await waitForDialog(page, title);
  await page.type('#hwp-save-password-input', password);
  await page.type('#hwp-save-password-confirmation', password);
  await clickDialogButton(page, title, '확인');
}

async function dismissLossNotice(page, path) {
  const notices = await page.$$('#rhwp-toast-container [role="status"]');
  let clicked = false;
  for (const notice of notices) {
    const text = await notice.evaluate((element) => element.textContent || '');
    if (!text.includes(NOTICE_PREFIX) || !text.includes(path)) {
      await notice.dispose();
      continue;
    }
    const buttons = await notice.$$('button');
    for (const button of buttons) {
      const label = await button.evaluate((element) => element.textContent?.trim() || '');
      if (label === '확인') {
        await button.click();
        clicked = true;
        await button.dispose();
        break;
      }
      await button.dispose();
    }
    await notice.dispose();
    if (clicked) break;
  }
  requireCondition(clicked, 'persistent content-loss notice has an acknowledgement button');
  await page.waitForFunction(({ noticePrefix, expectedPath }) =>
    ![...document.querySelectorAll('#rhwp-toast-container [role="status"]')]
      .some((node) => (node.textContent || '').includes(noticePrefix)
        && (node.textContent || '').includes(expectedPath)),
  { timeout: 3_000 }, { noticePrefix: NOTICE_PREFIX, expectedPath: path });
}

function assertExactReport(report, format) {
  requireCondition(JSON.stringify(report) === JSON.stringify(EXPECTED[format].report),
    `${format.toUpperCase()} report has the exact one-resource schema and location`);
}

function assertBlob(snapshot, format) {
  const expected = EXPECTED[format];
  requireCondition(snapshot.blobInfo !== null, `${format.toUpperCase()} persisted blob was captured`);
  assert(snapshot.blobInfo.type === expected.mimeType,
    `${format.toUpperCase()} persisted MIME is ${expected.mimeType}`);
  assert(snapshot.blobInfo.size > 0, `${format.toUpperCase()} persisted bytes are nonempty`);
  assert(expected.magic.every((byte, index) => snapshot.blobInfo.head[index] === byte),
    `${format.toUpperCase()} persisted bytes have the expected container magic`);
}

function assertNoLegacyExport(snapshot, label) {
  requireCondition(snapshot.legacyCalls.length === 0,
    `${label}: explicit serializer save never calls a legacy byte-only exporter`);
}

function assertReportedArtifact(
  snapshot,
  format,
  { passwordProtected = false, comparePersistedBlob = false } = {},
) {
  const method = expectedReportedMethod(format, passwordProtected);
  requireCondition(snapshot.reportedCalls.length === 1,
    `${format.toUpperCase()} save calls exactly one reported WasmBridge exporter`);
  const call = snapshot.reportedCalls[0];
  requireCondition(call.method === method, `${format.toUpperCase()} save calls ${method}`);
  requireCondition(call.argumentCount === (passwordProtected ? 1 : 0),
    `${method} receives the expected argument count`);
  requireCondition(call.hasNonEmptyPasswordArgument === passwordProtected,
    `${method} records only whether its password argument is present`);
  assertNoLegacyExport(snapshot, `${format.toUpperCase()} reported export`);

  requireCondition(snapshot.artifacts.length === 1,
    `${method} returns exactly one observed artifact`);
  const artifact = snapshot.artifacts[0];
  requireCondition(artifact.method === method, `${method} owns the observed artifact`);
  assertEventOrder(snapshot.events, [
    `reported:${method}:call`,
    `reported:${method}:report`,
    `reported:${method}:bytes`,
  ], `${method} returned artifact lifecycle`);
  assertExactReport(artifact.contentLoss, format);
  requireCondition(artifact.byteLength > 0, `${method} returns nonempty bytes`);
  if (comparePersistedBlob) {
    requireCondition(artifact.persistedBlobMatches === true,
      `${method} artifact bytes exactly match the persisted Blob`);
  }
}

function assertReportedCallWithoutArtifact(snapshot, format) {
  const method = expectedReportedMethod(format);
  requireCondition(snapshot.reportedCalls.length === 1
      && snapshot.reportedCalls[0].method === method,
  `failed ${format.toUpperCase()} export enters ${method} exactly once`);
  requireCondition(snapshot.reportedCalls[0].argumentCount === 0
      && snapshot.reportedCalls[0].hasNonEmptyPasswordArgument === false,
  `failed ${format.toUpperCase()} export enters the unprotected reported method without arguments`);
  requireCondition(snapshot.artifacts.length === 0,
    `failed ${format.toUpperCase()} export returns no artifact or bytes`);
  assertNoLegacyExport(snapshot, `failed ${format.toUpperCase()} export`);
}

function assertNoExporterCall(snapshot, label) {
  requireCondition(snapshot.reportedCalls.length === 0 && snapshot.artifacts.length === 0,
    `${label}: no reported artifact is created`);
  assertNoLegacyExport(snapshot, label);
}

function assertOneNotice(snapshot, format) {
  const expected = EXPECTED[format];
  const path = expected.report.losses[0].path;
  requireCondition(snapshot.observedNoticeTexts.length === 1,
    `${format.toUpperCase()} save inserts exactly one content-loss notice`);
  requireCondition(snapshot.currentNoticeTexts.length === 1,
    `${format.toUpperCase()} save leaves exactly one persistent content-loss notice`);
  const text = snapshot.currentNoticeTexts[0];
  assert(text.includes(`${expected.noticeFormat} ${NOTICE_PREFIX}`),
    `${format.toUpperCase()} notice names the persisted output format`);
  assert(text.includes(path), `${format.toUpperCase()} notice names the exact lost resource path`);
}

function assertNoNotice(snapshot, label) {
  assert(snapshot.observedNoticeTexts.length === 0 && snapshot.currentNoticeTexts.length === 0,
    `${label}: no stale or new content-loss notice is shown`);
}

function assertEventOrder(events, ordered, label) {
  let previous = -1;
  for (const event of ordered) {
    const index = events.indexOf(event);
    requireCondition(index > previous, `${label}: ${event} occurs in persistence order`);
    previous = index;
  }
}

async function verifyProtectedReopen(page, password, format) {
  return page.evaluate(({ passwordValue, outputFormat }) => {
    const state = window.__issue4430;
    return state.savedBlob.arrayBuffer().then((buffer) => {
      const bytes = new Uint8Array(buffer);
      let unprotectedFailed = false;
      try {
        window.__wasm.loadDocument(bytes, `plain-reopen.${outputFormat}`);
      } catch {
        unprotectedFailed = true;
      }
      const info = window.__wasm.loadDocumentWithPassword(
        bytes,
        passwordValue,
        `protected-reopen.${outputFormat}`,
      );
      return {
        unprotectedFailed,
        pageCount: info.pageCount,
        localHasPassword: JSON.stringify(localStorage).includes(passwordValue),
        sessionHasPassword: JSON.stringify(sessionStorage).includes(passwordValue),
      };
    });
  }, { passwordValue: password, outputFormat: format });
}

await runTest('Issue #4430 content-loss artifact reaches explicit Studio saves', async ({ page }) => {
  setTestCase('raw DocumentExport lifecycle and exact multiplexed-resource report');
  await loadMemoryPatchedSample(page);
  const lifecycle = await page.evaluate(() => {
    const doc = Reflect.get(window.__wasm, 'doc');
    if (!doc) throw new Error('raw HwpDocument is unavailable');
    const exported = doc.exportHwpxWithReport();
    try {
      const before = exported.contentLoss();
      const hasBytesBefore = exported.hasBytes();
      const bytes = exported.takeBytes();
      const hasBytesAfter = exported.hasBytes();
      const after = exported.contentLoss();
      let secondTakeError = '';
      try {
        exported.takeBytes();
      } catch (error) {
        secondTakeError = String(error);
      }
      return {
        before: JSON.parse(before),
        reportStable: before === after,
        hasBytesBefore,
        hasBytesAfter,
        byteLength: bytes.length,
        secondTakeError,
      };
    } finally {
      exported.free();
    }
  });
  assertExactReport(lifecycle.before, 'hwpx');
  assert(lifecycle.hasBytesBefore && !lifecycle.hasBytesAfter && lifecycle.byteLength > 0,
    'takeBytes transfers nonempty bytes exactly once');
  assert(lifecycle.reportStable, 'contentLoss remains readable and identical after takeBytes');
  assert(lifecycle.secondTakeError.includes('이미 가져갔습니다'),
    'a second takeBytes call is an explicit lifecycle error');

  setTestCase('primary save persists, notifies once, then exporter failure has no stale notice');
  await loadMemoryPatchedSample(page);
  await installPersistenceHarness(page, {
    picker: 'success',
    handleName: 'issue4430-primary.hwpx',
  });
  await markDocumentDirty(page);
  await clickFileCommand(page, 'file:save');
  await waitForNotice(page, expectedPath('hwpx'));
  await page.waitForFunction(() => window.__documentState?.isDirty?.() === false);
  let snapshot = await harnessSnapshot(page);
  assertEventOrder(snapshot.events, [
    'reported:exportHwpxWithReport:report',
    'picker:success',
    'write',
    'close',
    'notice',
  ], 'primary save');
  assert(!snapshot.events.includes('anchor:click'), 'primary save does not use download fallback');
  assertReportedArtifact(snapshot, 'hwpx', { comparePersistedBlob: true });
  assertBlob(snapshot, 'hwpx');
  assertOneNotice(snapshot, 'hwpx');
  assert(snapshot.dirty === false, 'successful primary save marks the document clean');

  await page.evaluate((delay) => new Promise((resolve) => setTimeout(resolve, delay)), PERSISTENCE_PROBE_MS);
  snapshot = await harnessSnapshot(page);
  assertOneNotice(snapshot, 'hwpx');
  await dismissLossNotice(page, expectedPath('hwpx'));
  await resetHarnessObservations(page);
  await markDocumentDirty(page);

  try {
    const originalCallbackIsFunction = await page.evaluate(() => {
      const state = window.__issue4430;
      state.originalOnBeforeExport = window.__wasm.onBeforeExport;
      window.__wasm.onBeforeExport = () => {
        state.events.push('before-export:throw');
        throw new Error('issue4430 deterministic exporter failure');
      };
      return typeof state.originalOnBeforeExport === 'function';
    });
    requireCondition(originalCallbackIsFunction,
      'export failure probe preserves the installed production onBeforeExport callback');
    await clickFileCommand(page, 'file:save');
    await waitForHarnessEvent(page, 'alert');
    snapshot = await harnessSnapshot(page);
    assertEventOrder(snapshot.events, [
      'reported:exportHwpxWithReport:call',
      'before-export:throw',
      'alert',
    ], 'export failure');
    assert(!snapshot.events.some((event) =>
      event.startsWith('picker:') || ['write', 'close', 'anchor:click', 'anchor:throw'].includes(event)),
    'export failure reaches no picker, writable, or download persistence boundary');
    assertReportedCallWithoutArtifact(snapshot, 'hwpx');
    assert(snapshot.blobInfo === null, 'export failure produces no persistable Blob');
    assert(snapshot.alerts.length === 1
      && snapshot.alerts[0].includes('issue4430 deterministic exporter failure'),
    'export failure uses the normal save-error alert exactly once');
    assertNoNotice(snapshot, 'export failure after a reported success');
    assert(snapshot.dirty === true, 'failed export leaves the document dirty');
  } finally {
    await page.evaluate(() => {
      const state = window.__issue4430;
      if (!Object.hasOwn(state, 'originalOnBeforeExport')) {
        throw new Error('original onBeforeExport callback was not preserved');
      }
      const original = state.originalOnBeforeExport;
      window.__wasm.onBeforeExport = original;
      if (window.__wasm.onBeforeExport !== original) {
        throw new Error('production onBeforeExport callback was not restored exactly');
      }
      delete state.originalOnBeforeExport;
    });
  }
  await markDocumentCleanForNavigation(page);

  setTestCase('picker failure falls back to HWP download and notifies after anchor click');
  await loadMemoryPatchedSample(page);
  await installPersistenceHarness(page, { picker: 'error', anchor: 'success' });
  await markDocumentDirty(page);
  await clickFileCommand(page, EXPECTED.hwp.command);
  await enterSaveAsName(page, 'issue4430-fallback', 'confirm');
  await waitForHarnessEvent(page, 'picker:error');
  await waitForDialog(page, '다른 이름으로 저장');
  snapshot = await harnessSnapshot(page);
  assertNoNotice(snapshot, 'primary picker failure while fallback name is pending');
  assert(!snapshot.events.includes('anchor:click'), 'fallback download has not started before name confirmation');
  assertReportedArtifact(snapshot, 'hwp');
  await enterSaveAsName(page, 'issue4430-fallback', 'confirm');
  await waitForNotice(page, expectedPath('hwp'));
  await waitForHarnessEvent(page, 'revokeObjectURL');
  await page.waitForFunction(() => window.__documentState?.isDirty?.() === false);
  snapshot = await harnessSnapshot(page);
  assertEventOrder(snapshot.events, [
    'reported:exportHwpWithReport:report',
    'picker:error',
    'objectURL',
    'anchor:click',
    'notice',
    'revokeObjectURL',
  ], 'fallback save');
  requireCondition(snapshot.events.filter((event) => event === 'revokeObjectURL').length === 1,
    'successful fallback revokes its object URL exactly once after anchor click');
  assert(snapshot.downloadName === 'issue4430-fallback.hwp',
    `fallback uses the confirmed HWP filename (expected="issue4430-fallback.hwp", observed=${JSON.stringify(snapshot.downloadName)}, events=${JSON.stringify(snapshot.events)})`);
  assertReportedArtifact(snapshot, 'hwp', { comparePersistedBlob: true });
  assertBlob(snapshot, 'hwp');
  assertOneNotice(snapshot, 'hwp');
  assert(snapshot.dirty === false, 'successful fallback download marks the document clean');
  await page.evaluate((delay) => new Promise((resolve) => setTimeout(resolve, delay)), PERSISTENCE_PROBE_MS);
  snapshot = await harnessSnapshot(page);
  assertOneNotice(snapshot, 'hwp');

  for (const format of ['hwp', 'hwpx']) {
    setTestCase(`password-protected ${format.toUpperCase()} save uses reported artifact`);
    await loadMemoryPatchedSample(page);
    await installPersistenceHarness(page, {
      picker: 'success',
      handleName: `issue4430-protected.${EXPECTED[format].extension}`,
    });
    await markDocumentDirty(page);
    await clickFileCommand(page, EXPECTED[format].command);
    await enterSaveAsName(page, `issue4430-protected-${format}`, 'password');
    const password = String.fromCharCode(116, 101, 115, 116, 45, 52, 52, 51, 48);
    await enterPasswordAndConfirm(page, password);
    await waitForNotice(page, expectedPath(format));
    await page.waitForFunction(() => window.__documentState?.isDirty?.() === false);
    snapshot = await harnessSnapshot(page);
    const method = expectedReportedMethod(format, true);
    assertEventOrder(snapshot.events, [
      `reported:${method}:report`,
      'picker:success',
      'write',
      'close',
      'notice',
    ],
      `protected ${format.toUpperCase()} save`);
    assertReportedArtifact(snapshot, format, {
      passwordProtected: true,
      comparePersistedBlob: true,
    });
    assertBlob(snapshot, format);
    assertOneNotice(snapshot, format);
    const reopened = await verifyProtectedReopen(page, password, format);
    assert(reopened.unprotectedFailed, `protected ${format.toUpperCase()} bytes reject ordinary reopen`);
    assert(reopened.pageCount >= 1, `protected ${format.toUpperCase()} bytes reopen with the entered password`);
    assert(!reopened.localHasPassword && !reopened.sessionHasPassword,
      `protected ${format.toUpperCase()} password is not stored in browser storage`);
  }

  setTestCase('picker cancellation has no fallback, alert, or content-loss notice');
  await loadMemoryPatchedSample(page);
  await installPersistenceHarness(page, { picker: 'abort' });
  await markDocumentDirty(page);
  await clickFileCommand(page, 'file:save');
  await waitForHarnessEvent(page, 'picker:abort');
  await settleUi(page);
  snapshot = await harnessSnapshot(page);
  assertEventOrder(snapshot.events, [
    'reported:exportHwpxWithReport:report',
    'picker:abort',
  ], 'picker cancellation');
  assert(!snapshot.events.some((event) =>
    ['write', 'close', 'objectURL', 'anchor:click', 'anchor:throw'].includes(event)),
  'AbortError stops before writable and download fallback boundaries');
  assertReportedArtifact(snapshot, 'hwpx');
  assert(snapshot.alerts.length === 0, 'picker cancellation is not reported as a save error');
  assertNoNotice(snapshot, 'picker cancellation');
  assert(snapshot.dirty === true, 'picker cancellation leaves the document dirty');
  await markDocumentCleanForNavigation(page);

  setTestCase('Save As and password-dialog validation/cancellation never notify');
  await loadMemoryPatchedSample(page);
  await installPersistenceHarness(page, { picker: 'success', handleName: 'unused.hwp' });
  await markDocumentDirty(page);
  await clickFileCommand(page, EXPECTED.hwp.command);
  await clickDialogButton(page, '다른 이름으로 저장', '취소');
  await waitForDialog(page, '다른 이름으로 저장', true);
  await settleUi(page);
  snapshot = await harnessSnapshot(page);
  assert(snapshot.events.length === 0 && snapshot.alerts.length === 0,
    'initial Save As cancellation reaches no exporter persistence or error boundary');
  assertNoExporterCall(snapshot, 'initial Save As cancellation');
  assertNoNotice(snapshot, 'initial Save As cancellation');
  assert(snapshot.dirty === true, 'initial Save As cancellation leaves the document dirty');
  await markDocumentCleanForNavigation(page);

  await resetHarnessObservations(page);
  await markDocumentDirty(page);
  await clickFileCommand(page, EXPECTED.hwpx.command);
  await enterSaveAsName(page, 'issue4430-password-cancel', 'password');
  const firstMismatch = String.fromCharCode(97, 108, 112, 104, 97, 49);
  const secondMismatch = String.fromCharCode(97, 108, 112, 104, 97, 50);
  await page.type('#hwp-save-password-input', firstMismatch);
  await page.type('#hwp-save-password-confirmation', secondMismatch);
  await clickDialogButton(page, '문서 암호 설정', '확인');
  await page.waitForFunction(() => {
    const alertNode = document.querySelector('[role="dialog"][aria-label="문서 저장 암호 설정"] [role="alert"]');
    return alertNode && !alertNode.hidden && alertNode.textContent?.includes('일치하지 않습니다');
  });
  snapshot = await harnessSnapshot(page);
  assert(snapshot.events.length === 0 && snapshot.alerts.length === 0,
    'password validation failure reaches no exporter or persistence boundary');
  assertNoExporterCall(snapshot, 'password validation failure');
  assertNoNotice(snapshot, 'password validation failure');
  await clickDialogButton(page, '문서 암호 설정', '취소');
  await waitForDialog(page, '문서 암호 설정', true);
  await settleUi(page);
  snapshot = await harnessSnapshot(page);
  assertNoExporterCall(snapshot, 'password dialog cancellation');
  assertNoNotice(snapshot, 'password dialog cancellation');
  assert(snapshot.dirty === true, 'password dialog cancellation leaves the document dirty');
  await markDocumentCleanForNavigation(page);

  setTestCase('fallback-name cancellation discards the report without download or notice');
  await loadMemoryPatchedSample(page);
  await installPersistenceHarness(page, { picker: 'error', anchor: 'success' });
  await markDocumentDirty(page);
  await clickFileCommand(page, EXPECTED.hwp.command);
  await enterSaveAsName(page, 'issue4430-fallback-cancel', 'confirm');
  await waitForHarnessEvent(page, 'picker:error');
  await waitForDialog(page, '다른 이름으로 저장');
  await clickDialogButton(page, '다른 이름으로 저장', '취소');
  await waitForDialog(page, '다른 이름으로 저장', true);
  await settleUi(page);
  snapshot = await harnessSnapshot(page);
  assertEventOrder(snapshot.events, [
    'reported:exportHwpWithReport:report',
    'picker:error',
  ], 'fallback-name cancellation');
  assert(!snapshot.events.some((event) =>
    ['objectURL', 'anchor:click', 'anchor:throw', 'revokeObjectURL'].includes(event)),
  'fallback-name cancellation reaches neither object URL nor anchor');
  assertReportedArtifact(snapshot, 'hwp');
  assert(snapshot.alerts.length === 0, 'fallback-name cancellation is not a save error');
  assertNoNotice(snapshot, 'fallback-name cancellation after a real reported export');
  assert(snapshot.dirty === true, 'fallback-name cancellation leaves the document dirty');
  await markDocumentCleanForNavigation(page);

  setTestCase('anchor failure reports save error but never publishes content-loss notice');
  await loadMemoryPatchedSample(page);
  await installPersistenceHarness(page, { picker: 'error', anchor: 'error' });
  await markDocumentDirty(page);
  await clickFileCommand(page, 'file:save');
  await waitForHarnessEvent(page, 'alert');
  await waitForHarnessEvent(page, 'revokeObjectURL');
  snapshot = await harnessSnapshot(page);
  assertEventOrder(snapshot.events, [
    'reported:exportHwpxWithReport:report',
    'picker:error',
    'objectURL',
    'anchor:throw',
    'alert',
    'revokeObjectURL',
  ],
    'failed download');
  requireCondition(snapshot.events.filter((event) => event === 'revokeObjectURL').length === 1,
    'throwing download revokes its object URL exactly once after anchor failure');
  assertReportedArtifact(snapshot, 'hwpx', { comparePersistedBlob: true });
  assert(snapshot.alerts.length === 1
    && snapshot.alerts[0].includes('issue4430 deterministic anchor failure'),
  'anchor failure uses the normal save-error alert exactly once');
  assertNoNotice(snapshot, 'anchor failure after a real reported export');
  assert(snapshot.dirty === true, 'anchor failure leaves the document dirty');
  await markDocumentCleanForNavigation(page);
}, { skipLoadApp: true });
