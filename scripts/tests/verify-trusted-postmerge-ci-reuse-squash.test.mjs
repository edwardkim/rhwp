import assert from "node:assert/strict";
import test from "node:test";

import { evaluateTrustedPostMergeReuse } from "../verify-trusted-postmerge-ci-reuse.mjs";

const base = "1".repeat(40);
const head = "2".repeat(40);
const merge = "3".repeat(40);
const tree = "4".repeat(40);

function candidate() {
  return {
    id: 123,
    event: "pull_request",
    head_sha: head,
    head_branch: "codex/6256",
    head_repository: { full_name: "edwardkim/rhwp" },
    created_at: "2026-08-27T10:01:00Z",
    updated_at: "2026-08-27T10:02:00Z",
    status: "completed",
    conclusion: "success",
  };
}

function input(overrides = {}) {
  return {
    eventName: "push",
    ref: "refs/heads/devel",
    repository: "edwardkim/rhwp",
    mergeSha: merge,
    mergeCommit: {
      sha: merge,
      parents: [{ sha: base }],
      commit: { tree: { sha: tree } },
    },
    sourceCommit: { sha: head, commit: { tree: { sha: tree } } },
    mergeBaseSha: base,
    pullRequests: [{
      number: 6256,
      state: "closed",
      merged_at: "2026-08-27T10:03:00Z",
      merge_commit_sha: merge,
      base: { ref: "devel" },
      head: {
        sha: head,
        ref: "codex/6256",
        repo: { full_name: "edwardkim/rhwp" },
      },
      created_at: "2026-08-27T10:00:00Z",
    }],
    pullFiles: [{ filename: "src/lib.rs" }],
    workflowRuns: [candidate()],
    ...overrides,
  };
}

test("reuses an exact green same-repository PR after a squash merge", () => {
  assert.deepEqual(evaluateTrustedPostMergeReuse(input()), {
    reuse: true,
    reason: "exact-green-pr-workflow-reused",
    sourceRunId: "123",
    pullNumber: "6256",
  });
});

test("fails closed when a squash merge tree differs from the reviewed PR head", () => {
  const result = evaluateTrustedPostMergeReuse(input({
    mergeCommit: {
      sha: merge,
      parents: [{ sha: base }],
      commit: { tree: { sha: "5".repeat(40) } },
    },
  }));
  assert.equal(result.reuse, false);
  assert.equal(result.reason, "merge-tree-does-not-match-pr-head");
});

test("fails closed when a squash merge has no unique associated PR", () => {
  const result = evaluateTrustedPostMergeReuse(input({ pullRequests: [] }));
  assert.equal(result.reuse, false);
  assert.equal(result.reason, "merge-commit-must-map-to-one-merged-same-repository-pr");
});
