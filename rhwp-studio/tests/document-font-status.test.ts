import test from 'node:test';
import assert from 'node:assert/strict';

import { analyzeDocumentFonts } from '../src/core/document-font-status.ts';

test('저장된 로컬 snapshot이 없으면 미확인 글꼴에 로컬 확인 필요 상태를 부여한다', () => {
  const report = analyzeDocumentFonts(['미설치원본', '함초롬바탕'], {
    localFonts: [],
    localSupported: true,
    localSnapshotLoaded: true,
    localSnapshotStored: false,
  });

  assert.equal(report.shouldPromptLocalAccess, true);
  assert.deepEqual(report.summary, {
    available: 1,
    needsLocalCheck: 1,
    webSubstitute: 0,
    missing: 0,
  });
  assert.deepEqual(report.fonts.map(f => [f.fontName, f.status]), [
    ['미설치원본', 'needs-local-check'],
    ['함초롬바탕', 'available'],
  ]);
});

test('저장된 로컬 snapshot에 있는 원본 글꼴은 사용 가능 상태가 된다', () => {
  const report = analyzeDocumentFonts(['문서원본', '없는글꼴'], {
    localFonts: ['문서원본'],
    localSupported: true,
    localSnapshotLoaded: true,
    localSnapshotStored: true,
  });

  assert.equal(report.shouldPromptLocalAccess, false);
  assert.deepEqual(report.fonts.map(f => [f.fontName, f.status, f.source]), [
    ['문서원본', 'available', 'local'],
    ['없는글꼴', 'missing', 'unknown'],
  ]);
});

test('문서 후보 probe snapshot은 아직 확인하지 않은 새 글꼴에 다시 prompt를 띄운다', () => {
  const report = analyzeDocumentFonts(['문서원본', '확인된미설치', '새글꼴'], {
    localFonts: ['문서원본'],
    localSupported: true,
    localSnapshotLoaded: true,
    localSnapshotStored: true,
    localSnapshotComplete: false,
    localSnapshotSource: 'font-presence-probe',
    localCheckedFonts: ['문서원본', '확인된미설치'],
    detectionMethod: 'font-presence-probe',
  });

  assert.equal(report.shouldPromptLocalAccess, true);
  assert.deepEqual(report.fonts.map(f => [f.fontName, f.status, f.source]), [
    ['문서원본', 'available', 'local'],
    ['새글꼴', 'needs-local-check', 'unknown'],
    ['확인된미설치', 'missing', 'unknown'],
  ]);
});

test('로컬 감지 미지원 환경에서는 prompt 없이 웹 대체와 누락을 구분한다', () => {
  const report = analyzeDocumentFonts(['휴먼명조', '없는글꼴'], {
    localFonts: [],
    localSupported: false,
    localSnapshotLoaded: true,
    localSnapshotStored: false,
  });

  assert.equal(report.shouldPromptLocalAccess, false);
  assert.deepEqual(report.fonts.map(f => [f.fontName, f.status, f.substituteFont]), [
    ['없는글꼴', 'missing', null],
    ['휴먼명조', 'web-substitute', 'HY신명조'],
  ]);
});
