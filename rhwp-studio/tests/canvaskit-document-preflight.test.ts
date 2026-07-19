import test from 'node:test';
import assert from 'node:assert/strict';

import {
  parseCanvasKitDocumentPreflight,
  withCanvasKitSurfaceBlockers,
} from '../src/core/canvaskit-document-preflight.ts';
import type { CanvasKitDocumentPreflight } from '../src/core/types.ts';

function validReport(): CanvasKitDocumentPreflight {
  return {
    schemaVersion: 1,
    mode: 'default',
    profile: 'screen',
    status: 'eligible',
    eligible: true,
    complete: true,
    pageCount: 2,
    scannedPages: 2,
    scannedWorkUnits: 12,
    limits: {
      maxPages: 128,
      maxWorkUnits: 50_000,
      maxBlockers: 32,
      maxRequiredFontFamilies: 256,
    },
    summary: {
      totalItems: 10,
      directItems: 10,
      directRequiredItems: 0,
      compatOverlayItems: 0,
      textFallbackItems: 0,
      unsupportedItems: 0,
      hiddenOverlayViolations: 0,
    },
    blockers: [],
    requiredFontFamilies: ['Noto Sans KR'],
    capabilityDigest: 'preflight-digest',
  };
}

test('CanvasKit document preflight parser accepts the complete bounded schema', () => {
  assert.deepEqual(
    parseCanvasKitDocumentPreflight(JSON.stringify(validReport())),
    validReport(),
  );
});

test('CanvasKit document preflight parser rejects malformed and partial reports', () => {
  assert.throws(
    () => parseCanvasKitDocumentPreflight('{'),
    /preflight parse 실패/,
  );
  assert.throws(
    () => parseCanvasKitDocumentPreflight(JSON.stringify({ ...validReport(), summary: {} })),
    /필수 필드가 없습니다/,
  );
  assert.throws(
    () => parseCanvasKitDocumentPreflight(JSON.stringify({ ...validReport(), scannedWorkUnits: -1 })),
    /필수 필드가 없습니다/,
  );
  assert.throws(
    () => parseCanvasKitDocumentPreflight(JSON.stringify({ ...validReport(), requiredFontFamilies: [42] })),
    /필수 필드가 없습니다/,
  );
});

test('surface blockers fail auto eligibility without mutating the WASM report', () => {
  const report = validReport();
  const blocked = withCanvasKitSurfaceBlockers(report, [
    'fontUnavailable:Missing Family',
    'fontUnavailable:Missing Family',
  ]);

  assert.equal(report.status, 'eligible');
  assert.equal(blocked.status, 'ineligible');
  assert.equal(blocked.eligible, false);
  assert.equal(blocked.summary.unsupportedItems, 1);
  assert.equal(blocked.blockers.at(-1)?.detail, 'fontUnavailable:Missing Family');
  assert.match(blocked.capabilityDigest, /^preflight-digest:surface-[0-9a-f]{8}$/);
});

test('surface blockers remain visible when the WASM blocker budget is full', () => {
  const report = {
    ...validReport(),
    blockers: Array.from({ length: 32 }, (_, pageIndex) => ({
      code: 'unsupported' as const,
      pageIndex,
      detail: `wasm:${pageIndex}`,
    })),
  };
  const blocked = withCanvasKitSurfaceBlockers(report, [
    'viewOption:showParagraphMarks',
    'viewOption:showControlCodes',
  ]);

  assert.equal(report.blockers.length, 32);
  assert.equal(blocked.blockers.length, 32);
  assert.deepEqual(
    blocked.blockers.slice(-2).map(blocker => blocker.detail),
    ['viewOption:showControlCodes', 'viewOption:showParagraphMarks'],
  );
});

test('surface blockers fail closed even when diagnostics cannot retain a blocker', () => {
  const report = validReport();
  report.limits.maxBlockers = 0;
  const blocked = withCanvasKitSurfaceBlockers(report, ['fontUnavailable:Missing Family']);

  assert.equal(blocked.status, 'ineligible');
  assert.equal(blocked.eligible, false);
  assert.deepEqual(blocked.blockers, []);
  assert.equal(blocked.summary.unsupportedItems, 1);
});

test('surface blocker details remain bounded while the summary counts every requirement', () => {
  const report = validReport();
  report.limits.maxBlockers = 2;
  const blocked = withCanvasKitSurfaceBlockers(report, [
    'viewOption:z',
    'viewOption:a',
    'viewOption:m',
  ]);

  assert.deepEqual(
    blocked.blockers.map(blocker => blocker.detail),
    ['viewOption:a', 'viewOption:m'],
  );
  assert.equal(blocked.summary.unsupportedItems, 3);
  assert.equal(blocked.summary.totalItems, report.summary.totalItems + 3);
});
