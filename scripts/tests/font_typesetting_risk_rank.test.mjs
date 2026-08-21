import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import test from 'node:test';
import { fileURLToPath } from 'node:url';

import {
  findSensitiveTypesettingRiskValues,
  rankTypesettingRiskAggregate,
  rankTypesettingRiskFile,
  validateRiskInputPreconditions,
  validateTypesettingRiskContract,
} from '../font_typesetting_risk_rank.mjs';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..', '..');
const INVESTIGATION = path.join(
  ROOT,
  'mydocs',
  'tech',
  'investigations',
  'issue-4962',
);
const CONTRACT = JSON.parse(fs.readFileSync(path.join(
  INVESTIGATION,
  'font_typesetting_risk_contract.json',
), 'utf8'));

function row(overrides = {}) {
  return {
    format: 'hwp',
    font: 'Face A',
    metricFace: 'Metric A',
    language: 'ko',
    ratio: 100,
    spacing: 0,
    kerning: false,
    bold: false,
    italic: false,
    context: 'body',
    alignment: 'left',
    storedLineSeg: true,
    normalizedFace: 'Face A',
    substFont: null,
    altType: 0,
    layoutFamily: 'Face A',
    metricRequestedFace: 'Metric A',
    metricResolvedFace: 'Metric A',
    matchKind: 'exact',
    metricEntry: 1,
    characterMatch: 'hit',
    widthSource: 'embeddedMetric',
    relationType: 'unknown',
    relationEvidenceStatus: 'unknown',
    coverageCategory: 'exact-hit',
    sourceJoinStatus: 'joined',
    documentCount: 1,
    paragraphCount: 1,
    runCount: 1,
    charCount: 1,
    ...overrides,
  };
}

function fixtureRows() {
  return [
    row({ charCount: 100 }),
    row({
      metricFace: null,
      metricRequestedFace: 'Missing Metric',
      metricResolvedFace: null,
      matchKind: 'none',
      metricEntry: null,
      characterMatch: 'notApplicable',
      widthSource: 'heuristicFullwidth',
      coverageCategory: 'face-miss',
      charCount: 10,
    }),
    row({
      format: 'hwpx',
      ratio: 90,
      spacing: -5,
      context: 'table-cell',
      storedLineSeg: false,
      characterMatch: 'miss',
      widthSource: 'heuristicHalfwidth',
      coverageCategory: 'char-miss',
      charCount: 5,
    }),
    row({
      ratio: 95,
      context: 'footnote+header',
      metricEntry: null,
      characterMatch: 'notApplicable',
      widthSource: 'areaDotFallback',
      coverageCategory: 'heuristic',
      charCount: 2,
    }),
    row({
      format: 'hwpx',
      font: 'Face B',
      normalizedFace: 'Face B',
      layoutFamily: 'Face B',
      metricFace: null,
      metricRequestedFace: 'Missing Metric',
      metricResolvedFace: null,
      matchKind: 'none',
      metricEntry: null,
      characterMatch: 'notApplicable',
      widthSource: 'heuristicFullwidth',
      coverageCategory: 'face-miss',
      storedLineSeg: false,
      charCount: 20,
    }),
  ];
}

function fixtureAggregate(rows = fixtureRows()) {
  return {
    schemaVersion: 1,
    kind: 'font-metric-coverage-aggregate',
    status: 'complete',
    format: 'mixed',
    counts: {
      layoutCharacters: 137,
      coverageCharacters: 137,
      decisionUsageRows: rows.length,
    },
    categories: {
      'measured-overlay': 0,
      'identity-alias-hit': 0,
      'metric-surrogate': 0,
      'exact-hit': 100,
      'char-miss': 5,
      'face-miss': 30,
      heuristic: 2,
    },
    joins: { joined: 137, layoutOnly: 0, excluded: 0 },
    decisionUsage: rows,
  };
}

test('W4 contract fixes compatibility, identity, proxy and lane boundaries', () => {
  assert.deepEqual(validateTypesettingRiskContract(CONTRACT), []);
  assert.deepEqual(CONTRACT.compatibilityProjection.riskCategories, [
    'char-miss',
    'face-miss',
    'heuristic',
  ]);
  assert.equal(CONTRACT.candidateIdentity.documentFaceKey, 'font');
  assert.equal(CONTRACT.candidateIdentity.mergeMetricClustersIntoDocumentFaces, false);
  assert.equal(
    CONTRACT.candidateIdentity.nullMetricRequestPolicy,
    'preserve-unavailable-cluster',
  );
  assert.equal(
    CONTRACT.editingAxes.fixedFrameContextProxy.outputField,
    'fixedFrameContextProxy',
  );
  assert.equal(CONTRACT.editingAxes.fixedFrameContextProxy.geometryClaim, false);
  assert.equal(CONTRACT.editingAxes.lineSegLanes.riskMultiplier, false);
});

test('same-row risk mass keeps document face identity and LineSeg lanes separate', () => {
  const result = rankTypesettingRiskAggregate(fixtureAggregate(), CONTRACT);
  assert.deepEqual(result.totals, {
    totalUsageCharacters: 137,
    riskCharacters: 37,
    storedRiskMass: 18,
    freshCandidateRiskMass: 70,
    baseRiskMass: 88,
  });
  assert.equal(result.documentFaces.length, 2);
  assert.deepEqual(result.documentFaces[0], {
    rank: 1,
    documentFace: 'Face A',
    totalUsageCharacters: 117,
    riskCharacters: 17,
    categoryRiskCharacters: {
      'char-miss': 5,
      'face-miss': 10,
      heuristic: 2,
    },
    compressedFixedContextRiskCharacters: 7,
    storedRiskMass: 18,
    freshCandidateRiskMass: 50,
    baseRiskMass: 68,
    formatCharacters: { hwp: 112, hwpx: 5 },
  });
  assert.equal(result.documentFaces[1].documentFace, 'Face B');
  assert.equal(result.documentFaces[1].baseRiskMass, 20);
});

test('metric request clusters explain shared causes without merging document faces', () => {
  const result = rankTypesettingRiskAggregate(fixtureAggregate(), CONTRACT);
  assert.deepEqual(
    result.metricRequestClusters.map(entry => [
      entry.rank,
      entry.metricRequestedFace,
      entry.documentFaceCount,
      entry.riskCharacters,
      entry.baseRiskMass,
    ]),
    [
      [1, 'Metric A', 1, 7, 58],
      [2, 'Missing Metric', 2, 30, 30],
    ],
  );
  assert.deepEqual(result.documentFaces.map(entry => entry.documentFace), ['Face A', 'Face B']);
});

test('a missing metric request remains an unavailable cluster instead of a guessed face', () => {
  const rows = fixtureRows();
  rows[1].metricRequestedFace = null;
  rows[4].metricRequestedFace = null;
  const result = rankTypesettingRiskAggregate(fixtureAggregate(rows), CONTRACT);
  const unavailable = result.metricRequestClusters.find(entry => (
    entry.metricRequestedFace === null
  ));
  assert.deepEqual(
    [unavailable.documentFaceCount, unavailable.riskCharacters, unavailable.baseRiskMass],
    [2, 30, 30],
  );
});

test('combined format counts are additive and do not use documentCount as reach', () => {
  const result = rankTypesettingRiskAggregate(fixtureAggregate(), CONTRACT);
  for (const candidate of result.documentFaces) {
    assert.equal(
      candidate.formatCharacters.hwp + candidate.formatCharacters.hwpx,
      candidate.totalUsageCharacters,
    );
    assert.equal('affectedDocuments' in candidate, false);
  }
  assert.equal(
    result.documentFaces.reduce((total, entry) => total + entry.riskCharacters, 0),
    37,
  );
});

test('row order does not change canonical public projection or hash', () => {
  const forward = rankTypesettingRiskAggregate(fixtureAggregate(), CONTRACT);
  const reversedRows = [...fixtureRows()].reverse();
  const reversed = rankTypesettingRiskAggregate(fixtureAggregate(reversedRows), CONTRACT);
  assert.deepEqual(reversed, forward);
  assert.match(forward.outputHash.value, /^[0-9a-f]{64}$/u);
});

test('streaming file path skips legacy rows and produces the fixture ranking', async t => {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), 'rhwp-4962-w4-rank-'));
  t.after(() => fs.rmSync(directory, { recursive: true, force: true }));
  const input = path.join(directory, 'fixture.json');
  const aggregate = fixtureAggregate();
  const fileAggregate = {
    schemaVersion: aggregate.schemaVersion,
    kind: aggregate.kind,
    status: aggregate.status,
    format: aggregate.format,
    checkpoint: { identity: { sourceHead: '1'.repeat(40) } },
    counts: aggregate.counts,
    categories: aggregate.categories,
    joins: aggregate.joins,
    documents: { attempted: 1, success: 1, failures: {} },
    aggregateHash: { algorithm: 'sha256', value: '2'.repeat(64) },
    legacyUsage: [{ ignored: 'the streaming ranker must not materialize this array' }],
    decisionUsage: aggregate.decisionUsage,
  };
  fs.writeFileSync(input, `${JSON.stringify(fileAggregate)}\n`, { mode: 0o600 });
  const streamed = await rankTypesettingRiskFile(input, CONTRACT, {
    enforceFrozenInput: false,
  });
  const inMemory = rankTypesettingRiskAggregate(aggregate, CONTRACT);
  assert.deepEqual(streamed.totals, inMemory.totals);
  assert.deepEqual(streamed.documentFaces, inMemory.documentFaces);
  assert.deepEqual(streamed.metricRequestClusters, inMemory.metricRequestClusters);
});

test('frozen input drift fails before ranking', () => {
  const observed = {
    primary: {
      mode: '0600',
      bytes: 110097106,
      fileSha256: CONTRACT.inputPreconditions.primary.fileSha256,
      aggregateSha256: CONTRACT.inputPreconditions.primary.aggregateSha256,
      sourceCommit: CONTRACT.inputPreconditions.primary.sourceCommit,
    },
    determinismPeer: {
      mode: '0600',
      bytes: 110097106,
      fileSha256: CONTRACT.inputPreconditions.determinismPeer.fileSha256,
      aggregateSha256: CONTRACT.inputPreconditions.determinismPeer.aggregateSha256,
      sourceCommit: CONTRACT.inputPreconditions.determinismPeer.sourceCommit,
    },
    postMergeIngress: {
      baselineMode: '0600',
      currentMode: '0600',
      documentCount: 32,
      baselineSourceCommit: CONTRACT.inputPreconditions.postMergeIngress.baselineSourceCommit,
      currentSourceCommit: CONTRACT.inputPreconditions.postMergeIngress.currentSourceCommit,
      semanticProjectionSha256:
        CONTRACT.inputPreconditions.postMergeIngress.semanticProjectionSha256,
    },
  };
  assert.deepEqual(validateRiskInputPreconditions(observed, CONTRACT), []);
  observed.primary.fileSha256 = '0'.repeat(64);
  assert.match(validateRiskInputPreconditions(observed, CONTRACT).join('\n'), /primary.*SHA-256/iu);
});

test('public projection rejects document identity, paths, raw rows, tokens and stacks', () => {
  const unsafe = {
    documentFace: 'Face A',
    inputRoot: '/home/example/private-corpus',
    riskDocuments: [{ fileName: 'private.hwpx' }],
    records: fixtureRows(),
    token: 'Bearer abcdefghijklmnopqrstuvwxyz123456',
    error: 'failure\n    at ranker (/tmp/private-ranker.mjs:10:4)',
  };
  const reasons = findSensitiveTypesettingRiskValues(unsafe, CONTRACT)
    .map(entry => entry.reason);
  assert.equal(reasons.includes('forbiddenKey'), true);
  assert.equal(reasons.includes('absoluteHomePath'), true);
  assert.equal(reasons.includes('accessToken'), true);
  assert.equal(reasons.includes('errorStack'), true);
  assert.deepEqual(findSensitiveTypesettingRiskValues({
    documentFace: 'Face A',
    riskCharacters: 17,
    baseRiskMass: 68,
  }, CONTRACT), []);
});

test('unknown decision dimensions and non-joined usage fail closed', () => {
  const extraField = fixtureAggregate();
  extraField.decisionUsage[0].newDimension = true;
  assert.throws(
    () => rankTypesettingRiskAggregate(extraField, CONTRACT),
    /schema drift/u,
  );

  const unknownCategory = fixtureAggregate();
  unknownCategory.decisionUsage[0].coverageCategory = 'new-risk-policy';
  assert.throws(
    () => rankTypesettingRiskAggregate(unknownCategory, CONTRACT),
    /coverageCategory is unclassified/u,
  );

  const layoutOnly = fixtureAggregate();
  layoutOnly.decisionUsage[0].sourceJoinStatus = 'layoutOnly';
  assert.throws(
    () => rankTypesettingRiskAggregate(layoutOnly, CONTRACT),
    /not a joined source row/u,
  );
});
