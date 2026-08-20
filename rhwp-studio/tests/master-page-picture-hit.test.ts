import assert from 'node:assert/strict';
import test from 'node:test';

import { isMasterPageDecoration } from '../src/engine/picture-hit-policy.ts';

test('master-page front image does not capture a body-text click', () => {
  assert.equal(isMasterPageDecoration({ plane: 1 }), true);
});

test('document and header/footer foreground images remain selectable', () => {
  assert.equal(isMasterPageDecoration({ plane: 2 }), false);
  assert.equal(isMasterPageDecoration({ plane: 1, headerFooter: { kind: 'header' } }), false);
});
