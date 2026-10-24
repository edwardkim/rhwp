import assert from 'node:assert/strict';
import { mkdirSync, writeFileSync } from 'node:fs';
import { runTest, createNewDocument, setTestCase } from './helpers.mjs';
const output = '../output/pr-review/issue-8104';
mkdirSync(output, { recursive: true });
await runTest('#8104 command recovery and embedding contract', async ({ page }) => {
  const results = [];
  for (const mode of ['rollback-clean', 'rollback-dirty', 'rejected', 'rollback-failed', 'partial', 'trap', 'render-failed', 'cursor-query-failed']) {
    setTestCase(mode);
    await createNewDocument(page);
    const result = await page.evaluate(async mode => {
      const ih = window.__inputHandler, w = window.__wasm, api = window.rhwpCommandErrors;
      const origin = { sectionIndex: 0, paragraphIndex: 0, charOffset: 0 };
      ih.cursor.moveTo(origin); ih.cursor.clearSelection();
      const edit = text => ih.executeOperation({ kind: 'snapshot', operationType: 'fixture', operation: wasm => {
        wasm.insertText(0, 0, 0, text); return { ...origin, charOffset: text.length };
      } });
      if (mode !== 'rollback-clean') {
        edit('original'); edit('redo'); ih.handleUndo();
        ih.cursor.selectRange(origin, { ...origin, charOffset: 4 });
      }
      const bytes = Array.from(w.exportHwp());
      const cursor = JSON.stringify({ position: ih.cursor.getPosition(), selection: ih.cursor.getSelection() });
      const history = ih.history.recoveryCheckpoint();
      const dirtyEvents = [], events = [];
      const offDirty = window.__eventBus.on('document-dirty-changed', e => dirtyEvents.push(e));
      const listener = e => events.push(e.detail);
      window.addEventListener('rhwp-command-error', listener);
      const cursorQuery = w.getCursorRect;
      const restore = w.restoreSnapshot, render = window.__canvasView.refreshDocumentAgentMutation;
      if (mode === 'rollback-failed') w.restoreSnapshot = () => { throw new Error('injected rollback'); };
      if (mode === 'render-failed') window.__canvasView.refreshDocumentAgentMutation = async () => { throw new Error('injected render'); };
      if (mode === 'cursor-query-failed') w.getCursorRect = () => { throw new Error('injected cursor query'); };
      if (mode === 'rejected') ih.setEditMode('form');
      let failure;
      try {
        const operation = wasm => {
          wasm.insertText(0, 0, 0, 'PARTIAL'); ih.cursor.moveTo({ ...origin, charOffset: 3 });
          throw mode === 'trap' ? new WebAssembly.RuntimeError('injected trap') : new Error('injected operation');
        };
        if (mode === 'partial') ih.executeOperation({ kind: 'command', command: {
          type: 'partial', timestamp: Date.now(), execute: operation, undo() { return origin; }, mergeWith() { return null; },
        } });
        else ih.executeOperation({ kind: 'snapshot', operationType: 'injected', operation });
      } catch (e) { failure = e; }
      const owned = api.isHandledError(failure), payload = owned ? await failure.report : null;
      w.getCursorRect = cursorQuery; w.restoreSnapshot = restore; window.__canvasView.refreshDocumentAgentMutation = render; ih.setEditMode('normal');
      const preservedBytes = JSON.stringify(bytes) === JSON.stringify(Array.from(w.exportHwp()));
      const preservedCursor = cursor === JSON.stringify({ position: ih.cursor.getPosition(), selection: ih.cursor.getSelection() });
      const preservedHistory = history(), failureDirtyEvents = dirtyEvents.length;
      let following = false;
      if (payload?.recovery === 'succeeded') {
        ih.cursor.clearSelection(); edit('NEXT'); ih.handleUndo();
        following = JSON.stringify(bytes) === JSON.stringify(Array.from(w.exportHwp()));
      }
      window.removeEventListener('rhwp-command-error', listener); offDirty();
      return { owned, payload, eventCount: events.length, preservedBytes, preservedCursor, preservedHistory,
        failureDirtyEvents, following, state: api.getState() };
    }, mode);
    results.push({ mode, ...result });
    writeFileSync(`${output}/browser-results.json`, JSON.stringify(results, null, 2));
    assert.equal(result.owned, true, `${mode}: typed failure`); assert.equal(result.eventCount, 1);
    const success = ['rollback-clean', 'rollback-dirty', 'rejected'].includes(mode);
    assert.equal(result.payload.recovery, success ? 'succeeded' : mode === 'rollback-failed' ? 'failed' : 'unknown', mode);
    if (success) {
      for (const key of ['preservedBytes', 'preservedCursor', 'preservedHistory', 'following']) assert.equal(result[key], true, `${mode}: ${key}`);
      assert.equal(result.failureDirtyEvents, 0);
    } else assert.equal(result.state.status, 'blocked');
  }
  setTestCase('cell selection rollback');
  await createNewDocument(page);
  await page.evaluate(async () => {
    const ih = window.__inputHandler;
    const { tableCommands } = await import('/src/command/commands/table.ts');
    tableCommands.find(c => c.id === 'table:create').execute({ getInputHandler: () => ih });
  });
  const grid = await page.waitForSelector('[data-row="1"][data-col="1"]', { visible: true });
  await grid.hover(); await grid.click();
  const cell = await page.evaluate(async () => {
    const ih = window.__inputHandler, w = window.__wasm, p = ih.cursor.getPosition();
    ih.executeOperation({ kind: 'snapshot', operationType: 'cell-setup', operation(wasm) {
      wasm.insertTextInCell(0, p.parentParaIndex, p.controlIndex, 0, 0, 0, 'alpha'); return p;
    } });
    ih.cursor.selectAllInCell();
    const before = Array.from(w.exportHwp()), selection = JSON.stringify(ih.cursor.getSelection());
    let failure;
    try { ih.executeOperation({ kind: 'snapshot', operationType: 'cell-delete-failed', operation(wasm) {
      wasm.deleteTextInCell(0, p.parentParaIndex, p.controlIndex, 0, 0, 0, 5);
      ih.cursor.clearSelection(); throw new Error('cell failure');
    } }); } catch (e) { failure = e; }
    const payload = await failure.report;
    const preserved = JSON.stringify(before) === JSON.stringify(Array.from(w.exportHwp()));
    const selected = selection === JSON.stringify(ih.cursor.getSelection());
    ih.deleteSelection(); ih.handleUndo();
    return { recovery: payload.recovery, preserved, selected,
      following: JSON.stringify(before) === JSON.stringify(Array.from(w.exportHwp())) };
  });
  assert.deepEqual(cell, { recovery: 'succeeded', preserved: true, selected: true, following: true });
  results.push({ cell });
  setTestCase('late completion after actual document replacement');
  await createNewDocument(page);
  const replacement = await page.evaluate(async () => {
    const ih = window.__inputHandler, view = window.__canvasView, api = window.rhwpCommandErrors;
    const original = view.refreshDocumentAgentMutation;
    let release;
    view.refreshDocumentAgentMutation = () => new Promise(resolve => { release = resolve; });
    let error;
    try { ih.executeOperation({ kind: 'snapshot', operationType: 'late', operation() { throw new Error('late'); } }); }
    catch (e) { error = e; }
    await Promise.resolve();
    const old = api.getState().documentSessionId;
    window.__wasm.createNewDocument();
    const current = api.getState().documentSessionId;
    release();
    const payload = await error.report;
    view.refreshDocumentAgentMutation = original;
    return { changed: old !== current, status: api.getState().status, recovery: payload.recovery,
      accepted: api.inspect(payload, current, new Set()) };
  });
  assert.deepEqual(replacement, { changed: true, status: 'ready', recovery: 'unknown', accepted: 'stale' });
  results.push({ replacement });
  setTestCase('owned failure at the real window error boundary');
  await createNewDocument(page);
  const global = await page.evaluate(() => new Promise(resolve => {
    let prevented = false;
    const onError = e => { prevented = e.defaultPrevented && window.rhwpCommandErrors.isHandledError(e.error); };
    window.addEventListener('error', onError);
    window.addEventListener('rhwp-command-error', e => {
      window.removeEventListener('error', onError);
      resolve({ prevented, recovery: e.detail.recovery });
    }, { once: true });
    setTimeout(() => window.__inputHandler.executeOperation({ kind: 'snapshot', operationType: 'global',
      operation() { throw new Error('injected DOM event failure'); } }), 0);
  }));
  assert.deepEqual(global, { prevented: true, recovery: 'succeeded' });
  results.push({ global });
  setTestCase('MessagePort negotiation and document replacement');
  await createNewDocument(page);
  const embedding = await page.evaluate(async () => {
    const channel = new MessageChannel(), messages = [];
    channel.port1.onmessage = e => messages.push(e.data); channel.port1.start();
    window.postMessage({ type: 'rhwp-connect', version: 1, sessionId: 'host', capabilities: ['transferable-array-buffer', 'command-error-v1'] }, location.origin, [channel.port2]);
    const wait = async predicate => {
      for (let i = 0; i < 200 && !predicate(); i++) await new Promise(r => setTimeout(r, 10));
      if (!predicate()) throw new Error('embedding timeout');
    };
    await wait(() => messages.some(m => m.type === 'rhwp-connected'));
    channel.port1.postMessage({ type: 'rhwp-request', version: 1, sessionId: 'host', id: 1, method: 'getCommandErrorState' });
    await wait(() => messages.some(m => m.id === 1));
    const session = messages.find(m => m.id === 1).result.documentSessionId;
    let failure;
    try { window.__inputHandler.executeOperation({ kind: 'snapshot', operationType: 'host-test', operation() { throw new Error('reject'); } }); }
    catch (e) { failure = e; }
    const payload = await failure.report;
    await wait(() => messages.some(m => m.event === 'commandError'));
    const event = messages.find(m => m.event === 'commandError'), seen = new Set(), api = window.rhwpCommandErrors;
    const result = { accepted: api.inspect(event.payload, session, seen), duplicate: api.inspect(payload, session, seen),
      stale: api.inspect(payload, 'replacement-session', new Set()), malformed: api.inspect({ ...payload, checks: {} }, session, new Set()),
      unsupported: api.inspect({ ...payload, schemaVersion: 2 }, session, new Set()),
      sessionMatches: payload.documentSessionId === session, negotiated: messages[0].capabilities.includes('command-error-v1') };
    channel.port1.close(); return result;
  });
  assert.deepEqual(embedding, { accepted: 'accept', duplicate: 'duplicate', stale: 'stale', malformed: 'invalid', unsupported: 'unsupported', sessionMatches: true, negotiated: true });
  results.push({ embedding });
  writeFileSync(`${output}/browser-results.json`, JSON.stringify(results, null, 2));
});
