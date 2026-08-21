import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { createHash } from 'node:crypto';
import test from 'node:test';
import { fileURLToPath } from 'node:url';

import { buildFullCoverageManifest } from '../font_metric_coverage_full_manifest.mjs';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..', '..');
const POLICY_BYTES = fs.readFileSync(path.join(
  ROOT,
  'mydocs',
  'tech',
  'investigations',
  'issue-4962',
  'font_metric_coverage_full_manifest_policy.json',
));
const BASE_POLICY = JSON.parse(POLICY_BYTES.toString('utf8'));
const CHECKPOINT_POLICY_BYTES = fs.readFileSync(path.join(
  ROOT,
  'mydocs',
  'tech',
  'investigations',
  'issue-4962',
  'font_metric_coverage_checkpoint_policy.json',
));

function temporary(t) {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), 'rhwp-4962-manifest-'));
  t.after(() => fs.rmSync(directory, { recursive: true, force: true }));
  return directory;
}

function fixturePolicy() {
  const policy = structuredClone(BASE_POLICY);
  policy.expected = {
    documents: 3,
    formats: { hwp: 2, hwpx: 1 },
    candidateBytes: 10,
    ignoredRegularFiles: 1,
    ignoredBytes: 2,
  };
  policy.execution.hashConcurrency = 2;
  return policy;
}

function fakeHash(filePath) {
  const bytes = fs.readFileSync(filePath);
  return Promise.resolve({
    hash: createHash('sha256').update(bytes).digest('hex'),
    bytes: bytes.length,
  });
}

test('full manifest is deterministic, local-only and preserves duplicate content instances', async t => {
  const directory = temporary(t);
  const corpus = path.join(directory, 'corpus');
  const nested = path.join(corpus, 'nested');
  fs.mkdirSync(nested, { recursive: true });
  fs.writeFileSync(path.join(corpus, 'a.hwp'), 'abc');
  fs.writeFileSync(path.join(nested, 'b.hwp'), 'abc');
  fs.writeFileSync(path.join(corpus, 'c.hwpx'), 'wxyz');
  fs.writeFileSync(path.join(corpus, 'metadata.tsv'), 'ok');
  const policy = fixturePolicy();
  const policyBytes = Buffer.from(`${JSON.stringify(policy)}\n`);
  const options = {
    corpusRoot: corpus,
    sourceHead: '1'.repeat(40),
    rhwpAgent: process.execPath,
    checkpointFilesystemPath: directory,
    policy,
    policyBytes,
    checkpointPolicyBytes: CHECKPOINT_POLICY_BYTES,
    hashFile: fakeHash,
  };
  const first = await buildFullCoverageManifest(options);
  const second = await buildFullCoverageManifest(options);
  assert.deepEqual(first.manifest, second.manifest);
  assert.equal(first.preflight.manifestSha256, second.preflight.manifestSha256);
  assert.equal(first.manifest.localOnly, true);
  assert.equal(first.manifest.documents.length, 3);
  assert.deepEqual(first.preflight.duplicateContent, { groups: 1, extraInstances: 1 });
  assert.deepEqual(first.preflight.formats, { hwp: 2, hwpx: 1 });
  assert.equal(first.preflight.candidateBytes, 10);
  assert.equal(first.preflight.privacy.containsDocumentIdentity, false);
  const safePreflight = JSON.stringify(first.preflight);
  assert.doesNotMatch(safePreflight, /(?:a\.hwp|b\.hwp|c\.hwpx|\/corpus)/u);
  assert.equal(new Set(first.manifest.documents.map(row => row.source)).size, 3);
  assert.equal(first.manifest.documents[0].format, 'hwp');
  assert.equal(first.manifest.documents[1].format, 'hwp');
  assert.equal(first.manifest.documents[2].format, 'hwpx');
});

test('full manifest fails closed on symlink, inventory drift and mutation during hash', async t => {
  const symlinkDirectory = temporary(t);
  const symlinkCorpus = path.join(symlinkDirectory, 'corpus');
  fs.mkdirSync(symlinkCorpus);
  fs.writeFileSync(path.join(symlinkCorpus, 'a.hwp'), 'abc');
  fs.symlinkSync(path.join(symlinkCorpus, 'a.hwp'), path.join(symlinkCorpus, 'linked.hwp'));
  await assert.rejects(
    buildFullCoverageManifest({
      corpusRoot: symlinkCorpus,
      sourceHead: '1'.repeat(40),
      rhwpAgent: process.execPath,
      checkpointFilesystemPath: symlinkDirectory,
      policy: fixturePolicy(),
      policyBytes: POLICY_BYTES,
      checkpointPolicyBytes: CHECKPOINT_POLICY_BYTES,
      hashFile: fakeHash,
    }),
    /contains a symlink/u,
  );

  const driftDirectory = temporary(t);
  const driftCorpus = path.join(driftDirectory, 'corpus');
  fs.mkdirSync(driftCorpus);
  fs.writeFileSync(path.join(driftCorpus, 'only.hwp'), 'abc');
  await assert.rejects(
    buildFullCoverageManifest({
      corpusRoot: driftCorpus,
      sourceHead: '1'.repeat(40),
      rhwpAgent: process.execPath,
      checkpointFilesystemPath: driftDirectory,
      policy: fixturePolicy(),
      policyBytes: POLICY_BYTES,
      checkpointPolicyBytes: CHECKPOINT_POLICY_BYTES,
      hashFile: fakeHash,
    }),
    /frozen Stage 4-A inventory/u,
  );

  const mutationDirectory = temporary(t);
  const mutationCorpus = path.join(mutationDirectory, 'corpus');
  fs.mkdirSync(mutationCorpus);
  fs.writeFileSync(path.join(mutationCorpus, 'one.hwp'), 'abc');
  const mutationPolicy = fixturePolicy();
  mutationPolicy.expected = {
    documents: 1,
    formats: { hwp: 1, hwpx: 0 },
    candidateBytes: 3,
    ignoredRegularFiles: 0,
    ignoredBytes: 0,
  };
  await assert.rejects(
    buildFullCoverageManifest({
      corpusRoot: mutationCorpus,
      sourceHead: '1'.repeat(40),
      rhwpAgent: process.execPath,
      checkpointFilesystemPath: mutationDirectory,
      policy: mutationPolicy,
      policyBytes: Buffer.from(`${JSON.stringify(mutationPolicy)}\n`),
      checkpointPolicyBytes: CHECKPOINT_POLICY_BYTES,
      hashFile: async filePath => {
        const result = await fakeHash(filePath);
        fs.appendFileSync(filePath, 'changed');
        return result;
      },
    }),
    /changed while hashing/u,
  );
});
