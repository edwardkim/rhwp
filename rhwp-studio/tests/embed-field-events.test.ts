import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';

import { createFieldChangeForwarder } from '../src/embed/field-events.ts';
import { installEmbedRuntime } from '../src/embed/runtime.ts';
import type { EmbedRpcHandlers } from '../src/embed/rpc-router.ts';

test('누름틀 이벤트는 들어감·바뀜·나감에서만 나오고 같은 누름틀 안의 이동은 거른다', () => {
  const names = new Map([[7, '기안자'], [9, '기안일']]);
  const forward = createFieldChangeForwarder((id) => names.get(id) ?? '');
  const info = (fieldId: number, guideName = '안내') => ({ fieldId, fieldType: 'clickhere', guideName });

  assert.equal(forward(null), null, '밖에서 밖으로는 보내지 않는다');
  assert.deepEqual(forward(info(7, '이름')), {
    schemaVersion: 1, inField: true, fieldId: 7, fieldType: 'clickhere', name: '기안자', guide: '이름',
  });
  assert.equal(forward(info(7, '이름')), null, '같은 누름틀 안의 캐럿 이동');
  assert.equal(forward(info(9))?.name, '기안일', '다른 누름틀로 바로 옮김');
  assert.deepEqual(forward(null), { schemaVersion: 1, inField: false });
  assert.equal(forward(undefined), null);
  assert.equal(forward(info(42))?.name, '', '이름을 못 찾으면 빈 문자열');
});

test('main.ts 는 field-info-changed 를 연결마다 새 forwarder 로 내보낸다', () => {
  const source = readFileSync(new URL('../src/main.ts', import.meta.url), 'utf8');
  assert.match(
    source,
    /subscribeFieldChanged: \(listener\) => \{[\s\S]*?createFieldChangeForwarder\([\s\S]*?eventBus\.on\('field-info-changed'/,
  );
});

async function connect(capabilities: string[]) {
  let messageListener: (event: MessageEvent) => void = () => {};
  let emitField: (payload: unknown) => void = () => {};
  let subscribed = 0;
  let unsubscribed = 0;
  const hostWindow = {
    addEventListener(_type: string, listener: (event: MessageEvent) => void) { messageListener = listener; },
    removeEventListener() {},
  };
  const parentWindow = { postMessage() {} };
  const cleanup = installEmbedRuntime({
    hostWindow: hostWindow as unknown as Window,
    parentWindow: parentWindow as unknown as Window,
    handlers: {} as EmbedRpcHandlers,
    subscribeFieldChanged(listener) {
      subscribed += 1;
      emitField = listener;
      return () => { unsubscribed += 1; };
    },
  });
  const channel = new MessageChannel();
  const messages: unknown[] = [];
  const connected = new Promise<void>((resolve) => {
    channel.port1.onmessage = ({ data }) => {
      messages.push(data);
      if (data.type === 'rhwp-connected') resolve();
    };
    channel.port1.start();
  });
  messageListener({
    data: { type: 'rhwp-connect', version: 1, sessionId: 'field-event', capabilities },
    source: parentWindow,
    origin: 'https://host.example',
    ports: [channel.port2],
  } as unknown as MessageEvent);
  await connected;
  return {
    messages, cleanup, channel,
    emit: (payload: unknown) => emitField(payload),
    counts: () => ({ subscribed, unsubscribed }),
  };
}

test('embed runtime 은 field-focus-events-v1 을 협상한 연결에만 fieldChanged 를 보낸다', async () => {
  const payload = { schemaVersion: 1, inField: false };
  const on = await connect(['transferable-array-buffer', 'field-focus-events-v1']);
  on.emit(payload);
  await new Promise((resolve) => setTimeout(resolve, 10));
  assert.deepEqual(on.messages.filter((m) => (m as { type: string }).type === 'rhwp-event'), [{
    type: 'rhwp-event', version: 1, sessionId: 'field-event', event: 'fieldChanged', payload,
  }]);
  on.cleanup();
  assert.deepEqual(on.counts(), { subscribed: 1, unsubscribed: 1 });
  on.channel.port1.close();

  const off = await connect(['transferable-array-buffer']);
  assert.deepEqual(off.counts(), { subscribed: 0, unsubscribed: 0 }, '협상 안 하면 구독하지 않는다');
  off.cleanup();
  off.channel.port1.close();
});
