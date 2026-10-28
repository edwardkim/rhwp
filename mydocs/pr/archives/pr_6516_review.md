# PR #6516 검토 - 동일 문단 float 그림 앵커

- 원 PR head: 43edfd4330e5e0412ea23bfe3cb3adb7267fff28
- 통합 cherry-pick: 7bd930ba8a6dea85c2f3b6aa33cd7373d966c281, 9d9debf3f6c7bc54fd4174da4a9cea497eede73e, 776e246730309f362cc59c15e703c536318dc02e
- 통합 기준: 76532b4da0e720026fb24211ad0c382884d3b970

## 판정: 메인터너 보정 됨 수용 가능

## 확인한 범위

동일 문단의 비-TAC float 두 개 이상을 문단 앵커에 묶고 cell containment 안전망을 둔다.

## 검증 및 증적

issue_6494_cell_float_stays_in_its_cell 3/3과 공통 전체 회귀를 통과했다.

원 PR 증적: mydocs/report/6494-cell-float-containment/{before,after,compare,m0,m1,m2,한글}.png.

## 다음 조건

private 기준 문서를 전용 Windows 경로에서 식별한 뒤 Hancom 2024 conversion으로 current-head 두 map의 앵커와 containment를 대조한다.

공통 검증 세부 내용은 pr_6489_6517_planet6897_integration_evidence.md를 따른다.
## 2026-08-31 메인터너 보정 검증

**최종 판정: 메인터너 보정 됨 수용 가능.**

- 비공개 `156489219` HWP는 Hancom Office 2018 저장본이므로 Hancom `2020` profile로 변환했다: SHA-256 `30a009dc811b96066a684917008a5392e405cae21e5500553d927e8683b214a8`.
- 5쪽의 두 지도는 PDF와 현재 후보 모두 표 셀 안에 있고 동일한 세로 band를 유지한다. visual sweep의 `column_text_flow_collapse` 한 건은 큰 표-그림을 본문 흐름으로 오인한 false positive이며, frame overflow와 table left-strip deficit은 없다.
- `b2` focused #6494 회귀 3/3 및 전체 nextest `8888 passed, 0 failed, 46 skipped`를 통과했다. 직접 review 이미지는 `maintainer-20260831/pr6516-p005-review.png`다.
