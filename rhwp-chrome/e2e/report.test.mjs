import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { mkdtemp, mkdir, readFile, writeFile, rm, cp, symlink } from 'node:fs/promises';
import path from 'node:path';
import os from 'node:os';
import { spawnSync } from 'node:child_process';
import test from 'node:test';
import { runChecks } from './run.mjs';
import { generateReport, inspectRun, parseReportArgs } from './report.mjs';
const assertShareable = bytes => assert.doesNotMatch(bytes, /SECRET|98765|\/private\/|<script>|smoke-0\.png|href=/i);
const sha = data => createHash('sha256').update(data).digest('hex');
const png = Buffer.from('iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mNk+A8AAQUBAScY42YAAAAASUVORK5CYII=', 'base64');
const env = { node: 'v24.0.0', platform: 'linux', arch: 'x64', puppeteer: '25.12.0', chrome: '147.0.0.0', executable: '/private/SECRET/chrome', harnessSha: 'a'.repeat(40), harnessDirty: false };
const pass = { status: 0, timedOut: false, signal: null, durationMs: 1, log: 'SECRET /private/user download id 98765' };
async function fixture(t, status = 'pass') {
  const directory = await mkdtemp(path.join(os.tmpdir(), 'rhwp-report-'));
  t.after(() => rm(directory, { recursive: true, force: true }));
  const dist = path.join(directory, 'candidate');
  await mkdir(path.join(dist, 'wasm'), { recursive: true });
  for (const [file, bytes] of Object.entries({ 'manifest.json': '{"manifest_version":3,"version":"1.2.3"}', 'viewer.html': 'viewer', 'wasm/rhwp.js': 'js', 'wasm/rhwp_bg.wasm': 'wasm' })) await writeFile(path.join(dist, file), bytes);
  const input = path.join(directory, 'run');
  const outcome = await runChecks({ dist, output: input }, { env: {}, environment: async () => {
    if (status === 'error') throw new Error('SECRET /private/error');
    return { ...env };
  }, runner: async (_args, { env: variables }) => {
    if (status !== 'fail') return pass;
    await writeFile(path.join(input, 'smoke.json'), JSON.stringify({ runId: variables.RHWP_EXTENSION_E2E_RUN_ID, stage: 'options-save', downloads: [{ id: 98765, filename: '/SECRET/private.hwp' }], error: '<script>SECRET</script>' }));
    await writeFile(path.join(input, 'smoke-0.png'), png);
    return { ...pass, status: 7 };
  } });
  return { input, output: path.join(directory, 'report'), directory, data: outcome.report };
}
async function update(f, change) {
  change(f.data);
  await writeFile(path.join(f.input, 'result.json'), JSON.stringify(f.data));
}

test('pass/fail/error/not-run preserve source semantics through runner and generator', async t => {
  for (const status of ['pass', 'fail', 'error']) {
    const f = await fixture(t, status);
    const result = await generateReport(f);
    assert.equal(result.sourceStatus, status);
    assert.equal(result.status, status);
    assert.deepEqual(result.results.map(x => x.status), status === 'pass' ? ['pass', 'pass', 'pass'] : status === 'fail' ? ['fail', 'not-run', 'not-run'] : ['not-run', 'not-run', 'not-run']);
    assert.deepEqual(result.issues, []);
    assert.equal(result.manualReview.status, '미확인');
    if (status !== 'error') {
      assert.equal(result.candidate.sha256, f.data.candidate.sha256);
      assert.equal(result.tool.sha, env.harnessSha);
      assert.equal(result.candidate.buildSha, '미관측 (검사 도구 SHA와 별개)');
    }
  }
});

test('running, legacy, missing suite, malformed and unsupported results never pass', async t => {
  const changes = [
    d => { d.status = 'running'; delete d.finishedAt; },
    d => { delete d.runId; delete d.evidenceVersion; },
    d => { d.results.pop(); },
    d => { d.schemaVersion = 999; },
    d => { d.results[0].status = 'wat'; },
    d => { d.results[0].exitCode = 1; },
    d => { d.candidateUnchanged = false; },
    d => { d.finishedAt = 'bad'; },
  ];
  for (const change of changes) {
    const f = await fixture(t); await update(f, change);
    assert.equal((await inspectRun(f.input)).summary.status, 'incomplete');
  }
  const f = await fixture(t);
  await writeFile(path.join(f.input, 'result.json'), '{');
  assert.deepEqual((await inspectRun(f.input)).summary.issues, ['result-json-parse-error']);
});

test('candidate inventory and tool SHA validation cannot be replaced by success flags', async t => {
  for (const change of [
    d => { d.candidate.sha256 = 'b'.repeat(64); },
    d => { d.candidate.files[0].sha256 = 'bad'; },
    d => { d.environment.harnessSha = 'unknown'; },
    d => { d.environment.harnessDirty = null; },
    d => { d.candidate.version = '/SECRET/private'; },
    d => { d.environment.executable = '/new/private/path'; d.environment.harnessSha = null; },
  ]) {
    const f = await fixture(t); await update(f, change);
    const { summary } = await inspectRun(f.input);
    assert.equal(summary.status, 'incomplete');
    assert.ok(summary.issues.some(x => /candidate|tool/.test(x)));
  }
});

test('missing logs/PNG, corrupt diagnostics, mutation and mixed-run evidence are incomplete', async t => {
  const cases = [
    async f => rm(path.join(f.input, 'extension-smoke.log')),
    async f => rm(path.join(f.input, 'smoke-0.png')),
    async f => writeFile(path.join(f.input, 'extension-smoke.log'), 'different run'),
    async f => writeFile(path.join(f.input, 'extra.log'), 'foreign'),
    async f => { f.data.results[0].artifacts[0].runId = 'b'.repeat(36); await update(f, () => {}); },
    async f => { const bytes = JSON.stringify({ runId: 'other-run' }); await writeFile(path.join(f.input, 'smoke.json'), bytes); f.data.results[0].artifacts.find(x => x.path === 'smoke.json').sha256 = sha(bytes); await update(f, () => {}); },
    async f => { await writeFile(path.join(f.input, 'smoke.json'), '{'); f.data.results[0].artifacts.find(x => x.path === 'smoke.json').sha256 = sha('{'); await update(f, () => {}); },
    async f => { await rm(path.join(f.input, 'smoke-0.png')); await symlink(path.join(f.input, 'smoke.json'), path.join(f.input, 'smoke-0.png')); },
  ];
  for (const mutate of cases) {
    const f = await fixture(t, 'fail'); await mutate(f);
    const { summary } = await inspectRun(f.input);
    assert.equal(summary.sourceStatus, 'fail'); assert.equal(summary.status, 'incomplete');
    assert.ok(summary.issues.length);
  }
});

test('omitted failure artifacts are explicit even if runner failed before any screenshot', async t => {
  const f = await fixture(t, 'fail');
  await rm(path.join(f.input, 'smoke-0.png'));
  await update(f, d => { d.results[0].artifacts = d.results[0].artifacts.filter(a => !a.path.endsWith('.png')); });
  assert.ok((await inspectRun(f.input)).summary.issues.includes('failure-png-missing'));
});

test('shareable summary excludes free-form diagnostics/paths/IDs; private links stay separate', async t => {
  const f = await fixture(t, 'fail');
  await update(f, d => { d.error = '<script>SECRET</script>'; d.limitations = ['SECRET']; d.candidate.directory = '/SECRET/path'; });
  await generateReport(f);
  for (const name of ['report.html', 'report.json']) {
    const bytes = await readFile(path.join(f.output, name), 'utf8');
    assertShareable(bytes);
  }
  const privateHtml = await readFile(path.join(f.output, 'private.html'), 'utf8');
  assert.match(privateHtml, /evidence-1/); assert.match(privateHtml, /smoke-0.png/);
  assert.match(await readFile(path.join(f.output, 'report.html'), 'utf8'), /options-save/);
  await assert.rejects(generateReport(f), /EEXIST/);
  await assert.rejects(generateReport({ ...f, output: path.join(f.input, 'nested') }), /outside/);
});

test('same CLI produces byte-identical summaries and nonzero failure/incomplete exits', async t => {
  for (const status of ['pass', 'fail']) {
    const f = await fixture(t, status);
    for (const suffix of ['one', 'two']) {
      const result = spawnSync(process.execPath, ['rhwp-chrome/e2e/report.mjs', '--input', f.input, '--output', path.join(f.directory, suffix)], { encoding: 'utf8' });
      assert.equal(result.error, undefined);
      assert.equal(result.status, status === 'pass' ? 0 : 1, result.stderr + result.stdout);
    }
    assert.deepEqual(await readFile(path.join(f.directory, 'one/report.json')), await readFile(path.join(f.directory, 'two/report.json')));
    assert.deepEqual(await readFile(path.join(f.directory, 'one/report.html')), await readFile(path.join(f.directory, 'two/report.html')));
  }
  assert.throws(() => parseReportArgs([]), /required/);
  assert.throws(() => parseReportArgs(['--input', 'x', '--input', 'y']), /Duplicate/);
});

test('copying an artifact from another run is detected by the original manifest', async t => {
  const a = await fixture(t, 'fail'), b = await fixture(t, 'fail');
  await cp(path.join(b.input, 'smoke.json'), path.join(a.input, 'smoke.json'));
  assert.ok((await inspectRun(a.input)).summary.issues.includes('artifact-hash-mismatch'));
});

test('independent expected candidate/tool hashes and source-bound manual review', async t => {
  const f = await fixture(t);
  const { summary } = await inspectRun(f.input, { 'expect-candidate-sha256': f.data.candidate.sha256, 'expect-tool-sha': env.harnessSha });
  assert.equal(summary.status, 'pass');
  for (const options of [{ 'expect-candidate-sha256': 'b'.repeat(64) }, { 'expect-tool-sha': 'b'.repeat(40) }]) {
    assert.equal((await inspectRun(f.input, options)).summary.status, 'incomplete');
  }
  const manualPath = path.join(f.directory, 'manual.json');
  const manual = { sourceSha256: summary.sourceSha256, checkedAt: '2026-10-23T09:00:00.000Z', readability: 'pass', sharingExclusions: 'fail', reviewer: 'SECRET' };
  await writeFile(manualPath, JSON.stringify(manual));
  const reviewed = (await inspectRun(f.input, { 'manual-review': manualPath })).summary;
  assert.equal(reviewed.manualReview.sharingExclusions, 'fail');
  assert.doesNotMatch(JSON.stringify(reviewed), /SECRET/);
  await writeFile(manualPath, JSON.stringify({ ...manual, sourceSha256: 'wrong' }));
  assert.equal((await inspectRun(f.input, { 'manual-review': manualPath })).summary.status, 'incomplete');
});

test('delegated manual review is explicit, source-bound and never promotes a failed run', async t => {
  const f = await fixture(t, 'fail');
  const { summary } = await inspectRun(f.input);
  const manualPath = path.join(f.directory, 'delegated.json');
  const manual = { sourceSha256: summary.sourceSha256, checkedAt: '2026-10-24T03:00:00.000Z',
    readability: 'pass', sharingExclusions: 'pass', reviewMode: 'delegated', reviewer: 'SECRET' };
  await writeFile(manualPath, JSON.stringify(manual));
  const result = await generateReport({ ...f, 'manual-review': manualPath });
  assert.equal(result.status, 'fail');
  assert.equal(result.manualReview.reviewMode, 'delegated');
  const html = await readFile(path.join(f.output, 'report.html'), 'utf8');
  assert.match(html, /사용자 위임에 따른 검토/);
  assertShareable(html);
  for (const reviewMode of ['SECRET', '<SCRIPT>SECRET</SCRIPT>', null]) {
    await writeFile(manualPath, JSON.stringify({ ...manual, reviewMode }));
    const { summary: invalid } = await inspectRun(f.input, { 'manual-review': manualPath });
    assert.equal(invalid.status, 'incomplete');
    assert.ok(invalid.issues.includes('manual-review-invalid-or-wrong-source'));
    assert.doesNotMatch(JSON.stringify(invalid.manualReview), /SECRET/);
  }
});

test('CLI keeps an incomplete otherwise-pass run nonzero', async t => {
  const f = await fixture(t);
  await rm(path.join(f.input, 'extension-smoke.log'));
  const result = spawnSync(process.execPath, ['rhwp-chrome/e2e/report.mjs', '--input', f.input, '--output', f.output], { encoding: 'utf8' });
  assert.equal(result.status, 1);
  assert.equal(JSON.parse(result.stdout).status, 'incomplete');
});

test('a foreign log cannot claim ownership by changing only its manifest hash', async t => {
  const a = await fixture(t), b = await fixture(t);
  const bytes = await readFile(path.join(b.input, 'extension-smoke.log'));
  await writeFile(path.join(a.input, 'extension-smoke.log'), bytes);
  await update(a, d => { d.results[0].artifacts[0].sha256 = sha(bytes); });
  assert.ok((await inspectRun(a.input)).summary.issues.includes('log-run-or-suite-mismatch'));
});

test('malformed artifact metadata is reported instead of crashing the report', async t => {
  const f = await fixture(t, 'fail');
  await update(f, d => { d.results[0].artifacts.push({ path: 123 }); });
  assert.equal((await inspectRun(f.input)).summary.status, 'incomplete');
});

test('truncated PNG evidence is incomplete even when its manifest hash matches', async t => {
  for (const bytes of [png.subarray(0, 8), png.subarray(0, -12), Buffer.concat([png, Buffer.from('trailing')])]) {
    const f = await fixture(t, 'fail');
    await writeFile(path.join(f.input, 'smoke-0.png'), bytes);
    await update(f, d => { d.results[0].artifacts.find(a => a.path === 'smoke-0.png').sha256 = sha(bytes); });
    const { summary, links } = await inspectRun(f.input);
    assert.equal(summary.sourceStatus, 'fail');
    assert.equal(summary.status, 'incomplete');
    assert.ok(summary.issues.includes('invalid-png'));
    assert.equal(links.some(x => x.kind === 'png'), false);
  }
});

test('contradictory overall error cannot coexist with a successful report', async t => {
  for (const error of ['verification failed', false, 0, {}]) {
    const f = await fixture(t);
    await update(f, d => { d.error = error; });
    const { summary } = await inspectRun(f.input);
    assert.equal(summary.sourceStatus, 'pass');
    assert.equal(summary.status, 'incomplete');
    assert.ok(summary.issues.includes('inconsistent-overall-pass'));
  }
});

test('candidate inventories require every runner-required file independently of their hash', async t => {
  for (const missing of ['manifest.json', 'viewer.html', 'wasm/rhwp.js', 'wasm/rhwp_bg.wasm']) {
    const f = await fixture(t);
    await update(f, d => {
      d.candidate.files = d.candidate.files.filter(file => file.path !== missing);
      d.candidate.sha256 = sha(JSON.stringify(d.candidate.files));
    });
    const { summary } = await inspectRun(f.input);
    assert.equal(summary.status, 'incomplete');
    assert.ok(summary.issues.includes('candidate-required-files-missing'));
  }
});

test('sharing guard rejects lower, upper and mixed-case script tags', () => {
  for (const payload of ['<script>alert(1)</script>', '<SCRIPT>alert(1)</SCRIPT>', '<ScRiPt>alert(1)</ScRiPt>']) {
    assert.throws(() => assertShareable(payload), assert.AssertionError);
  }
});
