---
kind: pr-review
status: approved-with-fidelity-residual
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-21
---

# PR #5823 검토 - 쪽 넘김 overlay 표 잔여 행

## 판정

- source head `53d109baffbe54794f1f160a4343f04b9eec9850`를 적용했다.
- split overlay 표가 모든 행을 유지하고 뒤 문단과 겹치지 않게 흐름 높이를 예약한다. 기능 회귀는 없다.

## 검증과 잔여사항

- `issue_5792_overlay_table_split_overlap` 1/1 및 통합 전체 nextest 8,109/8,109 통과.
- HWP 2020 MCP PDF `pdf/pr_open_20260821/2700727_animal_facility_standards-2020.pdf` p3를 비교했다. job `e72e4425-b44f-4510-8a5e-0c5f32678574`, validation ok, 6/6 pages, SHA-256 `8d2cf07a6fca9b05ebc32126001ef18614c44af2e8cd49e08d20a1b083a1cb46`다.
- pixel match 85.405%, proxy 9.149%이며 SVG는 5쪽, 기준 PDF는 6쪽이다. 표 행 보존/후속 문단 비중첩이라는 PR 계약은 통과했지만 전면 PDF 충실도 차이는 이 변경만으로 해소되지 않은 잔여사항이다. 대표 화면은 `mydocs/pr/assets/pr_5823_issue5792_review_003.png`에 보존했다.

## 최종 판단과 GitHub 기록

- **수용, fidelity 후속 보류**: #5844 code candidate CI와 행 보존·후속 문단 비중첩 계약은 성공했다. 현재 5쪽 SVG와 6쪽 기준 PDF의 전면 page-flow/typography 차이는 이 PR 범위를 넘어선 잔여 fidelity 과제로 **보류**하며, 이 기능 보정을 막지는 않는다. 추가 메인터너 코드 보정은 필요하지 않다.
- merge 뒤 원 PR에는 수용 근거와 함께 PDF fidelity 후속 보류를 명시하고 #5844 통합 수용으로 close한다.
