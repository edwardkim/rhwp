'use strict';

const assert = require('node:assert/strict');
const test = require('node:test');
const { selectLatestWorkflowRun } = require('../ci-impact-policy.cjs');

// #7926의 실제 API 응답: 실행 SHA는 고정되지만 연결 PR의 head는 문서 push 뒤 갱신된다.
const CANDIDATE = 'cbc5815631d6c221dc6ec6e0f03565859cb1c346';
const REVIEW_HEAD = '0b67266a6e53925f62cf8544cce9c521d3210cf8';
const BASE = '463c48cc228170b576c832591b76826116e86a49';
const BRANCH = 'review/planet6897-followup-20261011';
const REPOSITORY = 'edwardkim/rhwp';

function fixture(name = 'CI', workflow = 'ci.yml') {
  const identity = {
    name, path: `.github/workflows/${workflow}`, pullNumber: 7923,
    baseRef: 'devel', baseSha: BASE, headSha: CANDIDATE,
    headBranch: BRANCH, headRepository: REPOSITORY, allowPriorBase: true,
  };
  const run = {
    id: 38734213321, name, path: identity.path, event: 'pull_request',
    head_sha: CANDIDATE, head_branch: BRANCH,
    head_repository: { full_name: REPOSITORY },
    status: 'completed', conclusion: 'success', run_attempt: 1,
    run_started_at: '2026-10-15T12:23:04Z',
    pull_requests: [{
      number: 7923,
      base: { ref: 'devel', sha: BASE },
      head: { ref: BRANCH, sha: REVIEW_HEAD },
    }],
  };
  return { identity, run };
}

for (const [name, workflow] of [
  ['CI', 'ci.yml'], ['CodeQL', 'codeql.yml'], ['Render Diff', 'render-diff.yml'],
]) {
  test(`mutable PR association preserves the exact successful ${name} candidate`, () => {
    const { identity, run } = fixture(name, workflow);
    assert.notEqual(run.pull_requests[0].head.sha, run.head_sha);
    assert.equal(selectLatestWorkflowRun([run], identity), run);
  });
}

test('mutable PR association retains workflow, PR, source and exact run SHA rejection', () => {
  const { identity, run } = fixture();
  assert.equal(selectLatestWorkflowRun([run], identity), run);
  const mutations = [
    ['run SHA', r => { r.head_sha = REVIEW_HEAD; }],
    ['source branch', r => { r.head_branch = 'other'; }],
    ['source repository', r => { r.head_repository.full_name = 'other/rhwp'; }],
    ['workflow name', r => { r.name = 'Other'; }],
    ['workflow path', r => { r.path = '.github/workflows/other.yml'; }],
    ['event', r => { r.event = 'push'; }],
    ['PR number', r => { r.pull_requests[0].number = 7925; }],
    ['PR source branch', r => { r.pull_requests[0].head.ref = 'other'; }],
    ['PR base ref', r => { r.pull_requests[0].base.ref = 'main'; }],
  ];
  for (const [label, mutate] of mutations) {
    const wrong = structuredClone(run);
    mutate(wrong);
    assert.equal(selectLatestWorkflowRun([wrong], identity), null, label);
  }
});

test('mutable PR association keeps current-base and explicitly allowed prior-base distinct', () => {
  const { identity, run } = fixture();
  assert.equal(selectLatestWorkflowRun([run], { ...identity, allowPriorBase: false }), run);
  const prior = structuredClone(run);
  prior.pull_requests[0].base.sha = 'd'.repeat(40);
  assert.equal(selectLatestWorkflowRun([prior], { ...identity, allowPriorBase: false }), null);
  assert.equal(selectLatestWorkflowRun([prior], identity), prior);
});
