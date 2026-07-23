import test, { after } from 'node:test';
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { createRequire } from 'node:module';
import { mkdtempSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const studioRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const runtimeRoot = mkdtempSync(path.join(tmpdir(), 'rhwp-deferred-pagination-runner-'));
const compiler = process.env.RHWP_STUDIO_TSC
  ?? path.join(studioRoot, 'node_modules', '.bin', 'tsc');
const compilation = spawnSync(compiler, [
  '--ignoreConfig',
  'src/engine/deferred-pagination-runner.ts',
  '--target', 'ES2022',
  '--module', 'commonjs',
  '--rootDir', 'src',
  '--outDir', runtimeRoot,
  '--skipLibCheck',
  '--noCheck',
], {
  cwd: studioRoot,
  encoding: 'utf8',
});

assert.equal(
  compilation.status,
  0,
  `deferred pagination runner compile failed:\n${compilation.stdout}${compilation.stderr}`,
);

const require = createRequire(import.meta.url);
const { DeferredPaginationRunner } = require(
  path.join(runtimeRoot, 'engine', 'deferred-pagination-runner.js'),
);

after(() => {
  rmSync(runtimeRoot, { recursive: true, force: true });
});

function result(status, revision, fragmentsProcessed = 0, pageCount = 115) {
  return { ok: true, status, revision, fragmentsProcessed, pageCount };
}

class ManualTasks {
  constructor() {
    this.nextId = 1;
    this.tasks = new Map();
  }

  schedule(callback) {
    const id = this.nextId++;
    this.tasks.set(id, callback);
    return id;
  }

  cancel(id) {
    this.tasks.delete(id);
  }

  runOne() {
    const entry = this.tasks.entries().next().value;
    assert.ok(entry, 'scheduled continuation task');
    const [id, callback] = entry;
    this.tasks.delete(id);
    callback();
  }
}

class FakeClient {
  constructor(stepResults) {
    this.stepResults = [...stepResults];
    this.beginRevision = 1;
    this.calls = [];
  }

  beginDeferredPagination(budget) {
    this.calls.push(['begin', budget, this.beginRevision]);
    return result('pending', this.beginRevision);
  }

  stepDeferredPagination(budget) {
    this.calls.push(['step', budget]);
    const next = this.stepResults.shift();
    assert.ok(next, 'step fixture exhausted');
    return next;
  }

  cancelDeferredPagination() {
    this.calls.push(['cancel']);
    return true;
  }
}

test('한 macrotask당 한 budget만 처리하고 complete에서 한 번 commit callback을 호출한다', () => {
  const tasks = new ManualTasks();
  const client = new FakeClient([
    result('pending', 1, 1),
    result('pending', 1, 1),
    result('complete', 1, 1),
  ]);
  const completed = [];
  const fallbacks = [];
  const runner = new DeferredPaginationRunner(
    client,
    (value) => completed.push(value),
    (value) => fallbacks.push(value),
    1,
    (callback) => tasks.schedule(callback),
    (task) => tasks.cancel(task),
  );

  assert.equal(runner.start().status, 'pending');
  assert.equal(runner.isActive(), true);
  assert.deepEqual(client.calls, [['cancel'], ['begin', 1, 1]]);

  tasks.runOne();
  assert.deepEqual(client.calls.at(-1), ['step', 1]);
  assert.equal(runner.isActive(), true);
  assert.equal(completed.length, 0);

  tasks.runOne();
  assert.equal(runner.isActive(), true);
  tasks.runOne();
  assert.equal(runner.isActive(), false);
  assert.equal(completed.length, 1);
  assert.equal(completed[0].status, 'complete');
  assert.equal(fallbacks.length, 0);
});

test('새 입력 start는 예약 step과 이전 core job을 취소하고 최신 revision으로 교체한다', () => {
  const tasks = new ManualTasks();
  const client = new FakeClient([result('complete', 2, 1)]);
  const completed = [];
  const runner = new DeferredPaginationRunner(
    client,
    (value) => completed.push(value),
    () => assert.fail('fallback must not run'),
    1,
    (callback) => tasks.schedule(callback),
    (task) => tasks.cancel(task),
  );

  runner.start();
  assert.equal(tasks.tasks.size, 1);
  client.beginRevision = 2;
  runner.start();
  assert.equal(tasks.tasks.size, 1, 'old scheduled step must be replaced');
  assert.deepEqual(
    client.calls.filter(([name]) => name === 'cancel'),
    [['cancel'], ['cancel']],
  );

  tasks.runOne();
  assert.equal(completed.length, 1);
  assert.equal(completed[0].revision, 2);
});

test('unsupported begin은 step을 예약하지 않고 fallback callback으로 전달한다', () => {
  const tasks = new ManualTasks();
  const client = new FakeClient([]);
  client.beginDeferredPagination = (budget) => {
    client.calls.push(['begin', budget, 7]);
    return result('fallback', 7);
  };
  const fallbacks = [];
  const runner = new DeferredPaginationRunner(
    client,
    () => assert.fail('complete must not run'),
    (value) => fallbacks.push(value),
    1,
    (callback) => tasks.schedule(callback),
    (task) => tasks.cancel(task),
  );

  assert.equal(runner.start().status, 'fallback');
  assert.equal(runner.isActive(), false);
  assert.equal(tasks.tasks.size, 0);
  assert.equal(fallbacks.length, 1);
  assert.equal(fallbacks[0].revision, 7);
});
