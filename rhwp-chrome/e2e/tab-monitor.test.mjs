import assert from 'node:assert/strict';
import { EventEmitter } from 'node:events';
import test from 'node:test';
import { monitorTabs } from './tab-monitor.mjs';

test('a second page rejects an in-flight operation immediately, including transient tabs', async () => {
  const browser = new EventEmitter();
  const target = () => ({ type: () => 'page', url: () => '' });
  const monitor = monitorTabs(browser, target(), 1, []);
  const pending = monitor.guard(new Promise(() => {}));
  browser.emit('targetcreated', target());
  browser.emit('targetcreated', target());
  await assert.rejects(pending, /expected 1, created 2/);
  monitor.detach();
  assert.equal(browser.listenerCount('targetcreated'), 0);
  assert.equal(browser.listenerCount('targetchanged'), 0);
});

test('zero budget ignores the owned page and workers but rejects a blank popup', async () => {
  const browser = new EventEmitter();
  const owned = { type: () => 'page' };
  const monitor = monitorTabs(browser, owned, 0, []);
  browser.emit('targetcreated', owned);
  browser.emit('targetcreated', { type: () => 'service_worker' });
  await monitor.guard(Promise.resolve());
  browser.emit('targetcreated', { type: () => 'page', url: () => 'about:blank' });
  await assert.rejects(monitor.guard(Promise.resolve()), /expected 0, created 1/);
  monitor.detach();
});
