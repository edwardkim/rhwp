import test from 'node:test';
import assert from 'node:assert/strict';

import { exportDocumentForFormat } from '../src/command/save-document-format.ts';

test('선택한 SaveFormat 하나가 대응하는 exporter만 호출한다', () => {
  const calls: string[] = [];
  const exporter = {
    exportHml: () => { calls.push('hml'); return new Uint8Array([1]); },
    exportHwp: () => { calls.push('hwp'); return new Uint8Array([2]); },
    exportHwpx: () => { calls.push('hwpx'); return new Uint8Array([3]); },
  };

  assert.deepEqual(exportDocumentForFormat(exporter, 'hml'), new Uint8Array([1]));
  assert.deepEqual(exportDocumentForFormat(exporter, 'hwp'), new Uint8Array([2]));
  assert.deepEqual(exportDocumentForFormat(exporter, 'hwpx'), new Uint8Array([3]));
  assert.deepEqual(calls, ['hml', 'hwp', 'hwpx']);
});
