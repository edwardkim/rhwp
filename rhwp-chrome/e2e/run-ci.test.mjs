import assert from 'node:assert/strict';
import { existsSync } from 'node:fs';
import test from 'node:test';
import { runSuite } from './run-ci.mjs';

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
