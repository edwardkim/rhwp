import test from 'node:test';
import assert from 'node:assert/strict';

import { MIN_BODY_MM, pageBodyViolation, pageBodySize } from '../src/core/page-body-limits.ts';
import type { PageDef } from '../src/core/types.ts';

// [#4973] 마주보는 여백의 합이 용지를 넘으면 본문이 소멸하고, 렌더러가 용지 5% 여백으로
// 폴백한다(Rust model/page.rs [Task #1583]) — 화면은 그럴듯한데 저장되는 PageDef 만 쓸 수
// 없는 값이 된다. 눈금자 핀은 드래그를 가둬 막고 있었지만 편집 용지 대화상자는 그대로 받았다.

const HWPUNIT_PER_MM = 7200 / 25.4;
const mm = (v: number) => Math.round(v * HWPUNIT_PER_MM);

/** A4 세로, 여백 20mm */
function a4(over: Partial<PageDef> = {}): PageDef {
  return {
    width: mm(210), height: mm(297),
    marginLeft: mm(20), marginRight: mm(20),
    marginTop: mm(20), marginBottom: mm(20),
    marginHeader: mm(15), marginFooter: mm(15),
    marginGutter: 0, landscape: false, binding: 0,
    ...over,
  } as PageDef;
}

test('정상 여백은 통과한다', () => {
  assert.equal(pageBodyViolation(a4()), null);
});

test('좌우 여백 합이 용지를 넘으면 막는다', () => {
  const v = pageBodyViolation(a4({ marginLeft: mm(150), marginRight: mm(100) }));
  assert.match(String(v), /좌우 여백/);
});

test('제본 여백도 본문을 갉아먹는다 — 좌우만 보면 통과해 버린다', () => {
  const def = a4({ marginLeft: mm(90), marginRight: mm(90), marginGutter: mm(25) });
  assert.ok(210 - 90 - 90 > MIN_BODY_MM, '제본 여백 없이는 통과하는 값이어야 시험이 성립한다');
  assert.match(String(pageBodyViolation(def)), /좌우 여백/);
});

test('위아래는 머리말·꼬리말까지 합쳐서 본다', () => {
  // 297 − 130 − 130 = 37mm 라 위아래 여백만 보면 통과한다. 머리말 15 + 꼬리말 15 를 더해야
  // 본문이 7mm 만 남는 게 드러난다.
  const def = a4({ marginTop: mm(130), marginBottom: mm(130) });
  assert.ok(297 - 130 - 130 > MIN_BODY_MM, '머리말/꼬리말 없이는 통과하는 값이어야 시험이 성립한다');
  assert.match(String(pageBodyViolation(def)), /위아래 여백/);
});

test('가로 방향이면 용지 크기를 뒤바꿔 판정한다', () => {
  // 세로에서는 좌우 250mm 가 넘치지만, 가로(297mm)에서는 남는다.
  const def = a4({ landscape: true, marginLeft: mm(130), marginRight: mm(130) });
  const body = pageBodySize(def);
  assert.ok(body.width > 0, `가로 용지 폭 기준이어야 한다 (본문 ${body.width})`);
  assert.equal(pageBodyViolation(def), null);
});

test('한도는 본문 최소 크기 그대로다 — 딱 맞으면 통과, 1mm 모자라면 막는다', () => {
  const exact = a4({ marginLeft: mm(100), marginRight: mm(210 - 100 - MIN_BODY_MM) });
  assert.equal(pageBodyViolation(exact), null);
  const short = a4({ marginLeft: mm(100), marginRight: mm(210 - 100 - MIN_BODY_MM + 1) });
  assert.match(String(pageBodyViolation(short)), /좌우 여백/);
});
