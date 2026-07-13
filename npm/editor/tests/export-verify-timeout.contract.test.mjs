import test from 'node:test';
import assert from 'node:assert/strict';

import { requestTimeoutFor } from '../transport.js';

test('exportHwpVerify uses the documented export timeout', () => {
  assert.equal(requestTimeoutFor('exportHwpVerify'), 60_000);
});
