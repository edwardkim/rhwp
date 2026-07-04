import test from 'node:test';
import assert from 'node:assert/strict';

import {
  HWP_DOCUMENT_ACCEPT,
  canUseOpenFilePicker,
  isSupportedDocumentFileName,
  pickOpenFileHandle,
  readFileFromHandle,
  saveDocumentToFileSystem,
} from '../src/command/file-system-access.ts';

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

test('isSupportedDocumentFileName은 HWP/HWPX 확장자만 허용한다', () => {
  assert.equal(isSupportedDocumentFileName('sample.hwp'), true);
  assert.equal(isSupportedDocumentFileName('sample.HWPX'), true);
  assert.equal(isSupportedDocumentFileName(' sample.hwpx '), true);
  assert.equal(isSupportedDocumentFileName('sample.txt'), false);
  assert.equal(isSupportedDocumentFileName('sample.hwp.exe'), false);
  assert.equal(isSupportedDocumentFileName('sample'), false);
});

test('HWP_DOCUMENT_ACCEPT는 넓은 binary MIME을 등록하지 않는다', () => {
  assert.deepEqual(HWP_DOCUMENT_ACCEPT, {
    'application/x-hwp': ['.hwp'],
    'application/hwp+zip': ['.hwpx'],
  });
  assert.equal(Object.hasOwn(HWP_DOCUMENT_ACCEPT, 'application/octet-stream'), false);
  assert.equal(Object.hasOwn(HWP_DOCUMENT_ACCEPT, '*/*'), false);
});

test('canUseOpenFilePicker는 native open picker 지원 여부를 구분한다', () => {
  assert.equal(canUseOpenFilePicker({}), false);
  assert.equal(canUseOpenFilePicker({ showOpenFilePicker: async () => [] }), true);
});

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

test('pickOpenFileHandle는 native picker 취소 시 null을 반환한다', async () => {
  const result = await pickOpenFileHandle({
    showOpenFilePicker: async () => {
      throw new DOMException('cancelled', 'AbortError');
    },
  });

  assert.equal(result, null);
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
