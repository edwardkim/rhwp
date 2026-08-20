import assert from 'node:assert/strict';
import { EventEmitter } from 'node:events';
import test from 'node:test';
import { attachPageBudget } from './extension-smoke.test.mjs';

function createTarget(type, url = '') {
  return {
    type: () => type,
    url: () => url,
  };
}

test('unexpected page rejects an in-flight surface without waiting for it', { timeout: 1_000 }, async () => {
  const browser = new EventEmitter();
  const ownedTarget = createTarget('page', 'about:blank');
  const ownedPage = { target: () => ownedTarget };
  const diagnostics = { unexpectedPageTargets: [] };
  const budget = attachPageBudget(browser, ownedPage, diagnostics);

  try {
    const inFlightSurface = new Promise(() => {});
    const guardedSurface = budget.guard(inFlightSurface);
    browser.emit('targetcreated', createTarget('page', 'chrome-extension://unexpected/viewer.html'));

    await assert.rejects(
      guardedSurface,
      /예기치 않은 page target이 생성되었습니다/,
    );
    assert.deepEqual(
      diagnostics.unexpectedPageTargets,
      ['chrome-extension://unexpected/viewer.html'],
    );
  } finally {
    budget.detach();
  }
});

test('owned page and non-page targets stay within the page budget', async () => {
  const browser = new EventEmitter();
  const ownedTarget = createTarget('page', 'about:blank');
  const ownedPage = { target: () => ownedTarget };
  const diagnostics = { unexpectedPageTargets: [] };
  const budget = attachPageBudget(browser, ownedPage, diagnostics);

  try {
    browser.emit('targetcreated', ownedTarget);
    browser.emit('targetcreated', createTarget('service_worker', 'chrome-extension://worker/background.js'));
    assert.equal(await budget.guard(Promise.resolve('completed')), 'completed');
    assert.deepEqual(diagnostics.unexpectedPageTargets, []);
  } finally {
    budget.detach();
  }
});
