import test from 'node:test';
import assert from 'node:assert/strict';

import { openDocumentViaPicker } from '../src/command/file-open-picker.ts';

function createFileInput() {
  return {
    dataset: {} as Record<string, string | undefined>,
    clickCount: 0,
    click() {
      this.clickCount += 1;
    },
  };
}

test('교차 출처 SecurityError가 나면 열기 피커는 숨김 파일 입력으로 폴백한다', async () => {
  const input = createFileInput();
  let warnings = 0;
  let alerts = 0;

  await openDocumentViaPicker({
    canReplace: async () => true,
    windowLike: {
      showOpenFilePicker: async () => {
        throw new DOMException('blocked by cross-origin policy', 'SecurityError');
      },
    },
    findFileInput: () => input as unknown as HTMLInputElement,
    emitOpenDocument: () => assert.fail('파일 handle 없이 문서를 열면 안 된다'),
    warn: () => { warnings += 1; },
    alert: () => { alerts += 1; },
  });

  assert.equal(input.clickCount, 1);
  assert.equal(input.dataset.skipUnsavedGuard, 'true');
  assert.equal(warnings, 1);
  assert.equal(alerts, 0);
});

test('사용자가 native 열기 피커를 취소하면 폴백을 다시 열지 않는다', async () => {
  const input = createFileInput();

  await openDocumentViaPicker({
    canReplace: async () => true,
    windowLike: {
      showOpenFilePicker: async () => {
        throw new DOMException('cancelled', 'AbortError');
      },
    },
    findFileInput: () => input as unknown as HTMLInputElement,
    emitOpenDocument: () => assert.fail('취소한 picker가 문서를 열면 안 된다'),
    warn: () => assert.fail('사용자 취소는 경고가 아니어야 한다'),
    alert: () => assert.fail('사용자 취소는 오류 안내가 아니어야 한다'),
  });

  assert.equal(input.clickCount, 0);
  assert.equal(input.dataset.skipUnsavedGuard, undefined);
});
