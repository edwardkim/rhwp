import assert from "node:assert/strict";
import test from "node:test";

import { evaluateTrustedPostMergeReuse } from "../verify-trusted-postmerge-ci-reuse.mjs";

const base = "1".repeat(40);
const head = "2".repeat(40);
const merge = "3".repeat(40);
const tree = "4".repeat(40);
const code = "5".repeat(40);
const oldBase = "6".repeat(40);
const testedMerge = "7".repeat(40);

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
    prCommits: [{
      sha: head,
      parents: [{ sha: base }],
      files: [{ filename: "src/renderer/layout.rs", status: "modified" }],
    }],
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

test("reuses a stale-head PR when its tested merge ref exactly matches the final merge tree", () => {
  const result = evaluateTrustedPostMergeReuse(input({
    sourceCommit: { sha: head, commit: { tree: { sha: "8".repeat(40) } } },
    mergeBaseSha: oldBase,
    mergeTreeEvidenceByRunId: {
      123: {
        sha: testedMerge,
        parents: [base, head],
        treeSha: tree,
      },
    },
    fullLaneRunIds: ["123"],
  }));
  assert.deepEqual(result, {
    reuse: true,
    reason: "exact-merge-tree-green-pr-workflow-reused",
    sourceRunId: "123",
    pullNumber: "42",
  });
});

test("fails closed when stale-head merge-tree evidence is absent or mismatched", () => {
  const stale = {
    sourceCommit: { sha: head, commit: { tree: { sha: "8".repeat(40) } } },
    mergeBaseSha: oldBase,
    fullLaneRunIds: ["123"],
  };
  assert.equal(
    evaluateTrustedPostMergeReuse(input(stale)).reason,
    "pr-merge-tree-evidence-unavailable",
  );
  assert.equal(evaluateTrustedPostMergeReuse(input({
    ...stale,
    mergeTreeEvidenceByRunId: {
      123: { sha: testedMerge, parents: [base, head], treeSha: "9".repeat(40) },
    },
  })).reason, "pr-merge-tree-evidence-unavailable");
  assert.equal(evaluateTrustedPostMergeReuse(input({
    ...stale,
    mergeTreeEvidenceByRunId: {
      123: { sha: testedMerge, parents: [oldBase, head], treeSha: tree },
    },
  })).reason, "pr-merge-tree-evidence-unavailable");
});

test("stale-head exact-tree reuse still requires full-lane evidence", () => {
  const result = evaluateTrustedPostMergeReuse(input({
    sourceCommit: { sha: head, commit: { tree: { sha: "8".repeat(40) } } },
    mergeBaseSha: oldBase,
    mergeTreeEvidenceByRunId: {
      123: { sha: testedMerge, parents: [base, head], treeSha: tree },
    },
    fullLaneRunIds: [],
  }));
  assert.equal(result.reuse, false);
  assert.equal(result.reason, "candidate-full-lane-evidence-unavailable");
});

test("reuses the preceding full CI through a linear review-only tail", () => {
  const result = evaluateTrustedPostMergeReuse(input({
    prCommits: [
      {
        sha: code,
        parents: [{ sha: base }],
        files: [{ filename: "src/renderer/layout.rs", status: "modified" }],
      },
      {
        sha: head,
        parents: [{ sha: code }],
        files: [
          { filename: "mydocs/pr/archives/pr_6253_review.md", status: "added" },
          { filename: "pdf/pr_6253/reference.pdf", status: "added" },
        ],
      },
    ],
    workflowRuns: [candidate({ id: 456, head_sha: code })],
    fullLaneRunIds: ["456"],
  }));
  assert.deepEqual(result, {
    reuse: true,
    reason: "review-tail-green-pr-workflow-reused",
    sourceRunId: "456",
    pullNumber: "42",
  });
});

test("reuses an exact direct review-only PR fast pass after merge", () => {
  const result = evaluateTrustedPostMergeReuse(input({
    pullFiles: [
      { filename: "mydocs/pr/archives/pr_6456_review.md", status: "added" },
      { filename: "mydocs/orders/20260830.md", status: "modified" },
    ],
    prCommits: [{
      sha: head,
      parents: [{ sha: base }],
      files: [{ filename: "mydocs/pr/archives/pr_6456_review.md", status: "added" }],
    }],
    reviewOnlyFastPassRunIds: ["123"],
  }));
  assert.deepEqual(result, {
    reuse: true,
    reason: "direct-review-only-pr-fast-pass-reused",
    sourceRunId: "123",
    pullNumber: "42",
  });
});

test("fails closed when a direct review-only PR lacks worker-skip evidence", () => {
  const result = evaluateTrustedPostMergeReuse(input({
    pullFiles: [{ filename: "mydocs/orders/20260830.md", status: "modified" }],
    prCommits: [{
      sha: head,
      parents: [{ sha: base }],
      files: [{ filename: "mydocs/orders/20260830.md", status: "modified" }],
    }],
    reviewOnlyFastPassRunIds: [],
  }));
  assert.equal(result.reuse, false);
  assert.equal(result.reason, "direct-review-only-pr-fast-pass-evidence-unavailable");
});

test("prefers the final PR head when code and review evidence passed together", () => {
  const result = evaluateTrustedPostMergeReuse(input({
    prCommits: [
      {
        sha: code,
        parents: [{ sha: base }],
        files: [{ filename: "src/renderer/layout.rs", status: "modified" }],
      },
      {
        sha: head,
        parents: [{ sha: code }],
        files: [
          { filename: "mydocs/pr/archives/pr_6274_review.md", status: "added" },
          { filename: "pdf/pr_6274/reference.pdf", status: "added" },
        ],
      },
    ],
    workflowRuns: [candidate({ id: 789, head_sha: head })],
    fullLaneRunIds: ["789"],
  }));
  assert.deepEqual(result, {
    reuse: true,
    reason: "review-tail-final-head-green-pr-workflow-reused",
    sourceRunId: "789",
    pullNumber: "42",
  });
});

test("fails closed when a CI or CodeQL candidate lacks full-lane evidence", () => {
  const result = evaluateTrustedPostMergeReuse(input({ fullLaneRunIds: [] }));
  assert.equal(result.reuse, false);
  assert.equal(result.reason, "candidate-full-lane-evidence-unavailable");
});

test("fails closed when the review-only tail is not linear", () => {
  const result = evaluateTrustedPostMergeReuse(input({
    prCommits: [
      {
        sha: code,
        parents: [{ sha: base }],
        files: [{ filename: "src/renderer/layout.rs", status: "modified" }],
      },
      {
        sha: head,
        parents: [{ sha: "6".repeat(40) }],
        files: [{ filename: "mydocs/pr/archives/pr_6253_review.md", status: "added" }],
      },
    ],
  }));
  assert.equal(result.reuse, false);
  assert.equal(result.reason, "review-tail-evidence-unavailable");
});
