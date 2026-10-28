import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
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

test('observed glyph miss requires an explicit failure instead of a guessed face', () => {
  const trace = validTrace();
  trace.records[0].paint.native = {
    ...backend('complete', 'observed'),
    failures: ['nativeGlyphMissingAllCandidates'],
  };
  trace.normalizedHash.value = normalizedTraceHash(trace);
  assert.deepEqual(validateTraceEnvelope(trace), []);

  trace.records[0].paint.native.failures = [];
  trace.normalizedHash.value = normalizedTraceHash(trace);
  assert.match(validateTraceEnvelope(trace).join('\n'), /observed without a resolved face/);
});

function traceWithDirectRegistryRule(sourceBoundaryId, sourceFace, sourceOwner) {
  const registry = readJson(path.join(ROOT, 'assets/font-rules/font_rule_registry_v2.json'));
  const rule = registry.rules.find(candidate => candidate.status === 'active'
    && candidate.sourceBoundaryId === sourceBoundaryId && candidate.sourceFace === sourceFace);
  assert.ok(rule, '현재 활성 레지스트리가 직접 규칙의 근거다');
  const link = buildCandidateLink({
    sourceBoundaryId, candidateKind: 'finite-mapping', sourceFace,
    targetOrPolicy: rule.targetFaceOrPolicy, conditions: rule.conditions, order: rule.order,
  }, sourceOwner);
  const trace = validTrace();
  trace.records[0].provenance.push({
    candidateId: link.candidateId, ruleId: rule.ruleId,
    evidenceAnchor: `assets/font-rules/font_rule_registry_v2.json#${rule.ruleId}`,
    sourceOwner, relationType: rule.relationType, evidenceStatus: 'unknown',
    knownLimitations: ['선택 규칙의 출처이며 실제 paint 글꼴이나 윤곽 동일성의 증명이 아니다.'],
    reason: null,
  });
  return trace;
}

function refreshTraceHashes(trace) {
  trace.layoutHash.value = portableLayoutHash(trace);
  trace.normalizedHash.value = normalizedTraceHash(trace);
}

test('활성 v2 직접 규칙의 앵커는 과거 W1 후보 링크와 구분해 검증한다', () => {
  for (const [boundary, face, owner] of [
    ['rust-style-resolution.document-substitute', 'HCR Batang', 'rust-style-resolution'],
    ['rust-metric.metric-alias', 'RHWP Public Batang', 'rust-metric'],
  ]) {
    const trace = traceWithDirectRegistryRule(boundary, face, owner);
    refreshTraceHashes(trace);
    assert.deepEqual(validateTraceEnvelope(trace), [], face);
  }
});

test('v2 앵커의 후보·규칙·소유자·관계가 바뀌거나 비활성 규칙이면 거절한다', () => {
  const original = traceWithDirectRegistryRule(
    'rust-style-resolution.document-substitute', 'HCR Batang', 'rust-style-resolution',
  );
  const registry = readJson(path.join(ROOT, 'assets/font-rules/font_rule_registry_v2.json'));
  const retired = registry.rules.find(rule => rule.status === 'retired');
  assert.ok(retired);
  for (const mutate of [
    entry => { entry.candidateId = 'candidate.00000000000000000000'; },
    entry => { entry.ruleId = 'rule.rust-style-resolution.00000000000000000000';
      entry.evidenceAnchor = `assets/font-rules/font_rule_registry_v2.json#${entry.ruleId}`; },
    entry => { entry.evidenceAnchor = `other.json#${entry.ruleId}`; },
    entry => { entry.evidenceAnchor = `other.json#${entry.candidateId}`; },
    entry => { entry.evidenceAnchor = `assets/font-rules/font_rule_registry_v2.json#${entry.candidateId}`; },
    entry => { entry.sourceOwner = 'rust-metric'; },
    entry => { entry.relationType = 'identity-alias'; },
    entry => { entry.ruleId = retired.ruleId;
      entry.evidenceAnchor = `assets/font-rules/font_rule_registry_v2.json#${retired.ruleId}`; },
  ]) {
    const trace = structuredClone(original);
    mutate(trace.records[0].provenance.at(-1));
    refreshTraceHashes(trace);
    assert.match(validateTraceEnvelope(trace).join('\n'), /provenance/);
  }
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
  const original = structuredClone(snapshot);
  // Historical W1 digests stay frozen. Independently read the current bytes;
  // legitimate source edits must not require updating a hardcoded drift list.
  const candidates = new Map(snapshot.candidates.map(candidate => [candidate.path, candidate]));
  const expectedDrift = [...candidates.keys()].sort().flatMap(sourcePath => {
    const bytes = fs.readFileSync(path.join(ROOT, sourcePath));
    const actualSha256 = createHash('sha256').update(bytes).digest('hex');
    const expectedSha256 = candidates.get(sourcePath).sourceSha256;
    return actualSha256 === expectedSha256 ? [] : [{ path: sourcePath, expectedSha256, actualSha256 }];
  });
  const currentDrift = detectLedgerSourceDrift(snapshot, ROOT);
  assert.deepEqual(currentDrift, expectedDrift);
  for (const entry of currentDrift) {
    assert.match(entry.expectedSha256, /^[0-9a-f]{64}$/);
    assert.match(entry.actualSha256, /^[0-9a-f]{64}$/);
    assert.equal(path.isAbsolute(entry.path), false);
  }

  const changed = structuredClone(snapshot);
  const changedPath = changed.candidates.find(
    candidate => !expectedDrift.some(entry => entry.path === candidate.path),
  ).path;
  for (const candidate of changed.candidates) {
    if (candidate.path === changedPath) candidate.sourceSha256 = '0'.repeat(64);
  }
  const drift = detectLedgerSourceDrift(changed, ROOT);
  assert.equal(drift.length, currentDrift.length + 1);
  assert.ok(drift.some(entry => entry.path === changedPath));
  assert.equal(path.isAbsolute(drift[0].path), false);
  assert.equal(JSON.stringify(drift).includes(ROOT), false);
  assert.deepEqual(snapshot, original, 'Historical W1 evidence must remain unchanged');
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
  trace.reasons.push({ code: 'serializationFailed', detail: '/home/tester/private/input.hwp' });
  trace.records[0].paint.canvas2d.failures.push('Error: failed\n    at /workspace/app.ts:10:4');
  const syntheticToken = ['gh', 'p_', 'abcdefghijklmnopqrstuvwxyz123456'].join('');
  trace.records[0].paint.canvaskit.source = `Bearer ${syntheticToken}`;
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
  privatePath.fixtures[0].path = '/home/tester/private-corpus/corpus_10k/private.hwp';
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
  const ruleIdPattern = new RegExp(schema.$defs.provenance.properties.ruleId.oneOf[0].pattern);
  for (const id of ['rule.rust-style-resolution.8f1a484b6e8dff748728',
    'rule.rust-metric.ygodic230-name', 'rule.rust-metric.issue-7445.public-batang-name']) {
    assert.equal(ruleIdPattern.test(id), true, id);
  }
  for (const id of ['rule.', 'rule.Owner.name', 'rule.rust-metric..name', 'other.rust-metric.name']) {
    assert.equal(ruleIdPattern.test(id), false, id);
  }
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
