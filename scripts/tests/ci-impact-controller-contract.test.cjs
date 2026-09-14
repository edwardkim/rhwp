'use strict';

// Execute the actual workflow scripts, not a second implementation of their policy.
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const vm = require('node:vm');
const test = require('node:test');
const { statusDescription } = require('../ci-impact-policy.cjs');
const { classifyChanges } = require('../ci-impact-classifier.cjs');

const root = path.resolve(__dirname, '../..');
const readWorkflow = (name) => fs.readFileSync(path.join(root, '.github/workflows', name), 'utf8');
const controller = readWorkflow('ci-impact-policy.yml');
const HEAD = 'a'.repeat(40);
const BASE = 'b'.repeat(40);
const REPO = 'edwardkim/rhwp';
const repository = { full_name: REPO, owner: { login: 'edwardkim' } };
const clone = (value) => structuredClone(value);
const pull = () => ({ number: 123, state: 'open', commits: 2, created_at: '2026-09-14T00:00:00Z',
  user: { login: 'edwardkim' }, base: { ref: 'devel', sha: BASE, repo: clone(repository) },
  head: { sha: HEAD, ref: 'topic', repo: clone(repository) } });

function scriptAt(marker) {
  assert.equal(controller.split(marker).length, 2, 'unique workflow step: ' + marker);
  const section = controller.split(marker)[1];
  assert.ok(section.includes('script: |'), 'inline script missing');
  const lines = [];
  for (const line of section.split('script: |')[1].split('\n').slice(1)) {
    if (line.trim() && !line.startsWith('            ')) break;
    lines.push(line.slice(12));
  }
  assert.ok(lines.join('\n').trim(), 'empty inline script');
  return lines.join('\n');
}

const resolveScript = scriptAt('      - name: Resolve live pull request identity');
const publishScript = scriptAt('      - name: Publish exact-head policy status');
const context = () => ({ repo: { owner: 'edwardkim', repo: 'rhwp' }, eventName: 'pull_request_target',
  payload: { pull_request: pull() }, serverUrl: 'https://github.com', runId: 456 });
const auditContext = (linked = [{ number: 123, base: { ref: 'devel' } }]) => ({ ...context(),
  eventName: 'workflow_run', payload: { workflow_run: { id: 789, event: 'pull_request',
    head_sha: HEAD, head_branch: 'topic', head_repository: clone(repository), pull_requests: linked } } });

test('actual job condition skips known non-devel PRs and non-PR runs', () => {
  const condition = controller.split('    if: >-\n')[1].split('    runs-on:')[0].trim();
  assert.ok(condition.startsWith('${{') && condition.endsWith('}}'));
  // Only the workflow's property lookups are adapted. The Boolean expression itself is executed.
  const expression = condition.slice(3, -2).replace(/github(?:\.[a-zA-Z_]\w*|\[\d+\])+/g,
    (key) => 'lookup(' + JSON.stringify(key) + ')');
  const evaluate = (ctx) => Boolean(vm.runInNewContext(expression, { lookup: (key) => {
    const values = { github: { event_name: ctx.eventName, event: ctx.payload } };
    return key.replace(/\[(\d+)\]/g, '.$1').split('.').reduce((value, part) => value?.[part], values) ?? null;
  } }));
  assert.equal(evaluate(context()), true);
  const main = context(); main.payload.pull_request.base.ref = 'main';
  assert.equal(evaluate(main), false);
  assert.equal(evaluate(auditContext()), true);
  assert.equal(evaluate(auditContext([{ number: 123, base: { ref: 'main' } }])), false);
  assert.equal(evaluate(auditContext([{ number: 123, base: { ref: 'release' } }])), false);
  assert.equal(evaluate(auditContext([])), true, 'missing fork linkage needs live resolution');
  assert.equal(evaluate(auditContext([{ number: 123 }])), true);
  assert.equal(evaluate(auditContext([{ number: 123, base: { ref: 'main' } }, { number: 124 }])), true,
    'ambiguous/multiple links need exact live resolution');
  for (const event of ['push', 'workflow_dispatch', 'schedule']) {
    const ctx = auditContext(); ctx.payload.workflow_run.event = event;
    assert.equal(evaluate(ctx), false);
  }
});

async function execute(script, ctx, { live = pull(), candidates = [pull()], env = {}, apiError = false } = {}) {
  const outputs = {}, logs = [], posted = [], calls = [];
  const core = { setOutput: (key, value) => { outputs[key] = value; },
    info: (s) => logs.push(s), warning: (s) => logs.push(s), setFailed: (s) => logs.push(s) };
  const github = { rest: { pulls: {
    get: async ({ pull_number: number }) => {
      calls.push('get:' + number);
      if (apiError) throw Error('mock API failure');
      return { data: clone(Array.isArray(live) ? live.find((p) => p.number === number) : live) };
    }, list() {},
  }, repos: {
    getCollaboratorPermissionLevel: async () => { calls.push('permission'); return { data: { permission: 'admin' } }; },
    createCommitStatus: async (value) => posted.push(value),
  } }, paginate: async () => { calls.push('list'); if (apiError) throw Error('mock API failure'); return clone(candidates); } };
  await vm.runInNewContext('(async () => {\n' + script + '\n})()', {
    context: ctx, github, core, process: { env }, console,
  });
  return { outputs, logs, posted, calls };
}

const description = () => statusDescription({
  classification: classifyChanges({ eventName: 'pull_request', files: [{ filename: '.github/workflows/ci.yml', status: 'modified' }] }),
  expected_workflows: { CI: 'true', CodeQL: 'true', 'Render Diff': 'true' },
  decision: 'full', trusted_review_fast_pass: 'true', base_sha: BASE,
});
const publishEnv = () => ({ PULL_NUMBER: '123', HEAD_SHA: HEAD, BASE_REF: 'devel', BASE_SHA: BASE,
  HEAD_REPOSITORY: REPO, HEAD_BRANCH: 'topic', MODE: 'publish', DECISION: 'full', REVIEW_FAST_PASS: 'true',
  DESCRIPTION: description(), CI_EXPECTED: 'true', CODEQL_EXPECTED: 'true', RENDER_EXPECTED: 'true' });

for (const name of ['ci.yml', 'codeql.yml', 'render-diff.yml']) {
  const match = readWorkflow(name).match(/async function hasTrustedReviewReuse\(pr\) \{[\s\S]*?\n            \}/);
  assert.ok(match, 'missing consumer in ' + name);
  async function consume({ pr = pull(), status = {}, run = {} } = {}) {
    let polls = 0;
    const github = { rest: { repos: { listCommitStatusesForRef() {} }, actions: {
      getWorkflowRun: async () => ({ data: { name: 'CI Impact Policy Controller',
        path: '.github/workflows/ci-impact-policy.yml', event: 'pull_request_target', repository,
        status: 'completed', conclusion: 'success', ...run } }),
    } }, paginate: async () => { polls += 1; return [{ id: 1, context: 'CI Impact Policy',
      state: 'success', creator: { login: 'github-actions[bot]' }, description: description(),
      target_url: 'https://github.com/edwardkim/rhwp/actions/runs/456', ...status }]; } };
    const accepted = await vm.runInNewContext(match[0] + '; hasTrustedReviewReuse(pr)', {
      github, pr, owner: 'edwardkim', repo: 'rhwp', setTimeout: (fn) => fn(),
    });
    return { accepted, polls };
  }
  test(name + ': consumes the current producer status', async () => {
    assert.equal((await consume()).accepted, true);
    assert.equal((await consume({ run: { status: 'in_progress', conclusion: null } })).accepted, true);
  });
  test(name + ': rejects malformed, stale and untrusted reuse evidence', async () => {
    for (const bad of [description().replace(/^v=\d+;/, 'v=5;'), description().replace(/^v=\d+;/, 'v=999;'),
      description() + ';rfp=1', description().replace(';rfp=1', ''), description() + ';',
      description().replace('rfp=1', 'rfp=0'), description().replace(BASE, 'c'.repeat(40))]) {
      assert.equal((await consume({ status: { description: bad } })).accepted, false, bad);
    }
    for (const bad of [{ creator: { login: 'someone' } }, { state: 'pending' }, { target_url: '' }]) {
      assert.equal((await consume({ status: bad })).accepted, false);
    }
    for (const bad of [{ name: 'Other' }, { path: 'other.yml' }, { event: 'push' },
      { repository: { full_name: 'another/repo' } }, { conclusion: 'failure' }, { conclusion: 'cancelled' }]) {
      assert.equal((await consume({ run: bad })).accepted, false);
    }
  });
  test(name + ': main and fork requests do not poll the controller', async () => {
    const main = pull(); main.base.ref = 'main'; main.head.ref = 'devel';
    const fork = pull(); fork.head.repo.full_name = 'contributor/rhwp';
    for (const pr of [main, fork]) assert.deepEqual(await consume({ pr }), { accepted: false, polls: 0 });
  });
}

test('resolve accepts an exact open devel PR', async () => {
  const result = await execute(resolveScript, context());
  assert.equal(result.outputs.active, 'true');
  assert.equal(result.outputs.base_ref, 'devel');
});
test('resolve rejects retarget, stale and incomplete identity before permission lookup', async () => {
  const mutations = [(p) => { p.base.ref = 'main'; }, (p) => { p.state = 'closed'; },
    (p) => { p.head.sha = 'c'.repeat(40); }, (p) => { p.head.ref = 'other'; },
    (p) => { p.base.repo.full_name = 'other/repo'; }, (p) => { delete p.base.sha; },
    (p) => { delete p.head.repo; }];
  for (const mutate of mutations) {
    const live = pull(); mutate(live);
    const result = await execute(resolveScript, context(), { live });
    assert.equal(result.outputs.active, 'false', JSON.stringify(live));
    assert.ok(!result.calls.includes('permission'));
  }
});
test('audit uses the explicitly linked PR, not another devel PR on the branch', async () => {
  const live = pull(); live.base.ref = 'main';
  const result = await execute(resolveScript, auditContext([{ number: 123, base: { ref: 'main' } }]), { live });
  assert.equal(result.outputs.active, 'false');
  assert.ok(!result.calls.includes('list'));
});
test('audit accepts devel or missing linkage with unique verified live identity', async () => {
  for (const linked of [[{ number: 123 }], []]) {
    assert.equal((await execute(resolveScript, auditContext(linked))).outputs.active, 'true');
  }
  const ctx = auditContext([]); ctx.payload.workflow_run.head_repository = { full_name: 'contributor/rhwp', owner: { login: 'contributor' } };
  const live = pull(); live.head.repo = clone(ctx.payload.workflow_run.head_repository);
  assert.equal((await execute(resolveScript, ctx, { live, candidates: [live] })).outputs.active, 'true');
});
test('audit rejects ambiguous candidates, malformed linkage and missing run identity', async () => {
  const second = pull(); second.number = 124;
  for (const linked of [[], [{ number: 123 }, { number: 124 }], [{ base: { ref: 'main' } }]]) {
    const result = await execute(resolveScript, auditContext(linked), { candidates: [pull(), second], live: [pull(), second] });
    assert.equal(result.outputs.active, 'false');
  }
  const ctx = auditContext(); delete ctx.payload.workflow_run.head_repository;
  assert.equal((await execute(resolveScript, ctx)).outputs.active, 'false');
});
test('audit accepts only one matching devel PR among linked live identities', async () => {
  const main = pull(); main.number = 124; main.base.ref = 'main';
  const result = await execute(resolveScript, auditContext([{ number: 123 }, { number: 124 }]), { live: [pull(), main] });
  assert.equal(result.outputs.active, 'true');
  assert.equal(result.outputs.pull_number, '123');
});
test('audit rejects all-main multiple links and stale fallback identity', async () => {
  const first = pull(); first.base.ref = 'main';
  const second = clone(first); second.number = 124;
  const result = await execute(resolveScript, auditContext([{ number: 123 }, { number: 124 }]), { live: [first, second] });
  assert.equal(result.outputs.active, 'false');
  assert.ok(!result.calls.includes('permission'));
  const stale = pull(); stale.head.sha = 'c'.repeat(40);
  assert.equal((await execute(resolveScript, auditContext([]), { live: stale })).outputs.active, 'false');
  const excessive = Array.from({ length: 11 }, (_, i) => ({ number: 123 + i }));
  const bounded = await execute(resolveScript, auditContext(excessive));
  assert.equal(bounded.outputs.active, 'false');
  assert.equal(bounded.calls.length, 0);
});
test('non-PR completed workflows cannot activate an audit', async () => {
  for (const event of ['push', 'workflow_dispatch', 'schedule']) {
    const ctx = auditContext(); ctx.payload.workflow_run.event = event;
    const result = await execute(resolveScript, ctx);
    assert.equal(result.outputs.active, 'false');
    assert.equal(result.calls.length, 0);
  }
});
test('API failure cannot publish or synthesize an active identity', async () => {
  await assert.rejects(execute(resolveScript, context(), { apiError: true }), /mock API failure/);
});
test('publish rejects retarget, base advancement, head or repository changes and closed PR', async () => {
  for (const mutate of [(p) => { p.base.ref = 'main'; }, (p) => { p.base.sha = 'c'.repeat(40); },
    (p) => { p.base.repo.full_name = 'other/repo'; }, (p) => { p.head.sha = 'c'.repeat(40); },
    (p) => { p.head.repo.full_name = 'other/repo'; }, (p) => { p.head.ref = 'other'; },
    (p) => { p.state = 'closed'; }]) {
    const live = pull(); mutate(live);
    const result = await execute(publishScript, context(), { live, env: publishEnv() });
    assert.equal(result.posted.length, 0, JSON.stringify(live));
    assert.equal(result.outputs.stale, 'true');
  }
});
test('exact live identity preserves publish and audit status semantics', async () => {
  for (const [extra, expected] of [[{}, 'success'], [{ REVIEW_FAST_PASS: 'false' }, 'pending'],
    [{ DECISION: 'blocked' }, 'failure'], [{ MODE: 'audit', AUDIT_PUBLISH: 'true', AUDIT_CONCLUSION: 'success' }, 'success'],
    [{ MODE: 'audit', AUDIT_PUBLISH: 'true', REVIEW_FAST_PASS: 'false', AUDIT_CONCLUSION: 'pending' }, 'pending'],
    [{ MODE: 'audit', AUDIT_PUBLISH: 'true', AUDIT_CONCLUSION: 'failure' }, 'failure']]) {
    const result = await execute(publishScript, context(), { env: { ...publishEnv(), ...extra } });
    assert.equal(result.posted.length, 1); assert.equal(result.posted[0].state, expected);
  }
});

test('reporter distinguishes inactive/stale scope from actual controller failure without loading helpers', async () => {
  const script = scriptAt('      - name: Explain CI failure evidence');
  for (const [env, expected] of [
    [{ ACTIVE: 'false', SCOPE_REASON: 'not-devel-pr' }, /감사·게시 제외: not-devel-pr/],
    [{ ACTIVE: 'true', STALE: 'true', SCOPE_REASON: 'stale-or-non-devel-publication' }, /감사·게시 제외: stale-or-non-devel-publication/],
    [{ ACTIVE: 'false', OUTCOME_RESOLVE: 'failure' }, /Controller 오류 단계: RESOLVE/],
  ]) {
    let summary = '';
    let helperLoads = 0;
    await vm.runInNewContext('(async () => {\n' + script + '\n})()', {
      process: { env: { GITHUB_WORKSPACE: '/mock', GITHUB_STEP_SUMMARY: '/mock/summary', ...env } },
      require: (name) => {
        if (name === 'node:fs') return { appendFileSync: (_, text) => { summary += text; } };
        if (name === 'node:path') return path;
        helperLoads += 1; throw Error('mock unavailable trusted helper');
      },
    });
    assert.match(summary, expected);
    assert.equal(helperLoads, env.OUTCOME_RESOLVE === 'failure' ? 1 : 0);
  }
});
