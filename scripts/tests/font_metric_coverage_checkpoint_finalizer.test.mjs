import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import test from 'node:test';
import { fileURLToPath } from 'node:url';

import { finalizeCoverageCheckpoint } from '../font_metric_coverage_checkpoint_finalizer.mjs';
import { canonicalCoverageHash } from '../font_metric_coverage_contract.mjs';
import { runResumableCoverage } from '../font_metric_coverage_checkpoint_runner.mjs';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..', '..');
const CHECKPOINT_POLICY_BYTES = fs.readFileSync(path.join(
  ROOT,
  'mydocs',
  'tech',
  'investigations',
  'issue-4962',
  'font_metric_coverage_checkpoint_policy.json',
));
const CHECKPOINT_POLICY = JSON.parse(CHECKPOINT_POLICY_BYTES.toString('utf8'));
const SOURCE_HEAD = '1'.repeat(40);
const WORKER_SHA256 = '2'.repeat(64);

function manifest(count = 4) {
  return {
    schemaVersion: 1,
    kind: 'font-metric-coverage-private-corpus-manifest',
    policyVersion: 'finalizer-fixture-v1',
    localOnly: true,
    documents: Array.from({ length: count }, (_, index) => ({
      source: `/private/finalizer-${index}.${index % 2 === 0 ? 'hwp' : 'hwpx'}`,
      format: index % 2 === 0 ? 'hwp' : 'hwpx',
      blake3: (index + 100).toString(16).padStart(64, '0'),
    })),
  };
}

function legacyRow(characters) {
  return {
    font: 'FixtureFont',
    metricFace: 'FixtureMetric',
    language: 'ko',
    ratio: 100,
    spacing: 0,
    kerning: false,
    bold: false,
    italic: false,
    context: 'body',
    alignment: 'left',
    storedLineSeg: true,
    documentCount: 1,
    paragraphCount: 1,
    runCount: 1,
    charCount: characters,
  };
}

function decisionRow(characters) {
  return {
    ...legacyRow(characters),
    normalizedFace: 'FixtureFont',
    substFont: null,
    altType: 0,
    layoutFamily: 'FixtureFont',
    metricRequestedFace: 'FixtureFont',
    metricResolvedFace: 'FixtureMetric',
    matchKind: 'exact',
    metricEntry: 1,
    characterMatch: 'hit',
    widthSource: 'embeddedMetric',
    relationType: 'unknown',
    relationEvidenceStatus: 'unknown',
    coverageCategory: 'exact-hit',
    sourceJoinStatus: 'joined',
  };
}

function aggregate(index, mutate = value => value) {
  const characters = index + 1;
  const format = index % 2 === 0 ? 'hwp' : 'hwpx';
  const value = {
    schemaVersion: 1,
    kind: 'font-metric-coverage-aggregate',
    status: 'complete',
    format,
    counts: {
      paragraphsSeen: 1,
      sourceRunsSeen: 1,
      layoutCharacters: characters,
      coverageCharacters: characters,
      notApplicableCharacters: 0,
      excludedCharacters: 0,
      truncatedCharacters: 0,
      legacyUsageRows: 1,
      decisionUsageRows: 1,
    },
    categories: {
      'measured-overlay': 0,
      'identity-alias-hit': 0,
      'metric-surrogate': 0,
      'exact-hit': characters,
      'char-miss': 0,
      'face-miss': 0,
      heuristic: 0,
    },
    joins: { joined: characters, layoutOnly: 0, excluded: 0 },
    documents: {
      attempted: 1,
      success: 1,
      failures: {
        cancelled: 0,
        drm: 0,
        empty: 0,
        encrypted: 0,
        parser: 0,
        'resource-limit': 0,
        unsupported: 0,
      },
    },
    backends: { requested: 0, complete: 0, failed: 0, notObserved: 0, unsupported: 0 },
    legacyProjectionHash: { algorithm: 'sha256', value: '3'.repeat(64) },
    aggregateHash: { algorithm: 'sha256', value: '' },
    legacyUsage: [legacyRow(characters)],
    decisionUsage: [decisionRow(characters)],
  };
  mutate(value);
  value.aggregateHash.value = canonicalCoverageHash(value);
  return value;
}

function result(index) {
  if (index === 3) {
    return {
      status: 'failed',
      failure: 'parser',
      metrics: { elapsedMillis: 103, peakRssBytes: 1003 },
    };
  }
  return {
    status: 'complete',
    aggregate: aggregate(index),
    metrics: { elapsedMillis: 100 + index, peakRssBytes: 1000 + index },
  };
}

function options(directory, inputManifest = manifest()) {
  return {
    manifest: inputManifest,
    manifestBytes: Buffer.from(`${JSON.stringify(inputManifest)}\n`),
    checkpointPolicy: CHECKPOINT_POLICY,
    checkpointPolicyBytes: CHECKPOINT_POLICY_BYTES,
    checkpointDirectory: directory,
    workerPath: '/fixture/worker',
    workerSha256: WORKER_SHA256,
    sourceHead: SOURCE_HEAD,
  };
}

function temporary(t) {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), 'rhwp-4962-finalizer-'));
  t.after(() => fs.rmSync(directory, { recursive: true, force: true }));
  return directory;
}

test('resumed and uninterrupted checkpoints finalize to the exact aggregate', async t => {
  const resumedDirectory = temporary(t);
  const uninterruptedDirectory = temporary(t);
  let calls = 0;
  await assert.rejects(
    runResumableCoverage({
      ...options(resumedDirectory),
      runDocument: async source => {
        const index = Number(/(\d+)/u.exec(source)[1]);
        calls += 1;
        if (calls === 3) throw new Error('forced interruption');
        return result(index);
      },
    }),
    /forced interruption/u,
  );
  await runResumableCoverage({
    ...options(resumedDirectory),
    runDocument: async source => result(Number(/(\d+)/u.exec(source)[1])),
  });
  await runResumableCoverage({
    ...options(uninterruptedDirectory),
    runDocument: async source => result(Number(/(\d+)/u.exec(source)[1])),
  });

  const resumed = finalizeCoverageCheckpoint(resumedDirectory);
  const uninterrupted = finalizeCoverageCheckpoint(uninterruptedDirectory);
  assert.deepEqual(resumed, uninterrupted);
  assert.equal(resumed.aggregateHash.value, canonicalCoverageHash(resumed));
  assert.deepEqual(
    resumed.documents,
    {
      attempted: 4,
      success: 3,
      failures: {
        cancelled: 0,
        drm: 0,
        empty: 0,
        encrypted: 0,
        parser: 1,
        'resource-limit': 0,
        unsupported: 0,
      },
      formats: {
        hwp: {
          attempted: 2,
          success: 2,
          failures: {
            cancelled: 0,
            drm: 0,
            empty: 0,
            encrypted: 0,
            parser: 0,
            'resource-limit': 0,
            unsupported: 0,
          },
        },
        hwpx: {
          attempted: 2,
          success: 1,
          failures: {
            cancelled: 0,
            drm: 0,
            empty: 0,
            encrypted: 0,
            parser: 1,
            'resource-limit': 0,
            unsupported: 0,
          },
        },
      },
    },
  );
  assert.equal(resumed.counts.layoutCharacters, 6);
  assert.equal(resumed.counts.legacyUsageRows, 2);
  assert.equal(resumed.counts.decisionUsageRows, 2);
  assert.deepEqual(
    resumed.legacyUsage.map(row => [row.format, row.documentCount, row.charCount]),
    [['hwp', 2, 4], ['hwpx', 1, 2]],
  );
  assert.doesNotMatch(JSON.stringify(resumed), /\/private\//u);
});

test('finalizer rejects incomplete state, uncommitted tail and usage schema drift', async t => {
  const incompleteDirectory = temporary(t);
  let calls = 0;
  await assert.rejects(runResumableCoverage({
    ...options(incompleteDirectory),
    runDocument: async source => {
      calls += 1;
      if (calls === 2) throw new Error('stop');
      return result(Number(/(\d+)/u.exec(source)[1]));
    },
  }));
  assert.throws(
    () => finalizeCoverageCheckpoint(incompleteDirectory),
    /completed checkpoint state is invalid/u,
  );

  const tailDirectory = temporary(t);
  await runResumableCoverage({
    ...options(tailDirectory, manifest(1)),
    runDocument: async () => result(0),
  });
  const journal = path.join(tailDirectory, 'journal.ndjson');
  fs.appendFileSync(journal, '{"partial":');
  const tailedSize = fs.statSync(journal).size;
  assert.throws(
    () => finalizeCoverageCheckpoint(tailDirectory),
    /uncommitted tail/u,
  );
  assert.equal(fs.statSync(journal).size, tailedSize);

  const driftDirectory = temporary(t);
  const oneDocument = manifest(1);
  await runResumableCoverage({
    ...options(driftDirectory, oneDocument),
    runDocument: async () => ({
      status: 'complete',
      aggregate: aggregate(0, value => {
        value.legacyUsage[0].unexpected = true;
      }),
      metrics: { elapsedMillis: 1, peakRssBytes: 1 },
    }),
  });
  assert.throws(
    () => finalizeCoverageCheckpoint(driftDirectory),
    /legacyUsage row schema drift/u,
  );
});
