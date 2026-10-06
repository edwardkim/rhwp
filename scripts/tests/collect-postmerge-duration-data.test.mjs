import assert from 'node:assert/strict';
import test from 'node:test';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { collectPostmergeDurations, selectMeasuredDurationArtifacts, resolveMeasuredJobs, sameCodeReviewTail } from '../collect-postmerge-duration-data.mjs';
import { validateDurationReports } from '../trusted-postmerge-duration-evidence.mjs';
import { durationFixture, reportZip } from './helpers/postmerge-duration-fixtures.mjs';

function fixture(attempts = [1, 1, 1]) {
  const f = durationFixture({ attempt: 2 });
  Object.assign(f.pr, { created_at: '2026-09-06T10:00:00Z', merged: true, merge_commit_sha: '4'.repeat(40) });
  f.pr.head.sha = f.run.head_sha;
  Object.assign(f.run, { name: 'CI', path: '.github/workflows/ci.yml', head_branch: f.pr.head.ref, head_repository: f.pr.head.repo });
  f.jobs = f.reports.map((report, i) => ({ id: 200 + i, name: `test-archive-${report.archive_label}-shard-1`,
    run_id: f.run.id, head_sha: f.run.head_sha, run_attempt: attempts[i], status: 'completed', conclusion: 'success',
    started_at: '2026-09-06T11:05:00Z', completed_at: '2026-09-06T11:25:00Z' }));
  f.reports.forEach((r, i) => { r.run_attempt = String(attempts[i]); f.artifacts[i].name = `nextest-target-durations-123-attempt-${attempts[i]}-${r.archive_label}`; });
  return f;
}
for (const attempts of [[1, 1, 1], [1, 2, 1], [2, 2, 2]]) {
  test(`partial rerun uses actual successful worker attempts ${attempts}`, () => {
    const f = fixture(attempts);
    const selected = selectMeasuredDurationArtifacts(f);
    assert.deepEqual(selected.map(s => s.attempt), attempts);
    const measurementAttempts = Object.fromEntries(selected.map(s => [s.label, s.attempt]));
    assert.equal(validateDurationReports(f.reports, { ...f.context, measurementAttempts }).length, 3);
    if (attempts.includes(1)) assert.throws(() => validateDurationReports(f.reports, f.context));
  });
}
for (const [name, mutate] of Object.entries({
  'missing worker': f => f.jobs.pop(),
  'duplicate worker': f => f.jobs.push(f.jobs[0]),
  'failed worker': f => { f.jobs[0].conclusion = 'failure'; },
  'pending worker': f => { f.jobs[0].status = 'in_progress'; },
  'skipped worker': f => { f.jobs[0].conclusion = 'skipped'; },
  'different head': f => { f.jobs[0].head_sha = 'f'.repeat(40); },
  'different run': f => { f.jobs[0].run_id++; },
  'future attempt': f => { f.jobs[0].run_attempt = 3; },
  'missing attempt': f => { delete f.jobs[0].run_attempt; },
  'post merge worker': f => { f.jobs[0].completed_at = '2026-09-07T00:00:00Z'; },
  'invalid job times': f => { f.jobs[0].started_at = '2026-09-06T11:26:00Z'; },
  'missing artifact': f => f.artifacts.pop(),
  'duplicate artifact': f => f.artifacts.push(f.artifacts[0]),
  'expired': f => { f.artifacts[0].expired = true; },
  'too large': f => { f.artifacts[0].size_in_bytes = 9 * 1024 * 1024; },
  'old artifact': f => { f.artifacts[0].created_at = '2026-09-06T10:00:00Z'; },
  'wrong repository': f => { f.artifacts[0].workflow_run.repository_id++; },
  'wrong fork': f => { f.artifacts[0].workflow_run.head_repository_id++; },
  'wrong artifact head': f => { f.artifacts[0].workflow_run.head_sha = 'f'.repeat(40); },
  'wrong workflow': f => { f.run.path = '.github/workflows/other.yml'; },
  'different PR': f => { f.run.pull_requests = [{ number: 100 }]; },
  'failed run': f => { f.run.conclusion = 'failure'; },
  'invalid creation time': f => { delete f.pr.created_at; },
})) test(`rejects ${name}`, () => { const f = fixture(); mutate(f); assert.throws(() => selectMeasuredDurationArtifacts(f)); });

for (const measurementAttempts of [{}, { b: 1, c: 1 }, { b: 1, c: 1, d: 3 }, { b: 1, c: 1, d: 1, e: 1 }, { b: '1', c: 1, d: 1 }]) {
  test(`rejects incomplete or invalid attempt map ${JSON.stringify(measurementAttempts)}`, () => {
    const f = fixture(); assert.throws(() => validateDurationReports(f.reports, { ...f.context, measurementAttempts }));
  });
}
test('review tail requires a complete linear comparison of allowed files', () => {
  const c = { status: 'ahead', total_commits: 1, commits: [{ parents: [{}] }], files: [{ filename: 'mydocs/report/task.md', status: 'modified' }] };
  assert.equal(sameCodeReviewTail(c), true);
  for (const change of [{ status: 'diverged' }, { total_commits: 2 }, { commits: [{ parents: [{}, {}] }] },
    { files: [{ filename: 'src/lib.rs', status: 'modified' }] }, { files: [] }]) assert.equal(sameCodeReviewTail({ ...c, ...change }), false);
});

function client(f, options = {}) {
  let reads = 0;
  const get = data => async () => ({ data });
  const actions = {
    listWorkflowRuns: async () => [f.run], listJobsForWorkflowRun: async () => f.jobs,
    listWorkflowRunArtifacts: async () => f.artifacts,
    getWorkflowRun: async () => ({ data: options.changed && ++reads > 1 ? { ...f.run, run_attempt: 3 } : f.run }),
    downloadArtifact: async ({ artifact_id }) => ({ data: reportZip(f.reports[f.artifacts.findIndex(a => a.id === artifact_id)]) }),
  };
  return { rest: { actions, pulls: { get: get(f.pr) }, repos: { get: get({ id: f.repositoryId }),
    listPullRequestsAssociatedWithCommit: async () => options.noPr ? [] : [f.pr],
    getCommit: get({ parents: [{ sha: 'a'.repeat(40) }, { sha: options.wrongParent ? 'f'.repeat(40) : f.run.head_sha }] }),
    compareCommitsWithBasehead: get({ status: 'ahead', total_commits: 1, commits: [{ parents: [{}] }],
      files: [{ filename: options.codeTail ? 'src/lib.rs' : 'mydocs/report/task.md', status: 'modified' }] }),
  } }, paginate: async (method, args) => method(args) };
}
for (const options of [{}, { changed: true }, { noPr: true }, { wrongParent: true }, { reviewTail: true }, { codeTail: true }, { missing: true }]) {
  test(`collector publishes only verified measurements ${JSON.stringify(options)}`, async () => {
    const f = fixture();
    if (options.reviewTail || options.codeTail) f.pr.head.sha = '5'.repeat(40);
    if (options.missing) f.artifacts.pop();
    const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'rhwp-duration-test-'));
    try {
      const result = await collectPostmergeDurations({ github: client(f, options), owner: 'edwardkim', repo: 'rhwp', mergeSha: f.pr.merge_commit_sha, outputDir: dir });
      const expected = Object.keys(options).length === 0 || options.reviewTail === true;
      assert.equal(result.ready, expected);
      if (expected) {
        assert.deepEqual(result.measurementAttempts, { b: 1, c: 1, d: 1 });
        const saved = JSON.parse(fs.readFileSync(path.join(dir, 'duration-b/target-durations-b.json')));
        assert.equal(saved.run_attempt, '1');
      } else assert.deepEqual(fs.readdirSync(dir), []);
    } finally { fs.rmSync(dir, { recursive: true }); }
  });
}

for (const changed of [false, true]) {
  test(`copied rerun jobs require identical original execution times changed=${changed}`, async () => {
    const f = fixture();
    const copies = f.jobs.map(job => ({ ...job, id: job.id + 1000, run_attempt: 2,
      completed_at: changed ? '2026-09-06T11:26:00Z' : job.completed_at }));
    const args = { run: f.run, jobs: copies, artifacts: f.artifacts, listAttemptJobs: async attempt => {
      assert.equal(attempt, 1); return f.jobs;
    } };
    if (changed) await assert.rejects(resolveMeasuredJobs(args), /original measured worker unavailable/);
    else {
      const jobs = await resolveMeasuredJobs(args);
      assert.deepEqual(selectMeasuredDurationArtifacts({ ...f, jobs }).map(s => s.attempt), [1, 1, 1]);
    }
  });
}

test('#7068 real API copied job IDs/attempts resolve to original B/C/D artifacts', async () => {
  const f = JSON.parse(fs.readFileSync(new URL('./fixtures/ci-impact-policy/issue7070-copied-worker-attempts.json', import.meta.url)));
  assert.ok(f.jobs.every(job => job.run_attempt === 2));
  assert.ok(f.originalJobs.every(job => job.run_attempt === 1));
  const jobs = await resolveMeasuredJobs({ ...f, listAttemptJobs: async attempt => {
    assert.equal(attempt, 1); return f.originalJobs;
  } });
  const selected = selectMeasuredDurationArtifacts({ ...f, jobs });
  assert.deepEqual(selected.map(s => s.artifact.id), [10301251247, 10301655406, 10301346099]);
});
