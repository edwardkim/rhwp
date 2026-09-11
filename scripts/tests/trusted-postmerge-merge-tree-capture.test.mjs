import assert from "node:assert/strict";
import test from "node:test";
import { readFileSync } from "node:fs";

// Execute the deployed inline step itself, not a copy of its implementation.
const workflow = readFileSync(new URL("../../.github/workflows/trusted-postmerge-ci-reuse.yml", import.meta.url), "utf8");
const capture = workflow.split("- name: Capture PR merge-tree evidence")[1]
  .split("- name: Upload PR merge-tree evidence")[0].split("script: |\n")[1]
  .split("\n").map(line => line.replace(/^ {12}/, "")).join("\n");
const AsyncFunction = Object.getPrototypeOf(async function () {}).constructor;
const execute = new AsyncFunction("github", "context", "core", "process", "require", capture);
const oldBase = "a".repeat(40), testedBase = "b".repeat(40), currentBase = "c".repeat(40);
const head = "d".repeat(40), merge = "e".repeat(40), tree = "f".repeat(40);

function fixture({ stale = false, advanced = false, fork = true } = {}) {
  return {
    env: { CALLER_SHA: merge, CALLER_REF: "refs/pull/42/merge", WORKFLOW_FILE: "ci.yml",
      GITHUB_REPOSITORY: "owner/repo", GITHUB_REPOSITORY_ID: "10", GITHUB_RUN_ATTEMPT: "2" },
    context: { repo: { owner: "owner", repo: "repo" }, runId: 123, payload: { pull_request: {
      number: 42, base: { sha: stale ? oldBase : testedBase, ref: "devel", repo: { id: 10, full_name: "owner/repo" } },
      head: { sha: head, repo: { id: fork ? 20 : 10, full_name: fork ? "fork/repo" : "owner/repo", fork } },
    } } },
    commit: { sha: merge, parents: [{ sha: testedBase }, { sha: head }], commit: { tree: { sha: tree } } },
    branch: { name: "devel", commit: { sha: advanced ? currentBase : testedBase } },
    comparison: (base) => ({ status: "ahead", base_commit: { sha: base }, merge_base_commit: { sha: base } }),
  };
}

async function run(f) {
  const files = [], warnings = [], info = [], outputs = {}, calls = [];
  const github = { rest: { repos: {
    getCommit: async args => { assert.equal(args.ref, f.env.CALLER_SHA); return { data: f.commit }; },
    compareCommits: async args => {
      assert.equal(args.owner, "owner"); assert.equal(args.repo, "repo");
      calls.push([args.base, args.head]);
      return { data: f.comparison(args.base, args.head) };
    },
    getBranch: async args => {
      assert.deepEqual(args, { owner: "owner", repo: "repo", branch: "devel" });
      if (f.branchError) throw new Error("API unavailable");
      return { data: f.branch };
    },
  } } };
  await execute(github, f.context, {
    warning: x => warnings.push(x), info: x => info.push(x), setOutput: (k, v) => { outputs[k] = v; },
  }, { env: f.env }, name => {
    assert.equal(name, "node:fs");
    return { writeFileSync: (p, s) => { assert.equal(p, "trusted-postmerge-merge-tree.json"); files.push(JSON.parse(s)); } };
  });
  return { files, warnings, info, outputs, calls };
}

for (const fork of [true, false]) {
  for (const stale of [true, false]) {
    test(`실제 capture: fork=${fork}, 오래된 event base=${stale}`, async () => {
      const f = fixture({ fork, stale, advanced: true });
      const r = await run(f);
      assert.equal(r.warnings.length, 0);
      assert.equal(r.files.length, 1);
      assert.deepEqual(r.files[0].parents, [testedBase, head]);
      assert.equal(r.files[0].event_base_sha, stale ? oldBase : testedBase);
      assert.equal(r.files[0].merge_sha, merge);
      assert.equal(r.files[0].tree_sha, tree);
      assert.equal(r.files[0].run_id, "123");
      assert.equal(r.files[0].run_attempt, 2);
      assert.equal(r.files[0].head_repository_id, fork ? 20 : 10);
      assert.equal(r.outputs.artifact_name, fork
        ? `trusted-postmerge-fork-merge-tree-v1-42-10-20-2-${merge}-${tree}`
        : `trusted-postmerge-merge-tree-v1-${merge}-${tree}`);
      assert.deepEqual(r.calls, stale ? [[oldBase, testedBase], [testedBase, currentBase]] : []);
    });
  }
}

test("현재 devel과 tested base가 같으면 첫 계보만 대조한다", async () => {
  const r = await run(fixture({ stale: true }));
  assert.equal(r.files.length, 1);
  assert.deepEqual(r.calls, [[oldBase, testedBase]]);
});

for (const [name, mutate] of Object.entries({
  "실행 SHA 불일치": f => { f.commit.sha = head; },
  "부모 1개": f => { f.commit.parents.pop(); },
  "부모 3개": f => { f.commit.parents.push({ sha: oldBase }); },
  "잘못된 첫 부모": f => { f.commit.parents[0].sha = "bad"; },
  "다른 head": f => { f.commit.parents[1].sha = oldBase; },
  "잘못된 tree": f => { f.commit.commit.tree.sha = "bad"; },
  "다른 PR ref": f => { f.env.CALLER_REF = "refs/pull/43/merge"; },
  "fork 자체 실행": f => { f.env.GITHUB_REPOSITORY = "fork/repo"; },
  "upstream ID 불일치": f => { f.env.GITHUB_REPOSITORY_ID = "30"; },
  "head repository 누락": f => { delete f.context.payload.pull_request.head.repo.id; },
  "fork identity 거짓": f => { f.context.payload.pull_request.head.repo.fork = false; },
  "attempt 누락": f => { delete f.env.GITHUB_RUN_ATTEMPT; },
  "다른 base branch": f => { f.context.payload.pull_request.base.ref = "main"; },
  "event base 누락": f => { delete f.context.payload.pull_request.base.sha; },
  "base 역방향": f => { f.comparison = () => ({ status: "behind" }); },
  "base 분기": f => { f.comparison = () => ({ status: "diverged" }); },
  "다른 비교 base": f => { f.comparison = base => ({ status: "ahead", base_commit: { sha: head }, merge_base_commit: { sha: base } }); },
  "다른 공통 조상": f => { f.comparison = base => ({ status: "ahead", base_commit: { sha: base }, merge_base_commit: { sha: head } }); },
  "비교 API 오류": f => { f.comparison = () => { throw new Error("API unavailable"); }; },
  "devel 조회 오류": f => { f.branchError = true; },
  "다른 branch 응답": f => { f.branch.name = "main"; },
  "devel SHA 누락": f => { delete f.branch.commit.sha; },
  "devel 밖 tested base": f => { f.comparison = base => ({ status: base === testedBase ? "diverged" : "ahead", base_commit: { sha: base }, merge_base_commit: { sha: base } }); },
})) {
  test(`capture fail-closed: ${name}`, async () => {
    const f = fixture({ stale: true, advanced: true }); mutate(f);
    const r = await run(f);
    assert.equal(r.files.length, 0);
    assert.equal(r.outputs.artifact_name, undefined);
    assert.ok(r.warnings.length > 0);
  });
}
