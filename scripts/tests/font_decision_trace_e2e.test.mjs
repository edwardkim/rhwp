import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import test from 'node:test';
import { fileURLToPath, pathToFileURL } from 'node:url';

import {
  validateTraceEnvelope,
} from '../font_decision_trace_contract.mjs';
import { enrichFontDecisionTrace } from '../../rhwp-studio/src/core/font-decision-trace.ts';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..', '..');
const INVESTIGATION = path.join(ROOT, 'mydocs', 'tech', 'investigations', 'issue-4961');
const MANIFEST = JSON.parse(fs.readFileSync(path.join(
  INVESTIGATION,
  'font_decision_trace_e2e.json',
), 'utf8'));
const wasmModule = await import(pathToFileURL(path.join(ROOT, 'pkg', 'rhwp.js')).href);
await wasmModule.default({
  module_or_path: fs.readFileSync(path.join(ROOT, 'pkg', 'rhwp_bg.wasm')),
});

function readTrace(document, maxCharacters = MANIFEST.options.maxCharacters) {
  const bytes = new Uint8Array(fs.readFileSync(path.join(ROOT, document.path)));
  const hwp = new wasmModule.HwpDocument(bytes);
  try {
    return JSON.parse(hwp.getFontDecisionTrace(
      document.page,
      JSON.stringify({ maxCharacters }),
    ));
  } finally {
    hwp.free();
  }
}

function profileRecord(trace, profile) {
  const records = trace.records.filter(candidate => Object.entries(profile.sourceAddress)
    .every(([key, value]) => JSON.stringify(candidate.source[key]) === JSON.stringify(value)));
  assert.equal(records.length, 1, `${profile.id}: the source character has one trace owner`);
  return records[0];
}

function assertProfile(trace, profile) {
  const record = profileRecord(trace, profile);
  const expected = profile.expected;
  assert.equal(record.source.status, expected.sourceStatus);
  assert.equal(record.source.character, expected.character);
  assert.equal(record.document.face, expected.documentFace);
  assert.equal(record.document.substFont, expected.substFont);
  assert.equal(record.layoutName.normalizedFace, expected.normalizedFace);
  if (expected.layoutStepKind === null) {
    assert.deepEqual(record.layoutName.steps, []);
  } else {
    assert.ok(record.layoutName.steps.some(step => step.kind === expected.layoutStepKind));
  }
  assert.equal(record.layoutMetric.matchKind, expected.metricMatchKind);
  assert.equal(record.layoutMetric.characterMatch, expected.metricCharacterMatch);
  assert.equal(record.layoutMetric.widthSource, expected.widthSource);
}

const completeLocalState = {
  supported: true,
  method: 'local-font-access',
  loaded: true,
  stored: true,
  source: 'local-font-access',
  complete: true,
  storage: 'local-storage',
  count: 3,
  checkedFamilies: ['돋움', '바탕', 'Palatino Linotype'],
  probedFamilies: [],
  unresolvedFamilies: [],
  detectedAt: null,
  lastError: null,
};

test('공개 HWP/HWPX fixture의 exact, missing, substFont 계보가 WASM에서 끝까지 이어진다', () => {
  const traces = new Map();
  for (const document of MANIFEST.documents) {
    const first = readTrace(document);
    const second = readTrace(document);
    assert.deepEqual(first, second, `${document.id}: repeat trace`);
    assert.deepEqual(validateTraceEnvelope(first), [], document.id);
    assert.equal(first.status, document.expectedStatus, document.id);
    const deferredPins = new Set(document.renderPinsDeferredToIssue7445 ?? []);
    if (!deferredPins.has('expectedCounts')) {
      for (const [key, value] of Object.entries(document.expectedCounts)) {
        assert.equal(first.counts[key], value, `${document.id}: ${key}`);
      }
      if (document.validateRunOwnership) {
        const core = new wasmModule.HwpDocument(new Uint8Array(fs.readFileSync(path.join(ROOT, document.path))));
        let layout;
        try { layout = JSON.parse(core.getPageTextLayout(document.page)); }
        finally { core.free(); }
        assert.equal(first.counts.runsSeen, layout.runs.length);
        assert.equal(first.counts.charactersSeen, layout.runs.reduce((sum, run) => sum + Array.from(run.text).length, 0));
        assert.equal(new Set(first.records.map(record => record.recordId)).size, first.records.length);
        for (const record of first.records) {
          const owner = layout.runs[record.source.runIndex];
          const local = Number(record.recordId.split(':char:').at(-1));
          assert.equal(record.source.character, Array.from(owner.text)[local]);
          assert.equal(record.source.charShapeId, owner.charShapeId ?? null);
          if (owner.charStart == null) {
            assert.equal(record.source.charOffset, null);
            assert.equal(record.source.status, 'unavailable');
          } else {
            assert.equal(record.source.charOffset, owner.charStart + local);
            assert.equal(record.source.sectionIndex, owner.secIdx);
            assert.equal(record.source.paragraphIndex,
              owner.cellPath == null ? owner.paraIdx : owner.parentParaIdx);
          }
        }
      }
    }
    if (!deferredPins.has('expectedLayoutHash')) {
      assert.equal(first.layoutHash.value, document.expectedLayoutHash, document.id);
    }
    assert.equal(first.backendSummary.native.status, 'unsupported', document.id);
    assert.deepEqual(first.backendSummary.native.reasons, ['nativeSkiaFeatureUnavailable']);
    assert.ok(first.records.every(record => (
      record.paint.native.failures.includes('nativeSkiaFeatureUnavailable')
      && record.paint.canvas2d.failures.includes('studioSnapshotRequired')
      && record.paint.canvaskit.failures.includes('studioSnapshotRequired')
    )));
    traces.set(document.id, first);
  }

  for (const profile of MANIFEST.profiles) {
    const trace = traces.get(profile.documentId);
    assertProfile(trace, profile);
    const enriched = enrichFontDecisionTrace(JSON.stringify(trace), {
      localState: completeLocalState,
      detectedOsFonts: new Set(['돋움', '바탕', 'Palatino Linotype']),
    });
    const record = profileRecord(enriched, profile);
    assert.equal(record.paint.canvas2d.status, 'complete', profile.id);
    assert.equal(record.paint.canvas2d.certainty, 'notObserved', profile.id);
    assert.ok(record.paint.canvas2d.failures.includes('cssActualGlyphFaceUnobservable'));
    assert.equal(record.paint.canvaskit.status, 'notObserved', profile.id);
    assert.equal(record.paint.canvaskit.certainty, 'planned', profile.id);
    assert.ok(record.provenance.length > 0, `${profile.id}: provenance`);
    assert.notEqual(record.layoutMetric.finalAdvanceHwpunit, null, profile.id);
  }

  const feature = MANIFEST.comparisons.substFeatureDetection;
  const without = traces.get(feature.withoutSubstFont);
  const withSubstitution = traces.get(feature.withSubstFont);
  const hwpCounterpart = traces.get(feature.withHwpSubstFont);
  assert.ok(without.records.every(record => record.document.substFont === null));
  assert.ok(withSubstitution.records.some(record => record.document.substFont !== null));
  assert.ok(hwpCounterpart.records.some(record => record.document.substFont === feature.expectedHwpSubstFont));
  assert.notEqual(hwpCounterpart.layoutHash.value, withSubstitution.layoutHash.value);
  for (const declaration of MANIFEST.comparisons.documentSubstituteDeclarations) {
    const trace = traces.get(declaration.documentId);
    const records = trace.records.filter(record => (
      record.document.face === declaration.face
      && declaration.languageSlots.includes(record.document.languageSlot)
    ));
    assert.ok(records.length > 0, `${declaration.documentId}: 선언 대상 글꼴`);
    assert.ok(records.every(record => record.document.substFont === declaration.substFont));
  }

});

test('WASM 상한과 backend 미지원은 silent clamp나 빈 성공 없이 fail-closed한다', () => {
  const document = MANIFEST.documents.find(candidate => candidate.id === 'exact-face');
  const trace = readTrace(document, 1);
  assert.equal(trace.status, 'truncated');
  assert.equal(trace.counts.recordsEmitted, 1);
  assert.ok(trace.counts.recordsOmitted > 0);
  assert.ok(trace.reasons.some(reason => reason.code === 'characterLimitExceeded'));
  assert.equal(trace.backendSummary.native.status, 'unsupported');
  assert.equal(trace.backendSummary.canvas2d.status, 'unsupported');
  assert.equal(trace.backendSummary.canvaskit.status, 'unsupported');

  const bytes = new Uint8Array(fs.readFileSync(path.join(ROOT, document.path)));
  const hwp = new wasmModule.HwpDocument(bytes);
  try {
    assert.throws(
      () => hwp.getFontDecisionTrace(document.page, JSON.stringify({ maxCharacters: 4097 })),
      /maxCharacters/,
    );
  } finally {
    hwp.free();
  }
});
