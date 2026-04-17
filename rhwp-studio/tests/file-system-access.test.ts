import test from 'node:test';
import assert from 'node:assert/strict';

import {
  createHttpFileHandle,
  pickOpenFileHandle,
  readFileFromHandle,
  saveDocumentToFileSystem,
  type FileSystemFileHandleLike,
} from '../src/command/file-system-access.ts';
import * as fileSystemAccess from '../src/command/file-system-access.ts';

type FakeWriteCall = Blob;

interface FakeWritable {
  writes: FakeWriteCall[];
  closed: boolean;
  write(data: Blob): Promise<void>;
  close(): Promise<void>;
}

function createWritable(): FakeWritable {
  return {
    writes: [],
    closed: false,
    async write(data: Blob) {
      this.writes.push(data);
    },
    async close() {
      this.closed = true;
    },
  };
}

function createHandle(name: string, fileContent = 'fixture') {
  const writable = createWritable();
  return {
    kind: 'file' as const,
    name,
    writable,
    async getFile() {
      return new File([fileContent], name, { type: 'application/x-hwp' });
    },
    async createWritable() {
      return writable;
    },
  };
}

test('pickOpenFileHandle는 showOpenFilePicker가 있으면 첫 handle을 반환한다', async () => {
  const handle = createHandle('opened.hwp');
  let receivedOptions: Record<string, unknown> | undefined;

  const result = await pickOpenFileHandle({
    showOpenFilePicker: async (options) => {
      receivedOptions = options as Record<string, unknown>;
      return [handle];
    },
  });

  assert.equal(result, handle);
  assert.ok(receivedOptions);
});

test('readFileFromHandle은 handle 파일 내용을 Uint8Array로 읽는다', async () => {
  const handle = createHandle('opened.hwp', 'abc');

  const result = await readFileFromHandle(handle);

  assert.equal(result.name, 'opened.hwp');
  assert.deepEqual(Array.from(result.bytes), [97, 98, 99]);
});

test('saveDocumentToFileSystem은 current handle이 있으면 picker 없이 같은 파일에 저장한다', async () => {
  const currentHandle = createHandle('opened.hwp');
  let pickerCalled = false;
  const blob = new Blob(['saved'], { type: 'application/x-hwp' });

  const result = await saveDocumentToFileSystem({
    blob,
    suggestedName: 'opened.hwp',
    currentHandle,
    windowLike: {
      showSaveFilePicker: async () => {
        pickerCalled = true;
        return createHandle('picker.hwp');
      },
    },
  });

  assert.equal(result.method, 'current-handle');
  assert.equal(result.handle, currentHandle);
  assert.equal(result.fileName, 'opened.hwp');
  assert.equal(pickerCalled, false);
  assert.equal(currentHandle.writable.writes.length, 1);
  assert.equal(currentHandle.writable.closed, true);
});

test('saveDocumentToFileSystem은 current handle이 없으면 save picker를 사용한다', async () => {
  const pickerHandle = createHandle('picked.hwp');
  const blob = new Blob(['saved'], { type: 'application/x-hwp' });

  const result = await saveDocumentToFileSystem({
    blob,
    suggestedName: 'new-doc.hwp',
    currentHandle: null,
    windowLike: {
      showSaveFilePicker: async (options) => {
        assert.equal(options?.suggestedName, 'new-doc.hwp');
        return pickerHandle;
      },
    },
  });

  assert.equal(result.method, 'save-picker');
  assert.equal(result.handle, pickerHandle);
  assert.equal(result.fileName, 'picked.hwp');
  assert.equal(pickerHandle.writable.writes.length, 1);
  assert.equal(pickerHandle.writable.closed, true);
});

test('setupPwaFileLaunch는 launchQueue로 받은 파일을 읽어 기존 로더 계약으로 전달한다', async () => {
  const setupPwaFileLaunch = (fileSystemAccess as Record<string, unknown>).setupPwaFileLaunch as
    | ((windowLike: unknown, onLaunch: (payload: {
      bytes: Uint8Array;
      fileName: string;
      fileHandle: FileSystemFileHandleLike;
    }) => Promise<void> | void, onError?: (error: unknown) => void) => boolean)
    | undefined;

  assert.equal(typeof setupPwaFileLaunch, 'function');

  const handle = createHandle('launch-opened.hwp', 'xyz');
  let consumer: ((launchParams: { files?: FileSystemFileHandleLike[] }) => Promise<void> | void) | null = null;
  let receivedPayload:
    | { bytes: Uint8Array; fileName: string; fileHandle: FileSystemFileHandleLike }
    | null = null;

  const registered = setupPwaFileLaunch!(
    {
      launchQueue: {
        setConsumer(callback: typeof consumer) {
          consumer = callback;
        },
      },
    },
    async (payload) => {
      receivedPayload = payload;
    },
  );

  assert.equal(registered, true);
  assert.ok(consumer);

  await consumer!({ files: [handle] });

  assert.ok(receivedPayload);
  assert.equal(receivedPayload.fileName, 'launch-opened.hwp');
  assert.equal(receivedPayload.fileHandle, handle);
  assert.deepEqual(Array.from(receivedPayload.bytes), [120, 121, 122]);
});

test('setupPwaFileLaunch는 전달된 파일이 없으면 로더를 호출하지 않는다', async () => {
  const setupPwaFileLaunch = (fileSystemAccess as Record<string, unknown>).setupPwaFileLaunch as
    | ((windowLike: unknown, onLaunch: (payload: unknown) => Promise<void> | void) => boolean)
    | undefined;

  assert.equal(typeof setupPwaFileLaunch, 'function');

  let consumer: ((launchParams: { files?: FileSystemFileHandleLike[] }) => Promise<void> | void) | null = null;
  let called = false;

  setupPwaFileLaunch!(
    {
      launchQueue: {
        setConsumer(callback: typeof consumer) {
          consumer = callback;
        },
      },
    },
    async () => {
      called = true;
    },
  );

  assert.ok(consumer);
  await consumer!({ files: [] });
  assert.equal(called, false);
});

test('createHttpFileHandle는 Ctrl+S 저장을 local save endpoint로 전달한다', async () => {
  const originalFetch = globalThis.fetch;
  const calls: Array<{ input: RequestInfo | URL; init?: RequestInit }> = [];

  globalThis.fetch = async (input, init) => {
    calls.push({ input, init });
    return new Response(JSON.stringify({ ok: true }), {
      status: 200,
      headers: { 'content-type': 'application/json' },
    });
  };

  try {
    const handle = createHttpFileHandle({
      fileName: 'opened.hwp',
      fileUrl: 'http://127.0.0.1:7701/__opened/token.hwp',
      saveUrl: 'http://127.0.0.1:7701/__rhwp_save/token',
    });

    const result = await saveDocumentToFileSystem({
      blob: new Blob(['saved'], { type: 'application/x-hwp' }),
      suggestedName: 'opened.hwp',
      currentHandle: handle,
      windowLike: {},
    });

    assert.equal(result.method, 'current-handle');
    assert.equal(result.fileName, 'opened.hwp');
    assert.equal(calls.length, 1);
    assert.equal(String(calls[0].input), 'http://127.0.0.1:7701/__rhwp_save/token');
    assert.equal(calls[0].init?.method, 'PUT');
  } finally {
    globalThis.fetch = originalFetch;
  }
});
