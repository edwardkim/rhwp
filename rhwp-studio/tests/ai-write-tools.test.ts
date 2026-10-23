import test from 'node:test';
import assert from 'node:assert/strict';

import {
  WRITE_TOOL_DEFS,
  WriteRefused,
  isWriteTool,
  prepareWrite,
  type DocumentWriter,
  type WriteHost,
} from '../src/ai/tools/write-tools.ts';

/** 본문 문단 두 개와 표 칸 문단 하나, 누름틀 둘을 가진 작은 가짜 문서. */
function fakeDoc() {
  const body = ['사업명은 홈페이지 구축이다', '과업기간 X월 20일'];
  const cell = ['사 업 수 행 계 획 서'];
  const fields = [
    { fieldId: 11, name: '성명', value: '' },
    { fieldId: 12, name: '연락처', value: '010' },
    { fieldId: 13, name: '연락처', value: '' },
  ];
  const calls: string[] = [];
  const find = (q: string) => {
    const hits: unknown[] = [];
    body.forEach((text, para) => {
      for (let i = text.indexOf(q); i >= 0; i = text.indexOf(q, i + 1)) hits.push({ sec: 0, para, charOffset: i, length: q.length });
    });
    for (let i = cell[0].indexOf(q); i >= 0; i = cell[0].indexOf(q, i + 1)) {
      hits.push({ sec: 0, para: 5, charOffset: i, length: q.length, cellContext: { parentPara: 5, ctrlIdx: 0, cellIdx: 1, cellPara: 0 } });
    }
    return hits;
  };
  const doc: DocumentWriter = {
    searchAllText: (q) => JSON.stringify(find(q)),
    getParagraphLength: (_s, p) => body[p].length,
    getTextRange: (_s, p, o, c) => body[p].slice(o, o + c),
    // 글이 있는 칸은 칸 1 의 첫 문단뿐이다.
    getCellParagraphLength: (_s, _pp, _c, ci, cp) => (ci === 1 && cp === 0 ? cell[0].length : 0),
    getTextInCell: (_s, _pp, _c, ci, cp, o, n) => (ci === 1 && cp === 0 ? cell[0].slice(o, o + n) : ''),
    replaceText: (_s, p, o, l, text) => {
      calls.push(`replace ${p} ${o} ${l} ${text}`);
      body[p] = body[p].slice(0, o) + text + body[p].slice(o + l);
      return JSON.stringify({ ok: true });
    },
    deleteTextInCell: (_s, _pp, _c, _ci, _cp, o, n) => {
      calls.push(`cell-delete ${o} ${n}`);
      cell[0] = cell[0].slice(0, o) + cell[0].slice(o + n);
      return JSON.stringify({ ok: true });
    },
    insertTextInCell: (_s, _pp, _c, ci, cp, o, text) => {
      calls.push(`cell-insert ${ci === 1 ? '' : `c${ci} `}${cp > 0 ? `p${cp} ` : ''}${o} ${text}`);
      if (ci !== 1) return JSON.stringify({ ok: true });
      cell[0] = cell[0].slice(0, o) + text + cell[0].slice(o);
      return JSON.stringify({ ok: true });
    },
    getFieldList: () => JSON.stringify(fields),
    getSectionCount: () => 1,
    getParagraphCount: () => body.length,
    splitParagraph: (_s, p, o) => {
      calls.push(`split ${p} ${o}`);
      body.splice(p + 1, 0, body[p].slice(o));
      body[p] = body[p].slice(0, o);
      return JSON.stringify({ ok: true });
    },
    insertText: (_s, p, o, text) => {
      calls.push(`insert ${p} ${text}`);
      body[p] = body[p].slice(0, o) + text + body[p].slice(o);
      return JSON.stringify({ ok: true });
    },
    applyCharFormat: (_s, p, a, b, json) => { calls.push(`char ${p} ${a}-${b} ${json}`); return JSON.stringify({ ok: true }); },
    applyCharFormatInCell: (_s, _pp, _c, ci, _cp, a, b, json) => { calls.push(`cell-char ${ci} ${a}-${b} ${json}`); return JSON.stringify({ ok: true }); },
    applyParaFormat: (_s, p, json) => { calls.push(`para ${p} ${json}`); return JSON.stringify({ ok: true }); },
    applyParaFormatInCell: (_s, _pp, _c, ci, _cp, json) => { calls.push(`cell-para ${ci} ${json}`); return JSON.stringify({ ok: true }); },
    findOrCreateFontId: (name) => (name === '없는글꼴' ? -1 : 7),
    createTable: (_s, p, _o, rows, cols) => {
      calls.push(`table ${p} ${rows}x${cols}`);
      return JSON.stringify({ ok: true, paraIdx: p, controlIdx: 0 });
    },
    deleteParagraph: (_s, p) => { calls.push(`delete-para ${p}`); body.splice(p, 1); return JSON.stringify({ ok: true }); },
    deleteText: (_s, p, o, n) => { calls.push(`delete-text ${p} ${o} ${n}`); body[p] = body[p].slice(0, o) + body[p].slice(o + n); return JSON.stringify({ ok: true }); },
    // 구역 첫 문단은 구역 설정 컨트롤을 품는다.
    getControlTextPositions: (_s, p) => JSON.stringify(p === 0 ? [0, 0] : []),
    ensureDefaultBullet: () => 3,
    ensureDefaultNumbering: () => 4,
    getStyleList: () => JSON.stringify([{ id: 0, name: '바탕글', englishName: 'Normal' }, { id: 2, name: '개요 1', englishName: 'Outline 1' }]),
    applyStyle: (_s, p, id) => { calls.push(`style ${p} ${id}`); return JSON.stringify({ ok: true }); },
    // 표: 2행 2열, 칸 1 = (0,1) 이 「사 업 …」 칸
    getTableDimensions: () => JSON.stringify({ rowCount: 2, colCount: 2, cellCount: 4 }),
    getCellInfo: (_s, _pp, _c, ci) => JSON.stringify({ row: Math.floor(ci / 2), col: ci % 2, rowSpan: 1, colSpan: 1 }),
    insertTableRow: (_s, _pp, _c, r, below) => { calls.push(`row+ ${r} ${below}`); return JSON.stringify({ ok: true }); },
    insertTableColumn: (_s, _pp, _c, c, right) => { calls.push(`col+ ${c} ${right}`); return JSON.stringify({ ok: true }); },
    deleteTableRow: (_s, _pp, _c, r) => { calls.push(`row- ${r}`); return JSON.stringify({ ok: true }); },
    deleteTableColumn: (_s, _pp, _c, c) => { calls.push(`col- ${c}`); return JSON.stringify({ ok: true }); },
    mergeTableCells: (_s, _pp, _c, r1, c1, r2, c2) => { calls.push(`merge ${r1},${c1}-${r2},${c2}`); return JSON.stringify({ ok: true }); },
    splitParagraphInCell: (_s, _pp, _c, ci, cp, o) => { calls.push(`cell-split ${ci} ${cp} ${o}`); return JSON.stringify({ ok: true }); },
    createHeaderFooter: (_s, h) => { calls.push(`hf-create ${h}`); return JSON.stringify({ ok: true }); },
    getHeaderFooter: (_s, h) => JSON.stringify(h ? { ok: true, exists: true, text: '옛 머리말' } : { ok: true, exists: false }),
    insertTextInHeaderFooter: (_s, h, _a, _p, _o, text) => { calls.push(`hf-insert ${h} ${text}`); return JSON.stringify({ ok: true }); },
    deleteTextInHeaderFooter: (_s, h, _a, _p, o, n) => { calls.push(`hf-delete ${h} ${o} ${n}`); return JSON.stringify({ ok: true }); },
    insertFootnote: (_s, p, o) => { calls.push(`fn ${p} ${o}`); return JSON.stringify({ ok: true, paraIdx: p, controlIdx: 5 }); },
    insertTextInFootnote: (_s, p, c, _fp, _o, text) => { calls.push(`fn-text ${p} ${c} ${text}`); return JSON.stringify({ ok: true }); },
    setFieldValue: (id, value) => {
      calls.push(`field ${id} ${value}`);
      fields.find((f) => f.fieldId === id)!.value = value;
      return JSON.stringify({ ok: true });
    },
  };
  let transactions = 0;
  let blankAnswer = true;
  const host: WriteHost = {
    read: (fn) => fn(doc),
    transaction: (_label, fn) => { transactions += 1; return fn({ doc: () => doc }); },
    createBlankDocument: async () => {
      calls.push('blank');
      if (blankAnswer) body.splice(0, body.length, '');
      return blankAnswer;
    },
  };
  return {
    host, body, cell, fields, calls,
    transactions: () => transactions,
    cancelBlank: () => { blankAnswer = false; },
  };
}

const call = (name: string, input: unknown) => ({ id: 't', name, input });
const refused = (pattern: RegExp) => (error: unknown) => error instanceof WriteRefused && pattern.test(error.message);

test('쓰기 도구 목록과 이름 판정', () => {
  assert.deepEqual(WRITE_TOOL_DEFS.map((t) => t.name), [
    'replace_text', 'fill_field', 'insert_paragraphs', 'format_text', 'format_paragraph', 'insert_table',
    'delete_paragraph', 'set_list', 'apply_style', 'edit_table', 'set_cell_text', 'set_header_footer', 'insert_footnote',
    'create_document',
  ]);
  assert.ok(isWriteTool('replace_text') && isWriteTool('fill_field') && !isWriteTool('read_page_text'));
});

test('제안은 문서를 바꾸지 않고, 적용하면 트랜잭션 한 번으로 본문을 바꾼다', () => {
  const d = fakeDoc();
  const proposal = prepareWrite(d.host, call('replace_text', { find: '홈페이지 구축', replace: '누리집 구축' }));
  assert.deepEqual({ ...proposal, apply: undefined }, {
    tool: 'replace_text', location: { section: 0, paragraph: 0, inTable: false },
    prefix: '사업명은 ', removed: '홈페이지 구축', inserted: '누리집 구축', suffix: '이다', apply: undefined,
  });
  assert.equal(d.calls.length, 0, '제안 단계에서는 쓰지 않는다');
  proposal.apply();
  assert.equal(d.body[0], '사업명은 누리집 구축이다');
  assert.equal(d.transactions(), 1);
});

test('표 칸은 칸 전용 삭제·삽입으로 바꾼다(바깥 문단을 건드리지 않는다)', () => {
  const d = fakeDoc();
  const proposal = prepareWrite(d.host, call('replace_text', { find: '수 행 ', replace: '' }));
  assert.equal(proposal.location.inTable, true);
  proposal.apply();
  assert.equal(d.cell[0], '사 업 계 획 서');
  assert.deepEqual(d.calls, ['cell-delete 4 4'], '빈 replace 는 삭제만');
  assert.equal(d.body[0], '사업명은 홈페이지 구축이다');
});

test('찾을 글이 없거나 여러 곳이면 거절하고, occurrence 로 고를 수 있다', () => {
  const d = fakeDoc();
  assert.throws(() => prepareWrite(d.host, call('replace_text', { find: '없는 글', replace: 'x' })), refused(/not found/));
  assert.throws(() => prepareWrite(d.host, call('replace_text', { find: '업', replace: 'x' })), refused(/occurs 3 times/));
  const second = prepareWrite(d.host, call('replace_text', { find: '업', replace: '엽', occurrence: 3 }));
  assert.equal(second.location.inTable, true);
  assert.throws(() => prepareWrite(d.host, call('replace_text', { find: '업', replace: 'x', occurrence: 4 })), refused(/between 1 and 3/));
});

test('줄바꿈·제어 문자와 같은 글로 바꾸기는 거절한다', () => {
  const d = fakeDoc();
  assert.throws(() => prepareWrite(d.host, call('replace_text', { find: '사업명', replace: '사업\n명' })), refused(/line breaks/));
  assert.throws(() => prepareWrite(d.host, call('replace_text', { find: '사업명', replace: '사업명' })), refused(/same/));
  assert.throws(() => prepareWrite(d.host, call('replace_text', { find: '', replace: 'x' })), refused(/Invalid input/));
});

test('제안 뒤 문서가 바뀌었으면 적용하지 않는다', () => {
  const d = fakeDoc();
  const proposal = prepareWrite(d.host, call('replace_text', { find: '과업기간', replace: '사업기간' }));
  d.body[1] = '앞에 글 추가 과업기간 X월 20일';
  assert.throws(() => proposal.apply(), refused(/changed after this was proposed/));
  assert.equal(d.body[1], '앞에 글 추가 과업기간 X월 20일');
});

test('누름틀은 fieldId 로 지목하고, 같은 이름이 여럿이면 occurrence 가 필요하다', () => {
  const d = fakeDoc();
  const proposal = prepareWrite(d.host, call('fill_field', { name: '성명', value: '홍길동' }));
  assert.deepEqual([proposal.location.field, proposal.removed, proposal.inserted], ['성명', '', '홍길동']);
  proposal.apply();
  assert.deepEqual(d.calls, ['field 11 홍길동']);

  assert.throws(() => prepareWrite(d.host, call('fill_field', { name: '연락처', value: '1' })), refused(/2 fields/));
  const second = prepareWrite(d.host, call('fill_field', { name: '연락처', value: '010-1', occurrence: 2 }));
  assert.equal(second.location.field, '연락처 (2)');
  second.apply();
  assert.equal(d.fields[2].value, '010-1');
  assert.throws(() => prepareWrite(d.host, call('fill_field', { name: '없음', value: 'x' })), refused(/No field/));
});

test('문단 추가: 끝·앞·지정 문단 뒤, 트랜잭션 한 번', () => {
  const d = fakeDoc();
  prepareWrite(d.host, call('insert_paragraphs', { position: 'end', paragraphs: ['끝 하나', '', '끝 둘'] })).apply();
  assert.deepEqual(d.body, ['사업명은 홈페이지 구축이다', '과업기간 X월 20일', '끝 하나', '', '끝 둘']);
  prepareWrite(d.host, call('insert_paragraphs', { position: 'start', paragraphs: ['맨 앞'] })).apply();
  assert.equal(d.body[0], '맨 앞');
  assert.equal(d.body[1], '사업명은 홈페이지 구축이다', '나눈 뒤 원래 첫 문단은 그대로 뒤에 남는다');
  const after = prepareWrite(d.host, call('insert_paragraphs', { position: 'after', after: '홈페이지', paragraphs: ['끼운 문단'] }));
  assert.deepEqual([after.location.position, after.location.paragraph, after.inserted], ['after', 1, '끼운 문단']);
  after.apply();
  assert.deepEqual(d.body.slice(0, 4), ['맨 앞', '사업명은 홈페이지 구축이다', '끼운 문단', '과업기간 X월 20일']);
  assert.equal(d.transactions(), 3);
});

test('빈 새 문서에 끝으로 쓰면 첫 빈 문단부터 채운다(앞에 빈 줄을 남기지 않는다)', async () => {
  const d = fakeDoc();
  const create = prepareWrite(d.host, call('create_document', {}));
  assert.equal(d.calls.length, 0, '제안만으로는 문서를 바꾸지 않는다');
  await create.apply();
  assert.deepEqual(d.body, ['']);
  prepareWrite(d.host, call('insert_paragraphs', { position: 'end', paragraphs: ['제목', '본문'] })).apply();
  assert.deepEqual(d.body, ['제목', '본문']);
});

test('새 문서 만들기를 저장 확인에서 취소하면 거절로 알린다', async () => {
  const d = fakeDoc();
  d.cancelBlank();
  await assert.rejects(Promise.resolve(prepareWrite(d.host, call('create_document', {})).apply()), refused(/cancelled/));
  assert.equal(d.body[0], '사업명은 홈페이지 구축이다');
});

test('문단 추가 입력 검증: 줄바꿈·after 짝·개수', () => {
  const d = fakeDoc();
  assert.throws(() => prepareWrite(d.host, call('insert_paragraphs', { position: 'end', paragraphs: ['한\n둘'] })), refused(/one line/));
  assert.throws(() => prepareWrite(d.host, call('insert_paragraphs', { position: 'after', paragraphs: ['x'] })), refused(/needs the after/));
  assert.throws(() => prepareWrite(d.host, call('insert_paragraphs', { position: 'end', after: '사업', paragraphs: ['x'] })), refused(/only used/));
  assert.throws(() => prepareWrite(d.host, call('insert_paragraphs', { position: 'middle', paragraphs: ['x'] })), refused(/Invalid input/));
  assert.throws(() => prepareWrite(d.host, call('insert_paragraphs', { position: 'end', paragraphs: [] })), refused(/Invalid input/));
});

test('글자 모양: 찾은 범위만, studio 단위(크기 HWPUNIT·배경은 shadeColor·글꼴은 id)로 보낸다', () => {
  const d = fakeDoc();
  const proposal = prepareWrite(d.host, call('format_text', {
    find: '홈페이지', bold: true, fontSize: 16.5, textColor: '#C00000', highlightColor: '#FFFF00', fontName: '맑은 고딕',
  }));
  assert.deepEqual(proposal.format, { bold: true, fontSize: 16.5, textColor: '#C00000', highlightColor: '#FFFF00', fontName: '맑은 고딕' });
  assert.equal(proposal.inserted, '홈페이지');
  proposal.apply();
  assert.deepEqual(d.calls, ['char 0 5-9 {"bold":true,"textColor":"#C00000","fontSize":1650,"shadeColor":"#FFFF00","fontId":7}']);
  prepareWrite(d.host, call('format_text', { find: '계 획', italic: true })).apply();
  assert.equal(d.calls.at(-1), 'cell-char 1 8-11 {"italic":true}', '표 칸은 칸 전용 경로');
});

test('글자 모양 입력 검증: 속성 없음·색 형식·없는 글꼴', () => {
  const d = fakeDoc();
  assert.throws(() => prepareWrite(d.host, call('format_text', { find: '홈페이지' })), refused(/at least one/));
  assert.throws(() => prepareWrite(d.host, call('format_text', { find: '홈페이지', textColor: 'red0000' })), refused(/#RRGGBB/));
  assert.throws(() => prepareWrite(d.host, call('format_text', { find: '홈페이지', fontSize: 0 })), refused(/Invalid input/));
  const missing = prepareWrite(d.host, call('format_text', { find: '홈페이지', fontName: '없는글꼴' }));
  assert.throws(() => missing.apply(), refused(/could not be used/));
});

test('문단 모양: 정렬과 줄 간격(Percent)을 찾은 글의 문단에', () => {
  const d = fakeDoc();
  const proposal = prepareWrite(d.host, call('format_paragraph', { find: '과업기간', alignment: 'center', lineSpacing: 200 }));
  assert.deepEqual([proposal.location.paragraph, proposal.inserted], [1, '과업기간 X월 20일']);
  proposal.apply();
  assert.deepEqual(d.calls, ['para 1 {"alignment":"center","lineSpacing":200,"lineSpacingType":"Percent"}']);
  assert.throws(() => prepareWrite(d.host, call('format_paragraph', { find: '과업기간' })), refused(/alignment or lineSpacing/));
});

test('표 만들기: 문서 끝 빈 문단 자리에 만들고 칸을 행 우선으로 채운다', async () => {
  const d = fakeDoc();
  await prepareWrite(d.host, call('create_document', {})).apply();
  const proposal = prepareWrite(d.host, call('insert_table', { position: 'end', columns: 2, cells: ['항목', '금액', '인건비', ''] }));
  assert.deepEqual(proposal.format, { rows: 2, columns: 2 });
  assert.equal(proposal.inserted, '항목 | 금액\n인건비 | ');
  proposal.apply();
  assert.deepEqual(d.calls.slice(1), ['table 0 2x2', 'cell-insert c0 0 항목', 'cell-insert 0 금액', 'cell-insert c2 0 인건비'],
    '빈 칸은 쓰지 않는다');
});

test('표 만들기: 글이 있는 문서 끝이면 새 문단을 만들어 그 자리에', () => {
  const d = fakeDoc();
  prepareWrite(d.host, call('insert_table', { position: 'end', columns: 1, cells: ['하나'] })).apply();
  assert.deepEqual(d.calls.slice(0, 2), ['split 1 11', 'table 2 1x1']);
});

test('표 만들기 입력 검증: 칸 수·줄바꿈·표 칸 기준', () => {
  const d = fakeDoc();
  assert.throws(() => prepareWrite(d.host, call('insert_table', { position: 'end', columns: 2, cells: ['a', 'b', 'c'] })), refused(/multiple of columns/));
  assert.throws(() => prepareWrite(d.host, call('insert_table', { position: 'end', columns: 1, cells: ['a\nb'] })), refused(/line breaks/));
  assert.throws(() => prepareWrite(d.host, call('insert_table', { position: 'after', after: '수 행', columns: 1, cells: ['a'] })), refused(/inside a table cell/));
});

test('문단 지우기: 본문 문단만, 구역의 마지막 한 문단은 남긴다', () => {
  const d = fakeDoc();
  const proposal = prepareWrite(d.host, call('delete_paragraph', { find: '과업기간' }));
  assert.equal(proposal.removed, '과업기간 X월 20일');
  proposal.apply();
  assert.deepEqual(d.calls, ['delete-para 1']);
  assert.throws(() => prepareWrite(d.host, call('delete_paragraph', { find: '홈페이지' })), refused(/only this paragraph/));
});

test('개체(구역 설정·머리말·표)를 품은 문단은 통째로 지우지 않고 글만 지운다', () => {
  const d = fakeDoc();
  const proposal = prepareWrite(d.host, call('delete_paragraph', { find: '홈페이지' }));
  assert.deepEqual(proposal.format, { keepsObjects: true });
  assert.match(proposal.note ?? '', /only its text was deleted/);
  proposal.apply();
  assert.deepEqual(d.calls, ['delete-text 0 0 14']);
  assert.deepEqual(d.body, ['', '과업기간 X월 20일'], '빈 문단은 남는다');
  assert.throws(() => prepareWrite(d.host, call('delete_paragraph', { find: '수 행' })), refused(/table cell/));
});

test('글머리표·번호·해제는 studio 명령과 같은 속성으로', () => {
  const d = fakeDoc();
  prepareWrite(d.host, call('set_list', { find: '홈페이지', list: 'bullet' })).apply();
  prepareWrite(d.host, call('set_list', { find: '과업기간', list: 'number' })).apply();
  prepareWrite(d.host, call('set_list', { find: '수 행', list: 'none' })).apply();
  assert.deepEqual(d.calls, [
    'para 0 {"headType":"Bullet","numberingId":3,"paraLevel":0}',
    'para 1 {"headType":"Number","numberingId":4,"paraLevel":0}',
    'cell-para 1 {"headType":"None","numberingId":0}',
  ]);
});

test('스타일은 이름(또는 영문 이름)으로 고르고, 없으면 목록을 알려 준다', () => {
  const d = fakeDoc();
  prepareWrite(d.host, call('apply_style', { find: '홈페이지', style: 'outline 1' })).apply();
  assert.deepEqual(d.calls, ['style 0 2']);
  assert.throws(() => prepareWrite(d.host, call('apply_style', { find: '홈페이지', style: '제목' })), refused(/Available: 바탕글, 개요 1/));
});

test('표 구조: 기준 칸의 행·열로 추가·삭제·병합', () => {
  const d = fakeDoc();
  const act = (action: string, extra: Record<string, unknown> = {}) =>
    prepareWrite(d.host, call('edit_table', { find: '수 행', action, ...extra })).apply();
  // 기준 칸은 칸 1 = (0행, 1열)
  act('insert_row_above'); act('insert_row_below'); act('insert_column_left'); act('insert_column_right');
  act('delete_row'); act('delete_column'); act('merge_down');
  assert.deepEqual(d.calls, ['row+ 0 false', 'row+ 0 true', 'col+ 1 false', 'col+ 1 true', 'row- 0', 'col- 1', 'merge 0,1-1,1']);
  assert.throws(() => act('merge_right'), refused(/Only 1 cells to the right/));
  assert.throws(() => act('insert_row_above', { span: 3 }), refused(/only used with merge/));
  assert.throws(() => prepareWrite(d.host, call('edit_table', { find: '홈페이지', action: 'delete_row' })), refused(/inside a table cell/));
});

test('칸 글: 행·열(1부터)로 빈 칸도 채우고, 원래 글은 지운 뒤 넣는다', () => {
  const d = fakeDoc();
  const empty = prepareWrite(d.host, call('set_cell_text', { find: '수 행', row: 2, column: 1, text: '합계' }));
  assert.deepEqual([empty.removed, empty.inserted, empty.format], ['', '합계', { row: 2, column: 1 }]);
  empty.apply();
  assert.deepEqual(d.calls, ['cell-insert c2 0 합계']);
  assert.throws(() => prepareWrite(d.host, call('set_cell_text', { find: '수 행', row: 3, column: 1, text: 'x' })), refused(/2 rows and 2 columns/));
});

test('머리말은 있으면 첫 줄을 바꾸고, 꼬리말은 없으면 만든다', () => {
  const d = fakeDoc();
  const header = prepareWrite(d.host, call('set_header_footer', { kind: 'header', text: '새 머리말' }));
  assert.deepEqual([header.removed, header.inserted], ['옛 머리말', '새 머리말']);
  header.apply();
  prepareWrite(d.host, call('set_header_footer', { kind: 'footer', text: '- 1 -' })).apply();
  assert.deepEqual(d.calls, ['hf-delete true 0 5', 'hf-insert true 새 머리말', 'hf-create false', 'hf-insert false - 1 -']);
});

test('각주는 찾은 글 바로 뒤에 표시를 넣고 각주 글을 쓴다', () => {
  const d = fakeDoc();
  prepareWrite(d.host, call('insert_footnote', { find: '홈페이지', text: '2026년 기준' })).apply();
  assert.deepEqual(d.calls, ['fn 0 9', 'fn-text 0 5 2026년 기준']);
  assert.throws(() => prepareWrite(d.host, call('insert_footnote', { find: '수 행', text: 'x' })), refused(/table cell/));
});

test('표 칸 안 문단 뒤에 칸 문단을 추가한다', () => {
  const d = fakeDoc();
  const proposal = prepareWrite(d.host, call('insert_paragraphs', { position: 'after', after: '수 행', paragraphs: ['칸 둘째 줄'] }));
  assert.equal(proposal.location.inTable, true);
  proposal.apply();
  assert.deepEqual(d.calls, ['cell-split 1 0 13', 'cell-insert p1 0 칸 둘째 줄'], '칸 문단 0 끝에서 나누고 새 칸 문단 1 에 쓴다');
});
