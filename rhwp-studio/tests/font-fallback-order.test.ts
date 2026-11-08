// 관계형 회귀: 원래 요청과 별칭을 보존하고 모노크롬 기호 후보를 자동 이모지 후보보다 먼저 소비한다.
// 주석 갱신: 2026-11-06 09:56 KST. 직전 파일 변경: d0c751965a158f767791f7e70cfe4b3ebdc73970 (2026-11-06 09:34:59 +09:00).
// 실제 Studio 측정: 2026-11-06 09:35:39 KST, 각 1쪽, 96 DPI·Print·색상 경계 보정 전 2px raw. 직접 exporter와 실제 Studio 결과는 근거 문서에서 구분한다.
// 측정: 2026-11-06 KST, PR #8616 원본 HWPX 두 문서 각 1쪽 Native/fresh WASM raw 100.00%, 96 DPI·독립 Print.
// 실제 Canvas2D의 재활용 모양 색상은 별도 브라우저·원본 PDF 확대 판독으로 확인하며 이 순서 검사로 대신하지 않는다.
// 근거: mydocs/pr/assets/semanticist21_20261105/canvas-symbol-fallback-analysis.md.
import assert from 'node:assert/strict';
import test from 'node:test';
import * as fonts from '../src/core/font-substitution.ts';

const symbol = 'Noto Sans Symbols2';

test('모노크롬 기호 대체는 자동 이모지 후보보다 앞선다', () => {
  const families = fonts.ordinaryTextFontFamilies('Batang');
  assert.equal(families[0], 'Batang', '원래 요청한 글꼴을 먼저 사용한다');
  assert.ok(families.includes(symbol), '재활용 기호를 제공하는 공개 모노크롬 프로그램이 후보에 있다');
  assert.ok(families.indexOf(symbol) < families.indexOf('함초롬바탕'));
  assert.ok(families.indexOf('함초롬바탕') < families.indexOf('D2Coding'));
  assert.ok(families.indexOf('D2Coding') < families.indexOf('Segoe UI Emoji'));
});

test('표시 후보도 모노크롬 기호를 컬러 이모지보다 먼저 제공한다', () => {
  const families = fonts.fontFamilyCandidatesForDisplay('바탕');
  assert.ok(families.includes(symbol));
  assert.ok(families.indexOf(symbol) < families.indexOf('Noto Color Emoji'));
});

test('누락한 후보는 뒤의 공통 후보 앞에 들어가 generic 뒤로 사라지지 않는다', () => {
  const incoming = ['Batang', 'AppleMyungjo', 'Segoe UI Emoji', 'serif'];
  const fallback = ['AppleMyungjo', symbol, 'Segoe UI Emoji', 'serif'];
  assert.deepEqual(fonts.completeFontFamilyList(incoming, fallback),
    ['Batang', 'AppleMyungjo', symbol, 'Segoe UI Emoji', 'serif']);
  assert.deepEqual(incoming, ['Batang', 'AppleMyungjo', 'Segoe UI Emoji', 'serif'], '호출자의 배열을 수정하지 않는다');
});

test('기존 face·설치 별칭·명시적 순서와 중복 제거 계약을 유지한다', () => {
  const incoming = ['Requested', 'Installed Alias', 'C', 'B', 'serif'];
  assert.deepEqual(fonts.completeFontFamilyList(incoming, ['B', 'C', 'D', 'SERIF']),
    ['Requested', 'Installed Alias', 'C', 'B', 'D', 'serif']);
});

test('컬러 이모지를 명시적으로 요청하면 그 원래 face가 첫 후보다', () => {
  const families = fonts.ordinaryTextFontFamilies('Noto Color Emoji');
  assert.equal(families[0], 'Noto Color Emoji');
  assert.deepEqual(fonts.completeFontFamilyList(['Noto Color Emoji', 'serif'], [symbol, 'serif']),
    ['Noto Color Emoji', symbol, 'serif']);
});
