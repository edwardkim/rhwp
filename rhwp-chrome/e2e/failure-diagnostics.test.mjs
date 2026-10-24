import assert from 'node:assert/strict';
import test from 'node:test';
import { recordConsoleMessage } from './failure-diagnostics.mjs';

const error = url => ({ type: () => 'error', text: () => 'Failed to load resource', location: () => ({ url }) });

test('browser-owned downloads diagnostics remain visible without failing extension assertions', () => {
  const diagnostic = { stage: 'verify-persisted-history', console: [], errors: [] };
  recordConsoleMessage(diagnostic, error('chrome://file-icon/fixture'), 'chrome://downloads/');
  assert.equal(diagnostic.console.length, 1);
  assert.equal(diagnostic.browserUiErrors.length, 1);
  assert.deepEqual(diagnostic.errors, []);
});

test('extension and fixture errors stay fatal even after navigation to browser-owned UI', () => {
  const diagnostic = { stage: 'verify-persisted-history', console: [], errors: [] };
  recordConsoleMessage(diagnostic, error('chrome-extension://test/options.js'), 'chrome://downloads/');
  recordConsoleMessage(diagnostic, error('http://127.0.0.1:1234/fixture.html'), 'chrome://downloads/');
  recordConsoleMessage(diagnostic, error(''), 'chrome-extension://test/options.html');
  recordConsoleMessage(diagnostic, error(''), 'chrome://downloads/');
  assert.equal(diagnostic.errors.length, 4);
  assert.equal(diagnostic.browserUiErrors, undefined);
});

test('failure JSON carries the same runner run ID without copying profiles or documents', async t => {
  const { mkdtemp, readFile, readdir, rm, writeFile } = await import('node:fs/promises');
  const { default: os } = await import('node:os');
  const { default: path } = await import('node:path');
  const { saveFailureDiagnostics } = await import('./failure-diagnostics.mjs');
  const directory = await mkdtemp(path.join(os.tmpdir(), 'rhwp-diagnostic-binding-'));
  const oldOutput = process.env.RHWP_EXTENSION_E2E_OUTPUT_DIR;
  const oldRun = process.env.RHWP_EXTENSION_E2E_RUN_ID;
  t.after(async () => {
    if (oldOutput === undefined) delete process.env.RHWP_EXTENSION_E2E_OUTPUT_DIR; else process.env.RHWP_EXTENSION_E2E_OUTPUT_DIR = oldOutput;
    if (oldRun === undefined) delete process.env.RHWP_EXTENSION_E2E_RUN_ID; else process.env.RHWP_EXTENSION_E2E_RUN_ID = oldRun;
    await rm(directory, { recursive: true, force: true });
  });
  process.env.RHWP_EXTENSION_E2E_OUTPUT_DIR = directory;
  process.env.RHWP_EXTENSION_E2E_RUN_ID = '11111111-1111-1111-1111-111111111111';
  const page = { url: () => 'chrome-extension://fixture/options.html', screenshot: async ({ path }) => writeFile(path, 'synthetic screenshot') };
  await saveFailureDiagnostics({ targets: () => [], pages: async () => [page] }, 'smoke', { stage: 'options-save' });
  const data = JSON.parse(await readFile(path.join(directory, 'smoke.json')));
  assert.equal(data.runId, process.env.RHWP_EXTENSION_E2E_RUN_ID);
  assert.deepEqual((await readdir(directory)).sort(), ['smoke-0.png', 'smoke.json']);
});
