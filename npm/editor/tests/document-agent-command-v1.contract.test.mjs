import test from 'node:test';
import assert from 'node:assert/strict';

import { RhwpEditor } from '../index.js';

const SHA_A = 'a'.repeat(64);
const SHA_B = 'b'.repeat(64);
const SHA_C = 'c'.repeat(64);
const SHA_D = 'd'.repeat(64);

function target() {
  return {
    kind: 'body_paragraph',
    section: 0,
    paragraph: 2,
    charOffset: 0,
    length: 7,
  };
}

function state() {
  return {
    schemaVersion: 1,
    format: 'hwp',
    documentEpoch: 3,
    changeSeq: 11,
    dirty: true,
    pageCount: 2,
    documentSha256: SHA_A,
  };
}

function selection() {
  return {
    schemaVersion: 1,
    documentEpoch: 3,
    changeSeq: 11,
    page: 2,
    editable: true,
    collapsed: true,
    target: target(),
    selectedTextSha256: null,
  };
}

function applyCommand() {
  return {
    schemaVersion: 1,
    commandId: 'cmd-001',
    expectedDocumentEpoch: 3,
    expectedChangeSeq: 11,
    expectedDocumentSha256: SHA_A,
    target: target(),
    expectedBeforeSha256: SHA_B,
    expectedFormatSha256: SHA_C,
    expectedAdjacentContextSha256: SHA_D,
    replacement: '새 문단',
  };
}

function revertCommand() {
  return {
    schemaVersion: 1,
    commandId: 'cmd-001',
    expectedDocumentEpoch: 3,
    expectedChangeSeq: 12,
    expectedAfterDocumentSha256: SHA_B,
    expectedAfterSha256: SHA_C,
  };
}

function receipt(operation = 'apply') {
  return {
    schemaVersion: 1,
    commandId: 'cmd-001',
    operation,
    documentEpoch: 3,
    beforeChangeSeq: operation === 'apply' ? 11 : 12,
    afterChangeSeq: operation === 'apply' ? 12 : 13,
    beforeDocumentSha256: SHA_A,
    afterDocumentSha256: SHA_B,
    beforeTextSha256: SHA_C,
    afterTextSha256: SHA_D,
    formatSha256: SHA_A,
    adjacentContextSha256: SHA_B,
    pageCountBefore: 2,
    pageCountAfter: 2,
    target: target(),
  };
}

function editorHarness(results, capabilities = [
  'document-state-v1',
  'selection-context-v1',
  'document-agent-command-v1',
  'target-navigation-v1',
  'document-change-events-v1',
]) {
  const requests = [];
  const listeners = new Map();
  const transport = {
    request(method, params) {
      requests.push({ method, params });
      return Promise.resolve(results[method]);
    },
    supports(capability) { return capabilities.includes(capability); },
    on(event, listener) {
      listeners.set(event, listener);
      return () => listeners.delete(event);
    },
    destroy() {},
  };
  return {
    editor: new RhwpEditor({ remove() {} }, transport),
    requests,
    emit(event, payload) { listeners.get(event)?.(payload); },
    listeners,
  };
}

test('문서 에이전트 공개 API는 exact RPC 메서드와 파라미터를 사용한다', async () => {
  const results = {
    getDocumentState: state(),
    getSelectionContext: selection(),
    applyTextCommand: receipt('apply'),
    revertTextCommand: receipt('revert'),
    focusTarget: { focused: true, page: 2 },
  };
  const { editor, requests } = editorHarness(results);

  assert.deepEqual(await editor.getDocumentState(), state());
  assert.deepEqual(await editor.getSelectionContext(), selection());
  assert.deepEqual(await editor.applyTextCommand(applyCommand()), receipt('apply'));
  assert.deepEqual(await editor.revertTextCommand(revertCommand()), receipt('revert'));
  assert.deepEqual(await editor.focusTarget(target()), { focused: true, page: 2 });

  assert.deepEqual(requests, [
    { method: 'getDocumentState', params: {} },
    { method: 'getSelectionContext', params: {} },
    { method: 'applyTextCommand', params: { command: applyCommand() } },
    { method: 'revertTextCommand', params: { command: revertCommand() } },
    { method: 'focusTarget', params: { target: target() } },
  ]);
});

test('문서 에이전트 공개 API는 capability가 없으면 요청 전에 실패한다', async () => {
  const { editor, requests } = editorHarness({}, []);

  for (const call of [
    () => editor.getDocumentState(),
    () => editor.getSelectionContext(),
    () => editor.applyTextCommand(applyCommand()),
    () => editor.revertTextCommand(revertCommand()),
    () => editor.focusTarget(target()),
  ]) {
    await assert.rejects(call, (error) => error.code === 'CAPABILITY_UNSUPPORTED');
  }
  assert.deepEqual(requests, []);
});

test('문서 에이전트 공개 API는 extra key와 잘못된 SHA를 요청 전에 거부한다', async () => {
  const { editor, requests } = editorHarness({});

  await assert.rejects(
    () => editor.applyTextCommand({ ...applyCommand(), unexpected: true }),
    (error) => error.code === 'INVALID_COMMAND',
  );
  await assert.rejects(
    () => editor.applyTextCommand({ ...applyCommand(), expectedDocumentSha256: 'not-a-sha' }),
    (error) => error.code === 'INVALID_COMMAND',
  );
  await assert.rejects(
    () => editor.applyTextCommand({ ...applyCommand(), replacement: '두 문단\n금지' }),
    (error) => error.code === 'INVALID_COMMAND',
  );
  await assert.rejects(
    () => editor.focusTarget({ ...target(), charOffset: 1 }),
    (error) => error.code === 'INVALID_COMMAND',
  );
  assert.deepEqual(requests, []);
});

test('문서 에이전트 공개 API는 malformed 응답을 명시적으로 거부한다', async () => {
  const malformedState = { ...state(), pageCount: Number.NaN };
  const malformedSelection = { ...selection(), page: 0 };
  const malformedReceipt = { ...receipt(), extra: true };
  const malformedFocus = { focused: true, page: 1, extra: true };
  const { editor } = editorHarness({
    getDocumentState: malformedState,
    getSelectionContext: malformedSelection,
    applyTextCommand: malformedReceipt,
    focusTarget: malformedFocus,
  });

  await assert.rejects(
    () => editor.getDocumentState(),
    (error) => error.code === 'INVALID_RESPONSE',
  );
  await assert.rejects(
    () => editor.getSelectionContext(),
    (error) => error.code === 'INVALID_RESPONSE',
  );
  await assert.rejects(
    () => editor.applyTextCommand(applyCommand()),
    (error) => error.code === 'INVALID_RESPONSE',
  );
  await assert.rejects(
    () => editor.focusTarget(target()),
    (error) => error.code === 'INVALID_RESPONSE',
  );
});

test('문서 변경 이벤트는 capability와 strict v1 payload를 사용한다', () => {
  const { editor, emit, listeners } = editorHarness({});
  const received = [];
  const off = editor.onDocumentChanged((event) => received.push(event));
  const event = {
    schemaVersion: 1,
    reason: 'agent_apply',
    documentEpoch: 3,
    changeSeq: 12,
    commandId: 'cmd-001',
  };
  emit('documentChanged', event);
  assert.deepEqual(received, [event]);
  off();
  assert.equal(listeners.has('documentChanged'), false);

  const unsupported = editorHarness({}, []).editor;
  assert.throws(
    () => unsupported.onDocumentChanged(() => {}),
    (error) => error.code === 'CAPABILITY_UNSUPPORTED',
  );
});

test('문서 변경 이벤트는 모든 listener에 한 번 전달하고 stale epoch/seq를 버린다', () => {
  const { editor, emit } = editorHarness({});
  const first = [];
  const second = [];
  editor.onDocumentChanged(event => first.push(event.changeSeq));
  editor.onDocumentChanged(event => second.push(event.changeSeq));

  const event = (documentEpoch, changeSeq) => ({
    schemaVersion: 1,
    reason: 'agent_apply',
    documentEpoch,
    changeSeq,
    commandId: `cmd-${documentEpoch}-${changeSeq}`,
  });
  emit('documentChanged', event(3, 2));
  emit('documentChanged', event(3, 2));
  emit('documentChanged', event(2, 99));
  emit('documentChanged', event(3, 3));

  assert.deepEqual(first, [2, 3]);
  assert.deepEqual(second, [2, 3]);
});
