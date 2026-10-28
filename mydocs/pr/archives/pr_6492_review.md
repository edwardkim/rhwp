# PR #6492 검토 - 미주 용지 밖 관문

- 원 PR head: d31ecb7f1725dcfbf97f5ef94baf586b994c93a9
- 통합 cherry-pick: 7054c27407dde03283b8c614cbf4486c9a307938
- 통합 기준: 76532b4da0e720026fb24211ad0c382884d3b970

## 판정: 메인터너 보정 됨 수용 가능

## 확인한 범위

미주 관문의 bleed를 고정 56px과 실제 용지 여백의 최솟값으로 제한한다.

## 검증 및 증적

issue_5886_endnote_offcanvas_guard_respects_paper 2/2와 공통 전체 회귀를 통과했다.

원 PR 증적은 mydocs/report/5886-endnote-offcanvas-guard/{before,after,compare}.png이며 current-head sweep은 공통 증적 문서에 있다.

## 다음 조건

동일 입력·oracle에서 upstream/devel과 통합 head의 base-vs-head sweep을 만들어 12쪽 문제 구간의 변화를 분리한다.

공통 검증 세부 내용은 pr_6489_6517_planet6897_integration_evidence.md를 따른다.
## 2026-08-31 메인터너 보정 검증

**최종 판정: 메인터너 보정 됨 수용 가능.**

- 같은 HWPX와 Hancom 2024 PDF를 `upstream/devel` base `e718f40b` 및 현재 후보에 각각 적용했다. base는 5/5 페이지가 flag였고 현재 후보는 3/5로 감소했으며, 12쪽 terminal overflow와 13쪽 column text-flow collapse/marker drift가 해소됐다.
- 남은 19·23쪽 차이는 base에도 동일하게 존재해 이번 체리픽/보정의 신규 회귀로 귀속하지 않았다.
- base/current 12쪽 review 이미지는 각각 `maintainer-20260831/pr6492-base-p012-review.png`, `maintainer-20260831/pr6492-current-p012-review.png`에 보존했다.
