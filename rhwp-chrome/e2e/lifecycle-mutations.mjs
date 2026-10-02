#!/usr/bin/env node
import assert from 'node:assert/strict';
import { cp, mkdtemp, readFile, rm, writeFile } from 'node:fs/promises';
import os from 'node:os';
import path from 'node:path';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../..');
const temp = await mkdtemp(path.join(os.tmpdir(), 'rhwp-lifecycle-mutations-'));
function run(args, env = {}) {
  const result = spawnSync(process.execPath, args, {
    cwd: root, env: { ...process.env, ...env }, encoding: 'utf8', timeout: 120_000,
    maxBuffer: 2 * 1024 * 1024,
  });
  if (result.error) throw result.error;
  return { status: result.status, output: result.stdout + result.stderr };
}
async function replace(file, before, after) {
  const original = await readFile(file, 'utf8');
  assert.ok(original.includes(before), `mutation site missing: ${before}`);
  await writeFile(file, original.replace(before, after));
}
try {
  const stateTests = 'rhwp-shared/sw/download-observer-state.test.js';
  const control = run(['--test', stateTests]);
  assert.equal(control.status, 0, control.output);
  await writeFile(path.join(temp, 'package.json'), '{"type":"module"}');
  await cp(path.join(root, stateTests), path.join(temp, 'download-observer-state.test.js'));
  const state = path.join(temp, 'download-observer-state.js');
  await cp(path.join(root, 'rhwp-shared/sw/download-observer-state.js'), state);
  await replace(state, 'if (!item) return true;', 'return false; // mutation: accept old items\n  if (!item) return true;');
  const freshness = run(['--test', path.join(temp, 'download-observer-state.test.js')]);
  assert.equal(freshness.status, 1, freshness.output);
  assert.match(freshness.output, /past|old|stale/i);
  process.stdout.write(`PASS mutation freshness: Node state contract failed as expected\n${freshness.output}\n`);

  const dist = path.join(temp, 'dist');
  await cp(path.join(root, 'rhwp-chrome/dist'), dist, { recursive: true });
  const lifecycle = 'rhwp-chrome/e2e/extension-lifecycle.test.mjs';
  const env = { RHWP_EXTENSION_DIST_DIR: dist, RHWP_EXTENSION_LIFECYCLE_CASE: 'on-download-hwp', RHWP_EXTENSION_LIFECYCLE_REPEAT: '1' };
  const browserControl = run([lifecycle], env);
  assert.equal(browserControl.status, 0, browserControl.output);
  const adapter = path.join(dist, 'sw/download-interceptor.js');
  await replace(adapter, 'state && !state.handledAt && shouldRecheckDownload(delta)', 'state && shouldRecheckDownload(delta)');
  await replace(adapter, 'if (!item || state?.handledAt) return state;', 'if (!item) return state;');
  await replace(adapter, 'if (latestState?.handledAt) return latestState;', '// mutation: ignore handled marker');
  await replace(path.join(dist, 'sw/download-observer-state.js'), 'if (previousState.handledAt) {', 'if (false) { // mutation: ignore handled marker');
  const duplicate = run([lifecycle], env);
  assert.equal(duplicate.status, 1, duplicate.output);
  assert.match(duplicate.output, /Tab budget exceeded: expected 1, created 2/);
  process.stdout.write('PASS mutation duplicate: actual Chrome emitted a second viewer and the tab monitor failed immediately\n');
  process.stdout.write(duplicate.output + '\n');
  process.stdout.write('Freshness is proved by the Node state contract; completed Chrome history does not replay onCreated and is not claimed to detect that mutation.\n');
} finally {
  await rm(temp, { recursive: true, force: true, maxRetries: 3, retryDelay: 100 });
}
