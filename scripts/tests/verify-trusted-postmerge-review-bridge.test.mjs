import assert from "node:assert/strict";
import test from "node:test";
import { evaluateTrustedPostMergeReuse } from "../verify-trusted-postmerge-ci-reuse.mjs";

const base = "1".repeat(40), code = "2".repeat(40), docs = "3".repeat(40);
const bridge = "4".repeat(40), head = "5".repeat(40), merge = "6".repeat(40);
const testedMerge = "7".repeat(40), testedTree = "8".repeat(40), tree = "9".repeat(40);
const fullRunId = 34029158620, tailRunId = 34030157260;
const file = (filename) => ({ filename, status: "modified" });
const commit = (sha, parents, filename) => ({
  sha, parents: parents.map(sha => ({ sha })), files: [file(filename)],
});

function fixture() {
  const run = (id, sha) => ({
    id, head_sha: sha, event: "pull_request", head_branch: "fix/review-bridge",
    head_repository: { full_name: "edwardkim/rhwp" },
    status: "completed", conclusion: "success",
    created_at: "2026-09-06T11:04:00Z", updated_at: "2026-09-06T11:27:30Z",
  });
  return {
    eventName: "push", ref: "refs/heads/devel", repository: "edwardkim/rhwp",
    mergeSha: merge,
    mergeCommit: { sha: merge, parents: [{ sha: base }], commit: { tree: { sha: tree } } },
    sourceCommit: { sha: head, commit: { tree: { sha: tree } } }, mergeBaseSha: base,
    pullRequests: [{
      number: 6813, state: "closed", merge_commit_sha: merge,
      created_at: "2026-09-06T11:03:00Z", merged_at: "2026-09-06T11:29:16Z",
      base: { ref: "devel" },
      head: { sha: head, ref: "fix/review-bridge", repo: { full_name: "edwardkim/rhwp" } },
    }],
    pullFiles: [file("src/renderer/composer.rs"), file("mydocs/orders/day.md")],
    prCommits: [
      commit(code, [base], "src/renderer/composer.rs"),
      commit(docs, [code], "mydocs/orders/day.md"),
      commit(bridge, [docs, base], "mydocs/orders/day.md"),
      commit(head, [bridge], "mydocs/pr/review.md"),
    ],
    workflowRuns: [run(fullRunId, code), run(tailRunId, head)],
    fullLaneRunIds: [String(fullRunId)],
    mergeTreeEvidenceByRunId: {
      [fullRunId]: { sha: testedMerge, parents: [base, code], treeSha: testedTree },
    },
  };
}

test("#6813: a docs merge between a full candidate and fast-pass head fails closed without tree proof", () => {
  const result = evaluateTrustedPostMergeReuse(fixture());
  assert.equal(result.reuse, false);
  assert.equal(result.reason, "candidate-full-lane-evidence-unavailable");
});
