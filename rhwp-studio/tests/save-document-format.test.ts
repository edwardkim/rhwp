import test from 'node:test';
import assert from 'node:assert/strict';

import {
  exportDocumentForFormat,
  exportPasswordProtectedDocumentForFormat,
} from '../src/command/save-document-format.ts';

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

test('암호 저장은 HWP/HWPX 전용 serializer만 호출한다', () => {
  const calls: Array<[string, string]> = [];
  const exporter = {
    exportHml: () => new Uint8Array([1]),
    exportHwp: () => new Uint8Array([2]),
    exportHwpx: () => new Uint8Array([3]),
    exportHwpWithPassword: (password: string) => {
      calls.push(['hwp', password]);
      return new Uint8Array([4]);
    },
    exportHwpxWithPassword: (password: string) => {
      calls.push(['hwpx', password]);
      return new Uint8Array([5]);
    },
  };

  assert.deepEqual(exportPasswordProtectedDocumentForFormat(exporter, 'hwp', 'first'), new Uint8Array([4]));
  assert.deepEqual(exportPasswordProtectedDocumentForFormat(exporter, 'hwpx', 'second'), new Uint8Array([5]));
  assert.deepEqual(calls, [['hwp', 'first'], ['hwpx', 'second']]);
});
