import test from 'node:test';
import assert from 'node:assert/strict';

import {
  exportDocumentForFormat,
  exportDocumentWithReportForFormat,
  exportPasswordProtectedDocumentForFormat,
  exportPasswordProtectedDocumentWithReportForFormat,
} from '../src/command/save-document-format.ts';
import type {
  ContentLossReport,
  DocumentExportArtifact,
} from '../src/core/export-content-loss.ts';

const cleanHwpReport: ContentLossReport = {
  schemaVersion: 1,
  outputFormat: 'hwp',
  count: 0,
  losses: [],
};

const cleanHwpxReport: ContentLossReport = {
  ...cleanHwpReport,
  outputFormat: 'hwpx',
};

function artifact(bytes: number, contentLoss: ContentLossReport): DocumentExportArtifact {
  return { bytes: new Uint8Array([bytes]), contentLoss };
}

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

test('명시 저장은 HWP/HWPX reported exporter를 고르고 HML만 보고서 없음으로 둔다', () => {
  const calls: string[] = [];
  const exporter = {
    exportHml: () => { calls.push('hml'); return new Uint8Array([1]); },
    exportHwp: () => { throw new Error('byte-only HWP를 호출하면 안 됨'); },
    exportHwpx: () => { throw new Error('byte-only HWPX를 호출하면 안 됨'); },
    exportHwpWithReport: () => { calls.push('hwp:reported'); return artifact(2, cleanHwpReport); },
    exportHwpxWithReport: () => { calls.push('hwpx:reported'); return artifact(3, cleanHwpxReport); },
  };

  assert.deepEqual(exportDocumentWithReportForFormat(exporter, 'hml'), {
    bytes: new Uint8Array([1]),
    contentLoss: null,
  });
  assert.deepEqual(exportDocumentWithReportForFormat(exporter, 'hwp'), artifact(2, cleanHwpReport));
  assert.deepEqual(exportDocumentWithReportForFormat(exporter, 'hwpx'), artifact(3, cleanHwpxReport));
  assert.deepEqual(calls, ['hml', 'hwp:reported', 'hwpx:reported']);
});

test('암호 명시 저장도 같은 reported artifact 경계를 사용한다', () => {
  const calls: Array<[string, string]> = [];
  const exporter = {
    exportHml: () => new Uint8Array(),
    exportHwp: () => new Uint8Array(),
    exportHwpx: () => new Uint8Array(),
    exportHwpWithPassword: () => { throw new Error('byte-only HWP를 호출하면 안 됨'); },
    exportHwpxWithPassword: () => { throw new Error('byte-only HWPX를 호출하면 안 됨'); },
    exportHwpWithReport: () => artifact(1, cleanHwpReport),
    exportHwpxWithReport: () => artifact(1, cleanHwpxReport),
    exportHwpWithPasswordAndReport: (password: string) => {
      calls.push(['hwp', password]);
      return artifact(4, cleanHwpReport);
    },
    exportHwpxWithPasswordAndReport: (password: string) => {
      calls.push(['hwpx', password]);
      return artifact(5, cleanHwpxReport);
    },
  };

  assert.deepEqual(
    exportPasswordProtectedDocumentWithReportForFormat(exporter, 'hwp', 'first'),
    artifact(4, cleanHwpReport),
  );
  assert.deepEqual(
    exportPasswordProtectedDocumentWithReportForFormat(exporter, 'hwpx', 'second'),
    artifact(5, cleanHwpxReport),
  );
  assert.deepEqual(calls, [['hwp', 'first'], ['hwpx', 'second']]);
});
