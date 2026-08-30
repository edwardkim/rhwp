// 셀 줄나눔 계측기(내용 키 짝짓기)의 계약.
//
// 계측기가 틀리면 회귀가 통과하거나 개선이 회귀로 보인다. 특히
// (1) 중첩 표의 ls 줄이 바깥 셀로 새면 저장 줄 수가 부풀고,
// (2) 못 짝지은 셀을 조용히 버리면 "안 재서 통과"가 생기고,
// (3) 렌더 0줄 셀을 판정하면 빈 셀이 불일치로 잡힌다.
// (4) 저장 줄수 0 을 불일치로 세면 일치율이 73%로 깎이고 (#6363),
// (5) 쪽 나눔 조각을 통짜 저장 셀과 비교하면 "더 적게"가 부풀고.
import assert from 'node:assert/strict';
import test from 'node:test';

import {
  agreementPercent,
  compareAgreement,
  renderedCells,
  storedCells,
  tallyDocument,
  textKey,
} from '../cell-lineseg-agreement.mjs';

function totals() {
  return {
    documents: 0,
    unpairedStored: 0,
    unpairedRendered: 0,
    emptyCells: 0,
    noStoredRecord: 0,
    cells: 0,
    agree: 0,
    disagree: 0,
    renderedMore: 0,
    renderedFewer: 0,
  };
}

const cell = (row, col, text, lines, header) => {
  const out = { row, col, text, lines };
  if (header !== undefined) out.header = header;
  return out;
};

test('텍스트 키는 공백과 줄 구분 기호를 걷고 앞 12자만 쓴다', () => {
  assert.equal(textKey('성 명|  '), '성명');
  assert.equal(textKey('가나다라마바사아자차카타파하'), '가나다라마바사아자차카타');
});

test('dump 에서 셀·행·열·텍스트·저장 줄 수를 뽑는다', () => {
  const dump = [
    '  [0] 표: 1행×1열, 셀=1, padding=(0,0,0,0), cs=0',
    '  [0]   셀[0] r=0,c=0 rs=1,cs=1 h=100 w=200 pad=(0,0,0,0) valign=Top aim=false hdr=false bf=1 paras=1 text="가나"',
    '  [0]     p[0] ps_id=1 ctrls=0 text_len=2 ls[0] ts=0 vpos=0 lh=100 ls=0 cs=0 sw=200',
  ].join('\n');
  assert.deepEqual(storedCells(dump), [cell(0, 0, '가나', 1, false)]);
});

test('중첩 표의 ls 는 안쪽 셀에 붙고 바깥 셀로 새지 않는다', () => {
  const dump = [
    '  [0] 표: 1행×1열, 셀=1, padding=(0,0,0,0), cs=0',
    '  [0]   셀[0] r=0,c=0 rs=1,cs=1 h=1 w=1 pad=(0,0,0,0) valign=Top aim=false hdr=false bf=1 paras=2 text="밖"',
    '  [0]     p[0] ps_id=1 ctrls=0 text_len=1 ls[0] ts=0 vpos=0 lh=1 ls=0 cs=0 sw=1',
    '  [0]     p[1] 내부표: 1행×1열, 셀=1, cs=0, pad=(0,0,0,0)',
    '  [0]       셀[0] r=0,c=0 rs=1,cs=1 h=1 w=1 pad=(0,0,0,0) valign=Top aim=false hdr=false bf=1 paras=1 text="안"',
    '  [0]         p[0] ps_id=1 ctrls=0 text_len=1 ls[0] ts=0 vpos=0 lh=1 ls=0 cs=0 sw=1',
    '  [0]     p[2] ps_id=1 ctrls=0 text_len=1 ls[0] ts=0 vpos=9 lh=1 ls=0 cs=0 sw=1',
  ].join('\n');
  const cells = storedCells(dump);
  assert.deepEqual(cells, [cell(0, 0, '밖', 2, false), cell(0, 0, '안', 1, false)]);
});

test('render tree 에서 Cell 별 TextLine 수와 텍스트를 뽑는다', () => {
  const tree = {
    type: 'Page',
    children: [
      {
        type: 'Cell',
        row: 2,
        col: 3,
        children: [
          { type: 'TextLine', children: [{ type: 'TextRun', text: '가' }] },
          { type: 'TextLine', children: [{ type: 'TextRun', text: '나' }] },
        ],
      },
    ],
  };
  assert.deepEqual(renderedCells(tree), [cell(2, 3, '가|나', 2)]);
});

test('같은 내용 키끼리 짝지어 일치/불일치를 센다', () => {
  const t = totals();
  tallyDocument(
    [cell(0, 0, '가', 2), cell(0, 1, '나', 1)],
    [cell(0, 0, '가', 2), cell(0, 1, '나', 3)],
    t,
  );
  assert.equal(t.cells, 2);
  assert.equal(t.agree, 1);
  assert.equal(t.disagree, 1);
  assert.equal(t.renderedMore, 1);
  assert.equal(t.unpairedStored + t.unpairedRendered, 0);
});

test('개수가 달라도 문서를 버리지 않는다 — 남는 셀만 unpaired 로 센다', () => {
  const t = totals();
  tallyDocument(
    [cell(0, 0, '가', 1)],
    [cell(0, 0, '가', 1), cell(5, 5, '유령', 4)],
    t,
  );
  assert.equal(t.cells, 1);
  assert.equal(t.agree, 1);
  assert.equal(t.unpairedRendered, 1);
});

test('렌더 0줄 셀은 판정하지 않되 짝은 소비한다', () => {
  const t = totals();
  tallyDocument(
    [cell(0, 0, '가', 1), cell(0, 0, '가', 1)],
    [cell(0, 0, '가', 0), cell(0, 0, '가', 1)],
    t,
  );
  assert.equal(t.cells, 1);
  assert.equal(t.agree, 1);
  assert.equal(t.unpairedStored + t.unpairedRendered, 0);
});

test('같은 키가 여럿이면 순서대로 맞춘다', () => {
  const t = totals();
  tallyDocument(
    [cell(0, 0, '가', 1), cell(0, 0, '가', 2)],
    [cell(0, 0, '가', 1), cell(0, 0, '가', 2)],
    t,
  );
  assert.equal(t.agree, 2);
});

test('개행 든 셀 텍스트를 닫는 따옴표까지 재조립해 등록한다', () => {
  // dump 는 셀 텍스트의 개행을 물리 줄바꿈으로 찍는다 — 재조립하지 않으면 그 셀이
  // 통째로 빠지고 ls 가 직전 셀에 가산된다(k-water '운영중\n(사업대상)' 열).
  const dump = [
    '  [0]   셀[0] r=0,c=0 rs=1,cs=1 h=1 w=100 pad=(0,0,0,0) valign=Center aim=false hdr=false bf=1 paras=1 text="운영중',
    '(사업대상)"',
    '  [0]     p[0] ps_id=1 ctrls=0 text_len=10 ls[0] ts=0 vpos=0 lh=1000 ls=600 cs=0 sw=90',
    '  [0]   셀[1] r=0,c=1 rs=1,cs=1 h=1 w=100 pad=(0,0,0,0) valign=Center aim=false hdr=false bf=1 paras=1 text="이웃"',
    '  [0]     p[0] ps_id=1 ctrls=0 text_len=2 ls[0] ts=0 vpos=0 lh=1000 ls=600 cs=0 sw=90',
  ].join('\n');
  const cells = storedCells(dump);
  assert.equal(cells.length, 2);
  assert.equal(cells[0].lines, 1);
  assert.equal(cells[1].lines, 1);
  assert.ok(cells[0].text.includes('운영중'));
});

test('내용 키가 빈 셀은 짝짓기에서 빼고 emptyCells 로 센다 — 식별력이 없다', () => {
  // 빈 키는 (행, 열)만으로 수백 셀이 겹쳐 무관한 셀끼리 붙는다(편람: 저장 3줄 빈 셀이
  // 다른 쪽 1줄 빈 셀과 붙어 거짓 "더 적게"). 짝짓기 불가는 못 짝지음과 다른 사실이다.
  const t = totals();
  tallyDocument(
    [cell(0, 0, '||', 3), cell(0, 0, '가', 1)],
    [cell(0, 0, '|', 1), cell(0, 0, '가', 1)],
    t,
  );
  assert.equal(t.emptyCells, 2);
  assert.equal(t.cells, 1);
  assert.equal(t.agree, 1);
  assert.equal(t.disagree, 0);
  assert.equal(t.unpairedStored + t.unpairedRendered, 0);
});

test('일치율이 내려가면 회귀다', () => {
  const now = { ...totals(), cells: 100, agree: 90, unpairedStored: 0, unpairedRendered: 0 };
  const was = { ...totals(), cells: 100, agree: 95, unpairedStored: 0, unpairedRendered: 0 };
  const { regressions } = compareAgreement(now, was);
  assert.equal(regressions.length, 1);
  assert.equal(regressions[0].what, '일치율');
});

test('못 짝지은 셀이 늘면 회귀다', () => {
  const now = { ...totals(), cells: 100, agree: 100, unpairedStored: 3, unpairedRendered: 0 };
  const was = { ...totals(), cells: 100, agree: 100, unpairedStored: 1, unpairedRendered: 1 };
  const { regressions } = compareAgreement(now, was);
  assert.equal(regressions.length, 1);
  assert.equal(regressions[0].what, '못 짝지은 셀');
});

test('측정 셀이 줄면 일치율이 올라도 회귀다 — 안 재서 통과 금지', () => {
  const now = { ...totals(), cells: 50, agree: 50 };
  const was = { ...totals(), cells: 100, agree: 95 };
  const { regressions } = compareAgreement(now, was);
  assert.ok(regressions.some((r) => r.what === '측정 셀'));
});

test('일치율이 오르면 개선으로 보고한다', () => {
  const now = { ...totals(), cells: 100, agree: 97 };
  const was = { ...totals(), cells: 100, agree: 95 };
  const { improvements, regressions } = compareAgreement(now, was);
  assert.equal(regressions.length, 0);
  assert.equal(improvements.length, 1);
});

test('빈 집계의 일치율은 0 이다', () => {
  assert.equal(agreementPercent(totals()), 0);
});

test('dump 가 hdr=true 를 저장 셀에 붙인다', () => {
  const dump = [
    '  [0] 표: 1행×1열, 셀=1, padding=(0,0,0,0), cs=0',
    '  [0]   셀[0] r=0,c=0 rs=1,cs=1 h=100 w=200 pad=(0,0,0,0) valign=Top aim=false hdr=true bf=1 paras=1 text="제목"',
    '  [0]     p[0] ps_id=1 ctrls=0 text_len=2 ls[0] ts=0 vpos=0 lh=100 ls=0 cs=0 sw=200',
  ].join('\n');
  assert.deepEqual(storedCells(dump), [cell(0, 0, '제목', 1, true)]);
});

test('저장 줄수 0 은 불일치가 아니라 기록 없음이다', () => {
  const t = totals();
  tallyDocument([cell(0, 0, '가', 0)], [cell(0, 0, '가', 2)], t);
  assert.equal(t.noStoredRecord, 1);
  assert.equal(t.cells, 0);
  assert.equal(t.disagree, 0);
  assert.equal(t.renderedMore, 0);
  assert.equal(agreementPercent(t), 0);
});

test('제목 행 반복은 각 조각을 같은 저장 값과 개별 비교한다', () => {
  const t = totals();
  tallyDocument(
    [cell(0, 0, '제목', 2, true)],
    [cell(0, 0, '제목', 2), cell(0, 0, '제목', 2), cell(0, 0, '제목', 2)],
    t,
  );
  assert.equal(t.cells, 3);
  assert.equal(t.agree, 3);
  assert.equal(t.unpairedRendered, 0);
});

test('쪽 나눔 조각은 합산해 통짜 저장 셀과 비교한다', () => {
  const t = totals();
  tallyDocument(
    [cell(0, 0, '긴본문시작부분', 52, false)],
    [
      cell(0, 0, '긴본문시작부분', 15),
      cell(0, 0, '이어지는조각텍스트', 20),
      cell(0, 0, '마지막조각텍스트', 17),
    ],
    t,
  );
  assert.equal(t.cells, 1);
  assert.equal(t.agree, 1);
  assert.equal(t.renderedFewer, 0);
  assert.equal(t.unpairedRendered, 0);
});

test('같은 좌표의 저장 셀이 둘이면 쪽 나눔 합산을 하지 않는다', () => {
  const t = totals();
  tallyDocument(
    [cell(0, 0, '갑', 2, false), cell(0, 0, '을', 3, false)],
    [cell(0, 0, '갑', 2), cell(0, 0, '을', 3)],
    t,
  );
  assert.equal(t.agree, 2);
  assert.equal(t.unpairedRendered, 0);
});

test('기록 없는 셀이 늘면 회귀다', () => {
  const now = { ...totals(), cells: 100, agree: 100, noStoredRecord: 5 };
  const was = { ...totals(), cells: 100, agree: 100, noStoredRecord: 1 };
  const { regressions } = compareAgreement(now, was);
  assert.equal(regressions.length, 1);
  assert.equal(regressions[0].what, '기록 없는 셀');
});
