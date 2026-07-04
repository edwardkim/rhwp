import test from 'node:test';
import assert from 'node:assert/strict';

import {
  describeDraft,
  formatDraftSavedAt,
  formatDraftSize,
  recoveryFileName,
} from '../src/recovery/recovery-format.ts';

test('recoveryFileName은 원본을 덮어쓰지 않는 복구본 이름을 만든다', () => {
  assert.equal(recoveryFileName('sample.hwp'), 'sample 복구본.hwp');
  assert.equal(recoveryFileName('sample.hwpx'), 'sample 복구본.hwp');
  assert.equal(recoveryFileName('sample.hwpx', 'hwpx'), 'sample 복구본.hwp');
  assert.equal(recoveryFileName('새 문서.hwp'), '새 문서 복구본.hwp');
  assert.equal(recoveryFileName('memo'), 'memo 복구본.hwp');
  assert.equal(recoveryFileName(''), '문서 복구본.hwp');
});

test('formatDraftSize는 복구 후보 크기를 읽기 좋은 단위로 표시한다', () => {
  assert.equal(formatDraftSize(512), '512 B');
  assert.equal(formatDraftSize(1536), '1.5 KB');
  assert.equal(formatDraftSize(2 * 1024 * 1024), '2.0 MB');
  assert.equal(formatDraftSize(Number.NaN), '크기 알 수 없음');
});

test('describeDraft는 저장 시각, 크기, 출처 포맷을 포함한다', () => {
  const savedAt = new Date('2026-06-21T00:00:00+09:00').getTime();
  const text = describeDraft({
    id: 'd1',
    fileName: '문서.hwp',
    sourceFormat: 'hwp',
    savedAt,
    byteLength: 2048,
    data: new Uint8Array([1]),
  });

  assert.match(text, /HWP/);
  assert.match(text, /2\.0 KB/);
  assert.notEqual(formatDraftSavedAt(savedAt), '저장 시각 알 수 없음');
});

test('describeDraft는 HWPX 출처 draft가 HWP 복구본으로 열림을 표시한다', () => {
  const text = describeDraft({
    id: 'd2',
    fileName: '문서.hwpx',
    sourceFormat: 'hwpx',
    savedAt: 1,
    byteLength: 1024,
    data: new Uint8Array([1]),
  });

  assert.match(text, /HWPX/);
  assert.match(text, /HWP 복구본/);
});
