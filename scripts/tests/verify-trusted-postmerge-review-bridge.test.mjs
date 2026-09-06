import assert from "node:assert/strict";
import test from "node:test";
import { mkdtempSync, mkdirSync, writeFileSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { execFileSync } from "node:child_process";
import {
  evaluateTrustedPostMergeReuse, selectTrustedPostMergeCandidate,
  currentBaseReviewBridgeSource, verifyPostMergeReviewBridgeTree,
} from "../verify-trusted-postmerge-ci-reuse.mjs";

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
  assert.equal(result.reason, "review-bridge-tree-evidence-unavailable");
});

function provenFixture() {
  return { ...fixture(), reviewBridgeTreeEvidenceByRunId: { [fullRunId]: {
    baseSha: base, bridgeSha: bridge, mergeSha: merge, candidateSha: code,
    testedMergeSha: testedMerge, testedTreeSha: testedTree, finalTreeSha: tree,
  } } };
}

test("#6813: verified docs bridge selects the earlier full run, not the green fast-pass run", () => {
  const result = evaluateTrustedPostMergeReuse(provenFixture());
  assert.deepEqual(result, {
    reuse: true, reason: "current-base-review-bridge-green-pr-workflow-reused",
    sourceRunId: String(fullRunId), pullNumber: "6813",
  });
});

test("verified bridge supports a normal merge as well as squash", () => {
  const data = provenFixture();
  data.mergeCommit.parents.push({ sha: head });
  assert.equal(evaluateTrustedPostMergeReuse(data).reuse, true);
});

for (const key of ["baseSha", "bridgeSha", "mergeSha", "candidateSha", "testedMergeSha",
  "testedTreeSha", "finalTreeSha"]) {
  test(`rejects a stale or mismatched bridge proof: ${key}`, () => {
    const data = provenFixture();
    data.reviewBridgeTreeEvidenceByRunId[fullRunId][key] = "a".repeat(40);
    assert.equal(evaluateTrustedPostMergeReuse(data).reuse, false);
  });
}

for (const [name, mutate] of [
  ["missing full lane artifacts", d => { d.fullLaneRunIds = []; }],
  ["missing merge artifact", d => { d.mergeTreeEvidenceByRunId = {}; }],
  ["wrong tested base", d => { d.mergeTreeEvidenceByRunId[fullRunId].parents[0] = docs; }],
  ["wrong tested head", d => { d.mergeTreeEvidenceByRunId[fullRunId].parents[1] = head; }],
  ["failed full run", d => { d.workflowRuns[0].conclusion = "failure"; }],
  ["pending full run", d => { d.workflowRuns[0].status = "in_progress"; }],
  ["failed final head", d => { d.workflowRuns[1].conclusion = "failure"; }],
  ["pending final head", d => { d.workflowRuns[1].status = "in_progress"; }],
  ["missing final head run", d => { d.workflowRuns.pop(); }],
  ["full run after merge", d => { d.workflowRuns[0].updated_at = "2026-09-07T00:00:00Z"; }],
  ["fork PR", d => { d.pullRequests[0].head.repo.full_name = "fork/rhwp"; }],
  ["fork run", d => { d.workflowRuns[0].head_repository.full_name = "fork/rhwp"; }],
  ["stale base", d => { d.mergeBaseSha = docs; }],
  ["non-doc code tail", d => { d.prCommits[1].files = [file("src/new.rs")]; }],
  ["enforcement change", d => { d.pullFiles.push(file(".github/workflows/ci.yml")); }],
  ["head tree mismatch", d => { d.sourceCommit.commit.tree.sha = docs; }],
]) {
  test(`bridge cannot bypass ${name}`, () => {
    const data = provenFixture();
    mutate(data);
    assert.equal(evaluateTrustedPostMergeReuse(data).reuse, false);
  });
}

test("only one ordered current-base bridge is traversed", () => {
  const data = fixture(), pr = data.pullRequests[0];
  assert.equal(currentBaseReviewBridgeSource(data.prCommits[2], base), docs);
  assert.equal(currentBaseReviewBridgeSource(commit(bridge, [base, docs], "mydocs/a"), base), "");
  assert.equal(currentBaseReviewBridgeSource(commit(bridge, [base, base], "mydocs/a"), base), "");
  assert.equal(currentBaseReviewBridgeSource(commit(bridge, [docs, base, code], "mydocs/a"), base), "");
  data.prCommits[1].parents.push({ sha: base });
  assert.equal(selectTrustedPostMergeCandidate(pr, data.prCommits, base), null);
});

test("bridge source parent must connect to the preceding commit", () => {
  const data = fixture();
  data.prCommits[2].parents[0].sha = "a".repeat(40);
  assert.equal(selectTrustedPostMergeCandidate(data.pullRequests[0], data.prCommits, base), null);
});

function gitFixture(t, changedPath = "mydocs/pr/review.md") {
  const directory = mkdtempSync(path.join(tmpdir(), "postmerge-tree-"));
  t.after(() => rmSync(directory, { recursive: true, force: true }));
  const git = (...args) => execFileSync("git", ["-C", directory, ...args], {
    encoding: "utf8", stdio: ["ignore", "pipe", "pipe"],
  }).trim();
  const write = (name, content) => {
    const target = path.join(directory, name);
    mkdirSync(path.dirname(target), { recursive: true });
    writeFileSync(target, content);
  };
  const save = () => {
    git("add", ".");
    git("-c", "commit.gpgsign=false", "commit", "--quiet", "--allow-empty", "-m", "fixture");
    return git("rev-parse", "HEAD");
  };
  const tree = () => git("rev-parse", "HEAD^{tree}");
  git("init", "--quiet");
  git("config", "user.name", "Test");
  git("config", "user.email", "test@example.invalid");
  write("src/lib.rs", "base\n");
  const baseSha = save();
  write("src/lib.rs", "reviewed code\n");
  const candidateSha = save();
  const testedMergeSha = git("commit-tree", tree(), "-p", baseSha, "-p", candidateSha, "-m", "tested");
  write(changedPath, "final change\n");
  const sourceSha = save();
  const bridgeSha = git("commit-tree", tree(), "-p", sourceSha, "-p", baseSha, "-m", "bridge");
  const mergeSha = git("commit-tree", tree(), "-p", baseSha, "-m", "squash");
  return { directory, git, identity: { baseSha, bridgeSha, mergeSha, candidateSha, testedMergeSha } };
}

test("Git object proof accepts review documents, including spaces and newline paths", t => {
  const { directory, identity } = gitFixture(t, "mydocs/pr/review with space\nand newline.md");
  const result = verifyPostMergeReviewBridgeTree(directory, identity);
  assert.equal(result.mergeSha, identity.mergeSha);
  assert.notEqual(result.testedTreeSha, result.finalTreeSha);
});

for (const changedPath of ["src/lib.rs", "tests/cases/a.rs", ".github/workflows/ci.yml",
  "Cargo.toml", "pdf/new.pdf", "mydocs/tech/text-ir-v2.md",
  "mydocs/tech/canvaskit-parity-implementation.md"]) {
  test(`Git object proof rejects untested change: ${changedPath}`, t => {
    const { directory, identity } = gitFixture(t, changedPath);
    assert.throws(() => verifyPostMergeReviewBridgeTree(directory, identity), /non-review-tree-change/);
  });
}

test("Git object proof accepts identical trees and rejects wrong parents and invalid identities", t => {
  const { directory, identity, git } = gitFixture(t);
  const testedTree = git("show", "-s", "--format=%T", identity.testedMergeSha);
  const mergeSha = git("commit-tree", testedTree, "-p", identity.baseSha, "-m", "identical");
  const result = verifyPostMergeReviewBridgeTree(directory, { ...identity, mergeSha });
  assert.equal(result.testedTreeSha, result.finalTreeSha);
  assert.throws(() => verifyPostMergeReviewBridgeTree(directory, { ...identity, baseSha: identity.candidateSha }), /base-mismatch/);
  assert.throws(() => verifyPostMergeReviewBridgeTree(directory, { ...identity, candidateSha: identity.baseSha }), /base-mismatch/);
  assert.throws(() => verifyPostMergeReviewBridgeTree(directory, { ...identity, mergeSha: "--help" }), /invalid-review-bridge-identity/);
  assert.throws(() => verifyPostMergeReviewBridgeTree(directory, { ...identity, mergeSha: "a".repeat(40) }));
});
