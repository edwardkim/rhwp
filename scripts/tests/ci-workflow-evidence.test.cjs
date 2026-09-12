'use strict';
const test = require('node:test');
const assert = require('node:assert/strict');
const { collectWorkflowEvidence } = require('../ci-workflow-evidence.cjs');
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
