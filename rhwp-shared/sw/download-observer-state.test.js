import { test } from 'node:test';
import { strict as assert } from 'node:assert';

import {
  DEFAULT_FRESH_GRACE_MS,
  evaluateDownloadChanged,
  evaluateDownloadCreated,
  isDownloadStateExpired,
  markDownloadHandled,
  markDownloadTerminal,
  parseDownloadTime,
  shouldRecheckDownload,
} from './download-observer-state.js';

const NOW = Date.parse('2026-06-24T12:00:00.000Z');

test('parseDownloadTime returns milliseconds for valid ISO time', () => {
  assert.equal(parseDownloadTime('2026-06-24T12:00:00.000Z'), NOW);
});

test('parseDownloadTime returns null for invalid values', () => {
  assert.equal(parseDownloadTime(''), null);
  assert.equal(parseDownloadTime('not-a-date'), null);
  assert.equal(parseDownloadTime(undefined), null);
});

test('created past completed item is ignored', () => {
  const decision = evaluateDownloadCreated({
    id: 1,
    url: 'https://example.com/old.hwp',
    filename: 'old.hwp',
    state: 'complete',
    startTime: '2000-01-01T00:00:00.000Z',
    endTime: '2000-01-01T00:00:01.000Z',
  }, null, NOW);

  assert.equal(decision.action, 'ignore');
  assert.equal(decision.reason, 'past-created');
});

test('created complete item without time is ignored conservatively', () => {
  const decision = evaluateDownloadCreated({
    id: 2,
    url: 'https://example.com/old-no-time.hwp',
    filename: 'old-no-time.hwp',
    state: 'complete',
  }, null, NOW);

  assert.equal(decision.action, 'ignore');
  assert.equal(decision.reason, 'past-created');
});

test('fresh created item is tracked', () => {
  const decision = evaluateDownloadCreated({
    id: 3,
    url: 'https://example.com/fresh.hwp',
    filename: 'fresh.hwp',
    startTime: new Date(NOW).toISOString(),
  }, null, NOW);

  assert.equal(decision.action, 'track');
  assert.equal(decision.state.id, 3);
  assert.equal(decision.state.firstSeenAt, NOW);
  assert.equal(decision.state.itemStartTime, NOW);
});

test('fresh created item within grace window is tracked', () => {
  const decision = evaluateDownloadCreated({
    id: 4,
    url: 'https://example.com/fresh-grace.hwp',
    filename: 'fresh-grace.hwp',
    startTime: new Date(NOW - DEFAULT_FRESH_GRACE_MS + 1).toISOString(),
  }, null, NOW);

  assert.equal(decision.action, 'track');
});

test('changed event without stored state is ignored', () => {
  const decision = evaluateDownloadChanged(
    { id: 5, filename: { current: '/Downloads/new.hwp' } },
    { id: 5, filename: 'new.hwp', url: 'https://example.com/new.hwp' },
    null,
    NOW,
  );

  assert.equal(decision.action, 'ignore');
  assert.equal(decision.reason, 'untracked-changed');
});

test('tracked changed item becomes candidate', () => {
  const created = evaluateDownloadCreated({
    id: 6,
    url: 'https://example.com/download?id=6',
    filename: 'download',
    startTime: new Date(NOW).toISOString(),
  }, null, NOW);

  const changed = evaluateDownloadChanged(
    { id: 6, filename: { current: '/Downloads/fresh.hwp' } },
    { id: 6, url: 'https://example.com/download?id=6', filename: 'fresh.hwp' },
    created.state,
    NOW + 1000,
  );

  assert.equal(changed.action, 'candidate');
  assert.equal(changed.state.firstSeenAt, NOW);
  assert.equal(changed.state.lastReason, 'tracked-changed');
});

test('tracked changed item with old startTime is ignored', () => {
  const created = evaluateDownloadCreated({
    id: 7,
    url: 'https://example.com/download?id=7',
    filename: 'download',
  }, null, NOW);

  const changed = evaluateDownloadChanged(
    { id: 7, filename: { current: '/Downloads/old.hwp' } },
    {
      id: 7,
      url: 'https://example.com/old.hwp',
      filename: 'old.hwp',
      startTime: '2000-01-01T00:00:00.000Z',
      endTime: '2000-01-01T00:00:01.000Z',
    },
    created.state,
    NOW + 1000,
  );

  assert.equal(changed.action, 'ignore');
  assert.equal(changed.reason, 'past-changed');
});

test('handled state is not returned as candidate again', () => {
  const created = evaluateDownloadCreated({
    id: 8,
    url: 'https://example.com/fresh.hwp',
    filename: 'fresh.hwp',
  }, null, NOW);
  const handled = markDownloadHandled(created.state, NOW + 100);
  const changed = evaluateDownloadChanged(
    { id: 8, filename: { current: '/Downloads/fresh.hwp' } },
    { id: 8, filename: 'fresh.hwp', url: 'https://example.com/fresh.hwp' },
    handled,
    NOW + 1000,
  );

  assert.equal(changed.action, 'ignore');
  assert.equal(changed.reason, 'already-handled');
});

test('expired handled state does not block new created item forever', () => {
  const oldHandled = {
    id: 81,
    firstSeenAt: NOW - 120_000,
    handledAt: NOW - 120_000,
    terminalAt: null,
    lastReason: 'opened',
  };

  const created = evaluateDownloadCreated({
    id: 81,
    url: 'https://example.com/new-after-expired.hwp',
    filename: 'new-after-expired.hwp',
    startTime: new Date(NOW).toISOString(),
  }, oldHandled, NOW, { stateTtlMs: 30_000 });

  assert.equal(created.action, 'track');
  assert.equal(created.state.firstSeenAt, NOW);
});

test('terminal state keeps firstSeenAt and stores terminalAt', () => {
  const created = evaluateDownloadCreated({
    id: 9,
    url: 'https://example.com/fresh.hwp',
    filename: 'fresh.hwp',
  }, null, NOW);
  const terminal = markDownloadTerminal(created.state, NOW + 2000);

  assert.equal(terminal.firstSeenAt, NOW);
  assert.equal(terminal.terminalAt, NOW + 2000);
});

test('old stored state is expired', () => {
  const state = { id: 10, firstSeenAt: NOW - 60_000, handledAt: null, terminalAt: null };

  assert.equal(isDownloadStateExpired(state, NOW, 30_000), true);
  assert.equal(isDownloadStateExpired(state, NOW, 120_000), false);
});

test('shouldRecheckDownload matches filename finalUrl or complete state', () => {
  assert.equal(shouldRecheckDownload({ filename: { current: '/Downloads/a.hwp' } }), true);
  assert.equal(shouldRecheckDownload({ finalUrl: { current: 'https://example.com/a.hwp' } }), true);
  assert.equal(shouldRecheckDownload({ state: { current: 'complete' } }), true);
  assert.equal(shouldRecheckDownload({ state: { current: 'in_progress' } }), false);
});
