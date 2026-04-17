import test from 'node:test';
import assert from 'node:assert/strict';

test('save flow는 기존 handle 저장이 실패하면 다운로드 폴백 대신 오류 상태를 반환한다', async () => {
  const { performDocumentSave } = await import('../src/command/file-save-flow.js');

  const events: Array<{ type: string; message: string }> = [];
  let downloadTriggered = false;

  const result = await performDocumentSave({
    saveName: 'opened.hwp',
    bytes: new Uint8Array([1, 2, 3]),
    currentHandle: {
      name: 'opened.hwp',
      async getFile() {
        return new File(['broken'], 'opened.hwp', { type: 'application/x-hwp' });
      },
      async createWritable() {
        throw new Error('disk full');
      },
    },
    isNewDocument: false,
    windowLike: {},
    onStatus(message) {
      events.push({ type: 'status', message });
    },
    onAlert(message) {
      events.push({ type: 'alert', message });
    },
    onDownload() {
      downloadTriggered = true;
    },
  });

  assert.equal(result.ok, false);
  assert.equal(result.reason, 'existing-save-failed');
  assert.equal(downloadTriggered, false);
  assert.deepEqual(events.map((event) => event.type), ['status', 'alert']);
  assert.match(events[0].message, /저장 실패/);
  assert.match(events[1].message, /파일 저장에 실패했습니다/);
});
