import assert from 'node:assert/strict';
import crypto from 'node:crypto';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { spawnSync } from 'node:child_process';
import test, { after, before } from 'node:test';
import { fileURLToPath } from 'node:url';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..', '..');
const BINARY = path.join(ROOT, 'target', 'debug', 'font-metric-gen');
const PLAN = path.join(
  ROOT,
  'mydocs',
  'tech',
  'investigations',
  'issue-4964',
  'font_metric_generator_canary_plan.json',
);
const CORE = path.join(ROOT, 'src', 'renderer', 'font_metrics_data.rs');
const OVERLAY = path.join(ROOT, 'src', 'renderer', 'font_metrics_overlays.rs');
const GENERATED = path.join(ROOT, 'src', 'renderer', 'font_metrics_generated.rs');
let temporaryDirectory;

function sha256File(file) {
  return crypto.createHash('sha256').update(fs.readFileSync(file)).digest('hex');
}

function runGenerator(arguments_, expectedStatus = 0) {
  const result = spawnSync(BINARY, arguments_, {
    cwd: ROOT,
    encoding: 'utf8',
  });
  assert.equal(
    result.status,
    expectedStatus,
    `font-metric-gen status=${result.status}\nstdout=${result.stdout}\nstderr=${result.stderr}`,
  );
  return result;
}

before(() => {
  const build = spawnSync('cargo', ['build', '--bin', 'font-metric-gen'], {
    cwd: ROOT,
    encoding: 'utf8',
  });
  assert.equal(build.status, 0, `cargo build failed\n${build.stdout}\n${build.stderr}`);
  temporaryDirectory = fs.mkdtempSync(path.join(os.tmpdir(), 'rhwp-font-metric-gen-'));
});

after(() => {
  if (temporaryDirectory) fs.rmSync(temporaryDirectory, { recursive: true, force: true });
});

test('tracked public canary emits deterministic generated data and provenance metadata', () => {
  const firstSource = path.join(temporaryDirectory, 'first-generated.rs');
  const firstMetadata = path.join(temporaryDirectory, 'first-metadata.json');
  const secondSource = path.join(temporaryDirectory, 'second-generated.rs');
  const secondMetadata = path.join(temporaryDirectory, 'second-metadata.json');

  runGenerator([
    '--plan',
    PLAN,
    '--generated-output',
    firstSource,
    '--metadata-output',
    firstMetadata,
  ]);
  runGenerator([
    '--plan',
    PLAN,
    '--generated-output',
    secondSource,
    '--metadata-output',
    secondMetadata,
  ]);

  assert.equal(sha256File(firstSource), sha256File(secondSource));
  assert.equal(sha256File(firstMetadata), sha256File(secondMetadata));

  const source = fs.readFileSync(firstSource, 'utf8');
  assert.match(source, /static GENERATED_FONT_METRICS: \[FontMetric; 2\]/);
  assert.doesNotMatch(source, /pub struct FontMetric|fn find_metric/);

  const metadata = JSON.parse(fs.readFileSync(firstMetadata, 'utf8'));
  assert.equal(metadata.generatorContract, 'generated-data-and-provenance-only-v1');
  assert.equal(metadata.generatorVersion, '0.8.4');
  assert.match(metadata.generatorSourceSha256, /^[0-9a-f]{64}$/);
  assert.equal(metadata.targetRegion, 'canary');
  assert.equal(metadata.expectedEntryCount, 2);
  assert.equal(metadata.entries.length, 2);
  assert.equal(metadata.entries[0].order, 0);
  assert.equal(metadata.entries[0].faceIndex, 0);
  assert.equal(
    metadata.entries[0].sourceSha256,
    '6e06a7fe5d696ca719894a23f36bb2b1be8c816a5937cd4ad0f23ca67780dd74',
  );
  assert.equal(metadata.entries[0].license.declaration.spdx, 'OFL-1.1');
  assert.equal(metadata.entries[0].hangulCompression.status, 'verified');
  assert.equal(metadata.entries[0].hangulCompression.sampleCount, 11172);
  assert.ok(metadata.entries[0].hangulCompression.maxError >= 0);
  assert.ok(metadata.entries[0].hangulCompression.avgError >= 0);
  assert.ok(metadata.entries[0].namingRecords.some(record => record.nameId === 1));
  assert.ok(metadata.entries[0].namingRecords.some(record => record.nameId === 6));
  assert.equal(metadata.entries[1].order, 1);
  assert.equal(metadata.entries[1].faceIndex, 1);
  assert.equal(metadata.entries[1].familyName, 'RHWP Exact Face One');
  assert.equal(metadata.entries[1].hangulCompression.status, 'not-applicable');
});

test('generator refuses ownership of core and measured overlay outputs', () => {
  const coreBefore = sha256File(CORE);
  const overlayBefore = sha256File(OVERLAY);
  const generatedBefore = sha256File(GENERATED);
  const metadata = path.join(temporaryDirectory, 'ownership-metadata.json');

  const coreAttempt = runGenerator(
    ['--plan', PLAN, '--generated-output', CORE, '--metadata-output', metadata],
    1,
  );
  assert.match(coreAttempt.stderr, /generator ownership/);
  const overlayAttempt = runGenerator(
    ['--plan', PLAN, '--generated-output', OVERLAY, '--metadata-output', metadata],
    1,
  );
  assert.match(overlayAttempt.stderr, /generator ownership/);
  const incompleteGeneratedAttempt = runGenerator(
    ['--plan', PLAN, '--generated-output', GENERATED, '--metadata-output', metadata],
    1,
  );
  assert.match(incompleteGeneratedAttempt.stderr, /595-entry historical-generated/);

  assert.equal(sha256File(CORE), coreBefore);
  assert.equal(sha256File(OVERLAY), overlayBefore);
  assert.equal(sha256File(GENERATED), generatedBefore);
  assert.equal(fs.existsSync(metadata), false);
});

test('generation rejects implicit directory reconstruction and non-contiguous order', () => {
  const directoryAttempt = runGenerator(
    ['--dir', path.join(ROOT, 'ttfs', 'opensource'), '--output', 'ignored.rs'],
    1,
  );
  assert.match(directoryAttempt.stderr, /암묵적 sort\/dedupe/);

  const invalidPlan = path.join(temporaryDirectory, 'invalid-plan.json');
  const source = path.join(temporaryDirectory, 'invalid-generated.rs');
  const metadata = path.join(temporaryDirectory, 'invalid-metadata.json');
  const plan = JSON.parse(fs.readFileSync(PLAN, 'utf8'));
  plan.inputs[0].order = 1;
  fs.writeFileSync(invalidPlan, `${JSON.stringify(plan, null, 2)}\n`);

  const orderAttempt = runGenerator(
    [
      '--plan',
      invalidPlan,
      '--generated-output',
      source,
      '--metadata-output',
      metadata,
    ],
    1,
  );
  assert.match(orderAttempt.stderr, /명시적 연속값이 아님/);
  assert.equal(fs.existsSync(source), false);
  assert.equal(fs.existsSync(metadata), false);
});
