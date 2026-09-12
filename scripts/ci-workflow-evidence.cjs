'use strict';

// GitHub run and job endpoints are separate snapshots. Never synthesize success
// from successful steps, and never carry a prior snapshot across a rerun.
const NONTERMINAL = new Set(['queued', 'in_progress', 'waiting', 'pending', 'requested']);
function pending(value) {
  return NONTERMINAL.has(value?.status) && !value?.conclusion;
}
function sameIdentity(left, right) {
  return ['id', 'head_sha', 'head_branch', 'event', 'path', 'workflow_id']
    .every((key) => left[key] === right[key])
    && left.head_repository?.full_name === right.head_repository?.full_name;
}
function sameSnapshot(left, right) {
  return left.run_attempt === right.run_attempt
    && left.status === right.status && left.conclusion === right.conclusion;
}

async function collectWorkflowEvidence({
  run: selected, getRun, listJobs, warn = () => {},
  sleep = (ms) => new Promise((resolve) => setTimeout(resolve, ms)),
  now = Date.now, maxAttempts = 4, delayMs = 5000, maxElapsedMs = 45000,
}) {
  if (!Number.isInteger(maxAttempts) || maxAttempts < 1 || maxAttempts > 4
      || !Number.isFinite(delayMs) || delayMs < 0 || delayMs > 5000
      || !Number.isFinite(maxElapsedMs) || maxElapsedMs < 0 || maxElapsedMs > 45000) {
    throw new Error('invalid evidence retry budget');
  }
  const started = now();
  let last = { run: selected, jobs: [], jobsCollected: false };
  for (let attempt = 1; attempt <= maxAttempts; attempt += 1) {
    let reason = '';
    try {
      const before = await getRun(selected.id);
      if (!sameIdentity(selected, before)) {
        return { ...last, jobs: [], jobsCollected: false, collectionFailure: 'workflow-evidence-identity-mismatch' };
      }
      const jobs = await listJobs(selected.id);
      const after = await getRun(selected.id);
      if (!sameIdentity(selected, after)) {
        return { ...last, jobs: [], jobsCollected: false, collectionFailure: 'workflow-evidence-identity-mismatch' };
      }
      last = { run: after, jobs: [], jobsCollected: false, collectionAttempts: attempt };
      if (!sameSnapshot(before, after)) {
        reason = 'workflow-snapshot-changed';
      } else if (!Number.isSafeInteger(after.run_attempt) || after.run_attempt < 1) {
        return { ...last, collectionFailure: 'invalid-workflow-attempt' };
      } else if (!Array.isArray(jobs) || jobs.some((job) => (
        job.run_id !== after.id || job.head_sha !== after.head_sha
        || !Number.isSafeInteger(job.run_attempt) || job.run_attempt < 1
        || job.run_attempt > after.run_attempt
      ))) {
        return { ...last, collectionFailure: 'job-evidence-identity-mismatch' };
      } else if (jobs.length > 0 && !jobs.some((job) => job.run_attempt === after.run_attempt)) {
        // filter=latest includes unchanged jobs from earlier attempts. At least one
        // job must belong to the current attempt; otherwise rerun evidence is stale.
        reason = 'current-attempt-jobs-unavailable';
      } else {
        last = { ...last, jobs, jobsCollected: true };
        const incomplete = jobs.some((job) => pending(job) || (job.steps || []).some(pending));
        if (after.status !== 'completed' || after.conclusion !== 'success' || !incomplete) {
          return last; // The policy still checks missing/duplicate jobs and real failures.
        }
        reason = 'completed-workflow-nonterminal-evidence';
      }
    } catch (error) {
      last = { ...last, jobs: [], jobsCollected: false, collectionAttempts: attempt };
      reason = 'workflow-evidence-unavailable';
      warn(`Workflow evidence request failed: ${error.message}`);
    }
    last.collectionPendingReason = reason;
    if (attempt === maxAttempts || now() - started + delayMs > maxElapsedMs) break;
    warn(`Recollect workflow ${selected.id}: ${reason} (${attempt}/${maxAttempts}).`);
    await sleep(delayMs);
  }
  warn(`Workflow evidence remains pending: ${last.collectionPendingReason}; rerun this policy audit after API convergence.`);
  return last;
}

module.exports = { collectWorkflowEvidence };
