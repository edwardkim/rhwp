import assert from 'node:assert/strict';
import test from 'node:test';

import { isBodyControl, isMasterPageDecoration, isSupportedPictureControl } from '../src/engine/picture-hit-policy.ts';
import { findNearestConnectionPoint } from '../src/engine/input-handler-connector.ts';

test('master-page front image does not capture a body-text click', () => {
  assert.equal(isMasterPageDecoration({ plane: 1 }), true);
});

test('document and header/footer foreground images remain selectable', () => {
  assert.equal(isMasterPageDecoration({ plane: 2 }), false);
  assert.equal(isMasterPageDecoration({ plane: 1, headerFooter: { kind: 'header' } }), false);
});

test('HF source addresses never become body shape edit targets', () => {
  for (const type of ['group', 'shape', 'line', 'table', 'equation', 'ole']) {
    const control = { type, headerFooter: { kind: 'footer', outerParaIdx: 0, outerControlIdx: 1 } };
    assert.equal(isBodyControl(control), false);
    assert.equal(isSupportedPictureControl(control), false);
    assert.equal(isSupportedPictureControl({ type }), true);
  }
  assert.equal(isSupportedPictureControl({ type: 'image', headerFooter: {} }), true);
  assert.equal(isSupportedPictureControl({ type: 'image', headerFooter: {}, missing: true }), false);
  assert.equal(isSupportedPictureControl({ type: 'image', headerFooter: {}, cellPath: [{}] }), false);
});

test('connector lookup ignores HF groups and images even with valid section addresses', () => {
  const base = { x: 0, y: 0, w: 20, h: 20, secIdx: 0, paraIdx: 0, controlIdx: 0 };
  const controls = ['group', 'image'].map(type => ({ ...base, type, headerFooter: { kind: 'footer' } }));
  const host = { wasm: { getPageControlLayout: () => ({ controls }) } };
  assert.equal(findNearestConnectionPoint.call(host, 0, 10, 0), null);
  const withBody = { wasm: { getPageControlLayout: () => ({ controls: [...controls, { ...base, type: 'shape', controlIdx: 3 }] }) } };
  assert.deepEqual(findNearestConnectionPoint.call(withBody, 0, 10, 0), {
    x: 10, y: 0, index: 0, sec: 0, ppi: 0, ci: 3, instanceId: 0,
  });
});
