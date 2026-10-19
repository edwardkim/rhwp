import test from 'node:test';
import assert from 'node:assert/strict';

import { RhwpEditor } from '../index.js';
import { EditorTransport } from '../transport.js';

function editorHarness(capabilities = ['field-focus-events-v1']) {
  const listeners = new Map();
  const transport = {
    request() { return Promise.resolve(undefined); },
    supports(capability) { return capabilities.includes(capability); },
    on(event, listener) {
      listeners.set(event, listener);
      return () => listeners.delete(event);
    },
    destroy() {},
  };
  return {
    editor: new RhwpEditor({ remove() {} }, transport),
    emit(event, payload) { listeners.get(event)?.(payload); },
    listeners,
  };
}

const entered = {
  schemaVersion: 1, inField: true, fieldId: 7, fieldType: 'clickhere', name: '기안자', guide: '이름',
};

test('누름틀 이벤트는 capability와 strict v1 payload를 사용한다', () => {
  const { editor, emit, listeners } = editorHarness();
  const received = [];
  const off = editor.onFieldChanged((event) => received.push(event));
  emit('fieldChanged', entered);
  emit('fieldChanged', { schemaVersion: 1, inField: false });
  // 잘못된 payload 는 버린다.
  emit('fieldChanged', { ...entered, extra: 1 });
  emit('fieldChanged', { schemaVersion: 2, inField: false });
  emit('fieldChanged', { schemaVersion: 1, inField: false, fieldId: 7 });
  emit('fieldChanged', { ...entered, fieldId: -1 });
  emit('fieldChanged', { ...entered, name: null });
  assert.deepEqual(received, [entered, { schemaVersion: 1, inField: false }]);
  off();
  assert.equal(listeners.has('fieldChanged'), false);

  const unsupported = editorHarness([]).editor;
  assert.throws(
    () => unsupported.onFieldChanged(() => {}),
    (error) => error.code === 'CAPABILITY_UNSUPPORTED',
  );
});

test('누름틀 이벤트는 모든 listener에 전달하고 마지막 해지에서 구독을 푼다', () => {
  const { editor, emit, listeners } = editorHarness();
  const first = [];
  const second = [];
  const offFirst = editor.onFieldChanged((event) => first.push(event.inField));
  const offSecond = editor.onFieldChanged((event) => {
    second.push(event.inField);
    throw new Error('listener 오류는 다른 listener를 막지 않는다');
  });
  emit('fieldChanged', entered);
  assert.deepEqual(first, [true]);
  assert.deepEqual(second, [true]);
  offFirst();
  assert.equal(listeners.has('fieldChanged'), true);
  offSecond();
  assert.equal(listeners.has('fieldChanged'), false);
});

test('EditorTransport는 협상한 이벤트만 전달한다', async () => {
  let server;
  const contentWindow = {
    postMessage(message, _targetOrigin, ports) {
      server = ports[0];
      server.start();
      server.postMessage({
        type: 'rhwp-connected', version: 1, sessionId: message.sessionId,
        // 스튜디오가 누름틀 이벤트만 광고한다 — documentChanged 는 버려야 한다.
        capabilities: ['transferable-array-buffer', 'field-focus-events-v1'],
      });
      queueMicrotask(() => {
        for (const [event, payload] of [
          ['documentChanged', { changeSeq: 1 }],
          ['fieldChanged', { inField: false }],
          ['unknownEvent', { x: 1 }],
        ]) {
          server.postMessage({
            type: 'rhwp-event', version: 1, sessionId: message.sessionId, event, payload,
          });
        }
      });
    },
  };
  const transport = new EditorTransport(
    { contentWindow },
    'https://studio.example/app',
    { window: { addEventListener() {}, removeEventListener() {} } },
  );
  const received = [];
  for (const event of ['documentChanged', 'fieldChanged', 'unknownEvent']) {
    transport.on(event, (payload) => received.push([event, payload]));
  }
  await transport.connect();
  await new Promise((resolve) => setTimeout(resolve, 0));
  assert.deepEqual(received, [['fieldChanged', { inField: false }]]);
  transport.destroy();
  server.close();
});
