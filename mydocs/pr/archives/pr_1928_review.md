# PR #1928 검토 — Task #1880: 빈-앵커 스택 spacing_before 인코딩 불안정 수정

- 작성일: 2026-07-05 / planet6897 → devel / 시간순 3/7 (04:40)

## 요지
typeset의 "자리차지 sb 제외" 분기에 빈-앵커 스택 예외
`!(is_topbottom_empty_anchor && next_is_empty_table_anchor)` 추가 — #1863 보존 규칙과 동일
조건쌍. 한글 2022 오라클: 3075729 양 인코딩 모두 p13(sb 보존). razor-thin 인접 핀 전부
PASS(rowbreak 20/20 포함), big 5,000 A/B 완전 동일.

## 검토 — ⚠ 메인테이너 충돌 통합
#1927과 **같은 줄**(`let before = ...`)을 수정 — supersede 체인 패턴 (b) 절차로 통합:
`table_wrap_take_place`(#1927 판정) + 빈-앵커 스택 예외(#1928)를 합성. 두 수정은 층위가
다름: #1927 = convert-HWP를 HWPX 경로로 정합(판정 게이트), #1928 = 판정이 발동하는
HWP5-native 경로의 sb 보존 예외. 통합 후 두 PR의 핀이 모두 결합 게이트에서 검증됨.
보고서 add/add는 #1928판을 `_report_v2.md`로 분리 보존.

## 판단
머지 권고(통합 해소 상태). 컨트리뷰터 코멘트에 통합 diff 공유 예정.

## 결합 게이트 결과 (devel `d982067b` + 7건 시간순 통합, 2026-07-05)

fmt 통과 / clippy 경고 0 / `cargo test --profile release-test --tests` **2,889 통과·실패 0**
(신규 핀 11건 포함) / OVR baseline 5샘플 **개체 회귀 0건** (기준 00014ecf)
