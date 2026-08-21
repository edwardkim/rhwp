import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import test from 'node:test';
import { fileURLToPath } from 'node:url';

import { runResumableCoverage } from '../font_metric_coverage_checkpoint_runner.mjs';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..', '..');
const POLICY_BYTES = fs.readFileSync(path.join(
  ROOT,
  'mydocs',
  'tech',
  'investigations',
  'issue-4962',
  'font_metric_coverage_checkpoint_policy.json',
));
const POLICY = JSON.parse(POLICY_BYTES.toString('utf8'));
const SOURCE_HEAD = '1'.repeat(40);
const WORKER_SHA256 = '2'.repeat(64);

function manifest(count = 5) {
  return {
    schemaVersion: 1,
    kind: 'font-metric-coverage-private-pilot-cohort',
    policyVersion: 'fixture-policy',
    localOnly: true,
    selections: Array.from({ length: count }, (_, index) => ({
      source: `/private/fixture-${index}.hwp`,
      format: index % 2 === 0 ? 'hwp' : 'hwpx',
      blake3: (index + 100).toString(16).padStart(64, '0'),
    })),
  };
}

function aggregate(index) {
  const characters = index + 1;
  return {
    schemaVersion: 1,
    kind: 'font-metric-coverage-aggregate',
    status: 'complete',
    format: index % 2 === 0 ? 'hwp' : 'hwpx',
    counts: {
      layoutCharacters: characters,
      coverageCharacters: characters,
      notApplicableCharacters: 0,
      excludedCharacters: 0,
      truncatedCharacters: 0,
      paragraphsSeen: 1,
      sourceRunsSeen: 1,
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
    legacyUsage: [{ charCount: characters }],
    decisionUsage: [{ charCount: characters }],
    legacyProjectionHash: { algorithm: 'sha256', value: (index + 10).toString(16).padStart(64, '0') },
    aggregateHash: { algorithm: 'sha256', value: (index + 20).toString(16).padStart(64, '0') },
  };
}

function completeResult(index) {
  return {
    status: 'complete',
    aggregate: aggregate(index),
    metrics: { elapsedMillis: 100 + index, peakRssBytes: 1000 + index },
  };
}

function options(directory, inputManifest = manifest()) {
  const manifestBytes = Buffer.from(`${JSON.stringify(inputManifest)}\n`);
  return {
    manifest: inputManifest,
    manifestBytes,
    checkpointPolicy: POLICY,
    checkpointPolicyBytes: POLICY_BYTES,
    checkpointDirectory: directory,
    workerPath: '/fixture/worker',
    workerSha256: WORKER_SHA256,
    sourceHead: SOURCE_HEAD,
  };
}

function temporary(t) {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), 'rhwp-4962-checkpoint-'));
  t.after(() => fs.rmSync(directory, { recursive: true, force: true }));
  return directory;
}

test('forced interruption resumes to the exact uninterrupted state', async t => {
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
        return completeResult(index);
      },
    }),
    /forced interruption/u,
  );
  const interruptedState = JSON.parse(fs.readFileSync(
    path.join(resumedDirectory, 'state.json'),
    'utf8',
  ));
  assert.equal(interruptedState.nextIndex, 2);

  const resumed = await runResumableCoverage({
    ...options(resumedDirectory),
    runDocument: async source => completeResult(Number(/(\d+)/u.exec(source)[1])),
  });
  const uninterrupted = await runResumableCoverage({
    ...options(uninterruptedDirectory),
    runDocument: async source => completeResult(Number(/(\d+)/u.exec(source)[1])),
  });
  assert.equal(resumed.status, 'complete');
  assert.deepEqual(resumed, uninterrupted);
  assert.equal(resumed.nextIndex, 5);
  assert.equal(resumed.summary.documents.success, 5);
  assert.equal(resumed.summary.counts.layoutCharacters, 15);
  assert.deepEqual(
    resumed.summary.usageRowSums,
    { legacyUsageRows: 5, decisionUsageRows: 5 },
  );
  assert.doesNotMatch(
    fs.readFileSync(path.join(resumedDirectory, 'state.json'), 'utf8')
      + fs.readFileSync(path.join(resumedDirectory, 'journal.ndjson'), 'utf8'),
    /\/private\//u,
  );
  let completedCalls = 0;
  const alreadyComplete = await runResumableCoverage({
    ...options(resumedDirectory),
    runDocument: async () => {
      completedCalls += 1;
      return completeResult(0);
    },
  });
  assert.deepEqual(alreadyComplete, resumed);
  assert.equal(completedCalls, 0);
});

test('all run identity drift refuses resume before another document runs', async t => {
  const changedManifest = manifest();
  changedManifest.selections[0].blake3 = '4'.repeat(64);
  const driftCases = [
    ['source head', current => ({ ...current, sourceHead: '3'.repeat(40) })],
    ['worker', current => ({ ...current, workerSha256: '3'.repeat(64) })],
    ['manifest', current => options(current.checkpointDirectory, changedManifest)],
    ['manifest raw bytes', current => ({
      ...current,
      manifestBytes: Buffer.concat([current.manifestBytes, Buffer.from('\n')]),
    })],
    ['checkpoint policy', current => ({
      ...current,
      checkpointPolicyBytes: Buffer.concat([current.checkpointPolicyBytes, Buffer.from('\n')]),
    })],
    ['coverage contract', current => ({
      ...current,
      coverageContractBytes: Buffer.from('changed contract'),
    })],
    ['analysis options', current => ({ ...current, analysisOptions: { mode: 'changed' } })],
    ['isolation limits', current => ({ ...current, limits: { wallTimeoutMillis: 12345 } })],
  ];
  for (const [label, drift] of driftCases) {
    const directory = temporary(t);
    let calls = 0;
    await assert.rejects(runResumableCoverage({
      ...options(directory),
      runDocument: async source => {
        const index = Number(/(\d+)/u.exec(source)[1]);
        calls += 1;
        if (calls === 2) throw new Error('stop');
        return completeResult(index);
      },
    }));
    let resumedCalls = 0;
    await assert.rejects(
      runResumableCoverage({
        ...drift(options(directory)),
        runDocument: async () => {
          resumedCalls += 1;
          return completeResult(1);
        },
      }),
      /identity or schema drift/u,
      label,
    );
    assert.equal(resumedCalls, 0, label);
  }
});

test('manifest duplicate identity and aggregate format mismatch fail closed', async t => {
  const duplicate = manifest(2);
  duplicate.selections[1].format = duplicate.selections[0].format;
  duplicate.selections[1].blake3 = duplicate.selections[0].blake3;
  await assert.rejects(
    runResumableCoverage({
      ...options(temporary(t), duplicate),
      runDocument: async () => completeResult(0),
    }),
    /duplicate document/u,
  );

  const directory = temporary(t);
  await assert.rejects(
    runResumableCoverage({
      ...options(directory, manifest(2)),
      runDocument: async () => completeResult(1),
    }),
    /format does not match manifest/u,
  );
  const state = JSON.parse(fs.readFileSync(path.join(directory, 'state.json'), 'utf8'));
  assert.equal(state.nextIndex, 0);
  assert.equal(fs.statSync(path.join(directory, 'journal.ndjson')).size, 0);
});

test('journal storage budget fails before append', async t => {
  const directory = temporary(t);
  const constrainedPolicy = structuredClone(POLICY);
  constrainedPolicy.storage.maxJournalBytes = 16;
  const constrainedPolicyBytes = Buffer.from(`${JSON.stringify(constrainedPolicy)}\n`);
  await assert.rejects(
    runResumableCoverage({
      ...options(directory, manifest(1)),
      checkpointPolicy: constrainedPolicy,
      checkpointPolicyBytes: constrainedPolicyBytes,
      runDocument: async () => completeResult(0),
    }),
    /journal storage limit exceeded/u,
  );
  const state = JSON.parse(fs.readFileSync(path.join(directory, 'state.json'), 'utf8'));
  assert.equal(state.nextIndex, 0);
  assert.equal(fs.statSync(path.join(directory, 'journal.ndjson')).size, 0);
});

test('uncommitted journal tail is truncated before resume', async t => {
  const directory = temporary(t);
  let calls = 0;
  await assert.rejects(runResumableCoverage({
    ...options(directory),
    runDocument: async source => {
      const index = Number(/(\d+)/u.exec(source)[1]);
      calls += 1;
      if (calls === 2) throw new Error('stop');
      return completeResult(index);
    },
  }));
  const journal = path.join(directory, 'journal.ndjson');
  const committedBytes = fs.statSync(journal).size;
  fs.appendFileSync(journal, '{"partial":');
  const resumed = await runResumableCoverage({
    ...options(directory),
    runDocument: async source => completeResult(Number(/(\d+)/u.exec(source)[1])),
  });
  assert.equal(resumed.status, 'complete');
  assert.ok(fs.statSync(journal).size > committedBytes);
  assert.doesNotMatch(fs.readFileSync(journal, 'utf8'), /partial/u);
});

test('committed journal corruption fails closed', async t => {
  const directory = temporary(t);
  let calls = 0;
  await assert.rejects(runResumableCoverage({
    ...options(directory),
    runDocument: async source => {
      const index = Number(/(\d+)/u.exec(source)[1]);
      calls += 1;
      if (calls === 2) throw new Error('stop');
      return completeResult(index);
    },
  }));
  const journal = path.join(directory, 'journal.ndjson');
  const bytes = fs.readFileSync(journal);
  bytes[0] = 0x78;
  fs.writeFileSync(journal, bytes);
  await assert.rejects(
    runResumableCoverage({
      ...options(directory),
      runDocument: async () => completeResult(1),
    }),
  );
});
