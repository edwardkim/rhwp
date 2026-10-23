import test from 'node:test';
import assert from 'node:assert/strict';

import {
  MAX_RESULT_CHARS,
  READ_TOOL_DEFS,
  executeReadTool,
  pageTextFromLayout,
  type DocumentReader,
  type ToolHost,
} from '../src/ai/tools/read-tools.ts';

const BOUNDARY = 'abc123';

const body = [['제목\u0002 문단', '둘째 문단', ''], ['다른 구역']];
const cell = ['칸 안의 예산 항목'];

function reader(overrides: Partial<DocumentReader> = {}): DocumentReader {
  return {
    getDocumentInfo: () => JSON.stringify({ version: '5.1', pageCount: 3, sectionCount: 2 }),
    getSectionCount: () => body.length,
    getParagraphCount: (s) => body[s].length,
    getParagraphLength: (s, p) => body[s][p].length,
    getTextRange: (s, p, o, c) => body[s][p].slice(o, o + c),
    getOutlineNavigation: () => JSON.stringify({ outline: [
      { level: 1, number: '1.', title: '개요\r\n', page: 1, section: 0, paragraph: 0 },
    ] }),
    getPageTextLayout: (i) => JSON.stringify({ runs: [
      { text: `쪽${i + 1} 첫 줄`, secIdx: 0, paraIdx: 0 },
      { text: '둘째 줄', secIdx: 0, paraIdx: 1 },
    ] }),
    searchAllText: () => JSON.stringify([
      { sec: 0, para: 1, charOffset: 0, length: 2 },
      { sec: 0, para: 4, charOffset: 2, length: 2, cellContext: { parentPara: 4, ctrlIdx: 0, cellIdx: 1, cellPara: 0 } },
    ]),
    getFieldList: () => JSON.stringify([{ fieldId: 1, fieldType: 'clickhere', name: '성명', guide: '이름', value: '홍길동' }]),
    getTextInCell: (_s, _pp, _c, _ci, _cp, o, n) => cell[0].slice(o, o + n),
    getCellParagraphLength: () => cell[0].length,
    getChartDataByIndex: () => { throw new Error('차트 없음'); },
    ...overrides,
  };
}

function host(doc: DocumentReader = reader(), extra: Partial<ToolHost> = {}) {
  let reads = 0;
  const h: ToolHost & { reads: () => number } = {
    read: (fn) => { reads += 1; return fn(doc); },
    getSelectionText: () => null,
    getCaret: () => ({ sectionIndex: 0, parentParaIndex: 1, cellPath: [], charOffset: 2 }),
    reads: () => reads,
    ...extra,
  };
  return h;
}

/** 경계를 벗기고 JSON 을 읽는다. 경계가 정확히 한 쌍인지도 본다. */
function unwrap(content: string): any {
  const open = `<document_content id="${BOUNDARY}">\n`;
  const close = `\n</document_content id="${BOUNDARY}">`;
  assert.ok(content.startsWith(open) && content.endsWith(close), `경계: ${content.slice(0, 60)}`);
  const inner = content.slice(open.length, -close.length);
  assert.ok(!inner.includes('<') && !inner.includes('>'), '안쪽에는 꺾쇠가 없다');
  return JSON.parse(inner);
}

const call = (name: string, input: unknown = {}) => ({ id: 'c1', name, input });

test('읽기 도구 목록은 이름 순서가 고정이며 모든 schema 가 닫혀 있다', () => {
  assert.deepEqual(READ_TOOL_DEFS.map((t) => t.name), [
    'get_document_info', 'get_outline', 'read_page_text', 'read_paragraphs',
    'search_text', 'get_selection', 'list_fields', 'list_charts',
  ]);
  for (const tool of READ_TOOL_DEFS) {
    assert.equal(tool.inputSchema.additionalProperties, false, tool.name);
  }
});

test('get_document_info 는 쪽·구역·구역별 문단 수를 준다', () => {
  const result = executeReadTool(host(), call('get_document_info'), BOUNDARY);
  assert.equal(result.isError, false);
  assert.deepEqual(unwrap(result.content), { pageCount: 3, sectionCount: 2, paragraphCounts: [3, 1], version: '5.1' });
});

test('read_page_text 는 1부터 센 쪽을 읽는다(렌더 조각의 문단마다 한 줄)', () => {
  const result = unwrap(executeReadTool(host(), call('read_page_text', { page: 2 }), BOUNDARY).content);
  assert.deepEqual(result, { page: 2, pageCount: 3, offset: 0, text: '쪽2 첫 줄\n둘째 줄', truncated: false });
});

test('read_page_text 는 범위 밖 쪽과 0쪽을 거절한다', () => {
  const out = executeReadTool(host(), call('read_page_text', { page: 4 }), BOUNDARY);
  assert.deepEqual([out.isError, out.content], [true, 'page must be between 1 and 3.']);
  assert.equal(executeReadTool(host(), call('read_page_text', { page: 0 }), BOUNDARY).isError, true);
});

test('긴 쪽은 조용히 자르지 않고 nextOffset 으로 이어 읽게 한다', () => {
  const long = 'ㄱ'.repeat(MAX_RESULT_CHARS + 10);
  const h = host(reader({ getPageTextLayout: () => JSON.stringify({ runs: [{ text: long, secIdx: 0, paraIdx: 0 }] }) }));
  const first = unwrap(executeReadTool(h, call('read_page_text', { page: 1 }), BOUNDARY).content);
  assert.equal(first.truncated, true);
  assert.equal(first.text.length, MAX_RESULT_CHARS);
  assert.equal(first.nextOffset, MAX_RESULT_CHARS);
  const second = unwrap(executeReadTool(h, call('read_page_text', { page: 1, offset: first.nextOffset }), BOUNDARY).content);
  assert.deepEqual([second.text.length, second.truncated, second.nextOffset], [10, false, undefined]);
});

test('read_paragraphs 는 0부터 센 문단을 읽고 제어 문자를 지운다', () => {
  const result = unwrap(executeReadTool(host(), call('read_paragraphs', { section: 0, from: 0, count: 2 }), BOUNDARY).content);
  assert.deepEqual(result, {
    section: 0, paragraphCount: 3, truncated: false, nextFrom: 2,
    paragraphs: [{ index: 0, text: '제목 문단' }, { index: 1, text: '둘째 문단' }],
  });
  const last = unwrap(executeReadTool(host(), call('read_paragraphs', { section: 0, from: 2, count: 50 }), BOUNDARY).content);
  assert.equal(last.nextFrom, undefined, '끝까지 읽으면 nextFrom 이 없다');
  assert.equal(executeReadTool(host(), call('read_paragraphs', { section: 2, from: 0, count: 1 }), BOUNDARY).isError, true);
  assert.equal(executeReadTool(host(), call('read_paragraphs', { section: 0, from: 0, count: 51 }), BOUNDARY).isError, true);
});

test('read_paragraphs 는 상한에 걸리면 담은 데까지와 nextFrom 을 준다', () => {
  const big = ['가'.repeat(MAX_RESULT_CHARS - 5), '나'.repeat(10), '다'];
  const h = host(reader({
    getSectionCount: () => 1,
    getParagraphCount: () => big.length,
    getParagraphLength: (_s, p) => big[p].length,
    getTextRange: (_s, p, o, c) => big[p].slice(o, o + c),
  }));
  const result = unwrap(executeReadTool(h, call('read_paragraphs', { section: 0, from: 0, count: 3 }), BOUNDARY).content);
  assert.deepEqual([result.paragraphs.length, result.truncated, result.nextFrom], [1, true, 1]);

  const huge = host(reader({
    getSectionCount: () => 1,
    getParagraphCount: () => 2,
    getParagraphLength: () => MAX_RESULT_CHARS * 2,
    getTextRange: (_s, _p, o, c) => 'x'.repeat(MAX_RESULT_CHARS * 2).slice(o, o + c),
  }));
  const single = unwrap(executeReadTool(huge, call('read_paragraphs', { section: 0, from: 0, count: 2 }), BOUNDARY).content);
  assert.deepEqual([single.paragraphs[0].truncated, single.paragraphs[0].text.length, single.nextFrom],
    [true, MAX_RESULT_CHARS, 1], '한 문단이 혼자 넘으면 그 문단만 잘라 담는다');
});

test('search_text 는 본문과 표 칸의 문맥을 함께 준다', () => {
  const result = unwrap(executeReadTool(host(), call('search_text', { query: '예산' }), BOUNDARY).content);
  assert.equal(result.totalHits, 2);
  assert.deepEqual(result.hits[0], { section: 0, paragraph: 1, charOffset: 0, inTable: false, context: '둘째 문단' });
  assert.deepEqual(result.hits[1], { section: 0, paragraph: 4, charOffset: 2, inTable: true, context: '칸 안의 예산 항목' });
  assert.equal(executeReadTool(host(), call('search_text', { query: '' }), BOUNDARY).isError, true);
});

test('get_selection 은 선택이 없으면 null 과 캐럿을 준다', () => {
  const none = unwrap(executeReadTool(host(), call('get_selection'), BOUNDARY).content);
  assert.equal(none.selection, null);
  assert.equal(none.caret.parentParaIndex, 1);
  const selected = host(reader(), {
    getSelectionText: () => ({
      text: '고른\r\n글', truncated: false,
      start: { sectionIndex: 0, parentParaIndex: 0, cellPath: [], charOffset: 0 },
      end: { sectionIndex: 0, parentParaIndex: 1, cellPath: [], charOffset: 1 },
    }),
  });
  assert.equal(unwrap(executeReadTool(selected, call('get_selection'), BOUNDARY).content).selection.text, '고른\n글');
});

test('list_fields 는 이름·종류·값·안내문을 준다', () => {
  const result = unwrap(executeReadTool(host(), call('list_fields'), BOUNDARY).content);
  assert.deepEqual(result, {
    fields: [{ name: '성명', type: 'clickhere', value: '홍길동', guide: '이름' }], totalFields: 1, truncated: false,
  });
});

test('잘못된 입력·모르는 도구·문서 없음은 오류 결과로 돌려주고 던지지 않는다', () => {
  assert.deepEqual(executeReadTool(host(), call('nope'), BOUNDARY), { id: 'c1', content: 'Unknown tool: nope', isError: true });
  assert.match(executeReadTool(host(), call('get_outline', { extra: 1 }), BOUNDARY).content, /알 수 없는 필드/);
  assert.match(executeReadTool(host(), call('read_page_text', { page: '1' }), BOUNDARY).content, /정수/);
  assert.match(executeReadTool(host(), call('read_page_text', null), BOUNDARY).content, /객체/);
  const released = host(reader(), {
    read: () => { throw Object.assign(new Error('released'), { code: 'DOCUMENT_RELEASED' }); },
  });
  assert.deepEqual(executeReadTool(released, call('get_outline'), BOUNDARY),
    { id: 'c1', content: 'No document is open.', isError: true });
});

test('도구는 read 안에서만 문서를 본다(쓰기 경로 없음)', () => {
  const h = host();
  for (const name of ['get_document_info', 'get_outline', 'list_fields']) executeReadTool(h, call(name), BOUNDARY);
  assert.equal(h.reads(), 3);
  assert.ok(!('transaction' in h));
});

test('쪽 글은 표 칸까지 읽는 순서로 잇고, 칸에 들어설 때만 표·칸 표시를 붙인다', () => {
  const cell = (table: number, index: number, para: number) =>
    ({ secIdx: 0, paraIdx: 3, parentParaIdx: 3, cellPath: [{ controlIndex: table, cellIndex: index, cellParaIndex: para }] });
  const text = pageTextFromLayout(JSON.stringify({ runs: [
    { text: '본문 앞', secIdx: 0, paraIdx: 0 },
    { text: '화면에서 줄', secIdx: 0, paraIdx: 1 },
    { text: '바꿈된 문단', secIdx: 0, paraIdx: 1 },
    { text: '이름', ...cell(0, 0, 0) },
    { text: '홍길', ...cell(0, 1, 0) },
    { text: '동', ...cell(0, 1, 0) },
    { text: '둘째 칸 문단', ...cell(0, 1, 1) },
    { text: '', ...cell(0, 2, 0) },
    { text: '안쪽 표', secIdx: 0, paraIdx: 3, parentParaIdx: 3,
      cellPath: [{ controlIndex: 0, cellIndex: 3, cellParaIndex: 0 }, { controlIndex: 1, cellIndex: 0, cellParaIndex: 0 }] },
    { text: '다른 표', ...cell(2, 0, 0) },
    { text: '\u0002본문 뒤\r', secIdx: 0, paraIdx: 4 },
  ] }));
  assert.equal(text, [
    '본문 앞',
    '화면에서 줄바꿈된 문단',
    '[table 1 / cell 0] 이름',
    '[table 1 / cell 1] 홍길동',
    '  둘째 칸 문단',
    '[table 1 / cell 2]',
    '[table 2 / cell 0] 안쪽 표',
    '[table 3 / cell 0] 다른 표',
    '본문 뒤',
  ].join('\n'));
});
