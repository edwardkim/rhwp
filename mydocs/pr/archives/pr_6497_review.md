# PR #6497 검토 - aim=false 중첩 셀 안 여백

- 원 PR head: 7959436ce01fc1e7942e8bf91987f3bf50a582fe
- 통합 cherry-pick: ab9ec17a7faf57ffe2f13a44eb63e539e04f9419
- 통합 기준: 76532b4da0e720026fb24211ad0c382884d3b970

## 판정: 메인터너 보정 됨 수용 가능

## 확인한 범위

aim=false 중첩 셀이 cell margin 대신 table inner margin을 사용하도록 예외를 제거하고 #2308 허용치를 조정한다.

## 검증 및 증적

issue_5301_nested_cell_uses_table_inner_margin 3/3, issue_2308_render_normalized_derived_state 5/5와 공통 전체 회귀를 통과했다.

원 PR 증적: mydocs/report/5301-nested-cell-inner-margin/{before,after,compare,한글}.png.

## 다음 조건

samples/issue1891/76076_regulatory_analysis.hwpx와 확인된 2024 oracle PDF의 current-head 비교로 34·66쪽의 글자 소실과 잉크 경계를 판정한다.

공통 검증 세부 내용은 pr_6489_6517_planet6897_integration_evidence.md를 따른다.
## 2026-08-31 메인터너 보정 검증

**최종 판정: 메인터너 보정 됨 수용 가능.**

- 원본은 Hancom Office 2018 저장본이고, 기준 PDF는 `Creator: Hwp 2024 13.0.0.3622`, `Producer: Hancom PDF 1.3.0.550`임을 확인했다. 원본 저장 버전과 PDF 변환 엔진을 혼동하지 않는다.
- 현재 후보로 34·66쪽을 sweep해 누락 페이지, frame overflow, content-bottom drift, column text-flow collapse, legacy glyph flag가 모두 없었다.
- review 이미지는 `maintainer-20260831/pr6497-p034-review.png`, `maintainer-20260831/pr6497-p066-review.png`에 보존했다.
