import test from 'node:test';
import assert from 'node:assert/strict';
import { createServer } from 'vite';
import { fileURLToPath } from 'node:url';

const vite = await createServer({ root: fileURLToPath(new URL('..', import.meta.url)),
  appType: 'custom', logLevel: 'silent', server: { middlewareMode: true } });
const { CommandErrorBoundary, StudioCommandError, inspectCommandError, installCommandErrorGlobalRouting } =
  await vite.ssrLoadModule('/src/core/command-error.ts');
const { SnapshotCommand } = await vite.ssrLoadModule('/src/engine/command.ts');
const { CommandHistory } = await vite.ssrLoadModule('/src/engine/history.ts');
test.after(() => vite.close());
const pos = { sectionIndex: 0, paragraphIndex: 0, charOffset: 0 };

function fixture() {
  let content = 'original', next = 0, generation = 1;
  const snapshots = new Map();
  const events: any[] = [];
  const wasm = {
    saveSnapshot() { snapshots.set(++next, content); return next; },
    restoreSnapshot(id: number) { assert.ok(snapshots.has(id)); content = snapshots.get(id); },
    discardSnapshot(id: number) { snapshots.delete(id); },
    snapshotCapacity() { return 100; },
  };
  const options = { generation: () => generation, refresh: async () => {}, probe: () => {},
    publish: (payload: any) => { events.push(payload); } };
  const boundary = new CommandErrorBoundary(options);
  const history = new CommandHistory();
  const checkpoint = { restoreEditing() {}, editingUnchanged: () => true,
    historyUnchanged: history.recoveryCheckpoint(), dirtyUnchanged: () => true };
  return { wasm, boundary, options, history, checkpoint, snapshots, events,
    read: () => content, write: (v: string) => { content = v; }, replace: () => { generation++; } };
}

for (const rollback of ['rejected', 'restored', 'failed', 'unknown']) {
  test(`v1 recovery classification: ${rollback}`, async () => {
    const f = fixture();
    const e = f.boundary.fail('paste', new Error('failure'), rollback, f.checkpoint, f.boundary.sessionId());
    assert.ok(e instanceof StudioCommandError);
    assert.throws(() => f.boundary.assertReady(), value => value === e);
    const p = await e.report;
    assert.equal(p.recovery, ['rejected', 'restored'].includes(rollback) ? 'succeeded' : rollback);
    assert.equal(f.events.length, 1);
    assert.equal(inspectCommandError(p, f.boundary.sessionId(), new Set()), 'accept');
    if (p.recovery === 'succeeded') f.boundary.assertReady();
    else assert.throws(() => f.boundary.assertReady(), value => value === e);
  });
}

for (const bad of ['editing', 'history', 'dirty', 'render', 'probe', 'trap']) {
  test(`does not certify incomplete recovery: ${bad}`, async () => {
    const f = fixture();
    if (bad === 'editing') f.checkpoint.editingUnchanged = () => false;
    if (bad === 'history') f.checkpoint.historyUnchanged = () => false;
    if (bad === 'dirty') f.checkpoint.dirtyUnchanged = () => false;
    if (bad === 'render') f.options.refresh = async () => { throw new Error('paint'); };
    if (bad === 'probe') f.options.probe = () => { throw new Error('trap after rollback'); };
    const p = await f.boundary.fail('edit', bad === 'trap' ? new WebAssembly.RuntimeError('unreachable') : 'primitive',
      'restored', f.checkpoint, f.boundary.sessionId()).report;
    assert.equal(p.recovery, 'unknown');
    assert.equal(p.checks.ready, false);
  });
}

test('snapshot restores actual content, preserves redo/history and supports a following command', async () => {
  const f = fixture();
  f.history.execute(new SnapshotCommand('first', pos, pos, () => { f.write('first'); return pos; }), f.wasm);
  f.history.undo(f.wasm);
  const unchanged = f.history.recoveryCheckpoint();
  const cmd = new SnapshotCommand('failing', pos, pos, () => { f.write('partial'); throw new Error('injected'); });
  let error;
  try { f.history.execute(cmd, f.wasm); } catch (e) { error = e; }
  assert.equal(f.read(), 'original');
  assert.equal(f.history.lastFailureRecovery, 'restored');
  const p = await f.boundary.fail('failing', error, f.history.lastFailureRecovery,
    { ...f.checkpoint, historyUnchanged: unchanged }, f.boundary.sessionId()).report;
  assert.equal(p.recovery, 'succeeded');
  assert.ok(unchanged());
  f.history.redo(f.wasm); assert.equal(f.read(), 'first');
  f.history.undo(f.wasm); assert.equal(f.read(), 'original');
  f.history.execute(new SnapshotCommand('next', pos, pos, () => { f.write('next'); return pos; }), f.wasm);
  assert.equal(f.read(), 'next');
  f.history.undo(f.wasm); assert.equal(f.read(), 'original');
});

test('rollback failure keeps partial content and is never a recovered rejection', async () => {
  const f = fixture();
  f.wasm.restoreSnapshot = () => { throw new Error('rollback failed'); };
  let error;
  try { f.history.execute(new SnapshotCommand('failure', pos, pos, () => {
    f.write('partial'); throw new Error('mutation failed');
  }), f.wasm); } catch (e) { error = e; }
  assert.ok(error instanceof AggregateError);
  assert.equal(f.read(), 'partial');
  assert.equal(f.history.lastFailureRecovery, 'failed');
  assert.equal((await f.boundary.fail('failure', error, f.history.lastFailureRecovery,
    f.checkpoint, f.boundary.sessionId()).report).recovery, 'failed');
});

test('stale document completion neither restores nor blocks the replacement', async () => {
  const f = fixture(); let restored = false;
  f.checkpoint.restoreEditing = () => { restored = true; };
  const old = f.boundary.sessionId();
  const e = f.boundary.fail('old', new Error(), 'restored', f.checkpoint, old);
  f.replace();
  const payload = await e.report;
  assert.equal(restored, false);
  assert.equal(inspectCommandError(payload, f.boundary.sessionId(), new Set()), 'stale');
  f.boundary.assertReady();
});

test('duplicate, unsupported and malformed payloads cannot establish recovery', async () => {
  const f = fixture();
  const p = await f.boundary.fail('test', new Error(), 'restored', f.checkpoint, f.boundary.sessionId()).report;
  const seen = new Set();
  assert.equal(inspectCommandError(p, f.boundary.sessionId(), seen), 'accept');
  assert.equal(inspectCommandError(p, f.boundary.sessionId(), seen), 'duplicate');
  assert.equal(inspectCommandError({ ...p, schemaVersion: 2 }, f.boundary.sessionId(), seen), 'unsupported');
  assert.equal(inspectCommandError({ ...p, checks: { ...p.checks, history: false } }, f.boundary.sessionId(), seen), 'invalid');
  assert.equal(inspectCommandError({ ...p, code: 'INTEGRITY_UNKNOWN' }, f.boundary.sessionId(), seen), 'invalid');
  assert.equal(inspectCommandError({}, f.boundary.sessionId(), seen), 'unsupported');
});

test('only owned typed errors use the structured route; initialization errors retain global route', async () => {
  const target = new EventTarget();
  const off = installCommandErrorGlobalRouting(target);
  const f = fixture(); const error = f.boundary.fail('test', new Error(), 'unknown', f.checkpoint, f.boundary.sessionId());
  for (const type of ['error', 'unhandledrejection']) {
    for (const owned of [true, false]) {
      const event = new Event(type, { cancelable: true });
      Object.assign(event, { error: owned ? error : new Error('init'), reason: owned ? error : new Error('init') });
      target.dispatchEvent(event);
      assert.equal(event.defaultPrevented, owned);
    }
  }
  await error.report; off();
});

for (const negotiated of [true, false]) {
  test(`MessagePort negotiates errors and never returns success for failed RPC (${negotiated})`, async () => {
    const { installEmbedRuntime } = await vite.ssrLoadModule('/src/embed/runtime.ts');
    let onMessage: any;
    const parent = {};
    let subscribed = 0, unsubscribed = 0, emit = (_payload: any) => {};
    const f = fixture();
    f.options.publish = payload => emit(payload);
    const failure = f.boundary.fail('rpc', new Error('failure'), 'restored', f.checkpoint, f.boundary.sessionId());
    const cleanup = installEmbedRuntime({
      hostWindow: { addEventListener(_name: string, handler: any) { onMessage = handler; }, removeEventListener() {} },
      parentWindow: parent,
      handlers: { pageCount: async () => { throw failure; }, getCommandErrorState: async () => f.boundary.state() },
      subscribeCommandError(listener: any) { subscribed++; emit = listener; return () => { unsubscribed++; }; },
    });
    const channel = new MessageChannel();
    const received: any[] = [];
    const response = new Promise<any>(resolve => {
      channel.port1.onmessage = ({ data }) => {
        received.push(data);
        if (data.type === 'rhwp-connected') channel.port1.postMessage({ type: 'rhwp-request', version: 1,
          sessionId: 'transport', id: 1, method: 'pageCount' });
        if (data.id === 1) resolve(data);
      };
    });
    onMessage({ source: parent, origin: 'https://host.example', ports: [channel.port2], data: {
      type: 'rhwp-connect', version: 1, sessionId: 'transport',
      capabilities: ['transferable-array-buffer', ...(negotiated ? ['command-error-v1'] : [])],
    } });
    const result = await response;
    assert.equal('result' in result, false);
    assert.equal(subscribed, negotiated ? 1 : 0);
    if (negotiated) {
      assert.equal(result.error.commandError.eventId, (await failure.report).eventId);
      assert.equal(received.filter(e => e.event === 'commandError').length, 1);
      assert.equal(result.error.code, 'COMMAND_ROLLED_BACK');
    } else {
      assert.equal(result.error.commandError, undefined);
      assert.equal(result.error.code, 'RPC_ERROR');
    }
    cleanup(); channel.port1.close();
    assert.equal(unsubscribed, subscribed);
  });
}

test('boundary initializes without crypto.randomUUID in insecure contexts (#8353)', () => {
  // LAN IP의 http 등 비보안 컨텍스트에서는 crypto.randomUUID가 없다.
  const original = Object.getOwnPropertyDescriptor(globalThis.crypto, 'randomUUID');
  const proto = Object.getPrototypeOf(globalThis.crypto);
  const protoOriginal = Object.getOwnPropertyDescriptor(proto, 'randomUUID');
  Object.defineProperty(globalThis.crypto, 'randomUUID', { value: undefined, configurable: true });
  try {
    assert.equal(typeof globalThis.crypto.randomUUID, 'undefined');
    const a = fixture().boundary, b = fixture().boundary;
    assert.equal(a.state().status, 'ready');
    assert.match(a.sessionId(), /^.+:1$/);
    assert.notEqual(a.sessionId(), b.sessionId());
  } finally {
    if (original) Object.defineProperty(globalThis.crypto, 'randomUUID', original);
    else delete (globalThis.crypto as any).randomUUID;
    assert.equal(Object.getOwnPropertyDescriptor(proto, 'randomUUID')?.value, protoOriginal?.value);
  }
});
