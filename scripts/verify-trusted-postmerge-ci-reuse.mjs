#!/usr/bin/env node

const SHA = /^[0-9a-f]{40}$/;

function denied(reason) {
  return { reuse: false, reason, sourceRunId: "", pullNumber: "" };
}

function validSha(value) {
  return typeof value === "string" && SHA.test(value);
}

function timestamp(value) {
  const parsed = Date.parse(value || "");
  return Number.isFinite(parsed) ? parsed : Number.NaN;
}

function enforcementPathChanged(files) {
  const paths = (Array.isArray(files) ? files : [])
    .flatMap((file) => [file?.filename, file?.previous_filename])
    .filter((path) => typeof path === "string" && path.length > 0);
  return paths.some((path) => (
    path.startsWith(".github/workflows/")
    || path.startsWith(".github/actions/")
    || path === ".config/nextest.toml"
    || path === "scripts/ci-impact-classifier.cjs"
    || path === "scripts/ci-impact-policy.cjs"
    || path === "scripts/select-nextest-archive-targets.mjs"
    || path === "scripts/collect-nextest-target-durations.mjs"
    || path === "scripts/refresh-nextest-target-duration-policy.mjs"
    || path === "scripts/verify-trusted-postmerge-ci-reuse.mjs"
    || path === "tests/suites/nextest-target-duration-policy.json"
  ));
}

function latestCandidateRun(runs, pullRequest, repository) {
  const createdAt = timestamp(pullRequest.created_at);
  const mergedAt = timestamp(pullRequest.merged_at);
  if (!Number.isFinite(createdAt) || !Number.isFinite(mergedAt) || mergedAt < createdAt) {
    return null;
  }

  const matches = (Array.isArray(runs) ? runs : []).filter((run) => (
    run?.event === "pull_request"
    && run?.head_sha === pullRequest.head?.sha
    && run?.head_branch === pullRequest.head?.ref
    && run?.head_repository?.full_name === repository
    && timestamp(run.created_at) >= createdAt
    && timestamp(run.updated_at) <= mergedAt
  ));
  if (matches.length === 0) {
    return null;
  }
  return matches.sort((left, right) => (
    timestamp(right.updated_at) - timestamp(left.updated_at)
    || Number(right.run_attempt || 0) - Number(left.run_attempt || 0)
    || Number(right.id || 0) - Number(left.id || 0)
  ))[0];
}

export function evaluateTrustedPostMergeReuse(input) {
  if (input?.eventName !== "push" || input?.ref !== "refs/heads/devel") {
    return denied("not-a-devel-push");
  }
  if (!validSha(input?.mergeSha) || input.mergeCommit?.sha !== input.mergeSha) {
    return denied("merge-commit-unavailable");
  }
  const parents = Array.isArray(input.mergeCommit.parents)
    ? input.mergeCommit.parents.map((parent) => parent?.sha || parent).filter(validSha)
    : [];
  if (
    (parents.length !== 1 && parents.length !== 2)
    || new Set(parents).size !== parents.length
  ) {
    return denied("merge-commit-must-have-one-or-two-parents");
  }

  const pullRequests = (Array.isArray(input.pullRequests) ? input.pullRequests : []).filter((pullRequest) => (
    pullRequest?.state === "closed"
    && typeof pullRequest.merged_at === "string"
    && pullRequest.merge_commit_sha === input.mergeSha
    && pullRequest.base?.ref === "devel"
    && pullRequest.head?.repo?.full_name === input.repository
    && validSha(pullRequest.head?.sha)
  ));
  if (pullRequests.length !== 1) {
    return denied("merge-commit-must-map-to-one-merged-same-repository-pr");
  }
  const pullRequest = pullRequests[0];
  if (parents.length === 2 && !parents.includes(pullRequest.head.sha)) {
    return denied("merge-parent-does-not-match-pr-head");
  }
  const baseParent = parents.length === 1
    ? parents[0]
    : parents.find((parent) => parent !== pullRequest.head.sha);
  if (!baseParent || input.mergeBaseSha !== baseParent) {
    return denied("pr-head-does-not-contain-merge-base");
  }
  if (input.sourceCommit?.sha !== pullRequest.head.sha) {
    return denied("source-commit-does-not-match-pr-head");
  }
  if (
    input.mergeCommit?.commit?.tree?.sha !== input.sourceCommit?.commit?.tree?.sha
    || !input.mergeCommit?.commit?.tree?.sha
  ) {
    return denied("merge-tree-does-not-match-pr-head");
  }
  if (enforcementPathChanged(input.pullFiles)) {
    return denied("pr-changes-ci-enforcement-surface");
  }

  const candidate = latestCandidateRun(input.workflowRuns, pullRequest, input.repository);
  if (!candidate) {
    return denied("no-current-pr-workflow-candidate");
  }
  if (candidate.status !== "completed" || candidate.conclusion !== "success") {
    return denied("latest-pr-workflow-candidate-not-successful");
  }
  if (!Number.isInteger(candidate.id) && !/^[1-9][0-9]*$/.test(String(candidate.id || ""))) {
    return denied("candidate-run-id-unavailable");
  }
  return {
    reuse: true,
    reason: "exact-green-pr-workflow-reused",
    sourceRunId: String(candidate.id),
    pullNumber: String(pullRequest.number),
  };
}
