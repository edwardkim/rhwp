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

function allowedReviewOnlyFile(file) {
  if (!file || typeof file.filename !== "string") {
    return false;
  }
  if (file.filename.startsWith("mydocs/")) {
    return true;
  }

  const filename = file.filename;
  const lowerName = filename.toLowerCase();
  const sampleReference = filename.startsWith("samples/")
    && [".hwp", ".hwpx", ".pdf", ".png"].some((extension) => lowerName.endsWith(extension));
  const pdfReference = ["pdf/", "pdf-2020/", "pdf-large/"]
    .some((prefix) => filename.startsWith(prefix))
    && lowerName.endsWith(".pdf");
  if (pdfReference) {
    return file.status === "added" || file.status === "modified";
  }
  return file.status === "added" && sampleReference;
}

export function classifyReviewOnlyCommit(commit) {
  if (!validSha(commit?.sha) || !Array.isArray(commit?.files)) {
    return { kind: "invalid" };
  }
  if (
    commit.files.length === 0
    || commit.files.length >= 300
    || !Array.isArray(commit.parents)
  ) {
    return { kind: "code" };
  }
  if (!commit.files.every(allowedReviewOnlyFile)) {
    return { kind: "code" };
  }
  if (commit.parents.length !== 1 || !validSha(commit.parents[0]?.sha)) {
    return { kind: "code" };
  }
  return { kind: "review", parentSha: commit.parents[0].sha };
}

export function selectTrustedPostMergeCandidate(pullRequest, prCommits) {
  if (!validSha(pullRequest?.head?.sha) || !Array.isArray(prCommits) || prCommits.length === 0) {
    return null;
  }

  let expectedSha = pullRequest.head.sha;
  let hasReviewOnlyTail = false;
  for (let index = prCommits.length - 1; index >= 0; index -= 1) {
    const commit = prCommits[index];
    if (commit?.sha !== expectedSha) {
      return null;
    }
    const classification = classifyReviewOnlyCommit(commit);
    if (classification.kind === "invalid") {
      return null;
    }
    if (classification.kind === "code") {
      return { sha: commit.sha, hasReviewOnlyTail };
    }
    hasReviewOnlyTail = true;
    expectedSha = classification.parentSha;
  }

  return null;
}

function latestCandidateRun(runs, pullRequest, repository, candidateSha) {
  const createdAt = timestamp(pullRequest.created_at);
  const mergedAt = timestamp(pullRequest.merged_at);
  if (
    !validSha(candidateSha)
    || !Number.isFinite(createdAt)
    || !Number.isFinite(mergedAt)
    || mergedAt < createdAt
  ) {
    return null;
  }

  const matches = (Array.isArray(runs) ? runs : []).filter((run) => (
    run?.event === "pull_request"
    && run?.head_sha === candidateSha
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

function hasFullLaneEvidence(input, run) {
  if (!Array.isArray(input?.fullLaneRunIds)) {
    return true;
  }
  const runId = String(run?.id || "");
  return runId !== "" && input.fullLaneRunIds.some((id) => String(id) === runId);
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

  const candidateSource = selectTrustedPostMergeCandidate(pullRequest, input.prCommits);
  if (!candidateSource) {
    return denied("review-tail-evidence-unavailable");
  }

  const finalHeadCandidate = latestCandidateRun(
    input.workflowRuns,
    pullRequest,
    input.repository,
    pullRequest.head.sha,
  );
  if (
    finalHeadCandidate
    && finalHeadCandidate.status === "completed"
    && finalHeadCandidate.conclusion === "success"
    && hasFullLaneEvidence(input, finalHeadCandidate)
  ) {
    return {
      reuse: true,
      reason: candidateSource.hasReviewOnlyTail
        ? "review-tail-final-head-green-pr-workflow-reused"
        : "exact-green-pr-workflow-reused",
      sourceRunId: String(finalHeadCandidate.id),
      pullNumber: String(pullRequest.number),
    };
  }

  const candidate = latestCandidateRun(
    input.workflowRuns,
    pullRequest,
    input.repository,
    candidateSource.sha,
  );
  if (!candidate) {
    return denied("no-current-pr-workflow-candidate");
  }
  if (!hasFullLaneEvidence(input, candidate)) {
    return denied("candidate-full-lane-evidence-unavailable");
  }
  if (candidate.status !== "completed" || candidate.conclusion !== "success") {
    return denied("latest-pr-workflow-candidate-not-successful");
  }
  if (!Number.isInteger(candidate.id) && !/^[1-9][0-9]*$/.test(String(candidate.id || ""))) {
    return denied("candidate-run-id-unavailable");
  }
  return {
    reuse: true,
    reason: candidateSource.hasReviewOnlyTail
      ? "review-tail-green-pr-workflow-reused"
      : "exact-green-pr-workflow-reused",
    sourceRunId: String(candidate.id),
    pullNumber: String(pullRequest.number),
  };
}
