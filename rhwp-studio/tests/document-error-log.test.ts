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
    id: 2,
    parentId: 1,
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
    id: 2,
    parentId: 1,
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
    '    #2 layout_body_picture(section_index=0, para_index=4, y_offset=32) ' +
    '=> frame_height=200, y=232 2.346ms',
  ].join('\n'));
});

test('a long layout trace is emitted only to its fixed bound', () => {
  const trace = parseLayoutTrace(JSON.stringify(Array.from({ length: 80 }, (_, paraIndex) => ({
    id: paraIndex + 1,
    parentId: null,
    function: 'flow_advance_height',
    args: { para_index: paraIndex, result_height: paraIndex + 0.5 },
    durationMs: 0.1,
    depth: 0,
  }))));
  assert.equal(trace.length, 64);
  assert.equal(trace[0].args.para_index, 16);
  assert.equal(trace[63].args.para_index, 79);
  const line = attachDocumentErrorTrace('paint: [page=1 ratio=0.1]', trace);
  assert.equal(isDocumentErrorLine(line), true);
  assert.ok(line.length < 16_384);
  assert.equal(JSON.parse(line.slice(line.indexOf(' trace=') + 7)).length, 64);

  const verbose = parseLayoutTrace(JSON.stringify(Array.from({ length: 64 }, (_, index) => ({
    id: index + 1,
    parentId: index || null,
    function: 'layout_frame_commit_row',
    args: Object.fromEntries(Array.from({ length: 10 }, (__, field) => [
      `field_${field}`,
      `${index}-${field}-${'x'.repeat(240)}`,
    ])),
    durationMs: 1,
    depth: 1,
  }))));
  const verboseLine = attachDocumentErrorTrace('paint: [page=1 ratio=0.1]', verbose);
  const attached = JSON.parse(verboseLine.slice(verboseLine.indexOf(' trace=') + 7));
  assert.deepEqual(attached.map((entry: { id: number }) => entry.id), [59, 60, 61, 62, 63, 64]);
  assert.equal(attached[0].parentId, attached[0].id - 1, 'a bounded suffix may retain an orphan parent id');
  const nextOlder = verbose.find(entry => entry.id === 58)!;
  assert.ok(
    `paint: [page=1 ratio=0.1] trace=${JSON.stringify([nextOlder, ...attached])}`.length > 16_384,
    'the next older invocation does not fit',
  );
});

test('layout trace preserves repeated calls and exact parent links', () => {
  const trace = parseLayoutTrace(JSON.stringify([
    {
      id: 1,
      parentId: null,
      function: 'layout_table_item',
      args: { page_index: 1, para_index: 13, control_index: 0 },
      durationMs: 1,
      depth: 0,
    },
    {
      id: 2,
      parentId: 1,
      function: 'layout_table_control_block',
      args: {
        page_index: 1,
        para_index: 13,
        control_index: 0,
        flow_y: 594.4,
        blocking_bottom: 627.867,
        spacing_before: 16,
        outer_margin_top: 3.773,
        result_table_top: 490,
        result_table_bottom: 510,
      },
      durationMs: 1,
      depth: 1,
    },
    {
      id: 3,
      parentId: null,
      function: 'layout_table_item',
      args: { page_index: 1, para_index: 14, control_index: 0 },
      durationMs: 1,
      depth: 0,
    },
  ]));
  assert.deepEqual(trace.map(entry => [entry.id, entry.parentId, entry.function]), [
    [1, null, 'layout_table_item'],
    [2, 1, 'layout_table_control_block'],
    [3, null, 'layout_table_item'],
  ]);
  const tenArgs = Object.fromEntries(Array.from({ length: 10 }, (_, index) => [`field_${index}`, index]));
  assert.equal(parseLayoutTrace(JSON.stringify([{
    id: 4, parentId: null, function: 'layout_frame', args: tenArgs, durationMs: 1, depth: 0,
  }])).length, 1);
  assert.deepEqual(parseLayoutTrace(JSON.stringify([{
    id: 5,
    parentId: null,
    function: 'layout_frame',
    args: { ...tenArgs, extra: true },
    durationMs: 1,
    depth: 0,
  }])), []);
});
