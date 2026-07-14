import test from 'node:test';
import assert from 'node:assert/strict';

import {
  handlePwaLaunchFiles,
  installPwaFileHandling,
  type FileHandlingLaunchParamsLike,
  type LaunchQueueLike,
  type OpenDocumentBytesPayload,
  type PwaFileHandlingCallbacks,
} from '../src/command/pwa-file-handling.ts';

function createHandle(name: string, fileContent = 'fixture') {
  return {
    kind: 'file' as const,
    name,
    async getFile() {
      return new File([fileContent], name, { type: 'application/x-hwp' });
    },
    async createWritable() {
      throw new Error('write should not be called while opening PWA launch files');
    },
  };
}

function createCallbacks() {
  const opened: OpenDocumentBytesPayload[] = [];
  const unsupported: string[] = [];
  const errors: unknown[] = [];
  const multiple: number[] = [];

  const callbacks: PwaFileHandlingCallbacks = {
    openDocumentBytes(payload) {
      opened.push(payload);
    },
    notifyUnsupportedFile(fileName) {
      unsupported.push(fileName);
    },
    notifyError(error) {
      errors.push(error);
    },
    notifyMultipleFiles(count) {
      multiple.push(count);
    },
  };

  return { callbacks, opened, unsupported, errors, multiple };
}

test('installPwaFileHandling은 launchQueue가 없으면 false를 반환한다', () => {
  const { callbacks } = createCallbacks();

  const installed = installPwaFileHandling({}, callbacks);

  assert.equal(installed, false);
});

test('installPwaFileHandling은 launchQueue consumer를 등록한다', () => {
  const { callbacks } = createCallbacks();
  let consumer: ((params: FileHandlingLaunchParamsLike) => void) | null = null;
  const launchQueue: LaunchQueueLike = {
    setConsumer(nextConsumer) {
      consumer = nextConsumer;
    },
  };

  const installed = installPwaFileHandling({ launchQueue }, callbacks);

  assert.equal(installed, true);
  assert.equal(typeof consumer, 'function');
});

test('handlePwaLaunchFiles는 빈 launch를 무시한다', async () => {
  const { callbacks, opened, unsupported, errors } = createCallbacks();

  await handlePwaLaunchFiles({}, callbacks);
  await handlePwaLaunchFiles({ files: [] }, callbacks);

  assert.equal(opened.length, 0);
  assert.equal(unsupported.length, 0);
  assert.equal(errors.length, 0);
});

test('handlePwaLaunchFiles는 미지원 확장자를 로드하지 않는다', async () => {
  const { callbacks, opened, unsupported, errors } = createCallbacks();
  const handle = createHandle('memo.txt');

  await handlePwaLaunchFiles({ files: [handle] }, callbacks);

  assert.equal(opened.length, 0);
  assert.deepEqual(unsupported, ['memo.txt']);
  assert.equal(errors.length, 0);
});

test('handlePwaLaunchFiles는 HWP 파일 handle을 open-document-bytes payload로 만든다', async () => {
  const { callbacks, opened, unsupported, errors } = createCallbacks();
  const handle = createHandle('opened.hwp', 'abc');

  await handlePwaLaunchFiles({ files: [handle] }, callbacks);

  assert.equal(unsupported.length, 0);
  assert.equal(errors.length, 0);
  assert.equal(opened.length, 1);
  assert.equal(opened[0].fileName, 'opened.hwp');
  assert.equal(opened[0].fileHandle, handle);
  assert.equal(opened[0].skipUnsavedGuard, false);
  assert.deepEqual(Array.from(opened[0].bytes), [97, 98, 99]);
});

test('handlePwaLaunchFiles는 HWPX 파일도 허용하고 다중 파일은 첫 파일만 연다', async () => {
  const { callbacks, opened, unsupported, multiple } = createCallbacks();
  const first = createHandle('first.hwpx', 'one');
  const second = createHandle('second.hwp', 'two');

  await handlePwaLaunchFiles({ files: [first, second] }, callbacks);

  assert.deepEqual(multiple, [2]);
  assert.equal(unsupported.length, 0);
  assert.equal(opened.length, 1);
  assert.equal(opened[0].fileName, 'first.hwpx');
  assert.equal(opened[0].fileHandle, first);
});

test('handlePwaLaunchFiles는 HML 파일도 연다', async () => {
  const { callbacks, opened, unsupported, errors } = createCallbacks();
  const handle = createHandle('opened.hml', '<?xml version="1.0"?><HWPML />');

  await handlePwaLaunchFiles({ files: [handle] }, callbacks);

  assert.equal(unsupported.length, 0);
  assert.equal(errors.length, 0);
  assert.equal(opened.length, 1);
  assert.equal(opened[0].fileName, 'opened.hml');
  assert.equal(opened[0].fileHandle, handle);
});

test('handlePwaLaunchFiles는 getFile 실패를 notifyError로 전달한다', async () => {
  const { callbacks, opened, unsupported, errors } = createCallbacks();
  const boom = new Error('permission denied');
  const handle = {
    kind: 'file' as const,
    name: 'opened.hwp',
    async getFile() {
      throw boom;
    },
    async createWritable() {
      throw new Error('write should not be called');
    },
  };

  await handlePwaLaunchFiles({ files: [handle] }, callbacks);

  assert.equal(opened.length, 0);
  assert.equal(unsupported.length, 0);
  assert.deepEqual(errors, [boom]);
});
