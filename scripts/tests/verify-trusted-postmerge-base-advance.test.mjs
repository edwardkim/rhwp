import assert from "node:assert/strict";
import test from "node:test";
import { execFileSync } from "node:child_process";
import { mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { evaluateTrustedPostMergeReuse, verifyReviewOnlyBaseAdvance } from "../verify-trusted-postmerge-ci-reuse.mjs";

function fixture(t, advances = [{ "mydocs/pr/new.md": "review" }], finalChanges = {}) {
  const dir = mkdtempSync(join(tmpdir(), "rhwp-base-advance-"));
  t.after(() => rmSync(dir, { recursive: true, force: true }));
  const git = (args, input) => execFileSync("git", ["-C", dir, ...args], {
    encoding: "utf8", input, stdio: ["pipe", "pipe", "pipe"],
    env: { ...process.env, GIT_AUTHOR_NAME: "Fixture", GIT_AUTHOR_EMAIL: "fixture@example.invalid",
      GIT_COMMITTER_NAME: "Fixture", GIT_COMMITTER_EMAIL: "fixture@example.invalid" },
  }).trim();
  git(["init", "--quiet"]);
  const tree = (parent, changes) => {
    git(parent ? ["read-tree", `${parent}^{tree}`] : ["read-tree", "--empty"]);
    for (const [path, value] of Object.entries(changes)) {
      if (value === null) git(["update-index", "--force-remove", "--", path]);
      else {
        const blob = git(["hash-object", "-w", "--stdin"], typeof value === "string" ? value : value.text);
        git(["update-index", "--add", "--cacheinfo", `${value.mode || "100644"},${blob},${path}`]);
      }
    }
    return git(["write-tree"]);
  };
  const commit = (sha, parents) => git(["commit-tree", sha, ...parents.flatMap(p => ["-p", p]), "-m", "fixture"]);
  const old = commit(tree(null, { "src/lib.rs": "before", "mydocs/pr/old.md": "old" }), []);
  const headTree = tree(old, { "src/lib.rs": "after" });
  const head = commit(headTree, [old]);
  const tested = commit(headTree, [old, head]);
  let base = old;
  for (const changes of advances) base = commit(tree(base, changes), [base]);
  const finalTree = tree(base, { "src/lib.rs": "after", ...finalChanges });
  const merge = commit(finalTree, [base, head]);
  const identity = { baseSha: base, mergeSha: merge, candidateSha: head, testedMergeSha: tested };
  return { dir, git, tree, commit, old, head, headTree, tested, base, merge, finalTree, identity };
}

test("문서 base 전진과 문서 rename은 shallow raw 객체에서도 허용한다", t => {
  const f = fixture(t, [{ "mydocs/pr/old.md": null, "mydocs/pr/archive.md": "old" }]);
  const proof = verifyReviewOnlyBaseAdvance(f.dir, f.identity);
  assert.equal(proof.testedBaseSha, f.old);
  assert.equal(proof.finalTreeSha, f.finalTree);
  const shallow = mkdtempSync(join(tmpdir(), "rhwp-base-shallow-"));
  t.after(() => rmSync(shallow, { recursive: true, force: true }));
  execFileSync("git", ["-C", shallow, "init", "--quiet"]);
  execFileSync("git", ["-C", shallow, "fetch", "--quiet", "--depth=1", f.dir,
    f.merge, f.tested, f.base, f.old]);
  assert.deepEqual(verifyReviewOnlyBaseAdvance(shallow, f.identity), proof);
});

for (const path of ["src/lib.rs", ".github/workflows/ci.yml", "tests/case.rs", "samples/sample.hwp",
  "mydocs/state.json", "mydocs/tech/text-ir-v2.md", "mydocs/tech/canvaskit-parity-implementation.md"]) {
  test(`base의 비허용 변경은 거부한다: ${path}`, t => {
    const f = fixture(t, [{ [path]: "changed" }]);
    assert.throws(() => verifyReviewOnlyBaseAdvance(f.dir, f.identity), /non-review-change/);
  });
}

test("source 변경을 되돌려 최종 diff에서 숨겨도 거부한다", t => {
  const f = fixture(t, [{ "src/lib.rs": "unsafe" }, { "src/lib.rs": "before" }]);
  assert.throws(() => verifyReviewOnlyBaseAdvance(f.dir, f.identity), /non-review-change/);
});

test("rename의 원래 비문서 경로와 Markdown symlink 및 executable을 거부한다", t => {
  for (const changes of [
    { "src/lib.rs": null, "mydocs/source.md": "before" },
    { "mydocs/link.md": { mode: "120000", text: "../../src/lib.rs" } },
    { "mydocs/executable.md": { mode: "100755", text: "script" } },
  ]) {
    const f = fixture(t, [changes]);
    assert.throws(() => verifyReviewOnlyBaseAdvance(f.dir, f.identity), /non-review-change/);
  }
});

test("base는 문서뿐이어도 최종 병합의 코드 충돌 보정은 거부한다", t => {
  const f = fixture(t, undefined, { "src/lib.rs": "unreviewed resolution" });
  assert.throws(() => verifyReviewOnlyBaseAdvance(f.dir, f.identity), /non-review-change/);
});

test("다른 head와 누락 객체 및 전진하지 않은 base를 거부한다", t => {
  const f = fixture(t);
  assert.throws(() => verifyReviewOnlyBaseAdvance(f.dir, { ...f.identity, candidateSha: f.old }), /parent-mismatch/);
  assert.throws(() => verifyReviewOnlyBaseAdvance(f.dir, { ...f.identity, testedMergeSha: "f".repeat(40) }));
  const same = fixture(t, []);
  assert.throws(() => verifyReviewOnlyBaseAdvance(same.dir, same.identity), /parent-mismatch/);
});

test("first-parent 조상이 아닌 base와 64단계 초과 이력을 거부한다", t => {
  const f = fixture(t);
  const unrelated = f.commit(f.tree(null, { "mydocs/other.md": "other" }), []);
  const badBase = f.commit(f.tree(unrelated, { "mydocs/new.md": "new" }), [unrelated]);
  const badMerge = f.commit(f.finalTree, [badBase, f.head]);
  assert.throws(() => verifyReviewOnlyBaseAdvance(f.dir, {
    ...f.identity, baseSha: badBase, mergeSha: badMerge,
  }), /history-unavailable/);
  const long = fixture(t, Array.from({ length: 65 }, (_, i) => ({ "mydocs/count.md": String(i) })));
  assert.throws(() => verifyReviewOnlyBaseAdvance(long.dir, long.identity), /history-limit/);
});

test("최종 head 성공 및 신원과 tree 증거가 모두 맞을 때만 재사용한다", t => {
  const f = fixture(t);
  const proof = verifyReviewOnlyBaseAdvance(f.dir, f.identity);
  const repository = { id: 1, full_name: "edwardkim/rhwp" };
  const fork = { id: 2, full_name: "contributor/rhwp", fork: true };
  const run = { id: 10, run_attempt: 2, event: "pull_request", status: "completed", conclusion: "success",
    repository, head_repository: fork, head_branch: "feature", head_sha: f.head,
    path: ".github/workflows/codeql.yml", pull_requests: [],
    created_at: "2026-09-09T00:01:00Z", updated_at: "2026-09-09T00:02:00Z" };
  const input = { eventName: "push", ref: "refs/heads/devel", repository: repository.full_name,
    repositoryId: 1, workflowFile: "codeql.yml", mergeSha: f.merge,
    mergeCommit: { sha: f.merge, parents: [{ sha: f.base }, { sha: f.head }], commit: { tree: { sha: f.finalTree } } },
    sourceCommit: { sha: f.head, commit: { tree: { sha: f.headTree } } }, mergeBaseSha: f.old,
    pullRequests: [{ number: 42, state: "closed", merged_at: "2026-09-09T00:03:00Z",
      created_at: "2026-09-09T00:00:00Z", merge_commit_sha: f.merge,
      base: { ref: "devel", repo: repository }, head: { sha: f.head, ref: "feature", repo: fork } }],
    pullFiles: [{ filename: "src/lib.rs", status: "modified" }],
    prCommits: [{ sha: f.head, parents: [{ sha: f.old }], files: [{ filename: "src/lib.rs", status: "modified" }] }],
    workflowRuns: [run], fullLaneRunIds: ["10"],
    mergeTreeEvidenceByRunId: { 10: { sha: f.tested, parents: [f.old, f.head], treeSha: f.headTree,
      pullNumber: 42, repositoryId: 1, headRepositoryId: 2, runAttempt: 2 } },
    reviewOnlyBaseAdvanceByRunId: { 10: proof } };
  assert.equal(evaluateTrustedPostMergeReuse(input).reason, "review-only-base-advance-final-head-green-pr-workflow-reused");
  const reject = modify => { const copy = structuredClone(input); modify(copy); assert.equal(evaluateTrustedPostMergeReuse(copy).reuse, false); };
  reject(x => { x.reviewOnlyBaseAdvanceByRunId = {}; });
  for (const field of Object.keys(proof)) reject(x => { x.reviewOnlyBaseAdvanceByRunId[10][field] = "f".repeat(40); });
  for (const field of ["pullNumber", "repositoryId", "headRepositoryId", "runAttempt"]) reject(x => { x.mergeTreeEvidenceByRunId[10][field] = 99; });
  for (const conclusion of ["failure", "cancelled", "skipped"]) reject(x => { x.workflowRuns[0].conclusion = conclusion; });
  reject(x => { x.workflowRuns[0].status = "in_progress"; });
  reject(x => { x.workflowRuns[0].updated_at = "2026-09-09T00:04:00Z"; });
  reject(x => { x.fullLaneRunIds = []; });
  reject(x => { x.pullFiles = [{ filename: ".github/workflows/ci.yml" }]; });
});
