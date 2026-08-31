// [#6263] legacy 한양 face 를 설치 face 이름으로 해석하는 경로가 **선언 altType 에
// 좌우되면 안 된다**.
//
// 치환표 항목에는 altType 조건이 붙어 있다 — `한양중고딕` 은 `source:2->target:1` 하나뿐이다.
// 신고된 세 문서는 전부 `altType=2` 라 `ce1f87f88` 로 닫혔지만, 같은 코퍼스에는 같은 계열
// 이름을 `altType=1` 로 선언한 face 도 있다(`HY중고딕(altType=1)`·`HY헤드라인M(altType=1)`).
//
// 그 경우 조회가 통째로 비어 호스트에 `HY견고딕` 이 설치돼 있는데도 체인이 곧바로
// generic 으로 떨어졌다 — `한양견고딕` 은 **원 이름조차 체인에서 사라졌다**:
//
//   한양견고딕 alt=1 →  Malgun Gothic | Apple SD Gothic Neo | Noto Sans KR | …
//   한양견고딕 alt=2 →  HY견고딕 | Malgun Gothic | …
//
// 설치 face 를 다른 이름으로 찾는 일은 선언 altType 과 무관하므로, 타입 지정 조회가 비면
// 타입-무관 탐색으로 한 번 더 본다.
//
// **표면 범위.** Canvas2D paint 는 `fontFamilyChainForDisplay(name, 0, 0)` 로 altType 0 을
// 넘겨 이미 자동 탐색을 타므로 화면 결과는 바뀌지 않는다. 문서의 실제 altType 을 넘기는
// 곳은 폰트 판정 트레이스(`font-decision-trace.ts`)뿐이라, 이 수정은 **트레이스가 paint 와
// 같은 사슬을 보고하게** 만드는 정합 수정이다. 두 층이 갈리면 진단이 잘못된 원인을 가리킨다.
//
// 참고 — `Malgun Gothic` 으로 떨어지는 것이 왜 눈에 띄는지: 글꼴 파일 실측상 `Malgun Gothic`
// 만 `『`(U+300E)·`【`·`「` 를 **반각**으로 두고 잉크를 em 왼쪽 절반에 그린다
// (`『` advance 0.517em, 잉크 0.152..0.463 — 한양/HY/Batang/Dotum 은 모두 1.000em).

import assert from 'node:assert/strict';
import test from 'node:test';

import { fontFamilyCandidatesForDisplay } from '../src/core/font-substitution.ts';

/** 이 호스트에 한컴 legacy face 가 설치돼 있다고 가정한 목록. */
const INSTALLED = ['HY중고딕', 'HY견고딕', 'HY신명조', 'HY견명조', 'Malgun Gothic', 'Batang'];

const PAIRS: readonly (readonly [string, string])[] = [
  ['한양중고딕', 'HY중고딕'],
  ['한양견고딕', 'HY견고딕'],
  ['한양신명조', 'HY신명조'],
  ['한양견명조', 'HY견명조'],
];

test('legacy 한양 face 는 altType 과 무관하게 설치 face 를 체인 맨 앞에 둔다', () => {
  for (const [legacy, installed] of PAIRS) {
    for (const altType of [0, 1, 2]) {
      const chain = fontFamilyCandidatesForDisplay(legacy, altType, 0, {
        confirmedLocalFonts: INSTALLED,
      }) as string[];
      assert.equal(
        chain[0],
        installed,
        `${legacy}(altType=${altType}) 의 첫 후보가 ${installed} 여야 한다: ${chain.join(' | ')}`,
      );
    }
  }
});

test('설치 face 가 없으면 종전 체인을 그대로 유지한다', () => {
  // 이 수정은 "설치돼 있는데 못 찾는" 구멍만 막는다 — 없는 호스트의 동작은 바뀌지 않는다.
  const chain = fontFamilyCandidatesForDisplay('한양중고딕', 1, 0, {
    confirmedLocalFonts: [],
  }) as string[];
  assert.equal(chain[0], '한양중고딕');
  assert.ok(
    chain.includes('Malgun Gothic'),
    `설치 face 가 없으면 종전 generic 폴백이 남아야 한다: ${chain.join(' | ')}`,
  );
});
