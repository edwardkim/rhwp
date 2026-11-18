// 관계형: CSS 후보 순서와 예약 별칭의 투명성을 검사하며 실물 페이지 좌표를 고정하지 않는다.
// 주석 갱신: 2026-11-15 20:29 KST. 직전 변경: af0fe858c69db244fb44064a3646ae2f6b99163b
// (2026-09-11T18:29:40+09:00). #9077은 문서 없이 재현되는 CSS font 선택 계약이다.
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
import { registerLoadedLocalFontFace, clearLoadedLocalFontFaces } from '../src/core/font-loader.ts';

import {
  parseCssFontFamilyList,
  formatCssFontFamilyList,
  substituteCssFontFamily,
  setDocumentFontSubstitutions,
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

test('예약 별칭 뒤의 실제 웹 글꼴을 기준으로 표시 정책을 계산한다 (#9077)', () => {
  for (const family of ['함초롬바탕', '함초롬돋움', 'KoPub바탕체 Medium']) {
    const ordinary = `italic bold 14.667px "${family}", Batang, sans-serif`;
    const expected = substituteCssFontFamily(ordinary);
    for (const aliases of [
      [`__rhwp_ttf__${family}`],
      [`__rhwp_intrinsic__${family}`],
      [`__rhwp_intrinsic__${family}`, `__rhwp_ttf__${family}`],
    ]) {
      const actual = substituteCssFontFamily(
        ordinary.replace('px ', `px ${formatCssFontFamilyList(aliases)}, `),
      );
      assert.ok(actual.startsWith('italic bold 14.667px '));
      const names = parseCssFontFamilyList(actual.split('px ')[1]);
      assert.deepEqual(names.slice(0, aliases.length), aliases);
      assert.deepEqual(names.slice(aliases.length), parseCssFontFamilyList(expected.split('px ')[1]));
      assert.equal(names[aliases.length], family);
    }
  }
});

test('예약 별칭은 문서 대체와 미승인 원본 제외 정책을 바꾸지 않는다 (#9077)', () => {
  setDocumentFontSubstitutions([['없는 문서 글꼴', '함초롬바탕']]);
  try {
    for (const family of ['휴먼명조', '없는 문서 글꼴']) {
      const ordinary = `12px "${family}", "HY신명조", "함초롬바탕", serif`;
      const aliases = [`__rhwp_ttf__${family}`];
      const actual = substituteCssFontFamily(ordinary.replace('px ', `px "${aliases[0]}", `));
      const names = parseCssFontFamilyList(actual.slice(5));
      assert.equal(names[0], aliases[0]);
      assert.deepEqual(names.slice(1), parseCssFontFamilyList(substituteCssFontFamily(ordinary).slice(5)));
      assert.ok(!names.includes(family), '미승인 원본을 다시 넣지 않는다');
    }
  } finally {
    setDocumentFontSubstitutions(undefined);
  }
});

test('실제 이름 없는 예약 별칭과 px 없는 descriptor는 그대로 둔다 (#9077)', () => {
  for (const descriptor of ['12px "__rhwp_ttf__Only"', 'bold 1em "함초롬바탕"', '']) {
    assert.equal(substituteCssFontFamily(descriptor), descriptor);
  }
});

test('로드된 실제 로컬 face도 예약 별칭 뒤에서 시스템 대체보다 먼저 선택한다 (#9077)', () => {
  const previous = Object.getOwnPropertyDescriptor(globalThis, 'document');
  const faces = new Set<FontFace>();
  Object.defineProperty(globalThis, 'document', { configurable: true, value: { fonts: faces } });
  try {
    for (const family of ['궁서', 'Arial', 'Comic Sans MS', 'A, B']) {
      // 설치 글꼴에 의존하지 않는 공급 계약 검사. 실제 paint는 브라우저에서 별도 확인한다.
      registerLoadedLocalFontFace({ family, status: 'loaded' } as FontFace);
      const aliases = [`__rhwp_intrinsic__${family}`, `__rhwp_ttf__${family}`];
      const descriptor = '16px ' + formatCssFontFamilyList([...aliases, family, 'Malgun Gothic', 'serif']);
      const names = parseCssFontFamilyList(substituteCssFontFamily(descriptor).slice(5));
      assert.deepEqual(names.slice(0, 3), [...aliases, family]);
      assert.equal(new Set(names.map(name => name.toLowerCase())).size, names.length);
    }
  } finally {
    clearLoadedLocalFontFaces();
    if (previous) Object.defineProperty(globalThis, 'document', previous);
    else Reflect.deleteProperty(globalThis, 'document');
  }
});
