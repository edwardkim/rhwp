import fs from 'node:fs';
import path from 'node:path';
import { createRequire } from 'node:module';
import { decodeDurationArtifact, validateDurationReports } from './trusted-postmerge-duration-evidence.mjs';
const { isAllowedReviewFile } = createRequire(import.meta.url)('./ci-impact-policy.cjs');
const LABELS = ['b', 'c', 'd'];
const SHA = /^[0-9a-f]{40}$/;
const fail = (reason) => { throw new Error(`postmerge-duration: ${reason}`); };
const time = (value) => Date.parse(value || '');

// Metrics provenance is separate from approval/CI reuse. A lint-only rerun does
// not invalidate successful B/C/D measurements from the same head's earlier attempt.
export function selectMeasuredDurationArtifacts({ run, pr, jobs, artifacts, repositoryId }) {
  if (!Number.isSafeInteger(repositoryId) || repositoryId < 1
      || !Number.isSafeInteger(pr?.number) || pr.number < 1
      || !Number.isSafeInteger(run?.id) || run.id < 1 || !Number.isSafeInteger(run.run_attempt) || run.run_attempt < 1
      || run.repository?.id !== repositoryId || pr.base?.repo?.id !== repositoryId
      || run.head_repository?.id !== pr.head?.repo?.id || run.head_branch !== pr.head?.ref
      || run.event !== 'pull_request' || run.status !== 'completed' || run.conclusion !== 'success'
      || run.name !== 'CI' || String(run.path).split('@')[0] !== '.github/workflows/ci.yml'
      || !SHA.test(run.head_sha) || !Number.isFinite(time(pr.merged_at))
      || !Number.isFinite(time(pr.created_at))
      || !Number.isFinite(time(run.run_started_at || run.created_at))
      || time(run.created_at) < time(pr.created_at) || !Number.isFinite(time(run.created_at))
      || time(run.run_started_at || run.created_at) > time(pr.merged_at)) fail('invalid successful PR run');
  if (Array.isArray(run.pull_requests) && run.pull_requests.length
      && !run.pull_requests.some((entry) => entry.number === pr.number)) fail('run belongs to another PR');
  if (!Array.isArray(jobs) || !Array.isArray(artifacts)) fail('missing API evidence');
  const selected = LABELS.map((label) => {
    const names = [`test-archive-${label}-shard-1`, `test-archive-${label}-shard-1 / Default-feature tests (Archive ${label.toUpperCase()})`];
    const matches = jobs.filter((job) => names.includes(job.name));
    if (matches.length !== 1) fail(`missing or duplicate duration worker:${label}`);
    const job = matches[0];
    if (!Number.isSafeInteger(job.id) || job.id < 1 || job.status !== 'completed' || job.conclusion !== 'success'
        || job.run_id !== run.id || job.head_sha !== run.head_sha
        || !Number.isSafeInteger(job.run_attempt) || job.run_attempt < 1 || job.run_attempt > run.run_attempt
        || !Number.isFinite(time(job.started_at)) || !Number.isFinite(time(job.completed_at))
        || time(job.started_at) < time(run.created_at) || time(job.started_at) > time(job.completed_at)
        || time(job.completed_at) > time(pr.merged_at)) fail(`invalid successful duration worker:${label}`);
    const name = `nextest-target-durations-${run.id}-attempt-${job.run_attempt}-${label}`;
    const entries = artifacts.filter((artifact) => artifact.name === name);
    if (entries.length !== 1) fail(`missing or duplicate measurement:${label}`);
    const artifact = entries[0];
    if (!Number.isSafeInteger(artifact.id) || artifact.id < 1 || artifact.expired !== false
        || !Number.isSafeInteger(artifact.size_in_bytes) || artifact.size_in_bytes < 1 || artifact.size_in_bytes > 8 * 1024 * 1024
        || !Number.isFinite(time(artifact.created_at)) || time(artifact.created_at) < time(job.started_at)
        || time(artifact.created_at) > time(job.completed_at)
        || (artifact.workflow_run && (artifact.workflow_run.id !== run.id
          || artifact.workflow_run.repository_id !== repositoryId
          || artifact.workflow_run.head_repository_id !== pr.head.repo.id
          || artifact.workflow_run.head_sha !== run.head_sha))) fail(`invalid measurement artifact:${label}`);
    return { label, artifact, attempt: job.run_attempt, jobId: job.id };
  });
  if (new Set(selected.map((entry) => entry.artifact.id)).size !== 3) fail('duplicate artifact IDs');
  return selected;
}

// GitHub can copy unchanged jobs into a rerun with NEW IDs/run_attempt while
// retaining their original execution times. Resolve those copies using the
// attempt-specific endpoint; never substitute a different successful execution.
export async function resolveMeasuredJobs({ run, jobs, artifacts, listAttemptJobs }) {
  const result = [...jobs];
  const cache = new Map();
  for (const label of LABELS) {
    const matches = jobs.filter(job => job.name === `test-archive-${label}-shard-1`
      || job.name === `test-archive-${label}-shard-1 / Default-feature tests (Archive ${label.toUpperCase()})`);
    if (matches.length !== 1) fail(`missing or duplicate duration worker:${label}`);
    const latest = matches[0];
    if (!Number.isSafeInteger(latest.run_attempt) || latest.run_attempt < 1 || latest.run_attempt > run.run_attempt
        || latest.status !== 'completed' || latest.conclusion !== 'success') fail(`unsuccessful latest worker:${label}`);
    const currentName = `nextest-target-durations-${run.id}-attempt-${latest.run_attempt}-${label}`;
    if (artifacts.some(a => a.name === currentName)) continue;
    const pattern = new RegExp(`^nextest-target-durations-${run.id}-attempt-([1-9][0-9]*)-${label}$`);
    const attempts = [...new Set(artifacts.map(a => Number(pattern.exec(a.name)?.[1]))
      .filter(attempt => Number.isSafeInteger(attempt) && attempt > 0 && attempt < latest.run_attempt))]
      .sort((a, b) => b - a).slice(0, 20);
    let original;
    for (const attempt of attempts) {
      if (!cache.has(attempt)) cache.set(attempt, await listAttemptJobs(attempt));
      const candidates = cache.get(attempt).filter(job => job.run_attempt === attempt
        && ['name', 'run_id', 'head_sha', 'status', 'conclusion', 'started_at', 'completed_at']
          .every(key => job[key] === latest[key]));
      if (candidates.length === 1) { original = candidates[0]; break; }
    }
    if (!original) fail(`original measured worker unavailable:${label}`);
    result[result.indexOf(latest)] = original;
  }
  return result;
}

export function sameCodeReviewTail(comparison) {
  return comparison?.status === 'ahead'
    && comparison.total_commits > 0 && comparison.total_commits <= 250
    && comparison.commits?.length === comparison.total_commits
    && comparison.commits.every((commit) => commit.parents?.length === 1)
    && Array.isArray(comparison.files) && comparison.files.length > 0 && comparison.files.length < 300
    && comparison.files.every(isAllowedReviewFile);
}

export async function collectPostmergeDurations({ github, owner, repo, mergeSha, outputDir, notice = () => {} }) {
  if (!SHA.test(mergeSha)) fail('invalid merge SHA');
  const repository = (await github.rest.repos.get({ owner, repo })).data;
  const associations = await github.paginate(github.rest.repos.listPullRequestsAssociatedWithCommit, {
    owner, repo, commit_sha: mergeSha, per_page: 100,
  });
  const merged = associations.filter((pr) => pr.merged_at && pr.merge_commit_sha === mergeSha
    && pr.base?.ref === 'devel' && pr.base?.repo?.id === repository.id);
  if (merged.length !== 1) return { ready: false, reason: 'not-one-merged-devel-pr' };
  const pr = (await github.rest.pulls.get({ owner, repo, pull_number: merged[0].number })).data;
  if (!pr.merged || pr.merge_commit_sha !== mergeSha || pr.base?.ref !== 'devel'
      || pr.base?.repo?.id !== repository.id || !SHA.test(pr.head?.sha) || !pr.head?.repo?.id) {
    return { ready: false, reason: 'merged-pr-identity-unavailable' };
  }
  const runs = await github.paginate(github.rest.actions.listWorkflowRuns, {
    owner, repo, workflow_id: 'ci.yml', event: 'pull_request', branch: pr.head.ref, per_page: 100,
  });
  const candidates = runs.filter((run) => run.head_repository?.id === pr.head.repo.id
    && run.head_branch === pr.head.ref && run.name === 'CI' && run.status === 'completed'
    && run.conclusion === 'success' && time(run.created_at) >= time(pr.created_at)
    && time(run.run_started_at || run.created_at) <= time(pr.merged_at))
    .sort((a, b) => time(b.created_at) - time(a.created_at) || b.run_attempt - a.run_attempt).slice(0, 20);
  for (const candidate of candidates) {
    try {
      if (candidate.head_sha !== pr.head.sha) {
        const comparison = (await github.rest.repos.compareCommitsWithBasehead({
          owner, repo, basehead: `${candidate.head_sha}...${pr.head.sha}`, per_page: 100,
        })).data;
        if (!sameCodeReviewTail(comparison)) continue;
      }
      const run = (await github.rest.actions.getWorkflowRun({ owner, repo, run_id: candidate.id })).data;
      // Do not reuse an earlier snapshot if someone reran or replaced the run.
      if (run.head_sha !== candidate.head_sha || run.run_attempt !== candidate.run_attempt) continue;
      const jobs = await github.paginate(github.rest.actions.listJobsForWorkflowRun, {
        owner, repo, run_id: run.id, filter: 'latest', per_page: 100,
      });
      const artifacts = await github.paginate(github.rest.actions.listWorkflowRunArtifacts, {
        owner, repo, run_id: run.id, per_page: 100,
      });
      const measuredJobs = await resolveMeasuredJobs({ run, jobs, artifacts,
        listAttemptJobs: (attempt_number) => github.paginate(github.rest.actions.listJobsForWorkflowRunAttempt, {
          owner, repo, run_id: run.id, attempt_number, per_page: 100,
        }),
      });
      const selected = selectMeasuredDurationArtifacts({ run, pr, jobs: measuredJobs, artifacts, repositoryId: repository.id });
      const reports = [];
      for (const { label, artifact } of selected) {
        const response = await github.rest.actions.downloadArtifact({
          owner, repo, artifact_id: artifact.id, archive_format: 'zip', request: { timeout: 15000 },
        });
        reports.push(decodeDurationArtifact(response.data, label));
      }
      const testedMergeSha = reports[0]?.sha;
      if (!SHA.test(testedMergeSha)) fail('invalid tested merge SHA');
      const tested = (await github.rest.repos.getCommit({ owner, repo, ref: testedMergeSha })).data;
      if (tested.parents?.length !== 2 || tested.parents[1].sha !== run.head_sha) fail('tested merge/head mismatch');
      const measurementAttempts = Object.fromEntries(selected.map(({ label, attempt }) => [label, attempt]));
      const normalized = validateDurationReports(reports, {
        run, pr, repositoryId: repository.id, testedMergeSha, measurementAttempts,
      });
      const final = (await github.rest.actions.getWorkflowRun({ owner, repo, run_id: run.id })).data;
      if (final.status !== 'completed' || final.conclusion !== 'success'
          || final.head_sha !== run.head_sha || final.run_attempt !== run.run_attempt) fail('run changed during collection');
      for (const report of normalized) {
        const directory = path.join(outputDir, `duration-${report.archive_label}`);
        fs.mkdirSync(directory, { recursive: true });
        fs.writeFileSync(path.join(directory, `target-durations-${report.archive_label}.json`), `${JSON.stringify(report)}\n`);
      }
      return { ready: true, reason: 'successful-pr-worker-measurements', pullNumber: pr.number,
        sourceRunId: run.id, latestRunAttempt: run.run_attempt, measurementAttempts, testedMergeSha,
        sourceHeadSha: run.head_sha, artifactIds: selected.map(({ artifact }) => artifact.id) };
    } catch (error) {
      notice(`Duration candidate ${candidate.id} not used: ${error.message}`);
    }
  }
  return { ready: false, reason: 'no-verified-pr-duration-measurements' };
}
