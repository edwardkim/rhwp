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

// #9089: PR #9088에서 관측한 재사용 경계를 축약한 합성 입력으로 실제 preflight를 실행한다.
const TAIL_CODE = '0a8afd62bc955c8125e4238c9afc4117eeb4e683';
const TAIL_DOC = 'abbf3e21bf67c647ec633141e245cf389507ab4c';
const SUITE_MANIFEST = 'scripts/rust-test-suite-manifest.mjs';
const modified = filename => ({ filename, status: 'modified' });
const workerWorkflows = [
  { workflow: 'proptest-roundtrip.yml', worker: 'prop roundtrip', runId: 43680269699 },
  { workflow: 'adapter-diff.yml', worker: 'adapter inter-diff', runId: 43680269627 },
];

function workerTailFixture({ worker, runId }) {
  const docs = [modified('mydocs/orders/20261115.md'),
    { filename: 'mydocs/pr/archives/pr_9075_review.md', status: 'added' }];
  const code = [modified(SUITE_MANIFEST), modified('src/renderer/layout.rs')];
  const run = { id: runId, run_attempt: 1, event: 'pull_request', head_sha: TAIL_CODE,
    head_branch: 'topic', head_repository: { id: 20 }, status: 'completed', conclusion: 'success',
    created_at: '2026-11-15T01:00:00Z' };
  return { files: [...code, ...docs], changedFiles: code.length + docs.length,
    commits: [{ sha: TAIL_CODE, parents: [{ sha: BASE }] },
      { sha: TAIL_DOC, parents: [{ sha: TAIL_CODE }] }],
    details: { [TAIL_CODE]: code, [TAIL_DOC]: docs }, runs: [run],
    jobs: { [runId]: [{ id: 123, name: worker, run_id: runId, run_attempt: 1,
      head_sha: TAIL_CODE, status: 'completed', conclusion: 'success' }] } };
}

async function executeWorkerTail(config, change = () => {}) {
  const fixture = workerTailFixture(config);
  change(fixture);
  const lines = readWorkflow(config.workflow).split('          script: |\n')[1].split('\n');
  const script = [];
  for (const line of lines) {
    if (line.trim() && !line.startsWith('            ')) break;
    script.push(line.slice(12));
  }
  const outputs = {}, calls = [], errors = [];
  const endpoints = Object.fromEntries(['files', 'commits', 'runs', 'jobs'].map(name => [name, Symbol(name)]));
  const pr = { ...pull(), changed_files: fixture.changedFiles,
    head: { ...pull().head, sha: TAIL_DOC, repo: { id: 20 } } };
  let failure;
  try {
    await vm.runInNewContext('(async () => {\n' + script.join('\n') + '\n})()', {
      context: { eventName: 'pull_request', repo: { owner: 'edwardkim', repo: 'rhwp' },
        payload: { pull_request: pr } },
      core: { setOutput: (key, value) => { outputs[key] = String(value); },
        info() {}, warning: message => errors.push(message) },
      github: { rest: { pulls: { listFiles: endpoints.files, listCommits: endpoints.commits },
        repos: { getCommit: async ({ ref }) => ({ data: { files: fixture.details[ref] } }) },
        actions: { listWorkflowRuns: endpoints.runs, listJobsForWorkflowRun: endpoints.jobs } },
      paginate: async (endpoint, params) => {
        if (endpoint === endpoints.files) return clone(fixture.files);
        if (endpoint === endpoints.commits) return clone(fixture.commits);
        if (endpoint === endpoints.runs) {
          assert.equal(params.workflow_id, config.workflow);
          assert.equal(params.event, 'pull_request');
          return clone(fixture.runs);
        }
        if (endpoint === endpoints.jobs) {
          calls.push(params);
          assert.equal(params.filter, 'latest');
          assert.equal(params.per_page, 100);
          if (fixture.jobApiError) throw Error('worker API unavailable');
          return clone(fixture.jobs[params.run_id] || []);
        }
        throw Error('unexpected endpoint');
      } },
    });
  } catch (error) { failure = error; }
  // continue-on-error와 finalize의 기본값처럼, 판정 예외는 재사용을 허용하지 않는다.
  const accepted = !failure && (config.workflow === 'adapter-diff.yml'
    ? outputs.adapter_required === 'false' : outputs.fast_pass === 'true');
  return { accepted, outputs, calls, failure, errors };
}

for (const config of workerWorkflows) {
  test(`#9089 ${config.workflow}: 검증된 목록 변경 뒤 문서 tail은 실제 worker 성공을 재사용`, async () => {
    const result = await executeWorkerTail(config);
    assert.equal(result.failure, undefined);
    assert.equal(result.accepted, true, JSON.stringify(result.outputs));
    assert.equal(result.calls.length, 1);
    assert.equal(result.calls[0].run_id, config.runId);
    assert.ok(result.outputs.reason.endsWith(TAIL_CODE));
  });

  test(`#9089 ${config.workflow}: worker의 run·attempt·SHA·실행 성공이 모두 필요`, async () => {
    for (const jobChange of [
      { run_id: 1 }, { run_attempt: 2 }, { head_sha: BASE }, { status: 'in_progress' },
      { conclusion: 'skipped' }, { conclusion: 'failure' }, { conclusion: 'cancelled' },
      { conclusion: null }, { name: 'other worker' },
    ]) {
      const result = await executeWorkerTail(config, f => Object.assign(f.jobs[config.runId][0], jobChange));
      assert.equal(result.failure, undefined);
      assert.equal(result.accepted, false, JSON.stringify(jobChange));
      assert.equal(result.calls.length, 1);
    }
    for (const change of [
      f => { f.jobs[config.runId] = []; },
      f => { f.jobs[config.runId].push(clone(f.jobs[config.runId][0])); },
      f => { f.jobApiError = true; },
      f => { delete f.runs[0].id; },
      f => { delete f.runs[0].run_attempt; },
    ]) {
      const result = await executeWorkerTail(config, change);
      assert.equal(result.failure, undefined);
      assert.equal(result.accepted, false);
      assert.equal(result.outputs.reason, 'review-tail-candidate-worker-unavailable');
    }
  });

  test(`#9089 ${config.workflow}: 재실행의 최신 attempt 증거만 수용`, async () => {
    for (const jobAttempt of [1, 2]) {
      const result = await executeWorkerTail(config, f => {
        f.runs[0].run_attempt = 2;
        f.jobs[config.runId][0].run_attempt = jobAttempt;
      });
      assert.equal(result.accepted, jobAttempt === 2);
    }
  });

  test(`#9089 ${config.workflow}: 문서 후보의 skip을 성공으로 세지 않고 코드 성공까지 추적`, async () => {
    const result = await executeWorkerTail(config, f => {
      const docRunId = config.runId + 1;
      f.runs.unshift({ ...f.runs[0], id: docRunId, head_sha: TAIL_DOC,
        created_at: '2026-11-15T02:00:00Z' });
      f.jobs[docRunId] = [{ ...f.jobs[config.runId][0], run_id: docRunId,
        head_sha: TAIL_DOC, conclusion: 'skipped' }];
    });
    assert.equal(result.failure, undefined);
    assert.equal(result.accepted, true);
    assert.deepEqual(result.calls.map(call => call.run_id), [config.runId + 1, config.runId]);
    assert.ok(result.outputs.reason.endsWith(TAIL_CODE));
  });

  test(`#9089 ${config.workflow}: 실패·미완료·다른 source·PR 생성 전 run은 재사용하지 않음`, async () => {
    for (const runChange of [
      { status: 'in_progress', conclusion: null }, { conclusion: 'failure' },
      { conclusion: 'cancelled' }, { event: 'push' }, { head_sha: BASE },
      { head_branch: 'another-topic' }, { head_repository: { id: 99 } },
      { created_at: '2020-01-01T00:00:00Z' }, { created_at: 'invalid-date' },
    ]) {
      const result = await executeWorkerTail(config, f => Object.assign(f.runs[0], runChange));
      assert.equal(result.accepted, false, JSON.stringify(runChange));
      assert.equal(result.calls.length, 0);
    }
    const result = await executeWorkerTail(config, f => {
      f.runs.unshift({ ...f.runs[0], id: config.runId + 1,
        conclusion: 'failure', created_at: '2026-11-15T02:00:00Z' });
    });
    assert.equal(result.accepted, false, '같은 SHA의 과거 성공으로 최신 실패를 덮지 않는다');
    assert.equal(result.calls.length, 0);
  });

  test(`#9089 ${config.workflow}: 성공 뒤 새 목록 변경은 이전 성공을 재사용하지 않음`, async () => {
    const result = await executeWorkerTail(config, f => {
      const newCode = 'c'.repeat(40);
      f.commits.splice(1, 0, { sha: newCode, parents: [{ sha: TAIL_CODE }] });
      f.commits[2].parents = [{ sha: newCode }];
      f.details[newCode] = [modified(SUITE_MANIFEST)];
    });
    assert.equal(result.failure, undefined);
    assert.equal(result.accepted, false);
    assert.equal(result.calls.length, 0);
    const codeOnly = await executeWorkerTail(config, f => { f.commits.pop(); });
    assert.equal(codeOnly.accepted, false, '문서 tail 없는 코드 변경은 full');
  });

  test(`#9089 ${config.workflow}: 실행 정의 변경과 실행 파일의 문서 rename은 full`, async () => {
    const executionPaths = [`.github/workflows/${config.workflow}`,
      config.workflow === 'adapter-diff.yml' ? 'scripts/run-adapter-diff.mjs' : 'scripts/run-prop-roundtrip.mjs',
      config.workflow === 'adapter-diff.yml' ? 'scripts/tests/run-adapter-diff.test.mjs' : 'scripts/tests/run-prop-roundtrip.test.mjs',
      config.workflow === 'adapter-diff.yml' ? 'tools/adapter_diff/harness.py' : 'tools/proptest_roundtrip/gen_m04f_catalogs.py'];
    for (const filename of executionPaths) {
      const result = await executeWorkerTail(config, f => {
        f.files.push(modified(filename)); f.changedFiles += 1;
      });
      assert.equal(result.accepted, false, filename);
      assert.equal(result.calls.length, 0);
    }
    for (const previous_filename of [...executionPaths, SUITE_MANIFEST]) {
      const result = await executeWorkerTail(config, f => {
        f.files = [{ filename: 'mydocs/moved.md', status: 'renamed', previous_filename }];
        f.changedFiles = 1;
      });
      assert.equal(result.accepted, false, previous_filename);
      assert.equal(result.calls.length, 0);
    }
    // PR 합산 diff만 보는 것으로 부족하다. 중간 tail의 rename도 문서로 세면 안 된다.
    const renamedTail = await executeWorkerTail(config, f => {
      f.details[TAIL_DOC] = [{ filename: 'mydocs/moved.md', status: 'renamed', previous_filename: SUITE_MANIFEST }];
    });
    assert.equal(renamedTail.accepted, false);
  });

  test(`#9089 ${config.workflow}: 파일 목록 누락·잘림과 계보 불일치는 full`, async () => {
    for (const change of [
      f => { f.changedFiles += 1; },
      f => { f.details[TAIL_DOC] = []; },
      f => { f.details[TAIL_DOC] = Array.from({ length: 300 }, () => modified('mydocs/a.md')); },
      f => { f.commits[1].parents = [{ sha: BASE }]; },
    ]) {
      const result = await executeWorkerTail(config, change);
      assert.equal(result.failure, undefined);
      assert.equal(result.accepted, false);
      assert.equal(result.calls.length, 0);
    }
  });

  test(`#9089 ${config.workflow}: 목록 변경 후보의 성공도 base bridge 트리 검사를 생략하지 않음`, async () => {
    const mergeSha = 'd'.repeat(40);
    const result = await executeWorkerTail(config, f => {
      f.commits[0].parents = [{ sha: 'c'.repeat(40) }];
      f.commits.splice(1, 0, { sha: mergeSha, parents: [{ sha: TAIL_CODE }, { sha: BASE }] });
      f.commits[2].parents = [{ sha: mergeSha }];
    });
    assert.equal(result.failure, undefined);
    assert.equal(result.accepted, false, '트리 검증 전에는 worker skip을 확정하지 않는다');
    const key = config.workflow === 'adapter-diff.yml' ? 'adapter_required' : 'fast_pass';
    assert.equal(result.outputs[key], 'pending-base-merge-tree');
    assert.equal(result.outputs.candidate_sha, TAIL_CODE);
    assert.equal(result.outputs.base_merge_sha, mergeSha);
    assert.equal(result.outputs.source_parent_sha, TAIL_CODE);
    assert.equal(result.calls.length, 1);
  });
}

test('#9089 두 workflow가 같은 worker 증거 판정 함수를 사용', () => {
  const helpers = workerWorkflows.map(({ workflow }) => readWorkflow(workflow).split(
    '            async function candidateWorkerState(run, workerName) {\n')[1]?.split(
    '\n            const files = await github.paginate')[0]);
  assert.ok(helpers[0]);
  assert.equal(helpers[0], helpers[1]);
});

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
    for (const bad of [description().replace(/^v=\d+;/, 'v=5;'), description().replace(/^v=\d+;/, 'v=6;'), description().replace(/^v=\d+;/, 'v=7;'), description().replace(/^v=\d+;/, 'v=999;'),
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

async function executeCollection({ missingLists = 1, files = [{ filename: 'src/lib.rs', status: 'modified' }],
  runChanges = {}, listError = false } = {}) {
  const collected = {}, calls = { lists: 0, sleeps: 0, runs: 0, jobs: 0 };
  const paths = { CI: 'ci.yml', CodeQL: 'codeql.yml', 'Render Diff': 'render-diff.yml' };
  const runs = Object.entries(paths).map(([name, file], index) => ({ id: 100 + index,
    name, path: '.github/workflows/' + file, event: 'pull_request', head_sha: HEAD,
    head_branch: 'topic', head_repository: repository, run_attempt: 1, workflow_id: 200 + index,
    status: 'completed', conclusion: 'success', ...runChanges, pull_requests: [{ number: 123,
      base: { ref: 'devel', sha: BASE }, head: { ref: 'topic', sha: HEAD } }] }));
  const api = { files() {}, commits() {}, lists() {}, jobs() {} };
  const github = { rest: { pulls: { listFiles: api.files, listCommits: api.commits },
    actions: { listWorkflowRunsForRepo: api.lists, listJobsForWorkflowRun: api.jobs,
      getWorkflowRun: async ({ run_id }) => { calls.runs++; return { data: clone(runs.find((r) => r.id === run_id)) }; } } },
    paginate: async (method, args) => {
      if (method === api.files) return files;
      if (method === api.commits) return [];
      if (method === api.lists) {
        calls.lists++;
        if (listError) throw Error('mock list unavailable');
        return calls.lists <= missingLists ? [] : clone(runs);
      }
      if (method === api.jobs) { calls.jobs++; return [{ id: args.run_id + 10, run_id: args.run_id,
        run_attempt: 1, head_sha: HEAD, name: 'job', status: 'completed', conclusion: 'success', steps: [] }]; }
      throw Error('unexpected API');
    } };
  const evidence = require('../ci-workflow-evidence.cjs');
  const helper = Object.fromEntries(Object.entries(evidence).map(([key, fn]) => [key,
    (options) => fn({ ...options, sleep: async () => { calls.sleeps++; } })]));
  await vm.runInNewContext('(async () => {\n' + scriptAt('      - name: Collect policy and same-head workflow evidence') + '\n})()', {
    context: auditContext(), github,
    core: { setOutput() {}, warning() {} },
    process: { env: { GITHUB_WORKSPACE: '/mock', GITHUB_REPOSITORY: REPO, MODE: 'audit',
      PULL_NUMBER: '123', BASE_REF: 'devel', BASE_SHA: BASE, HEAD_SHA: HEAD,
      HEAD_BRANCH: 'topic', HEAD_REPOSITORY: REPO, PULL_COMMITS: '2' } },
    require: (name) => {
      if (name === 'node:fs') return { writeFileSync: (file, body) => { collected[path.basename(file)] = JSON.parse(body); } };
      if (name === 'node:path') return path;
      if (name.endsWith('ci-impact-policy.cjs')) return require('../ci-impact-policy.cjs');
      if (name.endsWith('ci-workflow-evidence.cjs')) return helper;
      throw Error('unexpected executable dependency');
    },
  });
  return { collected, calls, paths };
}

test('actual completion audit recollects missing lists before checking exact run/job evidence', async () => {
  const { collected, calls, paths } = await executeCollection();
  assert.deepEqual(Object.keys(collected['.ci-impact-workflows.json']).sort(), Object.keys(paths).sort());
  assert.equal(calls.lists, 2); assert.equal(calls.sleeps, 1);
  assert.equal(calls.runs, 6); assert.equal(calls.jobs, 3);
});

test('actual completion audit keeps unresolved listing absent without fabricating evidence', async () => {
  for (const options of [{ missingLists: 10 }, { listError: true }]) {
    const { collected, calls } = await executeCollection(options);
    assert.deepEqual(collected['.ci-impact-workflows.json'], {});
    assert.equal(calls.lists, 4); assert.equal(calls.sleeps, 3);
    assert.equal(calls.runs, 0); assert.equal(calls.jobs, 0);
  }
});

test('actual completion audit does not poll workflows excluded by path policy', async () => {
  const { calls } = await executeCollection({ files: [{ filename: 'README.md', status: 'modified' }], missingLists: 10 });
  assert.equal(calls.lists, 1); assert.equal(calls.sleeps, 0);
});

test('actual completion audit returns present running or failed workflows without list delay', async () => {
  for (const runChanges of [{ status: 'in_progress', conclusion: null }, { conclusion: 'failure' }]) {
    const { collected, calls } = await executeCollection({ missingLists: 0, runChanges });
    const workflow = collected['.ci-impact-workflows.json'].CI.run;
    assert.equal(workflow.status, runChanges.status || 'completed');
    assert.equal(workflow.conclusion, runChanges.conclusion || '');
    assert.equal(calls.lists, 1); assert.equal(calls.sleeps, 0);
    assert.equal(calls.runs, 6); assert.equal(calls.jobs, 3);
  }
});

// #8267: run each real workflow preflight with a complete PR file list.
for (const workflow of ['ci.yml', 'codeql.yml', 'render-diff.yml', 'adapter-diff.yml', 'proptest-roundtrip.yml']) {
  test(`#8267 ${workflow}: root instruction documents fast-pass; source and rename do not`, async () => {
    const lines = readWorkflow(workflow).split('          script: |\n')[1].split('\n');
    const script = [];
    for (const line of lines) {
      if (line.trim() && !line.startsWith('            ')) break;
      script.push(line.slice(12));
    }
    const { isAllowedReviewFile, selectReviewOnlyCandidate, workflowRunExpected } = require('../ci-impact-policy.cjs');
    for (const filename of ['AGENTS.md', 'CLAUDE.md']) {
      for (const check of ['CI', 'CodeQL', 'Render Diff']) {
        assert.equal(workflowRunExpected(check, [{ filename, status: 'modified' }]), true);
      }
      for (const [files, accepted] of [
        [[{ filename, status: 'modified' }], true],
        [[{ filename, status: 'added' }], true],
        [[{ filename, status: 'modified' }, { filename: 'src/lib.rs', status: 'modified' }], false],
        [[{ filename, status: 'removed' }], false],
        [[{ filename, previous_filename: 'src/lib.rs', status: 'renamed' }], false],
        [[{ filename: `tools/${filename}`, status: 'modified' }], false],
      ]) {
        const outputs = {}, errors = [];
        const listFiles = Symbol('files'), listCommits = Symbol('commits');
        const context = { eventName: 'pull_request', repo: { owner: 'edwardkim', repo: 'rhwp' },
          payload: { pull_request: { ...pull(), changed_files: files.length,
            base: { ...pull().base, repo: { id: 10 } },
            head: { ...pull().head, repo: { id: 20, full_name: 'contributor/rhwp' } } } } };
        await vm.runInNewContext('(async () => {\n' + script.join('\n') + '\n})()', {
          context, process: { env: {} },
          github: { rest: { pulls: { listFiles, listCommits } },
            paginate: async endpoint => {
              if (endpoint === listFiles) return files;
              if (endpoint === listCommits) return [];
              throw new Error('unexpected API call');
            } },
          core: { setOutput: (key, value) => { outputs[key] = String(value); },
            info() {}, warning: message => errors.push(message), setFailed: message => errors.push(message) },
        });
        const decision = workflow === 'adapter-diff.yml'
          ? outputs.adapter_required === 'false' : outputs.fast_pass === 'true';
        assert.equal(decision, accepted, `${workflow}: ${JSON.stringify(files)} ${JSON.stringify(outputs)} ${errors}`);
        assert.equal(files.every(isAllowedReviewFile), accepted);
        const result = selectReviewOnlyCandidate([
          { sha: 'c'.repeat(40), parents: [{ sha: BASE }], files: [{ filename: 'src/lib.rs', status: 'modified' }] },
          { sha: HEAD, parents: [{ sha: 'c'.repeat(40) }], files },
        ], BASE);
        assert.equal(result.candidateSha === 'c'.repeat(40), accepted);
        const classification = classifyChanges({ eventName: 'pull_request', files });
        if (accepted) {
          assert.equal(classification.rust_required, 'false');
          assert.equal(classification.codeql_languages, 'none');
        } else if (files.some(file => file.filename === 'src/lib.rs')) {
          assert.equal(classification.rust_required, 'true');
        }
      }
    }
  });
}
