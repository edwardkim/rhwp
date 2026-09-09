import assert from "node:assert/strict";
import test from "node:test";
import { mkdtempSync, mkdirSync, writeFileSync, readFileSync, copyFileSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { execFileSync } from "node:child_process";
import { createRequire } from "node:module";
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
  write("mydocs/pr/tail.md", "trailing review\n");
  save();
  const headSha = git("commit-tree", tree(), "-p", bridgeSha, "-m", "tail");
  const mergeSha = git("commit-tree", tree(), "-p", baseSha, "-m", "squash");
  return { directory, git, sourceSha, headSha,
    identity: { baseSha, bridgeSha, mergeSha, candidateSha, testedMergeSha } };
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

test("Git object proof works with the runner's depth=1 fetched commits", t => {
  const { directory, identity, git } = gitFixture(t);
  const shallow = mkdtempSync(path.join(tmpdir(), "postmerge-shallow-"));
  t.after(() => rmSync(shallow, { recursive: true, force: true }));
  const command = (...args) => execFileSync("git", ["-C", shallow, ...args], {
    encoding: "utf8", stdio: ["ignore", "pipe", "pipe"],
  }).trim();
  command("init", "--quiet");
  command("fetch", "--no-tags", "--depth=1", "--no-write-fetch-head", directory,
    identity.mergeSha, identity.bridgeSha, identity.testedMergeSha);
  assert.equal(command("rev-parse", "--is-shallow-repository"), "true");
  assert.equal(command("show", "-s", "--format=%P", identity.bridgeSha), "");
  const proof = verifyPostMergeReviewBridgeTree(shallow, identity);
  assert.equal(proof.testedTreeSha, git("rev-parse", `${identity.testedMergeSha}^{tree}`));
});

// Execute the actual github-script body over real Git objects, with only API/network calls mocked.
const workflow = readFileSync(new URL("../../.github/workflows/trusted-postmerge-ci-reuse.yml", import.meta.url), "utf8");
const script = workflow.split("- name: Evaluate exact PR workflow evidence")[1]
  .split("script: |\n")[1].split("\n").map(line => line.replace(/^ {12}/, "")).join("\n");
const AsyncFunction = Object.getPrototypeOf(async function () {}).constructor;
const require = createRequire(import.meta.url);

async function runWorkflow(t, workflowFile = "ci.yml", options = {}) {
  const f = gitFixture(t, options.changedPath);
  const { baseSha, candidateSha, bridgeSha, mergeSha, testedMergeSha } = f.identity;
  const commits = new Map();
  for (const sha of [baseSha, candidateSha, f.sourceSha, bridgeSha, f.headSha, mergeSha, testedMergeSha]) {
    const [parents, tree] = f.git("show", "-s", "--format=%P%n%T", sha).split("\n");
    commits.set(sha, { sha, parents: parents.split(" ").map(sha => ({ sha })),
      commit: { tree: { sha: tree } }, files: [file(
        sha === candidateSha ? "src/lib.rs" : "mydocs/pr/review.md",
      )] });
  }
  const data = fixture();
  const pr = { ...data.pullRequests[0], changed_files: 2, merge_commit_sha: mergeSha,
    head: { ...data.pullRequests[0].head, sha: f.headSha } };
  const summary = { ...pr };
  delete summary.changed_files;
  const runs = data.workflowRuns.map((run, index) => ({ ...run,
    head_sha: index === 0 ? candidateSha : f.headSha,
  }));
  const headers = [candidateSha, f.sourceSha, bridgeSha, f.headSha].map(sha => ({ sha }));
  const artifacts = ["b", "c", "d"].map(label => ({
    name: `nextest-target-durations-${fullRunId}-${label}`, expired: false,
  }));
  artifacts.push({ name: `trusted-postmerge-merge-tree-v1-${testedMergeSha}-${commits.get(testedMergeSha).commit.tree.sha}`,
    expired: false });
  const state = { pr, runs, headers, artifacts, commits };
  options.mutate?.(state);
  const apiCalls = [], gitCalls = [], outputs = {}, warnings = [];
  const endpoint = name => name;
  const github = { rest: {
    repos: {
      getCommit: async ({ ref }) => {
        apiCalls.push(["getCommit", ref]);
        assert.ok(commits.has(ref), `unknown commit ${ref}`);
        return { data: commits.get(ref) };
      },
      compareCommits: async () => ({ data: { merge_base_commit: { sha: baseSha } } }),
      listPullRequestsAssociatedWithCommit: endpoint("associated"),
    },
    pulls: { get: async () => ({ data: pr }), listFiles: endpoint("files"), listCommits: endpoint("commits") },
    actions: { listWorkflowRuns: endpoint("runs"), listWorkflowRunArtifacts: endpoint("artifacts"),
      listJobsForWorkflowRun: endpoint("jobs") },
  }, paginate: async (method, args) => {
    apiCalls.push([method, args]);
    switch (method) {
      case "associated": return [summary];
      case "files": return [file("src/lib.rs"), file("mydocs/pr/review.md")];
      case "commits": return headers;
      case "runs": return runs;
      case "artifacts": return args.run_id === fullRunId ? artifacts : [];
      case "jobs": return ["rust", "javascript-typescript", "python"].map(language => ({
        name: `Analyze (${language})`, status: "completed",
        conclusion: args.run_id === fullRunId ? "success" : "skipped",
      }));
      default: throw new Error(`unexpected API: ${method}`);
    }
  } };
  mkdirSync(path.join(f.directory, "scripts"));
  for (const name of ["verify-trusted-postmerge-ci-reuse.mjs", "ci-impact-classifier.cjs"]) {
    copyFileSync(new URL(`../${name}`, import.meta.url), path.join(f.directory, "scripts", name));
  }
  const testRequire = name => name === "node:child_process" ? {
    execFileSync: (command, args, settings) => {
      gitCalls.push([command, args]);
      assert.equal(command, "git");
      assert.ok(["cat-file", "fetch"].includes(args[0]));
      if (options.fetch && args[0] === "cat-file") throw new Error("object not present yet");
      if (args[0] === "fetch") {
        assert.deepEqual(args.slice(0, 7), ["fetch", "--no-tags", "--filter=blob:none", "--depth=1",
          "--no-write-fetch-head", "origin", mergeSha]);
        assert.deepEqual(args.slice(7), [bridgeSha, testedMergeSha]);
        if (options.fetch === "failure") throw new Error("fetch failed");
        return Buffer.alloc(0);
      }
      return execFileSync(command, args, settings);
    },
  } : require(name);
  await new AsyncFunction("require", "github", "context", "core", "process", script)(
    testRequire, github, { repo: { owner: "edwardkim", repo: "rhwp" } },
    { setOutput: (key, value) => { outputs[key] = value; }, warning: message => warnings.push(message), info: () => {} },
    { env: { GITHUB_WORKSPACE: f.directory, GITHUB_REPOSITORY: "edwardkim/rhwp", WORKFLOW_FILE: workflowFile,
      CALLER_EVENT_NAME: "push", CALLER_REF: "refs/heads/devel", CALLER_SHA: mergeSha,
      REQUIRE_DURATION_ARTIFACTS: workflowFile === "ci.yml" ? "true" : "false" } },
  );
  return { outputs, apiCalls, gitCalls, warnings, candidateSha };
}

for (const workflowFile of ["ci.yml", "codeql.yml"]) {
  test(`actual ${workflowFile} collector traverses bridge and reuses the tested full run`, async t => {
    const result = await runWorkflow(t, workflowFile);
    assert.deepEqual(result.outputs, { reuse: "true",
      reason: "current-base-review-bridge-green-pr-workflow-reused", source_run_id: String(fullRunId),
      refresh_duration_data: workflowFile === "ci.yml" ? "true" : "false" });
    assert.ok(result.apiCalls.some(([method, ref]) => method === "getCommit" && ref === result.candidateSha));
    assert.deepEqual(result.warnings, []);
  });
}

test("actual workflow fetches only missing immutable objects without checking out PR code", async t => {
  const result = await runWorkflow(t, "ci.yml", { fetch: "success" });
  assert.equal(result.outputs.reuse, "true");
  assert.equal(result.gitCalls.filter(([, args]) => args[0] === "fetch").length, 1);
});

for (const [name, options] of [
  ["fetch failure", { fetch: "failure" }],
  ["code conflict resolution", { changedPath: "src/lib.rs" }],
  ["expired immutable artifact", { mutate: d => { d.artifacts.at(-1).expired = true; } }],
  ["missing duration artifact", { mutate: d => { d.artifacts.shift(); } }],
  ["failed final head", { mutate: d => { d.runs[1].conclusion = "failure"; } }],
  ["truncated file listing", { mutate: d => { d.pr.changed_files += 1; } }],
  ["PR commit API cap", { mutate: d => { d.headers.push(...Array(246).fill(d.headers[0])); } }],
  ["PR detail identity changed", { mutate: d => { d.pr.merge_commit_sha = "f".repeat(40); } }],
]) {
  test(`actual workflow preserves full lane on ${name}`, async t => {
    const { outputs } = await runWorkflow(t, "ci.yml", options);
    assert.equal(outputs.reuse, "false");
    assert.equal(outputs.source_run_id, "");
    assert.equal(outputs.refresh_duration_data, "false");
  });
}
