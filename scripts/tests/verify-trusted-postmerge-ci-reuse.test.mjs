import assert from "node:assert/strict";
import test from "node:test";

import { evaluateTrustedPostMergeReuse } from "../verify-trusted-postmerge-ci-reuse.mjs";

const base = "1".repeat(40);
const head = "2".repeat(40);
const merge = "3".repeat(40);
const tree = "4".repeat(40);

function candidate(overrides = {}) {
  return {
    id: 123,
    event: "pull_request",
    status: "completed",
    conclusion: "success",
    head_sha: head,
    head_branch: "feature",
    head_repository: { full_name: "edwardkim/rhwp" },
    created_at: "2026-08-27T10:01:00Z",
    updated_at: "2026-08-27T10:02:00Z",
    ...overrides,
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
      parents: [{ sha: base }, { sha: head }],
      commit: { tree: { sha: tree } },
    },
    sourceCommit: { sha: head, commit: { tree: { sha: tree } } },
    mergeBaseSha: base,
    pullRequests: [{
      number: 42,
      state: "closed",
      merged_at: "2026-08-27T10:03:00Z",
      merge_commit_sha: merge,
      created_at: "2026-08-27T10:00:00Z",
      base: { ref: "devel" },
      head: { sha: head, ref: "feature", repo: { full_name: "edwardkim/rhwp" } },
    }],
    pullFiles: [{ filename: "src/renderer/layout.rs" }],
    workflowRuns: [candidate()],
    ...overrides,
  };
}

test("reuses only the latest exact green PR workflow run", () => {
  assert.deepEqual(evaluateTrustedPostMergeReuse(input()), {
    reuse: true,
    reason: "exact-green-pr-workflow-reused",
    sourceRunId: "123",
    pullNumber: "42",
  });
});

test("fails closed for direct pushes, stale bases, enforcement changes, and incomplete candidates", () => {
  assert.equal(evaluateTrustedPostMergeReuse(input({ eventName: "workflow_dispatch" })).reuse, false);
  assert.equal(evaluateTrustedPostMergeReuse(input({ mergeBaseSha: "5".repeat(40) })).reuse, false);
  assert.equal(evaluateTrustedPostMergeReuse(input({ pullFiles: [{ filename: ".github/workflows/ci.yml" }] })).reuse, false);
  assert.equal(evaluateTrustedPostMergeReuse(input({ workflowRuns: [candidate({ status: "in_progress", conclusion: null })] })).reuse, false);
});
