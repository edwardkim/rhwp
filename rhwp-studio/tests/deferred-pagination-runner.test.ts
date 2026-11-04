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
// npm bin 을 직접 spawn 하지 않는다: Windows 의 `.bin/tsc` 는 확장자 없는 shell 스크립트라
// 실행되지 않고, `.cmd` 는 Node 가 shell 없는 spawn 을 막는다(CVE-2024-27980 완화). 어느
// 쪽이든 status=null 로 죽어 테스트가 통째로 중단된다. typescript 의 JS 진입점을 현재
// node 로 실행하면 셸 없이도 모든 OS 에서 동일하게 동작한다.
const compiler = process.env.RHWP_STUDIO_TSC ?? process.execPath;
const compilerArgs = process.env.RHWP_STUDIO_TSC
  ? []
  : [path.join(studioRoot, 'node_modules', 'typescript', 'bin', 'tsc')];
const compilation = spawnSync(compiler, [
  ...compilerArgs,
  '--ignoreConfig',
  'src/engine/deferred-pagination-runner.ts',
  '--target', 'ES2022',
  '--module', 'commonjs',
  '--rootDir', '..',
  '--outDir', runtimeRoot,
  '--listEmittedFiles',
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

// Type-only imports also enter the compiler program. Shared sources must stay
// inside the owned temporary tree rather than emit beside workspace inputs.
for (const line of compilation.stdout.split(/\r?\n/).filter(line => line.startsWith('TSFILE: '))) {
  const relative = path.relative(runtimeRoot, line.slice(8).trim());
  assert.ok(relative !== '..' && !relative.startsWith(`..${path.sep}`) && !path.isAbsolute(relative),
    `test compiler escaped temporary output: ${line}`);
}

const require = createRequire(import.meta.url);
const { DeferredPaginationRunner } = require(
  path.join(runtimeRoot, 'rhwp-studio', 'src', 'engine', 'deferred-pagination-runner.js'),
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

  schedule(callback, delayMs = 0) {
    const id = this.nextId++;
    this.tasks.set(id, { callback, delayMs });
    return id;
  }

  cancel(id) {
    this.tasks.delete(id);
  }

  runOne() {
    const entry = this.tasks.entries().next().value;
    assert.ok(entry, 'scheduled continuation task');
    const [id, task] = entry;
    this.tasks.delete(id);
    task.callback();
  }

  first() {
    const entry = this.tasks.entries().next().value;
    assert.ok(entry, 'scheduled continuation task');
    return entry[1];
  }

  firstId() {
    const entry = this.tasks.entries().next().value;
    assert.ok(entry, 'scheduled continuation task');
    return entry[0];
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

test('최초 begin은 input stack 밖에서 실행하고 이후 한 macrotask당 한 budget만 처리한다', () => {
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
    (callback, delayMs) => tasks.schedule(callback, delayMs),
    (task) => tasks.cancel(task),
  );

  runner.requestStart(200, 100, 25);
  assert.equal(runner.isActive(), false);
  assert.equal(runner.hasPendingWork(), true);
  assert.deepEqual(client.calls, [['cancel']], 'begin must not run in the input call stack');
  assert.equal(tasks.first().delayMs, 100, 'first admission has a fixed paint timer target');

  tasks.runOne();
  assert.deepEqual(client.calls.at(-1), ['begin', 1, 1]);
  assert.equal(runner.isActive(), true);
  assert.equal(tasks.first().delayMs, 0, 'first fragment step starts on the regular task cadence');

  tasks.runOne();
  assert.deepEqual(client.calls.at(-1), ['step', 1]);
  assert.equal(runner.isActive(), true);
  assert.equal(completed.length, 0);
  assert.equal(tasks.first().delayMs, 25, 'the first fragment yields through the settle gap');

  tasks.runOne();
  assert.equal(runner.isActive(), true);
  assert.equal(tasks.first().delayMs, 0, 'later fragment steps keep the regular task cadence');
  tasks.runOne();
  assert.equal(runner.isActive(), false);
  assert.equal(completed.length, 1);
  assert.equal(completed[0].status, 'complete');
  assert.equal(fallbacks.length, 0);
});

test('공개 페이지 수는 async begin과 pending step 동안 유지되고 complete에서 한 번 교체된다', () => {
  const tasks = new ManualTasks();
  const client = new FakeClient([
    result('pending', 1, 1, 115),
    result('complete', 1, 1, 116),
  ]);
  let publicPageCount = 115;
  const published = [];
  const runner = new DeferredPaginationRunner(
    client,
    (value) => {
      publicPageCount = value.pageCount;
      published.push(value.pageCount);
    },
    () => assert.fail('fallback must not run'),
    1,
    (callback, delayMs) => tasks.schedule(callback, delayMs),
    (task) => tasks.cancel(task),
  );

  runner.requestStart(200, 100, 20);
  assert.equal(publicPageCount, 115);
  assert.deepEqual(published, []);

  tasks.runOne();
  assert.equal(publicPageCount, 115, 'begin result must stay private');
  assert.deepEqual(published, []);

  tasks.runOne();
  assert.equal(publicPageCount, 115, 'intermediate pending result must stay private');
  assert.deepEqual(published, []);

  tasks.runOne();
  assert.equal(publicPageCount, 116);
  assert.deepEqual(published, [116], 'final page count must publish once');
});

test('begin 전 반복 요청은 최초 timer target을 유지하고 최신 revision begin 하나만 남긴다', () => {
  const tasks = new ManualTasks();
  const client = new FakeClient([result('complete', 2, 1)]);
  const completed = [];
  const runner = new DeferredPaginationRunner(
    client,
    (value) => completed.push(value),
    () => assert.fail('fallback must not run'),
    1,
    (callback, delayMs) => tasks.schedule(callback, delayMs),
    (task) => tasks.cancel(task),
  );

  runner.requestStart(200, 100, 20);
  assert.equal(tasks.tasks.size, 1);
  assert.equal(tasks.first().delayMs, 100);
  const initialTaskId = tasks.firstId();
  client.beginRevision = 2;
  runner.requestStart(200, 100, 20);
  assert.equal(tasks.tasks.size, 1, 'initial timer must keep one scheduled begin');
  assert.equal(tasks.firstId(), initialTaskId, 'repeated input must not move the initial timer');
  assert.equal(tasks.first().delayMs, 100, 'unproven admission keeps the fixed initial timer target');
  assert.equal(client.calls.some(([name]) => name === 'begin'), false);
  assert.deepEqual(
    client.calls.filter(([name]) => name === 'cancel'),
    [['cancel']],
  );

  tasks.runOne();
  assert.deepEqual(client.calls.at(-1), ['begin', 1, 2]);
  tasks.runOne();
  assert.equal(completed.length, 1);
  assert.equal(completed[0].revision, 2);
});

test('active restart와 그 후 요청은 old step을 버리고 coalescing window를 다시 시작한다', () => {
  const tasks = new ManualTasks();
  const client = new FakeClient([result('complete', 3, 1)]);
  const completed = [];
  const runner = new DeferredPaginationRunner(
    client,
    (value) => completed.push(value),
    () => assert.fail('fallback must not run'),
    1,
    (callback, delayMs) => tasks.schedule(callback, delayMs),
    (task) => tasks.cancel(task),
  );

  runner.requestStart(200, 100, 20);
  tasks.runOne();
  assert.equal(runner.isActive(), true);
  const staleStep = tasks.first().callback;

  client.beginRevision = 2;
  runner.requestStart(200, 100, 20);
  assert.equal(runner.isActive(), false);
  assert.equal(tasks.tasks.size, 1);
  assert.equal(tasks.first().delayMs, 200);
  const staleRestartBegin = tasks.first().callback;
  const firstRestartTaskId = tasks.firstId();
  staleStep();
  assert.equal(
    client.calls.some(([name]) => name === 'step'),
    false,
    'a superseded callback must not enter the old core job',
  );

  client.beginRevision = 3;
  runner.requestStart(200, 100, 20);
  assert.equal(tasks.tasks.size, 1);
  assert.notEqual(tasks.firstId(), firstRestartTaskId, 'restart window must move to the latest input');
  assert.equal(tasks.first().delayMs, 200, 'latest input must restart the coalescing window');
  staleRestartBegin();
  assert.equal(
    client.calls.some(([name, , revision]) => name === 'begin' && revision === 2),
    false,
    'a superseded restart callback must not begin its revision',
  );
  tasks.runOne();
  assert.deepEqual(client.calls.at(-1), ['begin', 1, 3]);
  tasks.runOne();
  assert.equal(completed.length, 1);
  assert.equal(completed[0].revision, 3);
});

test('post-first settle task는 restart 뒤 core step을 실행할 수 없다', () => {
  const tasks = new ManualTasks();
  const client = new FakeClient([
    result('pending', 1, 1),
    result('complete', 2, 1),
  ]);
  const completed = [];
  const runner = new DeferredPaginationRunner(
    client,
    (value) => completed.push(value),
    () => assert.fail('fallback must not run'),
    1,
    (callback, delayMs) => tasks.schedule(callback, delayMs),
    (task) => tasks.cancel(task),
  );

  runner.requestStart(200, 100, 25);
  tasks.runOne();
  tasks.runOne();
  assert.equal(tasks.first().delayMs, 25);
  const staleSettledStep = tasks.first().callback;

  client.beginRevision = 2;
  runner.requestStart(200, 100, 25);
  assert.equal(tasks.first().delayMs, 200);
  staleSettledStep();
  assert.equal(
    client.calls.filter(([name]) => name === 'step').length,
    1,
    'superseded settle callback must not enter the old core job',
  );

  tasks.runOne();
  tasks.runOne();
  assert.equal(completed.length, 1);
  assert.equal(completed[0].revision, 2);
});

test('cancel은 queued begin과 이미 dequeue된 stale callback을 모두 무효화한다', () => {
  const tasks = new ManualTasks();
  const client = new FakeClient([]);
  const completed = [];
  const fallbacks = [];
  const runner = new DeferredPaginationRunner(
    client,
    (value) => completed.push(value),
    (value) => fallbacks.push(value),
    1,
    (callback, delayMs) => tasks.schedule(callback, delayMs),
    (task) => tasks.cancel(task),
  );

  runner.requestStart(200, 100);
  const staleBegin = tasks.first().callback;
  runner.cancel();
  assert.equal(runner.hasPendingWork(), false);
  assert.equal(tasks.tasks.size, 0);
  staleBegin();
  assert.equal(client.calls.some(([name]) => name === 'begin'), false);
  assert.equal(completed.length, 0);
  assert.equal(fallbacks.length, 0);
});

test('unsupported async begin은 step 없이 fallback callback으로 전달한다', () => {
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
    (callback, delayMs) => tasks.schedule(callback, delayMs),
    (task) => tasks.cancel(task),
  );

  runner.requestStart(200, 100);
  assert.equal(fallbacks.length, 0, 'fallback must not run in the input call stack');
  tasks.runOne();
  assert.equal(runner.isActive(), false);
  assert.equal(runner.hasPendingWork(), false);
  assert.equal(tasks.tasks.size, 0);
  assert.equal(fallbacks.length, 1);
  assert.equal(fallbacks[0].revision, 7);
});

test('begin과 step 예외는 각각 fallback을 정확히 한 번 게시한다', () => {
  for (const failure of ['begin', 'step']) {
    const tasks = new ManualTasks();
    const client = new FakeClient([]);
    if (failure === 'begin') {
      client.beginDeferredPagination = () => {
        throw new Error('begin failed');
      };
    } else {
      client.stepDeferredPagination = () => {
        throw new Error('step failed');
      };
    }
    const fallbacks = [];
    const runner = new DeferredPaginationRunner(
      client,
      () => assert.fail('complete must not run'),
      (value) => fallbacks.push(value),
      1,
      (callback, delayMs) => tasks.schedule(callback, delayMs),
      (task) => tasks.cancel(task),
    );

    runner.requestStart(200, 100);
    tasks.runOne();
    if (failure === 'step') tasks.runOne();
    assert.equal(fallbacks.length, 1, `${failure} fallback count`);
    assert.equal(fallbacks[0].status, 'fallback');
    assert.equal(runner.hasPendingWork(), false);
  }
});
