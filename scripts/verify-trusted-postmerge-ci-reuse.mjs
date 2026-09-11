#!/usr/bin/env node

import { execFileSync } from "node:child_process";

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

function positiveId(value) {
  return Number.isSafeInteger(value) && value > 0;
}

export function trustedPullRequestSource(pullRequest, repository, repositoryId) {
  const head = pullRequest?.head?.repo;
  if (head?.full_name === repository) {
    return true;
  }
  return positiveId(repositoryId)
    && pullRequest?.base?.repo?.full_name === repository
    && pullRequest.base.repo.id === repositoryId
    && positiveId(pullRequest.number)
    && head?.fork === true && positiveId(head.id) && head.id !== repositoryId
    && /^[A-Za-z0-9_.-]+\/[A-Za-z0-9_.-]+$/.test(head.full_name || "");
}

// Fork runs belong to upstream Actions, not the fork's Actions installation.
// An empty pull_requests array is normal for forks; the artifact binds the PR.
export function trustedPullRequestWorkflowRun(run, pullRequest, repository, repositoryId, workflowFile) {
  if (run?.event !== "pull_request" || !validSha(run.head_sha)
    || run.head_branch !== pullRequest?.head?.ref
    || run.head_repository?.full_name !== pullRequest?.head?.repo?.full_name) {
    return false;
  }
  if (pullRequest.head.repo.full_name === repository) {
    return true;
  }
  return trustedPullRequestSource(pullRequest, repository, repositoryId)
    && run.repository?.full_name === repository && run.repository.id === repositoryId
    && run.head_repository.id === pullRequest.head.repo.id
    && positiveId(run.id) && positiveId(run.run_attempt)
    && /^[A-Za-z0-9_-]+\.yml$/.test(workflowFile || "")
    && run.path === `.github/workflows/${workflowFile}`
    && Array.isArray(run.pull_requests)
    && (run.pull_requests.length === 0 || run.pull_requests.some((pr) => (
      pr.number === pullRequest.number && pr.head?.sha === run.head_sha
      && pr.head?.repo?.id === pullRequest.head.repo.id
      && pr.base?.repo?.id === repositoryId
    )));
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
    || path === "scripts/trusted-postmerge-duration-evidence.mjs"
    || path === "scripts/verify-trusted-postmerge-ci-reuse.mjs"
    || path === "tests/suites/nextest-target-duration-policy.json"
  ));
}

function allowedReviewOnlyFile(file) {
  if (!file || typeof file.filename !== "string") {
    return false;
  }
  if (file.previous_filename && !file.previous_filename.startsWith("mydocs/")) {
    return false;
  }
  if (file.filename.startsWith("mydocs/")) {
    return true;
  }

  const filename = file.filename;
  const lowerName = filename.toLowerCase();
  const sampleReference = filename.startsWith("samples/")
    && [".hwp", ".hwpx", ".pdf", ".png"].some((extension) => lowerName.endsWith(extension));
  const pdfReference = filename.startsWith("pdf/")
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

// A structural bridge is only a search hint. Reuse also requires independent tree proof.
export function currentBaseReviewBridgeSource(commit, baseSha) {
  const parents = commit?.parents;
  return validSha(commit?.sha) && validSha(baseSha) && Array.isArray(parents)
    && parents.length === 2 && parents[1]?.sha === baseSha
    && validSha(parents[0]?.sha) && parents[0].sha !== baseSha
    ? parents[0].sha : "";
}

export function selectTrustedPostMergeCandidate(pullRequest, prCommits, baseSha) {
  if (!validSha(pullRequest?.head?.sha) || !Array.isArray(prCommits) || prCommits.length === 0) {
    return null;
  }

  let expectedSha = pullRequest.head.sha;
  let hasReviewOnlyTail = false;
  const fullLaneCandidates = [];
  let bridgeSha = "";
  for (let index = prCommits.length - 1; index >= 0; index -= 1) {
    const commit = prCommits[index];
    if (commit?.sha !== expectedSha) {
      return null;
    }
    const bridgeSource = currentBaseReviewBridgeSource(commit, baseSha);
    if (bridgeSource) {
      if (bridgeSha) {
        return null;
      }
      bridgeSha = commit.sha;
      fullLaneCandidates.push({ sha: commit.sha, hasReviewOnlyTail });
      expectedSha = bridgeSource;
      hasReviewOnlyTail = true;
      continue;
    }
    const classification = classifyReviewOnlyCommit(commit);
    if (classification.kind === "invalid") {
      return null;
    }
    if (classification.kind === "code") {
      return {
        sha: commit.sha,
        hasReviewOnlyTail,
        ...(bridgeSha ? { bridgeSha } : {}),
        fullLaneCandidates: [
          ...fullLaneCandidates,
          { sha: commit.sha, hasReviewOnlyTail },
        ],
      };
    }
    fullLaneCandidates.push({ sha: commit.sha, hasReviewOnlyTail });
    hasReviewOnlyTail = true;
    expectedSha = classification.parentSha;
  }

  return null;
}

// Run only from the trusted base checkout; the inspected commits are data, never code.
function verifyPostMergeTree(repository, identity, requireBridge) {
  const { baseSha, bridgeSha, mergeSha, candidateSha, testedMergeSha } = identity;
  if (![baseSha, mergeSha, candidateSha, testedMergeSha].every(validSha)
    || (requireBridge && !validSha(bridgeSha))) {
    throw new Error("invalid-review-bridge-identity");
  }
  const git = (...args) => execFileSync("git", ["-C", repository, "--no-replace-objects", ...args], {
    encoding: "utf8", timeout: 30000, maxBuffer: 1024 * 1024,
    stdio: ["ignore", "pipe", "pipe"],
  });
  const readCommit = (sha) => {
    // Pretty-printing hides parents at a depth=1 boundary; raw headers retain the real identity.
    if (git("cat-file", "-t", sha).trim() !== "commit") {
      throw new Error("review-bridge-commit-unavailable");
    }
    const headers = git("cat-file", "commit", sha).split("\n\n", 1)[0].split("\n");
    const trees = headers.filter(line => line.startsWith("tree ")).map(line => line.slice(5));
    const parents = headers.filter(line => line.startsWith("parent ")).map(line => line.slice(7));
    if (trees.length !== 1 || !validSha(trees[0]) || !parents.every(validSha)) {
      throw new Error("review-bridge-commit-unavailable");
    }
    return { sha, treeSha: trees[0], parents: parents.map(sha => ({ sha })) };
  };
  const bridge = requireBridge ? readCommit(bridgeSha) : null;
  const tested = readCommit(testedMergeSha);
  const final = readCommit(mergeSha);
  if ((requireBridge && !currentBaseReviewBridgeSource(bridge, baseSha))
    || tested.parents.length !== 2 || tested.parents[0].sha !== baseSha
    || tested.parents[1].sha !== candidateSha || candidateSha === baseSha
    || final.parents.length < (requireBridge ? 1 : 2) || final.parents.length > 2
    || final.parents[0].sha !== baseSha) {
    throw new Error("review-bridge-base-mismatch");
  }
  const fields = git("diff", "--raw", "--no-abbrev", "-z", "--no-renames", "--no-ext-diff",
    "--no-textconv", "--ignore-submodules=none", tested.treeSha, final.treeSha, "--").split("\0");
  if (fields.pop() !== "" || fields.length % 2 !== 0) {
    throw new Error("review-bridge-incomplete-tree-diff");
  }
  for (let index = 0; index < fields.length; index += 2) {
    const path = fields[index + 1];
    if (!/^:(000000|100644) (000000|100644) [0-9a-f]{40} [0-9a-f]{40} [AMD]$/.test(fields[index])
      || !path.startsWith("mydocs/")
      || path === "mydocs/tech/text-ir-v2.md"
      || path === "mydocs/tech/canvaskit-parity-implementation.md") {
      throw new Error("review-bridge-non-review-tree-change");
    }
  }
  return { baseSha, ...(requireBridge ? { bridgeSha } : {}), mergeSha, candidateSha, testedMergeSha,
    testedTreeSha: tested.treeSha, finalTreeSha: final.treeSha };
}

export function verifyPostMergeReviewBridgeTree(repository, identity) {
  return verifyPostMergeTree(repository, identity, true);
}

export function verifyForkPostMergeTree(repository, identity) {
  return verifyPostMergeTree(repository, identity, false);
}

// Only the exact final fork head may reuse a run across a Markdown-only base advance.
// Inspect raw immutable objects, including every first-parent step, never PR code.
export function verifyReviewOnlyBaseAdvance(repository, identity) {
  const { baseSha, mergeSha, candidateSha, testedMergeSha } = identity;
  if (![baseSha, mergeSha, candidateSha, testedMergeSha].every(validSha)) {
    throw new Error("invalid-review-base-advance-identity");
  }
  const git = (...args) => execFileSync("git", ["-C", repository, "--no-replace-objects", ...args], {
    encoding: "utf8", timeout: 30000, maxBuffer: 1024 * 1024,
    stdio: ["ignore", "pipe", "pipe"],
  });
  const commit = (sha) => {
    const headers = git("cat-file", "commit", sha).split("\n\n", 1)[0].split("\n");
    const trees = headers.filter(line => line.startsWith("tree ")).map(line => line.slice(5));
    const parents = headers.filter(line => line.startsWith("parent ")).map(line => line.slice(7));
    if (trees.length !== 1 || !validSha(trees[0]) || !parents.every(validSha)
      || new Set(parents).size !== parents.length) {
      throw new Error("review-base-advance-commit-unavailable");
    }
    return { tree: trees[0], parents };
  };
  const assertReviewDiff = (before, after) => {
    const fields = git("diff", "--raw", "--no-abbrev", "-z", "--no-renames", "--no-ext-diff",
      "--no-textconv", "--ignore-submodules=none", before, after, "--").split("\0");
    if (fields.pop() !== "" || fields.length % 2 !== 0) {
      throw new Error("review-base-advance-incomplete-diff");
    }
    for (let index = 0; index < fields.length; index += 2) {
      const path = fields[index + 1];
      if (!/^:(000000|100644) (000000|100644) [0-9a-f]{40} [0-9a-f]{40} [AMD]$/.test(fields[index])
        || !path.startsWith("mydocs/") || !path.endsWith(".md")
        || path === "mydocs/tech/text-ir-v2.md"
        || path === "mydocs/tech/canvaskit-parity-implementation.md") {
        throw new Error("review-base-advance-non-review-change");
      }
    }
  };
  const tested = commit(testedMergeSha);
  const final = commit(mergeSha);
  if (tested.parents.length !== 2 || final.parents.length !== 2
    || tested.parents[1] !== candidateSha || final.parents[1] !== candidateSha
    || final.parents[0] !== baseSha || tested.parents[0] === baseSha) {
    throw new Error("review-base-advance-parent-mismatch");
  }
  const testedBaseSha = tested.parents[0];
  let cursor = baseSha;
  for (let count = 0; cursor !== testedBaseSha && count < 64; count += 1) {
    const current = commit(cursor);
    if (current.parents.length < 1 || current.parents.length > 2) {
      throw new Error("review-base-advance-history-unavailable");
    }
    const parent = commit(current.parents[0]);
    assertReviewDiff(parent.tree, current.tree);
    cursor = current.parents[0];
  }
  if (cursor !== testedBaseSha) {
    throw new Error("review-base-advance-history-limit");
  }
  assertReviewDiff(tested.tree, final.tree);
  return { baseSha, testedBaseSha, mergeSha, candidateSha, testedMergeSha,
    testedTreeSha: tested.tree, finalTreeSha: final.tree };
}

function hasReviewBridgeTreeEvidence(input, run, baseSha, bridgeSha, finalTreeSha) {
  const tested = input.mergeTreeEvidenceByRunId?.[String(run.id)];
  const proof = input.reviewBridgeTreeEvidenceByRunId?.[String(run.id)];
  return hasIntermediateCandidateMergeTreeEvidence(input, run)
    && tested.parents[0] === baseSha
    && proof?.baseSha === baseSha && proof.bridgeSha === bridgeSha
    && proof.mergeSha === input.mergeSha && proof.candidateSha === run.head_sha
    && proof.testedMergeSha === tested.sha && proof.testedTreeSha === tested.treeSha
    && proof.finalTreeSha === finalTreeSha;
}

function isDirectReviewOnlyPullRequest(pullRequest, pullFiles, prCommits) {
  if (
    !validSha(pullRequest?.head?.sha)
    || !Array.isArray(pullFiles)
    || pullFiles.length === 0
    || !pullFiles.every(allowedReviewOnlyFile)
    || !Array.isArray(prCommits)
    || prCommits.length === 0
  ) {
    return false;
  }

  let expectedSha = pullRequest.head.sha;
  for (let index = prCommits.length - 1; index >= 0; index -= 1) {
    const commit = prCommits[index];
    if (commit?.sha !== expectedSha) {
      return false;
    }
    const classification = classifyReviewOnlyCommit(commit);
    if (classification.kind !== "review") {
      return false;
    }
    expectedSha = classification.parentSha;
  }

  return true;
}

function latestCandidateRun(runs, pullRequest, repository, candidateSha, repositoryId, workflowFile) {
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
    trustedPullRequestWorkflowRun(run, pullRequest, repository, repositoryId, workflowFile)
    && run?.head_sha === candidateSha
    && timestamp(run.created_at) >= createdAt
    // Do not hide a newer rerun just because it completed after the merge.
    // Select it first, then reject non-pre-merge evidence at the caller.
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

// A classified frontend-only CI has no Rust timing data to reuse. Require the
// exact current CI job contract instead of treating any green aggregate as proof.
// Unknown, duplicate, missing, pending, or unexpectedly skipped jobs fail closed.
export function frontendOnlyCiRunIsReusable(impact, jobs) {
  if (
    impact?.classification_status !== "classified"
    || impact.rust_required !== "false"
    || impact.native_skia_required !== "false"
    || !["unit", "package"].includes(impact.frontend_mode)
    || !Array.isArray(jobs)
  ) {
    return false;
  }
  const expected = new Map([
    ["trusted_postmerge_reuse / Verify trusted post-merge reuse", "success"],
    ["CI preflight", "success"],
    ["Build & Test", "success"],
    ["Frontend unit gates", impact.frontend_mode === "unit" ? "success" : "skipped"],
    ["Frontend package gates", impact.frontend_mode === "package" ? "success" : "skipped"],
    ["WASM Build", "skipped"],
    ["Resolve nextest target duration policy", "skipped"],
    ["Lint (fmt, clippy, WASM check)", "skipped"],
    ["Native Skia tests", "skipped"],
    ["Workflow promotion preflight", "skipped"],
    ["Refresh nextest target duration data", "skipped"],
    ...["a", "b", "c", "d"].flatMap((label) => [
      [`build-test-archive-${label}`, "skipped"],
      [`test-archive-${label}-shard-1`, "skipped"],
    ]),
  ]);
  if (jobs.length !== expected.size) {
    return false;
  }
  for (const job of jobs) {
    if (
      !job
      || !expected.has(job.name)
      || job.status !== "completed"
      || job.conclusion !== expected.get(job.name)
    ) {
      return false;
    }
    expected.delete(job.name);
  }
  return expected.size === 0;
}

// Workflow success alone is not proof that the required workers actually ran.
export function fullLaneWorkflowJobsAreGreen(workflowFile, jobs) {
  const required = {
    "ci.yml": ["CI preflight", "Build & Test", "Lint (fmt, clippy, WASM check)"],
    "codeql.yml": ["CodeQL preflight", "Analyze (javascript-typescript)", "Analyze (python)", "Analyze (rust)"],
    "adapter-diff.yml": ["adapter inter-diff preflight", "adapter inter-diff"],
    "proptest-roundtrip.yml": ["Proptest preflight", "prop roundtrip"],
  }[workflowFile];
  if (!required || !Array.isArray(jobs) || jobs.length === 0
    || jobs.some((job) => job?.status !== "completed"
      || !["success", "skipped"].includes(job.conclusion))) {
    return false;
  }
  return required.every((name) => {
    const matches = jobs.filter((job) => job.name === name);
    return matches.length === 1 && matches[0].conclusion === "success";
  });
}

function hasFrontendOnlyEvidence(input, run) {
  const runId = String(run?.id || "");
  return Array.isArray(input?.frontendOnlyRunIds)
    && runId !== ""
    && input.frontendOnlyRunIds.some((id) => String(id) === runId);
}

function hasFullLaneEvidence(input, run) {
  if (!Array.isArray(input?.fullLaneRunIds)) {
    return false;
  }
  const runId = String(run?.id || "");
  return runId !== "" && input.fullLaneRunIds.some((id) => String(id) === runId);
}

function hasReviewOnlyFastPassEvidence(input, run) {
  const runId = String(run?.id || "");
  return Array.isArray(input?.reviewOnlyFastPassRunIds)
    && runId !== ""
    && input.reviewOnlyFastPassRunIds.some((id) => String(id) === runId);
}

function hasExactMergeTreeEvidence(input, run, pullRequest, parents, treeSha) {
  const runId = String(run?.id || "");
  const evidence = input?.mergeTreeEvidenceByRunId?.[runId];
  const evidenceParents = Array.isArray(evidence?.parents)
    ? evidence.parents.filter(validSha)
    : [];
  return (
    runId !== ""
    && validSha(evidence?.sha)
    && validSha(evidence?.treeSha)
    && parents.length === 2
    && evidenceParents.length === 2
    && evidenceParents.every((sha, index) => sha === parents[index])
    && evidenceParents.includes(pullRequest.head.sha)
    && evidence.treeSha === treeSha
  );
}

function hasIntermediateCandidateMergeTreeEvidence(input, run) {
  const runId = String(run?.id || "");
  const evidence = input?.mergeTreeEvidenceByRunId?.[runId];
  const evidenceParents = Array.isArray(evidence?.parents)
    ? evidence.parents.filter(validSha)
    : [];
  return (
    runId !== ""
    && validSha(evidence?.sha)
    && validSha(evidence?.treeSha)
    && evidenceParents.length === 2
    && evidenceParents[1] === run?.head_sha
  );
}

function hasForkRunBinding(input, run, pullRequest) {
  const evidence = input.mergeTreeEvidenceByRunId?.[String(run?.id)];
  return hasIntermediateCandidateMergeTreeEvidence(input, run)
    && evidence.pullNumber === pullRequest.number
    && evidence.repositoryId === input.repositoryId
    && evidence.headRepositoryId === pullRequest.head.repo.id
    && evidence.runAttempt === run.run_attempt;
}

function hasForkCandidateTreeEvidence(input, run, pullRequest, baseSha, finalTreeSha) {
  const tested = input.mergeTreeEvidenceByRunId?.[String(run?.id)];
  const proof = input.forkMergeTreeEvidenceByRunId?.[String(run?.id)];
  return hasForkRunBinding(input, run, pullRequest)
    && tested.parents[0] === baseSha
    && proof?.baseSha === baseSha && proof.mergeSha === input.mergeSha
    && proof.candidateSha === run.head_sha && proof.testedMergeSha === tested.sha
    && proof.testedTreeSha === tested.treeSha && proof.finalTreeSha === finalTreeSha;
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
    && trustedPullRequestSource(pullRequest, input.repository, input.repositoryId)
    && validSha(pullRequest.head?.sha)
  ));
  if (pullRequests.length !== 1) {
    return denied("merge-commit-must-map-to-one-trusted-merged-pr");
  }
  const pullRequest = pullRequests[0];
  const isFork = pullRequest.head.repo.full_name !== input.repository;
  if (isFork && parents.length !== 2) {
    return denied("fork-reuse-requires-two-parent-merge");
  }
  if (isFork && !Array.isArray(input.fullLaneRunIds)) {
    return denied("fork-worker-execution-evidence-unavailable");
  }
  if (parents.length === 2 && !parents.includes(pullRequest.head.sha)) {
    return denied("merge-parent-does-not-match-pr-head");
  }
  const baseParent = parents.length === 1
    ? parents[0]
    : parents.find((parent) => parent !== pullRequest.head.sha);
  if (!baseParent) {
    return denied("merge-base-parent-unavailable");
  }
  if (input.sourceCommit?.sha !== pullRequest.head.sha) {
    return denied("source-commit-does-not-match-pr-head");
  }
  if (enforcementPathChanged(input.pullFiles)) {
    return denied("pr-changes-ci-enforcement-surface");
  }
  const mergeTreeSha = input.mergeCommit?.commit?.tree?.sha;
  if (!validSha(mergeTreeSha)) {
    return denied("merge-tree-unavailable");
  }
  const headContainsBase = input.mergeBaseSha === baseParent;
  const finalHeadCandidate = latestCandidateRun(
    input.workflowRuns,
    pullRequest,
    input.repository,
    pullRequest.head.sha,
    input.repositoryId,
    input.workflowFile,
  );
  if (!finalHeadCandidate || finalHeadCandidate.status !== "completed"
    || finalHeadCandidate.conclusion !== "success") {
    return denied("final-head-pr-workflow-not-successful");
  }
  if (!Number.isFinite(timestamp(finalHeadCandidate.updated_at))
    || timestamp(finalHeadCandidate.updated_at) > timestamp(pullRequest.merged_at)) {
    return denied("final-head-pr-workflow-not-completed-before-merge");
  }
  const exactMergeTreeEvidence = hasExactMergeTreeEvidence(
    input,
    finalHeadCandidate,
    pullRequest,
    parents,
    mergeTreeSha,
  );
  const advanceProof = input.reviewOnlyBaseAdvanceByRunId?.[String(finalHeadCandidate?.id)];
  const testedFinalHead = input.mergeTreeEvidenceByRunId?.[String(finalHeadCandidate?.id)];
  const reviewOnlyBaseAdvance = isFork
    && finalHeadCandidate?.status === "completed" && finalHeadCandidate.conclusion === "success"
    && hasForkRunBinding(input, finalHeadCandidate, pullRequest)
    && advanceProof?.baseSha === baseParent
    && advanceProof.testedBaseSha === testedFinalHead.parents[0]
    && advanceProof.testedBaseSha !== baseParent
    && advanceProof.mergeSha === input.mergeSha
    && advanceProof.candidateSha === pullRequest.head.sha
    && advanceProof.testedMergeSha === testedFinalHead.sha
    && advanceProof.testedTreeSha === testedFinalHead.treeSha
    && advanceProof.finalTreeSha === mergeTreeSha;
  if (isFork && ((!exactMergeTreeEvidence && !reviewOnlyBaseAdvance)
    || !hasForkRunBinding(input, finalHeadCandidate, pullRequest))) {
    return denied("fork-pr-merge-tree-evidence-unavailable");
  }
  if (
    headContainsBase
    && mergeTreeSha !== input.sourceCommit?.commit?.tree?.sha
    && !exactMergeTreeEvidence
    && !reviewOnlyBaseAdvance
  ) {
    return denied("merge-tree-does-not-match-pr-head");
  }
  if (!headContainsBase && !exactMergeTreeEvidence && !reviewOnlyBaseAdvance) {
    return denied("pr-merge-tree-evidence-unavailable");
  }

  // A direct review-only PR has no code candidate. It is safe to reuse only
  // when this worker's exact PR run proves that preflight skipped the worker.
  if (isDirectReviewOnlyPullRequest(pullRequest, input.pullFiles, input.prCommits)) {
    if (!finalHeadCandidate) {
      return denied("direct-review-only-pr-workflow-unavailable");
    }
    if (
      finalHeadCandidate.status !== "completed"
      || finalHeadCandidate.conclusion !== "success"
    ) {
      return denied("direct-review-only-pr-workflow-not-successful");
    }
    if (!hasReviewOnlyFastPassEvidence(input, finalHeadCandidate)) {
      return denied("direct-review-only-pr-fast-pass-evidence-unavailable");
    }
    return {
      reuse: true,
      reason: "direct-review-only-pr-fast-pass-reused",
      sourceRunId: String(finalHeadCandidate.id),
      pullNumber: String(pullRequest.number),
    };
  }

  const candidateSource = selectTrustedPostMergeCandidate(pullRequest, input.prCommits, baseParent);
  if (!candidateSource) {
    return denied("review-tail-evidence-unavailable");
  }

  // Frontend evidence is collected only for this exact final PR head. Do not
  // generalize it to an older code candidate hidden behind a skipped review tail.
  const frontendOnly = hasFrontendOnlyEvidence(input, finalHeadCandidate);
  if (
    finalHeadCandidate
    && finalHeadCandidate.status === "completed"
    && finalHeadCandidate.conclusion === "success"
    && (hasFullLaneEvidence(input, finalHeadCandidate) || frontendOnly)
  ) {
    return {
      reuse: true,
      reason: reviewOnlyBaseAdvance
        ? "review-only-base-advance-final-head-green-pr-workflow-reused"
        : frontendOnly
        ? (candidateSource.hasReviewOnlyTail
          ? "review-tail-final-head-green-frontend-ci-reused"
          : "exact-green-frontend-ci-reused")
        : !headContainsBase
        ? (candidateSource.hasReviewOnlyTail
          ? "review-tail-exact-merge-tree-green-pr-workflow-reused"
          : "exact-merge-tree-green-pr-workflow-reused")
        : (candidateSource.hasReviewOnlyTail
          ? "review-tail-final-head-green-pr-workflow-reused"
          : "exact-green-pr-workflow-reused"),
      sourceRunId: String(finalHeadCandidate.id),
      pullNumber: String(pullRequest.number),
      ...(frontendOnly ? { refreshDurationData: false } : {}),
    };
  }

  let foundCandidateRun = false;
  let foundFullLaneCandidate = false;
  let missingIntermediateCandidateEvidence = false;
  let missingBridgeEvidence = false;
  let unsuccessfulCandidate = false;
  if (candidateSource.bridgeSha && (!finalHeadCandidate
    || finalHeadCandidate.status !== "completed" || finalHeadCandidate.conclusion !== "success")) {
    return denied("review-bridge-final-head-not-successful");
  }
  for (const candidateSourceEntry of candidateSource.fullLaneCandidates) {
    const candidate = latestCandidateRun(
      input.workflowRuns,
      pullRequest,
      input.repository,
      candidateSourceEntry.sha,
      input.repositoryId,
      input.workflowFile,
    );
    if (!candidate) {
      continue;
    }
    foundCandidateRun = true;
    // A newer failed or incomplete candidate must not be hidden by older green runs.
    if (candidate.status !== "completed" || candidate.conclusion !== "success") {
      return denied("latest-pr-workflow-candidate-not-successful");
    }
    if (!Number.isFinite(timestamp(candidate.updated_at))
      || timestamp(candidate.updated_at) > timestamp(pullRequest.merged_at)) {
      return denied("latest-pr-workflow-candidate-not-completed-before-merge");
    }
    if (!hasFullLaneEvidence(input, candidate)) {
      continue;
    }
    foundFullLaneCandidate = true;
    if (isFork && !hasForkCandidateTreeEvidence(input, candidate, pullRequest, baseParent, mergeTreeSha)) {
      missingIntermediateCandidateEvidence = true;
      continue;
    }
    if (candidateSource.bridgeSha && !hasReviewBridgeTreeEvidence(
      input, candidate, baseParent, candidateSource.bridgeSha, mergeTreeSha,
    )) {
      missingBridgeEvidence = true;
      continue;
    }
    if (
      candidateSourceEntry.sha !== candidateSource.sha
      && !hasIntermediateCandidateMergeTreeEvidence(input, candidate)
    ) {
      missingIntermediateCandidateEvidence = true;
      continue;
    }
    if (!Number.isInteger(candidate.id) && !/^[1-9][0-9]*$/.test(String(candidate.id || ""))) {
      unsuccessfulCandidate = true;
      continue;
    }
    return {
      reuse: true,
      reason: candidateSource.bridgeSha
        ? "current-base-review-bridge-green-pr-workflow-reused"
        : candidateSourceEntry.hasReviewOnlyTail
        ? "review-tail-green-pr-workflow-reused"
        : "exact-green-pr-workflow-reused",
      sourceRunId: String(candidate.id),
      pullNumber: String(pullRequest.number),
    };
  }
  if (!foundCandidateRun) {
    return denied("no-current-pr-workflow-candidate");
  }
  if (!foundFullLaneCandidate) {
    return denied("candidate-full-lane-evidence-unavailable");
  }
  if (missingBridgeEvidence) {
    return denied("review-bridge-tree-evidence-unavailable");
  }
  if (missingIntermediateCandidateEvidence) {
    return denied("review-tail-candidate-merge-tree-evidence-unavailable");
  }
  if (unsuccessfulCandidate) {
    return denied("latest-pr-workflow-candidate-not-successful");
  }
  return denied("candidate-run-id-unavailable");
}
