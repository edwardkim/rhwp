'use strict';
const test = require('node:test');
const assert = require('node:assert/strict');
const { collectWorkflowEvidence } = require('../ci-workflow-evidence.cjs');
const { collectWorkflowRunList } = require('../ci-workflow-evidence.cjs');
const { selectLatestWorkflowRun } = require('../ci-impact-policy.cjs');
const run = { id: 12, run_attempt: 1, status: 'completed', conclusion: 'success', head_sha: 'a'.repeat(40), head_branch: 'topic', event: 'pull_request', path: '.github/workflows/ci.yml', workflow_id: 2, head_repository: { full_name: 'owner/repo' } };
const job = { run_id: 12, run_attempt: 1, head_sha: run.head_sha, status: 'completed', conclusion: 'success', steps: [] };
function fixture(overrides = {}) {
  const calls = { runs: 0, jobs: 0, sleeps: 0 };
  return { calls, options: { run, getRun: async () => { calls.runs++; return structuredClone(run); }, listJobs: async () => { calls.jobs++; return [structuredClone(job)]; }, sleep: async () => { calls.sleeps++; }, ...overrides } };
}

test('stable exact run/job evidence needs no retry', async () => {
  const { calls, options } = fixture(); const result = await collectWorkflowEvidence(options);
  assert.equal(result.jobsCollected, true); assert.equal(calls.runs, 2); assert.equal(calls.sleeps, 0);
});
test('successful steps never promote nonterminal job to success; later snapshot converges', async () => {
  let reads = 0;
  const { calls, options } = fixture({ listJobs: async () => [++reads < 3 ? { ...job, status: 'in_progress', conclusion: null, steps: [{ status: 'completed', conclusion: 'success' }] } : job] });
  const result = await collectWorkflowEvidence(options);
  assert.equal(result.jobs[0].status, 'completed'); assert.equal(calls.sleeps, 2);
});
test('nonterminal evidence exhausts four reads and remains pending', async () => {
  const { calls, options } = fixture({ listJobs: async () => [{ ...job, status: 'in_progress', conclusion: null }] });
  const result = await collectWorkflowEvidence(options);
  assert.equal(result.collectionAttempts, 4); assert.equal(calls.sleeps, 3);
  assert.equal(result.collectionPendingReason, 'completed-workflow-nonterminal-evidence');
  assert.equal(result.jobs[0].status, 'in_progress');
});
test('network error cannot retain earlier success evidence', async () => {
  const { options } = fixture({ listJobs: async () => { throw new Error('unavailable'); } });
  const result = await collectWorkflowEvidence(options); assert.equal(result.jobsCollected, false); assert.deepEqual(result.jobs, []);
});
test('actual terminal failures are returned without retry or rewriting', async () => {
  for (const conclusion of ['failure', 'cancelled', 'timed_out']) {
    const { calls, options } = fixture({ listJobs: async () => [{ ...job, conclusion }] });
    const result = await collectWorkflowEvidence(options); assert.equal(result.jobs[0].conclusion, conclusion); assert.equal(calls.sleeps, 0);
  }
});
test('head/run identity changes cannot be accepted', async () => {
  for (const change of [{ head_sha: 'b'.repeat(40) }, { id: 13 }, { head_repository: { full_name: 'other/repo' } }]) {
    const { options } = fixture({ getRun: async () => ({ ...run, ...change }) });
    assert.equal((await collectWorkflowEvidence(options)).collectionFailure, 'workflow-evidence-identity-mismatch');
  }
});
test('job identity and future attempt are rejected', async () => {
  for (const change of [{ run_id: 13 }, { head_sha: 'b'.repeat(40) }, { run_attempt: 2 }]) {
    const { options } = fixture({ listJobs: async () => [{ ...job, ...change }] });
    assert.equal((await collectWorkflowEvidence(options)).collectionFailure, 'job-evidence-identity-mismatch');
  }
});
test('rerun crossing snapshot boundary discards old jobs, then recollects', async () => {
  let reads = 0; const second = { ...run, run_attempt: 2 };
  const { calls, options } = fixture({ getRun: async () => ++reads === 1 ? run : second, listJobs: async () => [{ ...job, run_attempt: 2 }] });
  const result = await collectWorkflowEvidence(options); assert.equal(result.run.run_attempt, 2); assert.equal(calls.sleeps, 1);
});
test('latest jobs may retain earlier attempt successes alongside current rerun', async () => {
  const { options } = fixture({ getRun: async () => ({ ...run, run_attempt: 2 }), listJobs: async () => [job, { ...job, run_attempt: 2 }] });
  assert.equal((await collectWorkflowEvidence(options)).jobsCollected, true);
});
test('latest jobs missing current attempt remain unavailable', async () => {
  const { options } = fixture({ getRun: async () => ({ ...run, run_attempt: 2 }) });
  const result = await collectWorkflowEvidence(options); assert.equal(result.jobsCollected, false); assert.equal(result.collectionPendingReason, 'current-attempt-jobs-unavailable');
});
test('elapsed retry budget stops additional waits', async () => {
  let clock = 0;
  const { calls, options } = fixture({ now: () => clock, listJobs: async () => { clock += 45000; return [{ ...job, status: 'queued', conclusion: null }]; } });
  const result = await collectWorkflowEvidence(options); assert.equal(calls.sleeps, 0); assert.equal(result.collectionAttempts, 1);
});

const identity = { name: 'CI', path: run.path, pullNumber: 123, baseRef: 'devel',
  baseSha: 'c'.repeat(40), headSha: run.head_sha, headBranch: run.head_branch,
  headRepository: run.head_repository.full_name };
const listedRun = { ...run, name: 'CI', pull_requests: [{ number: 123,
  base: { ref: 'devel', sha: identity.baseSha }, head: { ref: run.head_branch, sha: run.head_sha } }] };
function listFixture(snapshots, overrides = {}) {
  const calls = { lists: 0, sleeps: 0 };
  return { calls, options: { listRuns: async () => {
    const next = snapshots[Math.min(calls.lists++, snapshots.length - 1)];
    if (next instanceof Error) throw next;
    return structuredClone(next);
  }, missingWorkflows: (runs) => selectLatestWorkflowRun(runs, identity) ? [] : ['CI'],
  sleep: async () => { calls.sleeps++; }, ...overrides } };
}

test('missing workflow list converges without a new completion event', async () => {
  const { calls, options } = listFixture([[], [], [listedRun]]);
  const result = await collectWorkflowRunList(options);
  assert.equal(selectLatestWorkflowRun(result, identity).id, run.id);
  assert.equal(calls.lists, 3); assert.equal(calls.sleeps, 2);
});

test('present running or failed workflows do not wait or become successful', async () => {
  for (const change of [{ status: 'in_progress', conclusion: null }, { conclusion: 'failure' }]) {
    const { calls, options } = listFixture([[{ ...listedRun, ...change }]]);
    const result = await collectWorkflowRunList(options);
    assert.equal(result[0].status, change.status || 'completed');
    assert.equal(result[0].conclusion, change.conclusion);
    assert.equal(calls.lists, 1); assert.equal(calls.sleeps, 0);
  }
});

test('persistent missing workflow stays missing after bounded list retries', async () => {
  const { calls, options } = listFixture([[]]);
  assert.deepEqual(await collectWorkflowRunList(options), []);
  assert.equal(calls.lists, 4); assert.equal(calls.sleeps, 3);
});

test('partial listing retries only until every required workflow is selectable', async () => {
  const second = { ...listedRun, id: 13, name: 'CodeQL', path: '.github/workflows/codeql.yml' };
  const secondIdentity = { ...identity, name: second.name, path: second.path };
  const { calls, options } = listFixture([[listedRun], [listedRun, second]], {
    missingWorkflows: (runs) => [identity, secondIdentity]
      .filter((expected) => !selectLatestWorkflowRun(runs, expected)).map((expected) => expected.name),
  });
  const result = await collectWorkflowRunList(options);
  assert.equal(result.length, 2); assert.equal(calls.lists, 2); assert.equal(calls.sleeps, 1);
});

test('a failed newest listing never retains an older partial evidence snapshot', async () => {
  const { options } = listFixture([[{ ...listedRun, name: 'other' }], new Error('unavailable')]);
  assert.deepEqual(await collectWorkflowRunList(options), []);
});

test('list transport errors and malformed payloads retry without retaining old snapshots', async () => {
  for (const missing of [new Error('API unavailable'), null, {}]) {
    const { calls, options } = listFixture([missing, [listedRun]]);
    assert.equal((await collectWorkflowRunList(options))[0].id, run.id);
    assert.equal(calls.lists, 2); assert.equal(calls.sleeps, 1);
  }
  const { calls, options } = listFixture([new Error('API unavailable')]);
  assert.deepEqual(await collectWorkflowRunList(options), []);
  assert.equal(calls.lists, 4);
});

test('foreign run identities remain missing until exact selection converges', async () => {
  for (const change of [{ name: 'other' }, { path: '.github/workflows/other.yml' },
    { head_sha: 'd'.repeat(40) }, { head_branch: 'other' }, { event: 'push' },
    { head_repository: { full_name: 'other/repo' } },
    { pull_requests: [{ ...listedRun.pull_requests[0], number: 124 }] },
    { pull_requests: [{ ...listedRun.pull_requests[0], base: { ref: 'devel', sha: 'd'.repeat(40) } }] }]) {
    const { calls, options } = listFixture([[{ ...listedRun, ...change }], [listedRun]]);
    assert.equal((await collectWorkflowRunList(options))[0].id, run.id);
    assert.equal(calls.sleeps, 1);
  }
});

test('zero retry budget supports publish mode without audit polling', async () => {
  const { calls, options } = listFixture([[], [listedRun]], { maxAttempts: 1 });
  assert.deepEqual(await collectWorkflowRunList(options), []);
  assert.equal(calls.lists, 1); assert.equal(calls.sleeps, 0);
});

test('list retry elapsed budget never starts an additional sleep after exhaustion', async () => {
  let clock = 0;
  const { calls, options } = listFixture([[]], { now: () => clock,
    missingWorkflows: () => { clock += 45000; return ['CI']; } });
  assert.deepEqual(await collectWorkflowRunList(options), []);
  assert.equal(calls.lists, 1); assert.equal(calls.sleeps, 0);
});

test('invalid list retry budgets are rejected before any API request', async () => {
  for (const change of [{ maxAttempts: 0 }, { maxAttempts: 5 }, { delayMs: 5001 },
    { delayMs: -1 }, { maxElapsedMs: 45001 }, { maxElapsedMs: NaN }]) {
    const { calls, options } = listFixture([[]], change);
    await assert.rejects(collectWorkflowRunList(options), /invalid evidence retry budget/);
    assert.equal(calls.lists, 0);
  }
});
