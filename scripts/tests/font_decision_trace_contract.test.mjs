import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import test from 'node:test';
import { fileURLToPath } from 'node:url';

import {
  DEFAULT_CHARACTER_LIMIT,
  MAX_CHARACTER_LIMIT,
  attachLedgerEvidence,
  buildCandidateLink,
  detectLedgerSourceDrift,
  findSensitiveTraceValues,
  normalizeTraceLimits,
  normalizedTraceHash,
  portableLayoutHash,
  validatePublicFixtures,
  validateTraceEnvelope,
} from '../font_decision_trace_contract.mjs';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..', '..');
const INVESTIGATION = path.join(ROOT, 'mydocs', 'tech', 'investigations', 'issue-4961');
const LEDGER = JSON.parse(fs.readFileSync(path.join(
  ROOT,
  'mydocs',
  'tech',
  'investigations',
  'issue-4939',
  'font_rule_ledger.json',
), 'utf8'));

function readJson(file) {
  return JSON.parse(fs.readFileSync(file, 'utf8'));
}

function backend(status = 'unsupported', certainty = 'unsupported') {
  return {
    status,
    certainty,
    requested: 'HCR Batang',
    candidates: ['HCR Batang', 'serif'],
    resolved: null,
    source: null,
    capabilities: [],
    failures: status === 'unsupported' ? ['backendNotBuilt'] : [],
  };
}

function validTrace() {
  const vector = readJson(path.join(INVESTIGATION, 'font_decision_identity_vectors.json')).vectors[1];
  const provenance = attachLedgerEvidence(vector.identity, vector.sourceOwner, LEDGER);
  const record = {
    recordId: 'page-0-run-0-char-0',
    source: {
      status: 'complete',
      sectionIndex: 0,
      paragraphIndex: 0,
      nestedPath: [],
      runIndex: 0,
      charOffset: 0,
      character: '한',
      codePoint: 0xD55C,
      charShapeId: 0,
    },
    document: {
      languageSlot: 0,
      inheritedLanguageSlot: null,
      face: '함초롬바탕',
      altType: 1,
      embedded: false,
      substFont: null,
    },
    layoutName: {
      requestedFace: '함초롬바탕',
      normalizedFace: 'HCR Batang',
      cssFamilyChain: ['HCR Batang'],
      steps: [{ kind: 'ttf', input: '함초롬바탕', output: 'HCR Batang', reason: null }],
    },
    layoutMetric: {
      requestedFace: 'HCR Batang',
      aliasResolvedFace: 'HCR Batang',
      matchKind: 'exact',
      metricEntry: 0,
      characterMatch: 'hit',
      widthSource: 'metricGlyph',
      baseAdvanceHwpunit: 1000,
      transforms: [],
      finalAdvanceHwpunit: 1000,
    },
    paint: {
      native: backend(),
      canvas2d: backend('notObserved', 'notObserved'),
      canvaskit: backend(),
    },
    provenance: [provenance],
    oracle: {
      status: 'notProvided',
      profileId: null,
      knownLimitations: ['W5 oracle profile is not available'],
    },
  };
  const trace = {
    schemaVersion: 1,
    status: 'complete',
    scope: {
      pageIndex: 0,
      requestedLimits: { maxCharacters: DEFAULT_CHARACTER_LIMIT },
      appliedLimits: { maxCharacters: DEFAULT_CHARACTER_LIMIT },
    },
    counts: { runsSeen: 1, charactersSeen: 1, recordsEmitted: 1, recordsOmitted: 0 },
    records: [record],
    backendSummary: {
      layout: { status: 'complete', reasons: [] },
      native: { status: 'unsupported', reasons: ['backendNotBuilt'] },
      canvas2d: { status: 'notObserved', reasons: ['cssSelectedGlyphNotObservable'] },
      canvaskit: { status: 'unsupported', reasons: ['backendNotBuilt'] },
    },
    reasons: [],
    layoutHash: { algorithm: 'sha256', value: null },
    normalizedHash: { algorithm: 'sha256', value: null },
  };
  trace.layoutHash.value = portableLayoutHash(trace);
  trace.normalizedHash.value = normalizedTraceHash(trace);
  return trace;
}

test('W1 candidate identity vectors produce the exact candidateId and ruleId', () => {
  const vectors = readJson(path.join(INVESTIGATION, 'font_decision_identity_vectors.json')).vectors;
  for (const vector of vectors) {
    const actual = buildCandidateLink(vector.identity, vector.sourceOwner);
    assert.equal(actual.candidateId, vector.candidateId, vector.id);
    assert.equal(actual.ruleId, vector.ruleId, vector.id);
  }
});

test('every golden rule joins to the W1 ledger with the exact evidence anchor', () => {
  const vectors = readJson(path.join(INVESTIGATION, 'font_decision_identity_vectors.json')).vectors;
  for (const vector of vectors) {
    const joined = attachLedgerEvidence(vector.identity, vector.sourceOwner, LEDGER);
    assert.equal(joined.ruleId, vector.ruleId, vector.id);
    assert.equal(joined.reason, null, vector.id);
    assert.match(joined.evidenceAnchor, new RegExp(`#${vector.candidateId}$`), vector.id);
  }
});

test('a missing ledger row is explicit and never receives a guessed ruleId', () => {
  const identity = {
    sourceBoundaryId: 'rust-metric.metric-alias',
    candidateKind: 'finite-mapping',
    sourceFace: '__missing_font__',
    targetOrPolicy: '__missing_metric__',
    conditions: {},
    order: null,
  };
  const joined = attachLedgerEvidence(identity, 'rust-metric', LEDGER);
  assert.equal(joined.ruleId, null);
  assert.equal(joined.reason, 'ledgerRuleMissing');
  assert.match(joined.candidateId, /^candidate\.[0-9a-f]{20}$/);
});

test('limits use 1024 by default and reject invalid or oversized values without clamping', () => {
  assert.deepEqual(normalizeTraceLimits(undefined), { maxCharacters: DEFAULT_CHARACTER_LIMIT });
  assert.deepEqual(normalizeTraceLimits({ maxCharacters: MAX_CHARACTER_LIMIT }), {
    maxCharacters: MAX_CHARACTER_LIMIT,
  });
  for (const value of [0, -1, 1.5, MAX_CHARACTER_LIMIT + 1, Number.NaN, '1024']) {
    assert.throws(() => normalizeTraceLimits({ maxCharacters: value }), /maxCharacters/);
  }
});

test('valid v1 envelope passes and missing fields, inconsistent counts and silent truncation fail', () => {
  const trace = validTrace();
  assert.deepEqual(validateTraceEnvelope(trace), []);

  const missing = structuredClone(trace);
  delete missing.status;
  assert.match(validateTraceEnvelope(missing).join('\n'), /status is required/);

  const badCount = structuredClone(trace);
  badCount.counts.recordsEmitted = 2;
  assert.match(validateTraceEnvelope(badCount).join('\n'), /recordsEmitted/);

  const silentTruncation = structuredClone(trace);
  silentTruncation.status = 'truncated';
  silentTruncation.counts.recordsOmitted = 3;
  assert.match(validateTraceEnvelope(silentTruncation).join('\n'), /characterLimitExceeded/);

  const invalidPage = structuredClone(trace);
  invalidPage.scope.pageIndex = -1;
  assert.match(validateTraceEnvelope(invalidPage).join('\n'), /pageIndex/);

  const clamped = structuredClone(trace);
  clamped.scope.requestedLimits.maxCharacters = MAX_CHARACTER_LIMIT;
  assert.match(validateTraceEnvelope(clamped).join('\n'), /silent clamp/);
});

test('W1 source digest drift is detected without exposing an absolute checkout path', () => {
  const snapshot = readJson(path.join(
    ROOT,
    'mydocs',
    'tech',
    'investigations',
    'issue-4939',
    'font_rule_candidates.json',
  ));
  assert.deepEqual(detectLedgerSourceDrift(snapshot, ROOT), []);

  const changed = structuredClone(snapshot);
  const changedPath = changed.candidates[0].path;
  for (const candidate of changed.candidates) {
    if (candidate.path === changedPath) candidate.sourceSha256 = '0'.repeat(64);
  }
  const drift = detectLedgerSourceDrift(changed, ROOT);
  assert.equal(drift.length, 1);
  assert.equal(drift[0].path, changedPath);
  assert.equal(path.isAbsolute(drift[0].path), false);
});

test('hashes ignore object key order and unordered diagnostics but preserve fallback chain order', () => {
  const first = validTrace();
  first.records[0].paint.canvas2d.capabilities = ['rawProbe', 'localEnumeration'];
  first.records[0].paint.canvas2d.failures = ['permissionDenied', 'partialEnumeration'];
  first.records[0].oracle.knownLimitations = ['b', 'a'];

  const reordered = structuredClone(first);
  reordered.records[0].paint.canvas2d.capabilities.reverse();
  reordered.records[0].paint.canvas2d.failures.reverse();
  reordered.records[0].oracle.knownLimitations.reverse();
  assert.equal(normalizedTraceHash(first), normalizedTraceHash(reordered));

  const changedPolicyOrder = structuredClone(first);
  changedPolicyOrder.records[0].paint.canvas2d.candidates.reverse();
  assert.notEqual(normalizedTraceHash(first), normalizedTraceHash(changedPolicyOrder));
});

test('portable layout hash ignores backend state while normalized hash preserves it', () => {
  const first = validTrace();
  const changedBackend = structuredClone(first);
  changedBackend.records[0].paint.canvas2d = {
    ...backend('complete', 'resolved'),
    resolved: 'HCR Batang',
    source: 'local',
    capabilities: ['localEnumeration'],
    failures: [],
  };
  assert.equal(portableLayoutHash(first), portableLayoutHash(changedBackend));
  assert.notEqual(normalizedTraceHash(first), normalizedTraceHash(changedBackend));
});

test('absolute host paths, user directories, tokens and error stacks are rejected', () => {
  const trace = validTrace();
  trace.reasons.push({ code: 'serializationFailed', detail: '/home/edward/private/input.hwp' });
  trace.records[0].paint.canvas2d.failures.push('Error: failed\n    at /workspace/app.ts:10:4');
  trace.records[0].paint.canvaskit.source = 'Bearer ghp_abcdefghijklmnopqrstuvwxyz123456';
  const findings = findSensitiveTraceValues(trace);
  assert.equal(findings.some(finding => finding.reason === 'absoluteHomePath'), true);
  assert.equal(findings.some(finding => finding.reason === 'errorStack'), true);
  assert.equal(findings.some(finding => finding.reason === 'accessToken'), true);
  assert.match(validateTraceEnvelope(trace).join('\n'), /sensitive trace value/);
});

test('public fixture manifest is tracked, byte-stable and excludes private corpus paths', () => {
  const manifest = readJson(path.join(INVESTIGATION, 'public_fixtures.json'));
  assert.deepEqual(validatePublicFixtures(manifest, ROOT), []);

  const privatePath = structuredClone(manifest);
  privatePath.fixtures[0].path = '/home/edward/mygithub/hwpsamples/corpus_10k/private.hwp';
  assert.match(validatePublicFixtures(privatePath, ROOT).join('\n'), /repository-relative/);

  const wrongDigest = structuredClone(manifest);
  wrongDigest.fixtures[0].sha256 = '0'.repeat(64);
  assert.match(validatePublicFixtures(wrongDigest, ROOT).join('\n'), /sha256 mismatch/);
});

test('schema fixes v1 enums and the 4096 hard record limit', () => {
  const schema = readJson(path.join(INVESTIGATION, 'font_decision_trace.schema.json'));
  assert.equal(schema.properties.schemaVersion.const, 1);
  assert.equal(schema.properties.records.maxItems, MAX_CHARACTER_LIMIT);
  assert.equal(schema.$defs.limits.properties.maxCharacters.default, DEFAULT_CHARACTER_LIMIT);
  assert.equal(schema.$defs.limits.properties.maxCharacters.maximum, MAX_CHARACTER_LIMIT);
  assert.deepEqual(schema.$defs.traceStatus.enum, [
    'complete',
    'truncated',
    'unsupported',
    'failed',
  ]);
  assert.deepEqual(schema.$defs.certainty.enum, [
    'observed',
    'resolved',
    'planned',
    'notObserved',
    'unsupported',
  ]);
});
