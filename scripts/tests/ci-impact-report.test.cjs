'use strict';

const test = require('node:test');
const assert = require('node:assert/strict');
const { createReport, extractErrors, field, allowedLogUrl, transport, LIMITS, shortError } = require('../ci-impact-report.cjs');

const HEAD = '5f2ea958f14a3b1ce13525ebc482c21a7c7e5b6d';
const BASE = '5111c24c745863fde1f6a7db4b61ed48d2269195';
const RUN = 34220658247;
const JOB = 102045013116;
const STORAGE = 'https://productionresultssa1.blob.core.windows.net/logs/job.txt?sig=private-signature';
const LOG = [
  "thread 'text_overlap_baseline::text_overlaps_do_not_grow_partition_13' (2526) panicked at tests/cases/text_overlap_baseline.rs:226:5:",
  '보이는 글자끼리 겹치는 사건(layout-anomaly text-overlap)이 늘었다.',
  '신규 발생: confidential-document.hwp — 1건 (baseline 없음)',
  '##[error]Process completed with exit code 100.',
].join('\n');

function input() {
  return { repository: 'edwardkim/rhwp', headRepository: 'planet6897/rhwp', pullNumber: 6898,
    headSha: HEAD, baseSha: BASE, headBranch: 'fix/6879', active: 'true', auditPublish: 'true',
    conclusion: 'failure', decision: 'full', published: 'true', outcomes: { publish: 'failure' },
    workflows: { CI: { run: { id: RUN, attempt: 1, conclusion: 'failure' } } } };
}
function fixture(options = {}) {
  const calls = [];
  const pull = { number: 6898, state: 'open', head: { sha: HEAD, ref: 'fix/6879', repo: { full_name: 'planet6897/rhwp' } },
    base: { ref: 'devel', sha: BASE }, ...options.pull };
  const run = { id: RUN, run_attempt: 1, name: 'CI', path: '.github/workflows/ci.yml', event: 'pull_request',
    status: 'completed', conclusion: 'failure', head_sha: HEAD, head_branch: 'fix/6879',
    head_repository: { full_name: 'planet6897/rhwp' }, repository: { full_name: 'edwardkim/rhwp' },
    pull_requests: [], ...options.run };
  const worker = { id: JOB, run_id: RUN, run_attempt: 1, head_sha: HEAD,
    name: 'test-archive-b-shard-1 / Default-feature tests (Archive B)', status: 'completed', conclusion: 'failure',
    steps: [{ name: 'Run Archive B', number: 6, conclusion: 'failure' }], ...options.job };
  const jobs = options.jobs || [worker, { ...worker, id: JOB + 1, name: 'Build & Test' }];
  const fetchImpl = async (url, init) => {
    calls.push({ url, init });
    if (options.override) { const overridden = options.override(url, init); if (overridden) return overridden; }
    if (url.includes('/pulls/')) return Response.json(pull);
    if (url.includes('/attempts/')) return Response.json({ total_count: jobs.length, jobs });
    if (url.endsWith(`/runs/${RUN}`)) return Response.json(run);
    if (url.includes('/actions/jobs/')) return new Response(null, { status: 302, headers: { location: options.location || STORAGE } });
    if (url === STORAGE) return new Response(options.log ?? LOG);
    throw Error('Unexpected fake route');
  };
  return { calls, fetchImpl, worker, run };
}
async function example(options = {}, changes = {}) {
  const f = fixture(options);
  const report = await createReport({ ...input(), ...changes }, { token: 'unit-test-token', fetchImpl: f.fetchImpl });
  return { ...f, report };
}

test('reports #6898 original worker, step, test and overlap count without changing input', async () => {
  const source = input(); const original = structuredClone(source); const f = fixture();
  const report = await createReport(source, { token: 'test-token', fetchImpl: f.fetchImpl });
  assert.deepEqual(source, original);
  assert.equal(report.kind, 'upstream-failure');
  assert.deepEqual(report.internal, []);
  assert.ok(report.summary.includes(field('text_overlaps_do_not_grow_partition_13')));
  assert.match(report.summary, /신규 검출 1건/);
  assert.match(report.summary, /회귀 여부 미확정/);
  assert.match(report.summary, /6: Run Archive B/);
  assert.match(report.summary, new RegExp(`/runs/${RUN}/job/${JOB}`));
  assert.match(report.summary, /집계: Build/);
  assert.doesNotMatch(report.summary, /confidential-document|private-signature|unit-test-token/);
  assert.equal(report.stats.requests, 5);
  assert.equal(f.calls.at(-1).init.headers.Authorization, undefined);
  for (const call of f.calls) assert.equal(call.init.method, 'GET');
});

for (const state of ['success', 'pending']) test(`${state} adds no API calls`, async () => {
  const { report, calls } = await example({}, { conclusion: state });
  assert.equal(report.kind, state); assert.equal(calls.length, 0);
});
test('initial publish pending is not misreported as missing evidence', async () => {
  const { report, calls } = await example({}, { conclusion: '', publishedState: 'pending', outcomes: {} });
  assert.equal(report.kind, 'pending'); assert.equal(calls.length, 0);
});
test('unresolved/stale events never download logs', async () => {
  for (const change of [{ active: 'false' }, { stale: 'true' }, { auditPublish: 'false' }]) {
    const { report, calls } = await example({}, change);
    assert.equal(report.kind, 'stale/skipped'); assert.equal(calls.length, 0);
  }
});
test('internal failure distinguishes API publish error from intentional failure status', async () => {
  for (const stage of ['resolve', 'checkout', 'collect', 'classify', 'input', 'policy', 'publish']) {
    const { report, calls } = await example({}, { outcomes: { [stage]: 'failure' }, published: '' });
    assert.equal(report.kind, 'controller-error'); assert.deepEqual(report.internal, [stage]);
    assert.equal(calls.length, 0);
  }
});
test('live PR head/base change stops diagnostic downloads', async () => {
  for (const pull of [{ head: { sha: 'a'.repeat(40) } }, { base: { sha: 'b'.repeat(40) } }, { state: 'closed' }]) {
    const { report, calls } = await example({ pull });
    assert.equal(report.kind, 'stale/skipped'); assert.equal(calls.length, 1);
  }
});
test('different run head/repository/workflow/event fails closed', async () => {
  for (const run of [{ head_sha: 'a'.repeat(40) }, { repository: { full_name: 'other/repo' } },
    { name: 'other' }, { event: 'push' }, { path: '.github/workflows/other.yml' }]) {
    const { report, calls } = await example({ run });
    assert.deepEqual(report.runs[0].notes, ['identity-mismatch']); assert.equal(calls.length, 2);
  }
});
test('rerun attempt changes stop jobs/log collection', async () => {
  const { report, calls } = await example({ run: { run_attempt: 2 } });
  assert.deepEqual(report.runs[0].notes, ['attempt-changed']); assert.equal(calls.length, 2);
});
test('job attempt mismatch cannot leak older logs', async () => {
  const { report, calls } = await example({ job: { run_attempt: 2 } });
  assert.ok(report.runs[0].notes.includes('job-identity-mismatch'));
  assert.equal(calls.length, 3);
});
test('timed_out and cancelled upstream runs are reported as upstream failures', async () => {
  for (const conclusion of ['timed_out', 'cancelled']) {
    const source = input(); source.workflows.CI.run.conclusion = conclusion;
    const { report } = await example({ run: { conclusion }, job: { conclusion } }, source);
    assert.equal(report.kind, 'upstream-failure'); assert.ok(report.summary.includes(field(conclusion)));
  }
});
test('no selected metadata is unknown, not a fabricated cause', async () => {
  const { report } = await example({}, { workflows: {} });
  assert.equal(report.kind, 'evidence-unavailable');
});
test('API errors are bounded, non-retried and never copied verbatim', async () => {
  for (const status of [403, 404, 429, 500]) {
    const { report, calls } = await example({ override: () => new Response('SECRET response body', { status }) });
    assert.equal(calls.length, 1); assert.deepEqual(report.notes, [`http-${status}`]);
    assert.doesNotMatch(report.summary, /SECRET/);
  }
});
test('missing job log preserves direct link and shows evidence unavailable', async () => {
  const { report } = await example({ override: (url) => url.includes('/actions/jobs/')
    ? new Response('', { status: 404 }) : null });
  assert.match(report.summary, /http-404/); assert.match(report.summary, new RegExp(`/job/${JOB}`));
});
test('unrecognized log does not dump stdout or private document contents', async () => {
  const { report } = await example({ log: 'PRIVATE DOCUMENT CONTENT\npassword=hello\n' });
  assert.match(report.summary, /no-recognized-error/);
  assert.doesNotMatch(report.summary, /PRIVATE|hello/);
});
test('logs are capped during stream reading, not after a full download', async () => {
  let cancelled = false;
  const { report } = await example({ override: (url) => url === STORAGE
    ? new Response(new ReadableStream({ pull(c) { c.enqueue(new Uint8Array(65536).fill(65)); },
      cancel() { cancelled = true; } })) : null });
  assert.ok(cancelled); assert.match(report.summary, /log-truncated/);
});
test('oversize metadata is not parsed', async () => {
  const { report } = await example({ override: () => new Response('x'.repeat(LIMITS.metadataBytes + 10)) });
  assert.deepEqual(report.notes, ['oversize']);
});
test('invalid metadata JSON is explicitly unknown', async () => {
  const { report } = await example({ override: () => new Response('not JSON') });
  assert.deepEqual(report.notes, ['invalid-json']);
});
test('redirect trust boundary rejects arbitrary hosts, credentials, ports and protocols', async () => {
  for (const location of ['http://productionresultssa1.blob.core.windows.net/a', 'https://127.0.0.1/a',
    'https://evil.example/a', 'https://productionresultssa1.blob.core.windows.net.evil.example/a',
    'https://user:password@productionresultssa1.blob.core.windows.net/a',
    'https://productionresultssa1.blob.core.windows.net:8443/a']) {
    assert.equal(allowedLogUrl(location), null);
    const { report, calls } = await example({ location });
    assert.match(report.summary, /unsafe-redirect/); assert.equal(calls.length, 4);
  }
});
test('storage cannot redirect a second time', async () => {
  const { report } = await example({ override: (url) => url === STORAGE
    ? new Response(null, { status: 302, headers: { location: 'https://evil.example' } }) : null });
  assert.match(report.summary, /evidence-unavailable/);
});
test('failure list is capped at six jobs and aggregate is placed last', async () => {
  const f = fixture();
  const jobs = Array.from({ length: 9 }, (_, n) => ({ ...f.worker, id: JOB + n }));
  const { report, calls } = await example({ jobs });
  assert.equal(report.runs[0].jobs.length, 6); assert.match(report.summary, /failure-list-truncated/);
  assert.equal(calls.length, 15); assert.ok(Buffer.byteLength(report.summary) <= LIMITS.summaryBytes);
});
test('request cap applies to every request without retry', async () => {
  const client = transport({ fetchImpl: async () => Response.json({}) });
  for (let n = 0; n < 24; n++) await client.json('/repos/test/repo/check');
  await assert.rejects(client.json('/repos/test/repo/check'), { code: 'request-limit' });
});
test('total deadline prevents further requests', async () => {
  let clock = 0; const client = transport({ now: () => clock }); clock = 45001;
  await assert.rejects(client.json('/repos/test/repo/check'), { code: 'deadline' });
  assert.equal(client.stats().requests, 0);
});
test('hung fetch aborts within remaining overall deadline', async () => {
  let clock = 0; let signal;
  const client = transport({ now: () => clock, fetchImpl: async (_, options) => {
    signal = options.signal; return new Promise(() => {});
  } });
  clock = 44999;
  await assert.rejects(client.json('/repos/test/repo/check'), { code: 'request-timeout' });
  assert.ok(signal.aborted);
});
test('report escapes metadata and redacts credentials; extraction has a strict vocabulary', () => {
  const escaped = field('<script>[link](https://evil.example)\n::error:: ghp_abcdef secret=xxx');
  assert.doesNotMatch(escaped, /<script>|\[link\]|::error::|ghp_|xxx|evil.example/);
  assert.match(escaped, /redacted/);
  assert.equal(shortError(new Error('password=SECRET')), 'evidence-unavailable');
  assert.deepEqual(extractErrors('anything dangerous\nerror[E0308]: private text\nassertion failed: private'),
    ['Rust 컴파일 오류: E0308', 'assertion 검증 실패']);
});

test('policy blocked is distinguished from a test failure without log queries', async () => {
  const { report, calls } = await example({}, { decision: 'blocked' });
  assert.equal(report.kind, 'policy-blocked'); assert.equal(calls.length, 0);
});

test('job list pagination stops at two pages and reports omitted jobs', async () => {
  const f = fixture();
  const page = Array.from({ length: 100 }, (_, n) => ({ ...f.worker, id: JOB + n, conclusion: 'success' }));
  const { report, calls } = await example({ override: (url) => url.includes('/attempts/')
    ? Response.json({ total_count: 300, jobs: page }) : null });
  assert.ok(report.runs[0].notes.includes('job-list-truncated'));
  assert.equal(calls.length, 4);
});

test('multiple workflow failures are retained, not hidden by first policy failure', async () => {
  const source = input();
  source.workflows.CodeQL = { run: { id: RUN + 1, attempt: 1, conclusion: 'failure' } };
  const f = fixture();
  const report = await createReport(source, { fetchImpl: async (url, init) => {
    if (url.endsWith(`/runs/${RUN + 1}`)) return Response.json({ ...f.run,
      id: RUN + 1, name: 'CodeQL', path: '.github/workflows/codeql.yml' });
    if (url.includes(`/runs/${RUN + 1}/attempts/`)) return Response.json({ total_count: 1,
      jobs: [{ ...f.worker, id: JOB + 2, run_id: RUN + 1, name: 'Analyze (rust)' }] });
    return f.fetchImpl(url, init);
  } });
  assert.equal(report.runs.length, 2); assert.match(report.summary, /#### CI/);
  assert.match(report.summary, /#### CodeQL/); assert.ok(report.stats.requests <= 24);
});

test('full summary stays bounded for UTF-8 and expanded Markdown fields', async () => {
  const f = fixture();
  const jobs = Array.from({ length: 6 }, (_, n) => ({ ...f.worker, id: JOB + n,
    name: '한글<&>'.repeat(100), steps: Array.from({ length: 6 }, () => ({ number: 1,
      name: '한글<&>'.repeat(100), conclusion: 'failure' })) }));
  const { report } = await example({ jobs });
  assert.ok(Buffer.byteLength(report.summary) <= 16384);
  assert.doesNotMatch(report.summary, /\ufffd/);
  assert.match(report.summary, /summary-truncated/);
});

test('workflow bootstrap fallback works when trusted helper is missing or throws', async () => {
  const fs = require('node:fs');
  const path = require('node:path');
  const vm = require('node:vm');
  const yaml = fs.readFileSync(path.join(__dirname, '../../.github/workflows/ci-impact-policy.yml'), 'utf8');
  const step = yaml.split('      - name: Explain CI failure evidence')[1];
  const script = step.split('          script: |\n')[1].split('\n').map((line) => line.slice(12)).join('\n');
  for (const unavailable of ['missing', 'throws']) {
    let summary = '';
    const run = vm.runInNewContext(`(async function(require, process) {${script}\n})`);
    const fakeRequire = (module) => {
      if (module === 'node:fs') return { appendFileSync: (_, content) => { summary += content; } };
      if (module === 'node:path') return path;
      if (unavailable === 'missing') throw Error('SECRET loader error');
      return { loadInput: () => ({}), createReport: async () => { throw Error('SECRET reporter error'); } };
    };
    await run(fakeRequire, { env: { GITHUB_WORKSPACE: '/trusted', GITHUB_STEP_SUMMARY: '/summary', OUTCOME_RESOLVE: 'failure' } });
    assert.match(summary, /trusted reporter unavailable/); assert.match(summary, /RESOLVE/);
    assert.doesNotMatch(summary, /SECRET/);
  }
});
