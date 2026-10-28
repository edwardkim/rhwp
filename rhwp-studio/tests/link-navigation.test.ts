import test from 'node:test';
import assert from 'node:assert/strict';
import { navigateLink, setLinkHostHandler, type LinkDestination } from '../src/core/link-navigation.ts';

test('link navigation routes all six destinations only when requested and reports browser fallbacks', async () => {
  const events: string[] = [];
  const context = { fileName: 'source.hwp', sourceFormat: 'hwp' };
  const actions = { anchor: (name: string) => { events.push(`anchor:${name}`); return name === '목적지'; },
    web: (uri: string) => { events.push(`web:${uri}`); return false; },
    mail: (uri: string) => { events.push(`mail:${uri}`); return true; },
    browserFallback: (destination: LinkDestination) => { events.push(`fallback:${destination.kind}`); } };
  setLinkHostHandler(null);
  assert.equal(events.length, 0);
  assert.equal(await navigateLink({ kind: 'documentAnchor', uri: '#%EB%AA%A9%EC%A0%81%EC%A7%80' }, context, actions), true);
  assert.equal(await navigateLink({ kind: 'web', uri: 'https://example.com/' }, context, actions), false);
  assert.equal(await navigateLink({ kind: 'mail', uri: 'mailto:a@example.com' }, context, actions), true);
  for (const kind of ['relativeFile', 'localFile', 'registeredApplication'] as const) {
    assert.equal(await navigateLink({ kind, uri: 'destination' }, context, actions), false);
  }
  assert.deepEqual(events, ['anchor:목적지', 'web:https://example.com/', 'mail:mailto:a@example.com',
    'fallback:relativeFile', 'fallback:localFile', 'fallback:registeredApplication']);
  assert.equal(navigateLink({ kind: 'documentAnchor', uri: '#%ff' }, context, actions), false);
});

test('a host receives immutable copies, may decline, and failures remain observable', async () => {
  const destination: LinkDestination = { kind: 'localFile', uri: 'file:///tmp/target.hwp' };
  const context = { fileName: 'source.hwp', sourceFormat: 'hwp' };
  let fallback = 0;
  const actions = { anchor: () => false, web: () => false, mail: () => false, browserFallback: () => { fallback++; } };
  try {
    setLinkHostHandler((value, origin) => {
      assert.notEqual(value, destination); assert.notEqual(origin, context);
      assert.ok(Object.isFrozen(value)); assert.ok(Object.isFrozen(origin)); return true;
    });
    assert.equal(await navigateLink(destination, context, actions), true); assert.equal(fallback, 0);
    setLinkHostHandler(async () => false);
    assert.equal(await navigateLink(destination, context, actions), false); assert.equal(fallback, 1);
    setLinkHostHandler(async () => { throw new Error('host unavailable'); });
    await assert.rejects(navigateLink(destination, context, actions), /host unavailable/);
  } finally { setLinkHostHandler(null); }
});
