import assert from 'node:assert/strict';
import test from 'node:test';
import { DesktopHost, type DesktopEditor, type DesktopState } from '../src/desktop/desktop-host.ts';

const documentBytes = Uint8Array.of(0, 77, 127, 255);
const initialState: DesktopState = {
  hasDocument: true,
  fileName: 'old.hwp',
  format: 'hwp',
  dirty: true,
  documentEpoch: 3,
  changeSeq: 2,
  canNativeSave: true,
};

function createHost() {
  let receive: (message: unknown) => void = () => {};
  let id = 0;
  let locks = 0;
  let saved = 0;
  let discarded = 0;
  const state = { ...initialState };
  const emitted: Record<string, unknown>[] = [];
  let onLoad: (() => void) | undefined;
  const editor: DesktopEditor = {
    state: () => ({ ...state }),
    load: async (bytes, fileName, validate) => {
      onLoad?.();
      validate();
      assert.deepEqual(bytes, documentBytes);
      Object.assign(state, { fileName, documentEpoch: state.documentEpoch + 1, changeSeq: 0, dirty: false });
    },
    export: () => documentBytes,
    saved: async fileName => {
      saved++;
      Object.assign(state, { fileName, dirty: false });
    },
    discard: async () => {
      discarded++;
      state.dirty = false;
    },
    theme: () => {},
    lock: () => {
      locks++;
      return () => { locks--; };
    },
  };
  const host = new DesktopHost({
    version: 1,
    subscribe(listener) {
      receive = listener;
      return () => {};
    },
    postMessage(message) { emitted.push(message as Record<string, unknown>); },
  }, editor);
  async function send(method: string, params: object = {}) {
    const requestId = ++id;
    receive({ version: 1, kind: 'request', id: requestId, method, params });
    await new Promise(resolve => setImmediate(resolve));
    const response = emitted.find(message => message.kind === 'response' && message.id === requestId);
    assert.ok(response);
    return response;
  }
  return {
    host, send, receive, state, emitted,
    changeDuringOpen: () => { onLoad = () => { state.changeSeq++; }; },
    get locks() { return locks; },
    get saved() { return saved; },
    get discarded() { return discarded; },
  };
}

test('native open checks chunk order and the revision both before and during parsing', async () => {
  const app = createHost();
  const file = { fileName: '한글.hwp', byteLength: documentBytes.length, documentEpoch: 3, changeSeq: 2 };
  assert.equal((await app.send('load.begin', { ...file, changeSeq: 1 })).ok, false);
  assert.equal((await app.send('load.begin', file)).ok, true);
  assert.equal(app.locks, 1);
  assert.equal((await app.send('load.chunk', { offset: 1, bytes: 'AE1//w==' })).ok, false);
  assert.equal((await app.send('load.commit')).ok, false);
  assert.equal((await app.send('load.chunk', { offset: 0, bytes: 'AE0=' })).ok, true);
  assert.equal((await app.send('load.chunk', { offset: 2, bytes: 'f/8=' })).ok, true);
  assert.equal((await app.send('load.commit')).ok, true);
  assert.equal(app.state.fileName, file.fileName);
  assert.equal(app.locks, 0);

  const revision = { documentEpoch: app.state.documentEpoch, changeSeq: app.state.changeSeq };
  assert.equal((await app.send('load.begin', { ...file, ...revision })).ok, true);
  assert.equal((await app.send('load.chunk', { offset: 0, bytes: 'AE1//w==' })).ok, true);
  app.changeDuringOpen();
  assert.equal((await app.send('load.commit')).ok, false);
  assert.equal(app.state.documentEpoch, revision.documentEpoch);
  assert.equal(app.locks, 0);
});

test('save acknowledgement and recovery discard require the exact document revision', async () => {
  const app = createHost();
  const revision = { documentEpoch: 3, changeSeq: 2 };
  const fileName = 'saved.hwpx';
  app.host.notifyReady();
  assert.equal(app.emitted[0]?.event, 'ready');
  const first = await app.send('export', { format: 'hwpx' });
  assert.deepEqual(first.result, { ...revision, byteLength: documentBytes.length, format: 'hwpx' });
  assert.deepEqual((await app.send('export.chunk', { offset: 0, count: 4 })).result, { bytes: 'AE1//w==' });
  app.state.changeSeq++;
  assert.equal((await app.send('saved', { fileName, ...revision })).ok, false);
  assert.equal(app.saved, 0);
  assert.equal(app.state.dirty, true);
  assert.equal((await app.send('release')).ok, true);
  assert.equal(app.locks, 0);
  assert.equal((await app.send('discard', revision)).ok, false);
  assert.equal(app.discarded, 0);

  const next = { ...revision, changeSeq: 3 };
  assert.equal((await app.send('export', { format: 'hwpx' })).ok, true);
  assert.equal((await app.send('saved', { fileName, ...next })).ok, true);
  assert.equal(app.state.fileName, fileName);
  assert.equal(app.state.dirty, false);
  assert.equal(app.saved, 1);
  assert.equal(app.locks, 0);
  assert.equal((await app.send('discard', next)).ok, true);
  assert.equal(app.discarded, 1);
  app.state.canNativeSave = false;
  assert.equal((await app.send('export', { format: 'hwp' })).ok, false);
  assert.equal(app.locks, 0);
});

test('Studio file commands wait for the matching shell result', async () => {
  const app = createHost();
  const result = app.host.request('saveAs', 'hwpx', '보고서.hwpx');
  assert.deepEqual(app.emitted[0], {
    version: 1, kind: 'command', id: 1, command: 'saveAs', format: 'hwpx', suggestedName: '보고서.hwpx',
  });
  app.receive({ version: 1, kind: 'commandResult', id: 2, ok: true, result: 'failed' });
  app.receive({ version: 1, kind: 'commandResult', id: 1, ok: true, result: 'saved' });
  assert.equal(await result, 'saved');
});
