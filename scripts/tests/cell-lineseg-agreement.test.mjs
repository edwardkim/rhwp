import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import test from 'node:test';

import {
  BASELINE_PATH,
  agreementPercent,
  compareAgreement,
  renderedLineCounts,
  storedLineCounts,
  tallyDocument,
} from '../cell-lineseg-agreement.mjs';

function totals(over = {}) {
  return { documents: 0, skipped: 0, paragraphs: 0, agree: 0, disagree: 0, renderedMore: 0, renderedFewer: 0, ...over };
}

const CELL = (n, ls) =>
  `  [0]   셀[${n}] r=0,c=0 rs=1,cs=1 h=100 w=200 pad=(0,0,0,0) valign=Center aim=true hdr=false bf=3 paras=1 text="x"\n` +
  `  [0]     p[0] ps_id=0 ctrls=0 text_len=5 ${ls.map((_, i) => `ls[${i}] ts=0 vpos=0 lh=1000 ls=600 cs=0 sw=1440`).join(', ')}`;

test('셀 문단의 저장 줄 수를 셀 순서대로 뽑는다', () => {
  const dump = [CELL(0, [1, 2, 3]), CELL(1, [1])].join('\n');
  assert.deepEqual(storedLineCounts(dump), [3, 1]);
});

test('셀 바깥 문단은 세지 않는다', () => {
  // CELL 의 둘째 인자는 줄 목록이다 — 길이가 곧 저장 줄 수.
  const bodyPara = '  [0]     p[0] ps_id=0 ctrls=0 text_len=5 ls[0] ts=0 vpos=0 lh=1000 ls=600 cs=0 sw=1440';
  assert.deepEqual(storedLineCounts([bodyPara, CELL(0, ['a', 'b'])].join('\n')), [2]);
});

test('render tree 에서 Cell 별 TextLine 개수를 뽑는다 — 중첩도 따라간다', () => {
  const tree = {
    type: 'Page',
    children: [
      { type: 'Cell', children: [{ type: 'TextLine' }, { type: 'TextLine' }] },
      { type: 'Table', children: [{ type: 'Cell', children: [{ type: 'TextLine' }] }] },
    ],
  };
  assert.deepEqual(renderedLineCounts(tree), [2, 1]);
});

test('일치하면 agree 로 센다', () => {
  const t = totals();
  tallyDocument([3, 1], [3, 1], t);
  assert.equal(t.paragraphs, 2);
  assert.equal(t.agree, 2);
  assert.equal(agreementPercent(t), 100);
});

test('rhwp 가 줄을 더 많이 만들면 불일치로 잡는다 — 이게 못 잡히면 계측기가 아니다', () => {
  const t = totals();
  tallyDocument([3], [5], t);
  assert.equal(t.disagree, 1);
  assert.equal(t.renderedMore, 1);
  assert.equal(t.renderedFewer, 0);
});

test('줄이 적게 나온 경우도 따로 센다 — 방향이 뒤집히면 원인이 다르다', () => {
  const t = totals();
  tallyDocument([5], [3], t);
  assert.equal(t.renderedFewer, 1);
  assert.equal(t.renderedMore, 0);
});

test('글자가 없는 셀(렌더 0줄)은 모수에서 뺀다', () => {
  const t = totals();
  tallyDocument([1, 1], [0, 1], t);
  assert.equal(t.paragraphs, 1);
});

test('셀 개수가 다르면 비교하지 않고 건너뛴 것으로 센다', () => {
  const t = totals();
  tallyDocument([1, 2, 3], [1, 2], t);
  assert.equal(t.skipped, 1);
  assert.equal(t.paragraphs, 0, '억지로 짝지으면 거짓 불일치가 나온다');
});

test('일치율이 내려가면 회귀다', () => {
  const { regressions } = compareAgreement(
    totals({ paragraphs: 100, agree: 90 }),
    totals({ paragraphs: 100, agree: 95 }),
  );
  assert.ok(regressions.some((r) => r.what === '일치율'));
});

test('일치율이 올라가면 개선으로 잡고 실패시키지 않는다', () => {
  const { regressions, improvements } = compareAgreement(
    totals({ paragraphs: 100, agree: 95 }),
    totals({ paragraphs: 100, agree: 90 }),
  );
  assert.deepEqual(regressions, []);
  assert.ok(improvements.some((i) => i.what === '일치율'));
});

test('건너뛴 문서가 늘면 회귀다 — 모수가 줄어 일치율이 착시로 오른다', () => {
  const { regressions } = compareAgreement(
    totals({ paragraphs: 10, agree: 10, skipped: 5 }),
    totals({ paragraphs: 100, agree: 90, skipped: 1 }),
  );
  assert.ok(regressions.some((r) => r.what === '짝 못 맞춘 문서'));
});

test('측정 문단이 줄어도 회귀다 — 안 재고 통과하는 길을 막는다', () => {
  const { regressions } = compareAgreement(
    totals({ paragraphs: 1, agree: 1 }),
    totals({ paragraphs: 100, agree: 90 }),
  );
  assert.ok(regressions.some((r) => r.what === '측정 문단'));
});

test('문단이 0 이면 일치율은 0 이다 — NaN 을 만들지 않는다', () => {
  assert.equal(agreementPercent(totals()), 0);
});

test('기준선 파일이 필요한 필드를 담는다', () => {
  const b = JSON.parse(readFileSync(BASELINE_PATH, 'utf8'));
  for (const k of ['documents', 'skipped', 'paragraphs', 'agree', 'disagree', 'renderedMore', 'renderedFewer']) {
    assert.equal(typeof b[k], 'number', `${k} 가 기준선에 없다`);
  }
  assert.ok(b.paragraphs > 0, '문단 0 으로 기록된 기준선은 아무것도 막지 못한다');
});
