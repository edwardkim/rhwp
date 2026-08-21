import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import process from 'node:process';
import test from 'node:test';
import { fileURLToPath } from 'node:url';

import {
  runIsolatedCoverageDocument,
  runIsolatedCoverageDocuments,
} from '../font_metric_coverage_supervisor.mjs';

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..', '..');
const MOCK = path.join(ROOT, 'scripts', 'tests', 'fixtures', 'font_metric_coverage_mock_worker.mjs');

function fixture(mode) {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), 'rhwp-4962-supervisor-'));
  const input = path.join(directory, 'document.fixture');
  fs.writeFileSync(input, mode);
  return { input, cleanup: () => fs.rmSync(directory, { recursive: true, force: true }) };
}

function mockOptions(limits = {}) {
  return {
    workerPath: process.execPath,
    workerArguments: [MOCK],
    limits: {
      wallTimeoutMillis: 1000,
      cpuSeconds: 2,
      addressSpaceBytes: 2 * 1024 * 1024 * 1024,
      maxStdoutBytes: 64 * 1024,
      rssPollMillis: 5,
      ...limits,
    },
  };
}

test('real prlimit applies finite address-space and CPU limits', async t => {
  const sample = fixture('limits');
  t.after(sample.cleanup);
  const result = await runIsolatedCoverageDocument(sample.input, mockOptions());
  assert.equal(result.status, 'complete');
  assert.equal(result.aggregate.kind, 'font-metric-coverage-aggregate');
  assert.ok(result.metrics.peakRssBytes >= 0);
});

test('OS address-space exhaustion is a safe resource-limit failure', async t => {
  const sample = fixture('success');
  t.after(sample.cleanup);
  const result = await runIsolatedCoverageDocument(sample.input, mockOptions({
    addressSpaceBytes: 64 * 1024 * 1024,
  }));
  assert.equal(result.status, 'failed');
  assert.equal(result.failure, 'resource-limit');
});

test('wall timeout kills a hung worker and returns only a safe failure', async t => {
  const sample = fixture('hang');
  t.after(sample.cleanup);
  const result = await runIsolatedCoverageDocument(sample.input, mockOptions({
    wallTimeoutMillis: 50,
  }));
  assert.deepEqual(
    Object.keys(result).sort(),
    ['failure', 'kind', 'metrics', 'schemaVersion', 'status'],
  );
  assert.equal(result.status, 'failed');
  assert.equal(result.failure, 'resource-limit');
});

test('wall timeout terminates the whole worker process group', async t => {
  const sample = fixture('placeholder');
  t.after(sample.cleanup);
  const marker = path.join(path.dirname(sample.input), 'descendant-survived');
  fs.writeFileSync(sample.input, `descendant:${marker}`);
  const result = await runIsolatedCoverageDocument(sample.input, mockOptions({
    wallTimeoutMillis: 100,
  }));
  assert.equal(result.failure, 'resource-limit');
  await new Promise(resolve => setTimeout(resolve, 350));
  assert.equal(fs.existsSync(marker), false);
});

test('stdout overflow, signal/exit and sensitive payloads fail closed', async t => {
  for (const mode of ['overflow', 'exit', 'sensitive']) {
    const sample = fixture(mode);
    t.after(sample.cleanup);
    const result = await runIsolatedCoverageDocument(sample.input, mockOptions({
      maxStdoutBytes: 1024,
    }));
    assert.equal(result.status, 'failed', mode);
    assert.equal(result.failure, 'resource-limit', mode);
    assert.equal(JSON.stringify(result).includes(sample.input), false, mode);
    assert.equal(JSON.stringify(result).includes('/home/private'), false, mode);
  }
});

test('known worker failure is mapped without raw diagnostics', async t => {
  const sample = fixture('parser');
  t.after(sample.cleanup);
  const result = await runIsolatedCoverageDocument(sample.input, mockOptions());
  assert.equal(result.status, 'failed');
  assert.equal(result.failure, 'parser');
  assert.equal(JSON.stringify(result).includes(sample.input), false);
});

test('batch boundary continues after failure and retains no per-document path', async t => {
  const failed = fixture('hang');
  const passed = fixture('success');
  t.after(failed.cleanup);
  t.after(passed.cleanup);
  let completed = 0;
  const summary = await runIsolatedCoverageDocuments(
    [failed.input, passed.input],
    {
      ...mockOptions({ wallTimeoutMillis: 50 }),
      onDocumentComplete: aggregate => {
        assert.equal(aggregate.status, 'complete');
        completed += 1;
      },
    },
  );
  assert.equal(summary.documents.attempted, 2);
  assert.equal(summary.documents.success, 1);
  assert.equal(summary.documents.failures['resource-limit'], 1);
  assert.equal(completed, 1);
  assert.equal(JSON.stringify(summary).includes(failed.input), false);
  assert.equal(JSON.stringify(summary).includes(passed.input), false);
});
