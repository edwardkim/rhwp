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
