# 화학제품 표시 조사보고서의 한컴 Print 대조

2026-11-23, 96 DPI에서 보정 전 2px 실루엣을 측정했다. 독립 한컴 2020 Print와
같은 103쪽을 출력했으며 [Native](native.tsv)와 [fresh WASM](wasm.tsv)의 전쪽
최저는 각각 99.53%다. 23·83·99쪽은 양쪽 모두 100.00%였다.

Print의 23·99쪽에는 휴먼모음T(`MoeumTR-HM`)를 사용한 글줄이 있다. 같은 원본
프로그램 `HMFMMUEX.TTC`를 기존 TrueType Collection 공급 순서에 추가하자
fresh WASM의 해당 두 쪽이 96.76%·97.04%에서 각각 100.00%로 올랐다.
프로그램 SHA-256은 `be52401ea71e34d7c52bffacf25f95e3cc0bfdadc8a2a1f377d458f22484f7b7`이다.

| 자료 | SHA-256 |
| --- | --- |
| `samples/issue6782/1480000-201900042-chemical-product-labeling-study.hwp` | `398d03a5d5e4d6e857086be532d6d9ed0cec9c8ad06f95c17bbb7f83056ae860` |
| `pdf/1480000-201900042-chemical-product-labeling-study-2020.pdf` | `f8e5c0408e221080ede9a9a67b153d02d792d22961c738e46749641f32a32e79` |
| Native 전쪽 TSV | `2d40ed836fa5d1cc7981ce4c8239de58e8993bf4b458b2681e29fe633d724b05` |
| fresh WASM 전쪽 TSV | `c2da526baf421ee89ff73ef8b47c7049bad304b4b3c44feb6094bffd53525ceb` |

## 83쪽 후속 제목

표 문단과 그 다음 빈 문단의 원본 바깥여백, 후속 ‘일반인 대상 선호도 조사’ 제목의
소속과 순서를 검사한다. 최종 조판 소스를 반영한 Native·fresh WASM에서 83쪽은
각각 100.00%, 전쪽 최저는 양쪽 99.53%로 재현했다.
[83쪽 표와 후속 제목 확대 비교](page-83-table-followup.png)
