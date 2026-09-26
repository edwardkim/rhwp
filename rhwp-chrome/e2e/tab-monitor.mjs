import assert from 'node:assert/strict';

// Keep the creation history: a second tab that immediately closes is still a violation.
export function monitorTabs(browser, ownedTarget, expected, events) {
  const created = new Set();
  let rejectFailure;
  let failure;
  const failed = new Promise((_, reject) => { rejectFailure = reject; });
  // An event can arrive between guarded operations.
  void failed.catch(() => {});
  const onCreated = target => {
    if (target.type() !== 'page' || target === ownedTarget || created.has(target)) return;
    created.add(target);
    events.push({ event: 'page-created', url: target.url(), count: created.size });
    if (created.size > expected && !failure) {
      failure = new Error(`Tab budget exceeded: expected ${expected}, created ${created.size}`);
      rejectFailure(failure);
    }
  };
  const onChanged = target => {
    if (created.has(target)) events.push({ event: 'page-url', url: target.url() });
  };
  browser.on('targetcreated', onCreated);
  browser.on('targetchanged', onChanged);
  return {
    guard(operation) { return Promise.race([failed, Promise.resolve(operation)]); },
    assertComplete(extensionId) {
      if (failure) throw failure;
      assert.equal(created.size, expected, 'viewer creation count');
      for (const target of created) {
        assert.ok(browser.targets().includes(target), 'created viewer must remain open');
        assert.ok(target.url().startsWith(`chrome-extension://${extensionId}/viewer.html?`),
          `unexpected page: ${target.url()}`);
      }
    },
    targets() { return [...created]; },
    detach() {
      browser.off('targetcreated', onCreated);
      browser.off('targetchanged', onChanged);
    },
  };
}
