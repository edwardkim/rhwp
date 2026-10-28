---
kind: pr-review
status: accepted-with-maintainer-correction
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-26
---

# PR #6097 review - #6025 CELL 분할 조각 회계 회귀 핀

## 라우팅

- 원 PR: https://github.com/edwardkim/rhwp/pull/6097
- 작성자: `planet6897`
- 원 PR head: `1c1dd76ac235`
- 통합 검토 브랜치: `review/open-prs-20260826-r1`
- 기준: `upstream/devel@1011a89475c9`
- 원 PR 상태: non-draft, source CI 녹색, comments/reviews 0건

## 검토 판단

**메인터너 보정 포함 수용 가능**. 원 PR은 #6025의 CELL 분할 조각 회계와 페인트 정합을 회귀 테스트로
고정한다. 통합 과정에서 #6025/#6035 조합이 드러낸 CENTER 셀 source-frame 판정 차이가 있어, 메인터너
보정 commit `cd4866211`로 셀 source-frame 분류를 함께 정리했다.

## 메인터너 보정

- 보정 이유: source PR 자체는 테스트 핀이지만, 같은 통합 후보의 table fragment 계열 변경과 결합되면
  CENTER 셀의 기준 frame을 잘못 해석할 수 있어 CI/회귀에서 재노출될 수 있었다.
- 보정 범위: `src/renderer/layout/table_layout.rs`의 source-frame 판정 경계.
- 보정 후 전체 nextest와 native-Skia 범위를 모두 통과했다.

## 증적과 검증

- 추가 테스트: `tests/cases/issue_6025_cell_fragment_budget_pin.rs`
- 직접 visual_sweep 증적:
  - `rhwp info --json`: `mydocs/pr/assets/sample_issue6025_3232693_employment_support_criteria_info.json`
  - MCP Hancom 2020 PDF: `pdf/pr_6088_6144/hancom2020/pr_6088_6144_issue6025_employment_support_criteria_3232693_employment_support_criteria-2020.pdf` (`sha256=985dab08280f0cb7a2489c796614337e230871428bc0ac98b5cd3be4b5abd579`)
  - 대표 review PNG: `mydocs/pr/assets/pr_6097_issue6025_mcp2020_visual_review_001.png`
  - metrics: `mydocs/pr/assets/pr_6088_6144_mcp2020_visual_sweep_metrics.tsv` (`page=1`, `flagged_page_count=0`, pixel match `80.49104%`)
- 통합 후보 전체 검증: rustfmt, clippy, 전체 nextest 8,399 pass, WASM, native-Skia 통과

## 후속

원 PR comment에는 source PR 테스트 핀을 수용하면서 통합 후보에서 필요한 메인터너 보정을 별도 수행한
이유를 함께 적는다.
