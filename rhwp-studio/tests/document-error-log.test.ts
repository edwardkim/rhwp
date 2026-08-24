import assert from 'node:assert/strict';
import test from 'node:test';
import { formatDocumentError } from '../src/dev/document-error-log.ts';

test('formats one typed CLI line from flat document attributes', () => {
  assert.equal(formatDocumentError('line-break', [
    ['page', 3],
    ['target', 's0/p4/c0.0.0/g2'],
    ['at', 1],
    ['expected', '0,37,77,114'],
    ['actual', '0,39,80,119'],
  ]), 'line-break: [page=3 target=s0/p4/c0.0.0/g2 at=1 expected=0,37,77,114 actual=0,39,80,119]');
  for (const invalid of ['visible\u001bred', 'two words', 'a\ud800b']) {
    assert.throws(() => formatDocumentError('paint', [['detail', invalid]]), /invalid document error value: detail/);
  }
});
