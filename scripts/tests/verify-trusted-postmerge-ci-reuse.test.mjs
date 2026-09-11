import "./verify-trusted-postmerge-green-merge.test.mjs";
import assert from "node:assert/strict";
import "./verify-trusted-postmerge-base-advance.test.mjs";
import test from "node:test";
import { execFileSync } from "node:child_process";
import { mkdtempSync, mkdirSync, writeFileSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

import { evaluateTrustedPostMergeReuse, verifyForkPostMergeTree } from "../verify-trusted-postmerge-ci-reuse.mjs";

const base = "1".repeat(40);
const head = "2".repeat(40);
const merge = "3".repeat(40);
const tree = "4".repeat(40);
const code = "5".repeat(40);
const oldBase = "6".repeat(40);
const testedMerge = "7".repeat(40);
const reviewed = "8".repeat(40);

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
    fullLaneRunIds: ["123"],
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
    workflowRuns: [candidate(), candidate({ id: 456, head_sha: code })],
    fullLaneRunIds: ["456"],
  }));
  assert.deepEqual(result, {
    reuse: true,
    reason: "review-tail-green-pr-workflow-reused",
    sourceRunId: "456",
    pullNumber: "42",
  });
});

test("reuses a full review-evidence candidate before a later fast-pass tail", () => {
  const result = evaluateTrustedPostMergeReuse(input({
    prCommits: [
      {
        sha: code,
        parents: [{ sha: base }],
        files: [{ filename: "src/renderer/layout.rs", status: "modified" }],
      },
      {
        sha: reviewed,
        parents: [{ sha: code }],
        files: [{ filename: "pdf/pr_6279/reference.pdf", status: "added" }],
      },
      {
        sha: head,
        parents: [{ sha: reviewed }],
        files: [{ filename: "mydocs/pr/archives/pr_6279_review.md", status: "added" }],
      },
    ],
    workflowRuns: [
      candidate({ id: 123, head_sha: head }),
      candidate({ id: 456, head_sha: reviewed }),
    ],
    fullLaneRunIds: ["456"],
    mergeTreeEvidenceByRunId: {
      456: { sha: testedMerge, parents: [base, reviewed], treeSha: tree },
    },
  }));
  assert.deepEqual(result, {
    reuse: true,
    reason: "review-tail-green-pr-workflow-reused",
    sourceRunId: "456",
    pullNumber: "42",
  });
});

test("reuses a full review-evidence candidate through a stale-base tail", () => {
  const result = evaluateTrustedPostMergeReuse(input({
    sourceCommit: { sha: head, commit: { tree: { sha: "9".repeat(40) } } },
    mergeBaseSha: oldBase,
    prCommits: [
      {
        sha: code,
        parents: [{ sha: base }],
        files: [{ filename: "src/renderer/layout.rs", status: "modified" }],
      },
      {
        sha: reviewed,
        parents: [{ sha: code }],
        files: [{ filename: "pdf/pr_6279/reference.pdf", status: "added" }],
      },
      {
        sha: head,
        parents: [{ sha: reviewed }],
        files: [{ filename: "mydocs/pr/archives/pr_6279_review.md", status: "added" }],
      },
    ],
    workflowRuns: [
      candidate({ id: 123, head_sha: head }),
      candidate({ id: 456, head_sha: reviewed }),
    ],
    fullLaneRunIds: ["456"],
    mergeTreeEvidenceByRunId: {
      123: { sha: testedMerge, parents: [base, head], treeSha: tree },
      456: { sha: "a".repeat(40), parents: [base, reviewed], treeSha: tree },
    },
  }));
  assert.deepEqual(result, {
    reuse: true,
    reason: "review-tail-green-pr-workflow-reused",
    sourceRunId: "456",
    pullNumber: "42",
  });
});

test("fails closed when an intermediate full review candidate lacks merge-tree evidence", () => {
  const result = evaluateTrustedPostMergeReuse(input({
    prCommits: [
      {
        sha: code,
        parents: [{ sha: base }],
        files: [{ filename: "src/renderer/layout.rs", status: "modified" }],
      },
      {
        sha: reviewed,
        parents: [{ sha: code }],
        files: [{ filename: "pdf/pr_6279/reference.pdf", status: "added" }],
      },
      {
        sha: head,
        parents: [{ sha: reviewed }],
        files: [{ filename: "mydocs/pr/archives/pr_6279_review.md", status: "added" }],
      },
    ],
    workflowRuns: [candidate(), candidate({ id: 456, head_sha: reviewed })],
    fullLaneRunIds: ["456"],
  }));
  assert.equal(
    result.reason,
    "review-tail-candidate-merge-tree-evidence-unavailable",
  );
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

// #6779: successful frontend-only CI must not depend on Rust timing artifacts.
const { default: frontendAssert } = await import('node:assert/strict');
const { test: frontendTest } = await import('node:test');
const {
  frontendOnlyCiRunIsReusable,
  evaluateTrustedPostMergeReuse: evaluateFrontendReuse,
} = await import('../verify-trusted-postmerge-ci-reuse.mjs');

function frontendImpact(mode = 'package') {
  return {
    classification_status: 'classified', rust_required: 'false',
    native_skia_required: 'false', frontend_mode: mode,
  };
}

function frontendJobs(mode = 'package') {
  const skipped = [
    'WASM Build', 'Resolve nextest target duration policy',
    'Lint (fmt, clippy, WASM check)', 'Native Skia tests',
    'Workflow promotion preflight', 'Refresh nextest target duration data',
    ...['a', 'b', 'c', 'd'].flatMap(label => [
      `build-test-archive-${label}`, `test-archive-${label}-shard-1`,
    ]),
  ];
  return [
    ...['trusted_postmerge_reuse / Verify trusted post-merge reuse', 'CI preflight', 'Build & Test']
      .map(name => ({ name, status: 'completed', conclusion: 'success' })),
    ...['unit', 'package'].map(lane => ({
      name: `Frontend ${lane} gates`, status: 'completed',
      conclusion: lane === mode ? 'success' : 'skipped',
    })),
    ...skipped.map(name => ({ name, status: 'completed', conclusion: 'skipped' })),
  ];
}

function frontendReuseInput(tail = false) {
  const base = 'a'.repeat(40), code = 'b'.repeat(40);
  const head = (tail ? 'e' : 'b').repeat(40), merge = 'c'.repeat(40), tree = 'd'.repeat(40);
  const files = [{ filename: 'rhwp-studio/src/ui/picture-props-apply-model.ts', status: 'modified' }];
  const run = {
    id: 6779, event: 'pull_request', head_sha: head, head_branch: 'feature',
    head_repository: { full_name: 'owner/repo' }, status: 'completed', conclusion: 'success',
    created_at: '2026-09-05T01:01:00Z', updated_at: '2026-09-05T01:02:00Z',
  };
  return {
    eventName: 'push', ref: 'refs/heads/devel', repository: 'owner/repo', mergeSha: merge,
    mergeCommit: { sha: merge, parents: [{ sha: base }, { sha: head }], commit: { tree: { sha: tree } } },
    sourceCommit: { sha: head, commit: { tree: { sha: tree } } }, mergeBaseSha: base,
    pullRequests: [{
      number: 6779, state: 'closed', created_at: '2026-09-05T01:00:00Z',
      merged_at: '2026-09-05T01:03:00Z', merge_commit_sha: merge,
      base: { ref: 'devel' }, head: { sha: head, ref: 'feature', repo: { full_name: 'owner/repo' } },
    }],
    pullFiles: files,
    prCommits: [
      { sha: code, parents: [{ sha: base }], files },
      ...(tail ? [{ sha: head, parents: [{ sha: code }], files: [{ filename: 'mydocs/pr/review.md', status: 'added' }] }] : []),
    ],
    workflowRuns: [run], fullLaneRunIds: [], frontendOnlyRunIds: ['6779'],
  };
}

for (const mode of ['unit', 'package']) {
  frontendTest(`accepts the exact ${mode} job contract with normal Rust skips`, () => {
    frontendAssert.equal(frontendOnlyCiRunIsReusable(frontendImpact(mode), frontendJobs(mode)), true);
  });
  for (const conclusion of ['skipped', 'failure', 'cancelled', 'neutral']) {
    frontendTest(`rejects ${mode} required worker ${conclusion}`, () => {
      const jobs = frontendJobs(mode);
      jobs.find(job => job.name === `Frontend ${mode} gates`).conclusion = conclusion;
      frontendAssert.equal(frontendOnlyCiRunIsReusable(frontendImpact(mode), jobs), false);
    });
  }
}

frontendTest('rejects missing, duplicate, unknown and pending jobs, including a missing aggregate', () => {
  const original = frontendJobs();
  for (const jobs of [
    original.filter(job => job.name !== 'Build & Test'),
    original.filter(job => job.name !== 'CI preflight'),
    original.filter(job => job.name !== 'test-archive-b-shard-1'),
    [...original, original[0]],
    original.map((job, index) => index ? job : { ...job, name: 'Unknown worker' }),
    original.map((job, index) => index ? job : { ...job, status: 'in_progress' }),
    original.map(job => job.name === 'build-test-archive-b' ? { ...job, conclusion: 'success' } : job),
    original.map(job => job.name === 'Build & Test' ? { ...job, conclusion: 'failure' } : job),
    null,
  ]) {
    frontendAssert.equal(frontendOnlyCiRunIsReusable(frontendImpact(), jobs), false);
  }
});

frontendTest('does not weaken Rust, Skia, unknown classification or unsupported lane requirements', () => {
  for (const impact of [
    { ...frontendImpact(), rust_required: 'true' },
    { ...frontendImpact(), native_skia_required: 'true' },
    { ...frontendImpact(), classification_status: 'full' },
    { ...frontendImpact(), rust_required: false },
    frontendImpact('none'), frontendImpact('unknown'), null,
  ]) {
    frontendAssert.equal(frontendOnlyCiRunIsReusable(impact, frontendJobs()), false);
  }
});

for (const tail of [false, true]) {
  frontendTest(`reuses exact frontend evidence without timings, review tail=${tail}`, () => {
    const result = evaluateFrontendReuse(frontendReuseInput(tail));
    frontendAssert.equal(result.reuse, true);
    frontendAssert.equal(result.refreshDurationData, false);
    frontendAssert.equal(result.sourceRunId, '6779');
    frontendAssert.equal(result.reason, tail
      ? 'review-tail-final-head-green-frontend-ci-reused' : 'exact-green-frontend-ci-reused');
  });
}

frontendTest('frontend evidence cannot bypass failed runs, provenance, tree or enforcement guards', () => {
  const mutations = [
    data => { data.workflowRuns[0].conclusion = 'failure'; },
    data => { data.workflowRuns[0].conclusion = 'cancelled'; },
    data => { data.workflowRuns[0].status = 'in_progress'; },
    data => { data.workflowRuns[0].head_sha = 'f'.repeat(40); },
    data => { data.workflowRuns[0].head_repository.full_name = 'foreign/repo'; },
    data => { data.workflowRuns[0].updated_at = '2026-09-05T01:04:00Z'; },
    data => { data.mergeCommit.commit.tree.sha = 'f'.repeat(40); },
    data => { data.mergeBaseSha = 'f'.repeat(40); },
    data => { data.pullFiles.push({ filename: '.github/workflows/ci.yml', status: 'modified' }); },
    data => { data.pullFiles.push({ filename: 'scripts/ci-impact-classifier.cjs', status: 'modified' }); },
    data => { data.frontendOnlyRunIds = []; },
  ];
  for (const mutate of mutations) {
    const data = frontendReuseInput(); mutate(data);
    frontendAssert.equal(evaluateFrontendReuse(data).reuse, false);
  }
});

frontendTest('does not reuse an earlier frontend run through an untested final review head', () => {
  const data = frontendReuseInput(true);
  data.workflowRuns[0].head_sha = 'b'.repeat(40);
  frontendAssert.equal(evaluateFrontendReuse(data).reuse, false);
});

function forkInput() {
  const data = input({ repositoryId: 10, workflowFile: "ci.yml", fullLaneRunIds: ["123"] });
  data.pullRequests[0].base.repo = { id: 10, full_name: data.repository };
  data.pullRequests[0].head.repo = { id: 20, full_name: "contributor/rhwp", fork: true };
  Object.assign(data.workflowRuns[0], {
    repository: { id: 10, full_name: data.repository },
    head_repository: { id: 20, full_name: "contributor/rhwp" },
    run_attempt: 1, path: ".github/workflows/ci.yml", pull_requests: [],
  });
  data.mergeTreeEvidenceByRunId = {
    123: { sha: testedMerge, parents: [base, head], treeSha: tree,
      pullNumber: 42, repositoryId: 10, headRepositoryId: 20, runAttempt: 1 },
  };
  return data;
}

function forkTailInput() {
  const data = forkInput();
  data.prCommits = [
    { sha: code, parents: [{ sha: base }], files: [{ filename: "src/renderer/layout.rs", status: "modified" }] },
    { sha: head, parents: [{ sha: code }], files: [{ filename: "mydocs/pr/archives/review.md", status: "added" }] },
  ];
  data.workflowRuns.push({ ...data.workflowRuns[0], id: 456, head_sha: code });
  data.fullLaneRunIds = ["456"];
  data.mergeTreeEvidenceByRunId[456] = {
    ...data.mergeTreeEvidenceByRunId[123], sha: "a".repeat(40), parents: [base, code], treeSha: "b".repeat(40),
  };
  data.forkMergeTreeEvidenceByRunId = {
    456: { baseSha: base, mergeSha: merge, candidateSha: code,
      testedMergeSha: "a".repeat(40), testedTreeSha: "b".repeat(40), finalTreeSha: tree },
  };
  return data;
}

test("fork PR은 upstream run과 PR 신원이 결합된 정확한 merge-tree 증거로 재사용한다", () => {
  const result = evaluateTrustedPostMergeReuse(forkInput());
  assert.equal(result.reuse, true);
  assert.equal(result.sourceRunId, "123");
});

test("fork 문서 trailing head는 코드 CI의 동일 base 및 최종 tree 대조 증명을 요구한다", () => {
  const result = evaluateTrustedPostMergeReuse(forkTailInput());
  assert.equal(result.reuse, true);
  assert.equal(result.sourceRunId, "456");
});

const forkRejections = {
  "fork 자체 Actions 실행": d => { d.workflowRuns[0].repository = { id: 20, full_name: "contributor/rhwp" }; },
  "다른 upstream 저장소 ID": d => { d.workflowRuns[0].repository.id = 11; },
  "다른 fork 저장소 ID": d => { d.workflowRuns[0].head_repository.id = 21; },
  "다른 fork 이름": d => { d.workflowRuns[0].head_repository.full_name = "other/rhwp"; },
  "다른 branch": d => { d.workflowRuns[0].head_branch = "other"; },
  "다른 head": d => { d.workflowRuns[0].head_sha = code; },
  "다른 workflow": d => { d.workflowRuns[0].path = ".github/workflows/untrusted.yml"; },
  "PR base 저장소 불일치": d => { d.pullRequests[0].base.repo.id = 11; },
  "fork 표시 누락": d => { delete d.pullRequests[0].head.repo.fork; },
  "다른 PR 연결": d => { d.workflowRuns[0].pull_requests = [{ number: 43 }]; },
  "다른 PR artifact": d => { d.mergeTreeEvidenceByRunId[123].pullNumber = 43; },
  "다른 base 저장소 artifact": d => { d.mergeTreeEvidenceByRunId[123].repositoryId = 11; },
  "다른 fork artifact": d => { d.mergeTreeEvidenceByRunId[123].headRepositoryId = 21; },
  "이전 attempt artifact": d => { d.workflowRuns[0].run_attempt = 2; },
  "run attempt 누락": d => { delete d.workflowRuns[0].run_attempt; },
  "신원 없는 기존 v1 artifact": d => { delete d.mergeTreeEvidenceByRunId[123].pullNumber; },
  "merge-tree 증거 누락": d => { d.mergeTreeEvidenceByRunId = {}; },
  "tree 불일치": d => { d.mergeTreeEvidenceByRunId[123].treeSha = code; },
  "base 불일치": d => { d.mergeTreeEvidenceByRunId[123].parents[0] = oldBase; },
  "worker 실행 증거 누락": d => { delete d.fullLaneRunIds; },
  "실행 worker 없음": d => { d.fullLaneRunIds = []; },
  "실패한 run": d => { d.workflowRuns[0].conclusion = "failure"; },
  "취소한 run": d => { d.workflowRuns[0].conclusion = "cancelled"; },
  "미완료 run": d => { d.workflowRuns[0].status = "in_progress"; d.workflowRuns[0].conclusion = null; },
  "머지 이후 실행": d => { d.workflowRuns[0].updated_at = "2026-08-27T10:04:00Z"; },
  "enforcement 수정": d => { d.pullFiles.push({ filename: ".github/workflows/ci.yml" }); },
  "증거 없는 squash": d => { d.mergeCommit.parents = [{ sha: base }]; },
};
for (const [name, mutate] of Object.entries(forkRejections)) {
  test(`fork 재사용 거부: ${name}`, () => {
    const data = forkInput();
    mutate(data);
    assert.equal(evaluateTrustedPostMergeReuse(data).reuse, false);
  });
}

test("fork 최신 rerun 실패를 이전 green run으로 대체하지 않는다", () => {
  const data = forkInput();
  data.workflowRuns.push({ ...data.workflowRuns[0], id: 124, run_attempt: 2,
    conclusion: "failure", updated_at: "2026-08-27T10:02:30Z" });
  assert.equal(evaluateTrustedPostMergeReuse(data).reuse, false);
});

for (const mutate of [
  d => { d.forkMergeTreeEvidenceByRunId = {}; },
  d => { d.forkMergeTreeEvidenceByRunId[456].baseSha = oldBase; },
  d => { d.forkMergeTreeEvidenceByRunId[456].finalTreeSha = code; },
  d => { d.forkMergeTreeEvidenceByRunId[456].testedTreeSha = tree; },
  d => { d.mergeTreeEvidenceByRunId[456].parents[0] = oldBase; },
  d => { d.mergeTreeEvidenceByRunId[456].runAttempt = 2; },
]) {
  test("fork 코드 후보의 tree 또는 identity 증명이 없거나 변조되면 거부한다", () => {
    const data = forkTailInput();
    mutate(data);
    assert.equal(evaluateTrustedPostMergeReuse(data).reuse, false);
  });
}

test("실제 Git tree 대조는 fork의 문서 trailing만 허용하고 source 변경과 base 이동은 거부한다", () => {
  const root = mkdtempSync(join(tmpdir(), "rhwp-fork-reuse-"));
  const git = (...args) => execFileSync("git", ["-C", root, ...args], {
    encoding: "utf8", stdio: ["ignore", "pipe", "pipe"],
  }).trim();
  try {
    git("init");
    git("config", "user.name", "CI contract");
    git("config", "user.email", "ci-contract@example.invalid");
    git("config", "commit.gpgsign", "false");
    writeFileSync(join(root, "source.txt"), "base\n");
    git("add", "."); git("commit", "-m", "base");
    const baseSha = git("rev-parse", "HEAD");
    git("switch", "-c", "feature");
    writeFileSync(join(root, "source.txt"), "candidate\n");
    git("commit", "-am", "candidate");
    const candidateSha = git("rev-parse", "HEAD");
    git("switch", "--detach", baseSha);
    git("merge", "--no-ff", "feature", "-m", "tested merge");
    const testedMergeSha = git("rev-parse", "HEAD");
    git("switch", "feature");
    mkdirSync(join(root, "mydocs"));
    writeFileSync(join(root, "mydocs/review.md"), "review\n");
    git("add", "."); git("commit", "-m", "review");
    git("switch", "--detach", baseSha);
    git("merge", "--no-ff", "feature", "-m", "final merge");
    const mergeSha = git("rev-parse", "HEAD");
    const identity = { baseSha, candidateSha, testedMergeSha, mergeSha };
    assert.equal(verifyForkPostMergeTree(root, identity).finalTreeSha, git("rev-parse", "HEAD^{tree}"));
    assert.throws(() => verifyForkPostMergeTree(root, { ...identity, baseSha: oldBase }));
    git("switch", "feature");
    writeFileSync(join(root, "source.txt"), "untested source\n");
    git("commit", "-am", "source changed");
    git("switch", "--detach", baseSha);
    git("merge", "--no-ff", "feature", "-m", "different final merge");
    assert.throws(() => verifyForkPostMergeTree(root, {
      ...identity, mergeSha: git("rev-parse", "HEAD"),
    }), /non-review-tree-change/);
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});

test("worker evidence is mandatory even when workflow status is green", () => {
  const data = input();
  delete data.fullLaneRunIds;
  assert.equal(evaluateTrustedPostMergeReuse(data).reason, "candidate-full-lane-evidence-unavailable");
});
