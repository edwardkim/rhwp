import test from 'node:test';
import assert from 'node:assert/strict';
import { DocumentOpenSession } from '../src/core/document-open-session.ts';

test('startup remains idle while an explicit open is pending, then becomes eligible again', () => {
  const session = new DocumentOpenSession(new Error('superseded'));
  assert.equal(session.pending, false);
  const signal = session.begin();
  assert.equal(session.pending, true);
  session.finish(signal);
  assert.equal(session.pending, false);
  assert.equal(signal.aborted, false, 'completion preserves the current document signal');
  assert.equal(session.signal, signal, 'failed replacement can restore the previous initialization with the current signal');
});

test('late completion of a superseded open cannot make the newer open idle', () => {
  const reason = new Error('superseded');
  const session = new DocumentOpenSession(reason);
  const previous = session.begin();
  const current = session.begin();
  assert.equal(previous.reason, reason);
  assert.equal(current.aborted, false);
  session.finish(previous);
  assert.equal(session.pending, true);
  session.finish(current);
  assert.equal(session.pending, false);
});

test('a failed open releases its pending state and a later open still cancels its signal', async () => {
  const reason = new Error('superseded');
  const session = new DocumentOpenSession(reason);
  const signal = session.begin();
  await assert.rejects(async () => {
    try { throw new Error('parse failed'); }
    finally { session.finish(signal); }
  }, /parse failed/);
  assert.equal(session.pending, false);
  session.begin();
  assert.equal(signal.reason, reason);
});
