import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import test from "node:test";
import { classifyReviewOnlyCommit, fullLaneWorkflowJobsAreGreen } from "../verify-trusted-postmerge-ci-reuse.mjs";

const base = "b".repeat(40);
const code = "c".repeat(40);
const bridge = "d".repeat(40);
const head = "e".repeat(40);
const workflows = ["ci.yml", "codeql.yml", "render-diff.yml", "adapter-diff.yml", "proptest-roundtrip.yml"];
const AsyncFunction = Object.getPrototypeOf(async function () {}).constructor;
const scripts = new Map(workflows.map(name => {
  const text = readFileSync(new URL(`../../.github/workflows/${name}`, import.meta.url), "utf8");
  const blocks = [...text.matchAll(/^          script: \|\n((?:            .*\n|\n)+)/gm)];
  const source = blocks.map(match => match[1].replace(/^ {12}/gm, ""))
    .find(block => block.includes("const reviewOnlyCandidates = []"));
  assert.ok(source, `preflight script missing: ${name}`);
  return [name, source];
}));

async function preflight(name, options = {}) {
  const pr = { number: 6990, created_at: "2026-09-10T09:00:00Z",
    base: { ref: "devel", sha: base },
    head: { sha: head, ref: "review/6990", repo: { id: 123 } } };
  const docs = { filename: "mydocs/orders/20260910.md", status: "modified" };
  const source = { filename: "src/lib.rs", status: "modified" };
  const commits = [
    { sha: code, parents: [{ sha: "a".repeat(40) }], files: [source] },
    { sha: bridge, parents: [{ sha: code }, { sha: base }], files: [docs] },
    { sha: head, parents: [{ sha: bridge }], files: [docs] },
  ];
  const candidate = options.olderCandidate ? code : bridge;
  const runs = [{ id: 10, path: `.github/workflows/${name}`, event: "pull_request",
    head_sha: candidate, head_branch: pr.head.ref, head_repository: pr.head.repo,
    created_at: "2026-09-10T10:00:00Z", run_started_at: "2026-09-10T10:00:00Z",
    status: "completed", conclusion: "success" }];
  const names = name === "ci.yml" ? ["Build & Test"]
    : name === "codeql.yml" ? ["Analyze (javascript-typescript)", "Analyze (python)", "Analyze (rust)"]
    : name === "render-diff.yml" ? ["Canvas visual diff"] : [];
  const jobs = names.map((name, index) => ({ id: index + 100, name, status: "completed", conclusion: "success",
    started_at: "2026-09-10T10:01:00Z",
    steps: [{ name: `Render Diff identity PR #6990 base ${base}`, status: "completed", conclusion: "success" }] }));
  const checks = [{ id: 200, name: "CodeQL", app: { slug: "github-advanced-security" }, head_sha: candidate,
    started_at: "2026-09-10T10:01:00Z", status: "completed", conclusion: "success" }];
  const files = [source, docs];
  options.mutate?.({ commits, runs, jobs, checks, files, pr });
  const outputs = {};
  const requested = [];
  const github = { rest: {
    pulls: { listFiles: "files", listCommits: "commits" },
    actions: { listWorkflowRuns: "runs", listJobsForWorkflowRun: "jobs" },
    checks: { listForRef: "checks" },
    repos: { getCommit: async ({ ref }) => ({ data: commits.find(commit => commit.sha === ref) }) },
  }, paginate: async (method, args) => {
    if (method === "files") return files;
    if (method === "commits") return commits;
    if (method === "runs") { requested.push(args.head_sha || "all"); return runs.filter(run => !args.head_sha || args.head_sha === run.head_sha); }
    if (method === "jobs") return jobs;
    if (method === "checks") return checks.filter(check => check.head_sha === args.ref);
    throw new Error(`Unexpected API ${method}`);
  } };
  const warnings = [];
  await new AsyncFunction("github", "context", "core", scripts.get(name))(github,
    { eventName: "pull_request", repo: { owner: "edwardkim", repo: "rhwp" }, payload: { pull_request: pr } },
    { setOutput: (key, value) => { outputs[key] = value; }, info: () => {}, warning: message => warnings.push(message) });
  return { outputs, requested, warnings };
}
const reused = (name, outputs) => name === "adapter-diff.yml" ? outputs.adapter_required === "false" : outputs.fast_pass === "true";
for (const name of workflows) {
  test(`#6990 ${name}: green base-update merge remains a candidate behind docs`, async () => {
    const { outputs, warnings } = await preflight(name);
    assert.equal(reused(name, outputs), true, JSON.stringify({ outputs, warnings }));
    assert.equal(outputs.base_merge_sha, undefined);
    assert.deepEqual(warnings, []);
  });
  test(`${name}: earlier green source still requires independent merge-tree proof`, async () => {
    const { outputs } = await preflight(name, { olderCandidate: true });
    assert.equal(name === "adapter-diff.yml" ? outputs.adapter_required : outputs.fast_pass, "pending-base-merge-tree", JSON.stringify(outputs));
    assert.equal(outputs.candidate_sha, code);
    assert.equal(outputs.base_merge_sha, bridge);
  });
  for (const [label, mutate] of [
    ["failed merge run", data => { data.runs[0].conclusion = "failure"; }],
    ["cancelled merge run", data => { data.runs[0].conclusion = "cancelled"; }],
    ["pending merge run", data => { data.runs[0].status = "in_progress"; data.runs[0].conclusion = null; }],
    ["missing merge run", data => { data.runs.length = 0; }],
    ["wrong repository", data => { data.runs[0].head_repository = { id: 999 }; }],
    ["wrong branch", data => { data.runs[0].head_branch = "unrelated"; }],
    ["code tail", data => { data.commits[2].files = [{ filename: "src/new.rs", status: "added" }]; }],
    ["disconnected tail", data => { data.commits[2].parents[0].sha = "f".repeat(40); }],
    ["wrong base", data => { data.pr.base.sha = "f".repeat(40); }],
    ["second bridge", data => { data.commits[0].parents.push({ sha: base }); }],
    ["execution policy change", data => { data.files.push({ filename: `.github/workflows/${name}`, status: "modified" }); }],
  ]) {
    test(`${name}: fail closed on ${label}`, async () => {
      const { outputs } = await preflight(name, { mutate });
      assert.equal(reused(name, outputs), false, JSON.stringify(outputs));
    });
  }
}

test("Render Diff merge candidate must match current base, not a prior base", async () => {
  const { outputs } = await preflight("render-diff.yml", { mutate: data => {
    data.jobs[0].steps[0].name = `Render Diff identity PR #6990 base ${"a".repeat(40)}`;
  } });
  assert.equal(outputs.fast_pass, "false");
  assert.equal(outputs.reason, "canvas-visual-diff-identity-mismatch");
});

for (const [name, required] of Object.entries({
  "ci.yml": ["CI preflight", "Build & Test", "Lint (fmt, clippy, WASM check)"],
  "codeql.yml": ["CodeQL preflight", "Analyze (javascript-typescript)", "Analyze (python)", "Analyze (rust)"],
  "adapter-diff.yml": ["adapter inter-diff preflight", "adapter inter-diff"],
  "proptest-roundtrip.yml": ["Proptest preflight", "prop roundtrip"],
})) {
  test(`${name}: required worker evidence rejects missing, skipped, duplicate and non-green jobs`, () => {
    const jobs = required.map(name => ({ name, status: "completed", conclusion: "success" }));
    assert.equal(fullLaneWorkflowJobsAreGreen(name, jobs), true);
    assert.equal(fullLaneWorkflowJobsAreGreen(name, jobs.slice(1)), false);
    assert.equal(fullLaneWorkflowJobsAreGreen(name, [...jobs, jobs[0]]), false);
    for (const conclusion of ["skipped", "neutral", "failure", "cancelled"]) {
      assert.equal(fullLaneWorkflowJobsAreGreen(name, [{ ...jobs[0], conclusion }, ...jobs.slice(1)]), false);
    }
    assert.equal(fullLaneWorkflowJobsAreGreen(name, [{ ...jobs[0], status: "in_progress" }, ...jobs.slice(1)]), false);
  });
}
test("a source rename into mydocs is not review-only evidence", () => {
  assert.equal(classifyReviewOnlyCommit({ sha: head, parents: [{ sha: bridge }],
    files: [{ filename: "mydocs/a.md", previous_filename: "src/lib.rs", status: "renamed" }] }).kind, "code");
});
