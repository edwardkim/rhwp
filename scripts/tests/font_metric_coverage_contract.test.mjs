import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import test from 'node:test';
import { fileURLToPath } from 'node:url';

import {
  auditW1SemanticDrift,
  canonicalCoverageHash,
  classifyCoverageDecision,
  findSensitiveAggregateValues,
  pocV2Projection,
  reconcileCoverageAggregate,
  validateCoverageContract,
  validatePocFormatAdditivity,
  validatePocV2Baseline,
  validateW1LedgerBaseline,
} from '../font_metric_coverage_contract.mjs';
import { verifyHistoricalSourceCandidates } from '../font_rule_ledger.mjs';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..', '..');
const INVESTIGATION = path.join(
  ROOT,
  'mydocs',
  'tech',
  'investigations',
  'issue-4962',
);
const W1 = path.join(ROOT, 'mydocs', 'tech', 'investigations', 'issue-4939');

function readJson(file) {
  return JSON.parse(fs.readFileSync(file, 'utf8'));
}

const CONTRACT = readJson(path.join(INVESTIGATION, 'font_metric_coverage_contract.json'));

function decision(overrides = {}) {
  return {
    widthSource: 'embeddedMetric',
    characterMatch: 'hit',
    matchKind: 'exact',
    metricEntry: 7,
    provenance: [],
    ...overrides,
  };
}

test('W3 contract has seven exclusive categories and disjoint reuse/delta fields', () => {
  assert.deepEqual(validateCoverageContract(CONTRACT), []);
  assert.deepEqual(
    CONTRACT.categories.map(entry => entry.id),
    [
      'measured-overlay',
      'identity-alias-hit',
      'metric-surrogate',
      'exact-hit',
      'char-miss',
      'face-miss',
      'heuristic',
    ],
  );
  const overlap = CONTRACT.reusedPocFields.filter(field => CONTRACT.deltaFields.includes(field));
  assert.deepEqual(overlap, []);
  assert.deepEqual(CONTRACT.resourcePolicy, {
    failureMode: 'explicit-document-failure',
    partialAggregateAccepted: false,
    deadlineChecks: true,
    cancellationChecks: true,
    workUnitBudget: true,
    nestingDepthBudget: true,
    aggregateRowBudget: true,
    outputByteBudget: true,
    corpusWorkerIsolation: 'required',
    workerProcessPerDocument: true,
    parentWallTimeout: true,
    osAddressSpaceLimit: true,
    processGroupTermination: true,
    supervisorOutputByteBudget: true,
    deidentifiedFailureEnvelope: true,
    workerFailureRecovery: true,
  });
});

test('classification priority produces every W3 category exactly once', () => {
  const cases = [
    [decision({ widthSource: 'metricSpaceOverlay' }), 'measured-overlay'],
    [decision({ provenance: [{
      relationType: 'identity-alias',
      evidenceStatus: 'verified-by-bytes',
    }] }), 'identity-alias-hit'],
    [decision({ provenance: [{
      relationType: 'metric-surrogate',
      evidenceStatus: 'historical',
    }] }), 'metric-surrogate'],
    [decision(), 'exact-hit'],
    [decision({
      widthSource: 'heuristicHalfwidth',
      characterMatch: 'miss',
    }), 'char-miss'],
    [decision({
      widthSource: 'heuristicFullwidth',
      characterMatch: 'notApplicable',
      matchKind: 'none',
      metricEntry: null,
    }), 'face-miss'],
    [decision({
      widthSource: 'areaDotFallback',
      characterMatch: 'notApplicable',
      matchKind: 'none',
      metricEntry: null,
    }), 'heuristic'],
  ];

  for (const [record, expected] of cases) {
    assert.deepEqual(classifyCoverageDecision(record, CONTRACT), {
      status: 'classified',
      category: expected,
    });
  }
});

test('non-font advances are explicit non-applicable records, not heuristic successes', () => {
  for (const widthSource of [
    'clusterContinuation',
    'inlineObjectPlaceholder',
    'hwpPuaFiller',
    'figureSpace',
    'tabAdvance',
  ]) {
    const result = classifyCoverageDecision(decision({
      widthSource,
      characterMatch: 'notApplicable',
      matchKind: 'none',
      metricEntry: null,
    }), CONTRACT);
    assert.equal(result.status, 'not-applicable');
    assert.equal(result.reason, widthSource);
  }
});

test('unknown or contradictory decision combinations fail closed', () => {
  assert.throws(
    () => classifyCoverageDecision(decision({ widthSource: 'newUnreviewedPolicy' }), CONTRACT),
    /unclassified widthSource/,
  );
  assert.throws(
    () => classifyCoverageDecision(decision({ matchKind: 'newFallback' }), CONTRACT),
    /unclassified matchKind/,
  );
  assert.throws(
    () => classifyCoverageDecision(decision({
      characterMatch: 'miss',
      metricEntry: null,
    }), CONTRACT),
    /character miss requires a metric entry/,
  );
  assert.throws(
    () => classifyCoverageDecision(decision({
      provenance: [{ relationType: 'identity-alias', evidenceStatus: 'unknown' }],
    }), CONTRACT),
    /identity-alias requires verified evidence/,
  );
});

test('layout, coverage, join, parse and backend denominators reconcile independently', () => {
  const aggregate = {
    schemaVersion: 1,
    counts: {
      layoutCharacters: 10,
      coverageCharacters: 7,
      notApplicableCharacters: 2,
      excludedCharacters: 1,
      truncatedCharacters: 0,
    },
    categories: Object.fromEntries(CONTRACT.categories.map(entry => [entry.id, 1])),
    joins: { joined: 7, layoutOnly: 2, excluded: 1 },
    documents: {
      attempted: 4,
      success: 3,
      failures: {
        cancelled: 0,
        drm: 1,
        empty: 0,
        encrypted: 0,
        unsupported: 0,
        parser: 0,
        'resource-limit': 0,
      },
    },
    backends: { requested: 4, complete: 1, unsupported: 1, notObserved: 1, failed: 1 },
  };
  assert.deepEqual(reconcileCoverageAggregate(aggregate, CONTRACT), []);

  aggregate.categories['exact-hit'] += 1;
  assert.match(reconcileCoverageAggregate(aggregate, CONTRACT).join('\n'), /category sum/);
  aggregate.categories['exact-hit'] -= 1;
  aggregate.backends.notObserved -= 1;
  assert.match(reconcileCoverageAggregate(aggregate, CONTRACT).join('\n'), /backend state sum/);
});

test('truncation and silently omitted join, parser or backend states fail reconciliation', () => {
  const aggregate = {
    schemaVersion: 1,
    counts: {
      layoutCharacters: 10,
      coverageCharacters: 7,
      notApplicableCharacters: 2,
      excludedCharacters: 1,
      truncatedCharacters: 1,
    },
    categories: Object.fromEntries(CONTRACT.categories.map(entry => [entry.id, 1])),
    joins: { joined: 7, layoutOnly: 2, excluded: 1 },
    documents: {
      attempted: 4,
      success: 3,
      failures: {
        cancelled: 0,
        drm: 1,
        empty: 0,
        encrypted: 0,
        unsupported: 0,
        'resource-limit': 0,
      },
    },
    backends: { requested: 4, complete: 1, unsupported: 1, notObserved: 1, failed: 1 },
  };
  const errors = reconcileCoverageAggregate(aggregate, CONTRACT).join('\n');
  assert.match(errors, /long-page truncation is forbidden/);
  assert.match(errors, /document failure states/);

  aggregate.counts.truncatedCharacters = 0;
  aggregate.documents.failures.parser = 0;
  delete aggregate.joins.layoutOnly;
  delete aggregate.backends.unsupported;
  const omissions = reconcileCoverageAggregate(aggregate, CONTRACT).join('\n');
  assert.match(omissions, /join states/);
  assert.match(omissions, /backend states/);
});

test('canonical coverage hash ignores volatile execution fields and object key order', () => {
  const first = {
    schemaVersion: 1,
    sourceCommit: 'a'.repeat(40),
    generatedAt: '2026-08-21T12:00:00+09:00',
    elapsedMillis: 10,
    categories: { heuristic: 2, 'exact-hit': 3 },
  };
  const second = {
    categories: { 'exact-hit': 3, heuristic: 2 },
    elapsedMillis: 99,
    generatedAt: '2026-08-22T12:00:00+09:00',
    sourceCommit: 'a'.repeat(40),
    schemaVersion: 1,
  };
  assert.equal(canonicalCoverageHash(first), canonicalCoverageHash(second));
  second.categories.heuristic += 1;
  assert.notEqual(canonicalCoverageHash(first), canonicalCoverageHash(second));
});

test('aggregate privacy rejects document identity, raw records, paths, tokens and stacks', () => {
  const unsafe = {
    sourceCommit: 'a'.repeat(40),
    source: 'private-document.hwp',
    records: [{ count: 1 }],
    note: '/home/example/private/file.hwp',
    token: 'Bearer abcdefghijklmnopqrstuvwxyz123456',
    error: 'failure\n    at worker (/tmp/collector.mjs:10:4)',
  };
  const reasons = findSensitiveAggregateValues(unsafe, CONTRACT).map(entry => entry.reason);
  assert.equal(reasons.includes('forbiddenKey'), true);
  assert.equal(reasons.includes('absoluteHomePath'), true);
  assert.equal(reasons.includes('accessToken'), true);
  assert.equal(reasons.includes('errorStack'), true);
  assert.deepEqual(findSensitiveAggregateValues({
    sourceCommit: 'a'.repeat(40),
    categories: { 'exact-hit': 3 },
  }, CONTRACT), []);
});

test('historical W1 source stays verifiable while ownership drift is not semantic drift', () => {
  const oldCandidates = readJson(path.join(W1, 'font_rule_candidates.json'));
  const ledger = readJson(path.join(W1, 'font_rule_ledger.json'));
  const currentCandidates = structuredClone(oldCandidates);
  for (const candidate of currentCandidates.ruleCandidates) {
    if (candidate.sourceBoundaryId !== 'rust-metric.metric-table') continue;
    candidate.sourceLocation.path = 'src/renderer/font_metrics_generated.rs';
    candidate.sourceLocation.symbol = 'GENERATED_FONT_METRICS';
    candidate.sourceLocation.selector = 'static GENERATED_FONT_METRICS:';
    candidate.sourceLocation.sourceSha256 = 'f'.repeat(64);
  }
  const audit = auditW1SemanticDrift(oldCandidates, currentCandidates);

  assert.deepEqual(verifyHistoricalSourceCandidates(oldCandidates, ROOT), []);
  assert.equal(audit.boundaryCount, 30);
  assert.equal(audit.candidateCount, 1352);
  assert.deepEqual(audit.addedCandidateIds, []);
  assert.deepEqual(audit.removedCandidateIds, []);
  assert.deepEqual(audit.changedCandidateIds, []);
  assert.equal(audit.ownershipDriftCandidateIds.length, 600);
  assert.deepEqual(validateW1LedgerBaseline(ledger, CONTRACT, oldCandidates), []);
});

test('the retained local POC v2 aggregate matches the de-identified frozen projection', {
  skip: !fs.existsSync(path.join(ROOT, CONTRACT.existingPoc.localArtifact)),
}, () => {
  const summary = readJson(path.join(ROOT, CONTRACT.existingPoc.localArtifact));
  assert.deepEqual(validatePocV2Baseline(summary, CONTRACT), []);
  assert.equal(pocV2Projection(summary).inputRoot, undefined);
  assert.equal(pocV2Projection(summary).riskDocuments, undefined);
  const hwp = readJson(path.join(ROOT, CONTRACT.existingPoc.localFormatArtifacts.hwp));
  const hwpx = readJson(path.join(ROOT, CONTRACT.existingPoc.localFormatArtifacts.hwpx));
  assert.deepEqual(validatePocFormatAdditivity(summary, hwp, hwpx), []);
});
