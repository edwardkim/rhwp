import assert from 'node:assert/strict';
import test from 'node:test';
import {
  attachDocumentErrorTrace,
  formatDocumentError,
  formatDocumentErrorForTerminal,
  isDocumentErrorLine,
  parseLayoutTrace,
} from '../src/dev/document-error-log.ts';

test('a broken document formats as one recognizable CLI error line', () => {
  assert.equal(formatDocumentError('line-break', [
    ['page', 3],
    ['target', 's0/p4/c0.0.0/g2'],
    ['at', 1],
    ['expected', '0,37,77,114'],
    ['actual', '0,39,80,119'],
  ]), 'line-break: [page=3 target=s0/p4/c0.0.0/g2 at=1 expected=0,37,77,114 actual=0,39,80,119]');
});

test('document text cannot inject extra terminal output into the error line', () => {
  for (const invalid of ['visible\u001bred', 'two words', 'a\ud800b']) {
    assert.throws(() => formatDocumentError('paint', [['detail', invalid]]), /invalid document error value: detail/);
  }
});

test('actual layout inputs and computed heights stay attached to the document error', () => {
  const trace = parseLayoutTrace(JSON.stringify([{
    function: 'layout_body_picture',
    args: {
      section_index: 0,
      para_index: 4,
      y_offset: 32,
      result_frame_height: 200,
      result_y: 232,
    },
    durationMs: 2.3456,
    depth: 1,
  }]));
  assert.deepEqual(trace, [{
    function: 'layout_body_picture',
    args: {
      section_index: 0,
      para_index: 4,
      y_offset: 32,
      result_frame_height: 200,
      result_y: 232,
    },
    durationMs: 2.346,
    depth: 1,
  }]);
  const line = attachDocumentErrorTrace(
    'line-break: [page=3 target=s0/p4 at=1 expected=37:single actual=35:single]',
    trace,
  );
  assert.equal(isDocumentErrorLine(line), true);
  assert.equal(formatDocumentErrorForTerminal(line), [
    'line-break: [page=3 target=s0/p4 at=1 expected=37:single actual=35:single]',
    'trace:',
    '    layout_body_picture(section_index=0, para_index=4, y_offset=32) ' +
    '=> frame_height=200, y=232 2.346ms',
  ].join('\n'));
});

test('a long layout trace is emitted only to its fixed bound', () => {
  const trace = parseLayoutTrace(JSON.stringify(Array.from({ length: 24 }, (_, paraIndex) => ({
    function: 'flow_advance_height',
    args: { para_index: paraIndex, result_height: paraIndex + 0.5 },
    durationMs: 0.1,
    depth: 0,
  }))));
  assert.equal(trace.length, 16);
  assert.equal(trace[0].args.para_index, 8);
  assert.equal(trace[15].args.para_index, 23);
});
