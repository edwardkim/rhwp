import test from 'node:test';
import assert from 'node:assert/strict';
import { unsupportedHwpJsonControls } from '../src/engine/hwpjson-paste-guard.ts';

const ctrl = (ci: number, co: string) => ({ cc: 11, ci, co });

test('코어가 아는 제어문자만 있으면 빈 배열', () => {
  const model = JSON.stringify({ bo: [{ ch: [ctrl(1_936_024_420, 'a'), ctrl(1_952_607_264, 'b'), { t: '글자' }] }] });
  assert.deepEqual(unsupportedHwpJsonControls(model), []);
});

test('누름틀(%clk)과 필드 끝은 코어가 안다', () => {
  // 한/글 2020 이 누름틀 문서를 복사할 때 싣는 ci=627272811("%clk")·157510763("	clk")
  const model = JSON.stringify({ bo: [{ ch: [ctrl(627_272_811, 'c'), { t: 'TGT_DPT' }, ctrl(157_510_763, 'd')] }] });
  assert.deepEqual(unsupportedHwpJsonControls(model), []);
});

test('모르는 필드(%hlk 하이퍼링크)가 있으면 그 ctrlId 를 돌려준다', () => {
  const hlk = (0x25 << 24) | (0x68 << 16) | (0x6c << 8) | 0x6b;   // "%hlk"
  const model = JSON.stringify({ bo: [{ ch: [ctrl(1_952_607_264, 'b'), { nested: [ctrl(hlk, 'c')] }] }] });
  assert.deepEqual(unsupportedHwpJsonControls(model), ['%hlk']);
});

test('그리기 개체 안의 ci(cc·co 없음)는 제어문자로 보지 않는다', () => {
  const model = JSON.stringify({ cs: { x: { rc: { ci: 1_234 } } } });
  assert.deepEqual(unsupportedHwpJsonControls(model), []);
});

test('JSON 이 아니면 null', () => {
  assert.equal(unsupportedHwpJsonControls('{깨진'), null);
});

test('글자 안 탭(cc=9)은 코어가 알고, 모르는 특수 문자는 cc 번호로 돌려준다', () => {
  const tab = { cc: 9, wi: 1820, le: 0, ty: 1 };
  assert.deepEqual(unsupportedHwpJsonControls(JSON.stringify({ x: { ch: [{ t: '가' }, tab] } })), []);
  const odd = { cc: 24 };
  assert.deepEqual(unsupportedHwpJsonControls(JSON.stringify({ x: { ch: [odd, tab] } })), ['cc:24']);
});
