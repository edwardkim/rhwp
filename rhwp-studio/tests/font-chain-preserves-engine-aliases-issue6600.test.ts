/**
 * [Issue #6600] 캔버스 font 치환이 엔진이 준 체인을 버리지 않는다.
 *
 * 엔진(`renderer::canvas_font_family_chain`)은 설치 face 별칭을 담은 체인을 준다:
 *   `"한양중고딕", "HY중고딕", "HYGothic", "HYGothic-Medium", "HCR Dotum", "함초롬돋움", 'Malgun Gothic', …`
 *
 * 종전 치환은 **첫 이름만 떼어 studio 체인으로 통째 대체**해서 별칭이 사라지고 곧바로
 * `Malgun Gothic` 으로 떨어졌다. Windows DirectWrite 가 `-Medium` 을 스타일 토큰으로 떼어내
 * `HY중고딕`·`HYGothic-Medium` 둘 다 해석하지 못하고 `HYGothic` 만 해석하므로
 * (`src/renderer/mod.rs:1674` 실측), 별칭이 빠지면 `【`·`『` 가 Malgun 의 반각 글리프
 * (0.518em)로 폴백해 1em 상자 왼쪽에 붙는다 — headless Chrome 실측 0.518em → 1.000em.
 */
import test from 'node:test';
import assert from 'node:assert/strict';

import {
  parseCssFontFamilyList,
  formatCssFontFamilyList,
} from '../src/core/font-substitution.ts';

const ENGINE_FAMILY_PART =
  '"한양중고딕", "HY중고딕", "HYGothic", "HYGothic-Medium", "HCR Dotum", "함초롬돋움", '
  + "'Malgun Gothic','맑은 고딕','Apple SD Gothic Neo','Noto Sans KR',sans-serif";

test('엔진 체인의 혼합 인용(큰따옴표·작은따옴표·무인용)을 모두 가른다 (#6600)', () => {
  const names = parseCssFontFamilyList(ENGINE_FAMILY_PART);
  assert.equal(names[0], '한양중고딕');
  for (const expected of ['HY중고딕', 'HYGothic', 'HYGothic-Medium', 'HCR Dotum', '함초롬돋움']) {
    assert.ok(names.includes(expected), `${expected} 가 보존돼야 한다: ${names.join(' / ')}`);
  }
  assert.ok(names.includes('Malgun Gothic'), '작은따옴표 항목도 읽어야 한다');
  assert.equal(names.at(-1), 'sans-serif', '무인용 generic 도 읽어야 한다');
});

test('설치 face 별칭이 generic 폴백보다 앞에 남는다 (#6600)', () => {
  const names = parseCssFontFamilyList(ENGINE_FAMILY_PART);
  const hyGothic = names.indexOf('HYGothic');
  const malgun = names.indexOf('Malgun Gothic');
  assert.ok(hyGothic >= 0, 'HYGothic 이 있어야 한다 — Windows 에서 유일하게 해석되는 이름이다');
  assert.ok(
    hyGothic < malgun,
    `HYGothic(${hyGothic}) 이 Malgun Gothic(${malgun}) 보다 앞이어야 한다 — 뒤면 괄호가 반각으로 떨어진다`,
  );
});

test('가른 뒤 되돌려도 이름 집합과 순서가 보존된다 (#6600)', () => {
  const names = parseCssFontFamilyList(ENGINE_FAMILY_PART);
  const round = parseCssFontFamilyList(formatCssFontFamilyList(names));
  assert.deepEqual(round, names);
});

test('이스케이프와 쉼표 포함 이름을 깨뜨리지 않는다 (#6600)', () => {
  assert.deepEqual(parseCssFontFamilyList('"A, B", C'), ['A, B', 'C']);
  assert.deepEqual(parseCssFontFamilyList(String.raw`"He said \"hi\"", serif`), ['He said "hi"', 'serif']);
  assert.deepEqual(parseCssFontFamilyList(''), []);
});
