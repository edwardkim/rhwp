import assert from 'node:assert/strict';
import { existsSync } from 'node:fs';
import test from 'node:test';
import { runSuite } from './run.mjs';

test('runner preserves failures, bounds diagnostic text and cleans its temporary files', async () => {
  const result = await runSuite(['-e', `
    const fs = require('node:fs');
    const os = require('node:os');
    fs.writeFileSync(os.tmpdir() + '/owned-download', 'fixture');
    // One pipe preserves the marker after the large payload on every OS.
    // stdout/stderr are independent pipes and their delivery order may differ.
    process.stdout.write('x'.repeat(300000) + '\\n' + os.tmpdir() + '\\n');
    process.exitCode = 7;
  `], { timeoutMs: 5_000 });
  assert.equal(result.status, 7);
  assert.equal(result.timedOut, false);
  assert.ok(result.log.length <= 256 * 1024);
  const directory = result.log.trim().split('\n').at(-1);
  assert.match(directory, /rhwp-e2e-runner-/);
  assert.equal(existsSync(directory), false);
});

test('runner terminates a stuck child without treating the timeout as success', async () => {
  const result = await runSuite(['-e', `process.on('SIGTERM', () => {}); setInterval(() => {}, 1000);`], {
    timeoutMs: 500, killGraceMs: 100,
  });
  assert.equal(result.timedOut, true);
  assert.notEqual(result.status, 0);
  assert.ok(result.durationMs < 5_000);
  assert.match(result.log, /stopped without retry/);
});

import { mkdtemp, mkdir, readFile, writeFile, rm, symlink, realpath } from 'node:fs/promises';
import os from 'node:os';
import path from 'node:path';
import { inspectCandidate, parseArgs, runChecks } from './run.mjs';

async function fixture(t) {
  const dir = await mkdtemp(path.join(os.tmpdir(), 'rhwp-run-contract-'));
  t.after(() => rm(dir, { recursive: true, force: true }));
  const dist = path.join(dir, 'candidate with spaces');
  await mkdir(path.join(dist, 'wasm'), { recursive: true });
  for (const [name, text] of Object.entries({ 'manifest.json': '{"manifest_version":3,"version":"1.2.3"}',
    'viewer.html': 'viewer', 'wasm/rhwp.js': 'js', 'wasm/rhwp_bg.wasm': 'wasm' })) {
    await writeFile(path.join(dist, name), text);
  }
  return { dist, output: path.join(dir, 'report'), dir };
}
const success = async () => ({ status: 0, signal: null, timedOut: false, durationMs: 1, log: 'ok' });
const dependencies = { runner: success, environment: async () => ({ chrome: 'test-double' }), env: {} };

test('CLI requires an explicit candidate and rejects ambiguous arguments', () => {
  assert.throws(() => parseArgs([]), /--dist is required/);
  assert.throws(() => parseArgs(['--dist', '--output']), /invalid argument/);
  assert.throws(() => parseArgs(['--dist', 'x', '--dist', 'y']), /Duplicate/);
  assert.equal(parseArgs(['--help']).help, true);
  assert.equal(parseArgs(['--dist', 'x']).dist, path.resolve('x'));
});

test('candidate inventory binds paths and bytes, independently of directory location', async t => {
  const { dist, dir } = await fixture(t);
  const before = await inspectCandidate(dist);
  const { cp } = await import('node:fs/promises');
  await cp(dist, path.join(dir, 'copy'), { recursive: true });
  assert.equal((await inspectCandidate(path.join(dir, 'copy'))).sha256, before.sha256);
  await writeFile(path.join(dist, 'viewer.html'), 'changed');
  assert.notEqual((await inspectCandidate(dist)).sha256, before.sha256);
  await symlink(path.join(dist, 'viewer.html'), path.join(dist, 'linked'));
  await assert.rejects(inspectCandidate(dist), /symlinks/);
});

test('all suites receive the same selected candidate and an isolated report directory', async t => {
  const options = await fixture(t);
  const calls = [];
  const { report, output } = await runChecks(options, { ...dependencies, runner: async (args, settings) => {
    calls.push({ args, settings }); return success();
  } });
  assert.equal(report.status, 'pass');
  assert.equal(calls.length, 3);
  const selectedPath = await realpath(options.dist);
  assert.ok(calls.every(call => call.settings.env.RHWP_EXTENSION_DIST_DIR === selectedPath));
  assert.ok(calls.every(call => call.settings.env.RHWP_EXTENSION_E2E_OUTPUT_DIR === output));
  assert.equal(report.candidateUnchanged, true);
  assert.equal(JSON.parse(await readFile(path.join(output, 'result.json'))).status, 'pass');
  await assert.rejects(runChecks(options, dependencies), /EEXIST/);
});

test('failed suite remains a failure and later suites are explicitly not run', async t => {
  const { report } = await runChecks(await fixture(t), { ...dependencies,
    runner: async () => ({ status: 7, timedOut: false, log: 'failure' }) });
  assert.equal(report.status, 'fail');
  assert.deepEqual(report.results.map(x => x.status), ['fail', 'not-run', 'not-run']);
  assert.equal(report.results[0].exitCode, 7);
});

test('timeout or spawn error is never counted as pass', async t => {
  for (const failure of [{ timedOut: true }, { error: 'spawn failed' }]) {
    const { report } = await runChecks(await fixture(t), { ...dependencies,
      runner: async () => ({ status: 0, log: '', ...failure }) });
    assert.equal(report.status, 'fail');
  }
});

test('preflight failure writes a fresh error report with no successful tests', async t => {
  const options = await fixture(t);
  await rm(path.join(options.dist, 'wasm/rhwp_bg.wasm'));
  const { report, output } = await runChecks(options, dependencies);
  assert.equal(report.status, 'error');
  assert.match(report.error, /Incomplete candidate/);
  assert.ok(report.results.every(x => x.status === 'not-run'));
  assert.equal(JSON.parse(await readFile(path.join(output, 'result.json'))).status, 'error');
});

test('partial selection cannot masquerade as a complete run', async t => {
  const { report } = await runChecks(await fixture(t), { ...dependencies,
    env: { RHWP_EXTENSION_LIFECYCLE_CASE: 'off-reenter' } });
  assert.equal(report.status, 'error');
  assert.match(report.error, /must be unset/);
});

test('candidate mutation during execution invalidates otherwise successful suites', async t => {
  const options = await fixture(t);
  const { report } = await runChecks(options, { ...dependencies, runner: async () => {
    await writeFile(path.join(options.dist, 'viewer.html'), 'changed'); return success();
  } });
  assert.equal(report.status, 'error');
  assert.equal(report.candidateUnchanged, false);
});

test('output cannot change the candidate, including a symlinked parent', async t => {
  const options = await fixture(t);
  await assert.rejects(runChecks({ ...options, output: path.join(options.dist, 'out') }, dependencies), /outside/);
  await assert.rejects(runChecks({ ...options, dist: path.parse(options.dist).root }, dependencies), /outside/);
  const alias = path.join(options.dir, 'alias');
  await symlink(options.dist, alias);
  await assert.rejects(runChecks({ ...options, output: path.join(alias, 'new', 'out') }, dependencies), /outside/);
  assert.equal(existsSync(path.join(options.dist, 'new')), false);
});
