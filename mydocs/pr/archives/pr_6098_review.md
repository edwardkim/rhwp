---
kind: pr-review
status: accepted-pending-integration-pr
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-26
---

# PR #6098 review - #5966 강제 새 쪽 표 각주

## 라우팅

- 원 PR: https://github.com/edwardkim/rhwp/pull/6098
- 작성자: `planet6897`
- 원 PR head: `ba4eb15162b9`
- 통합 검토 브랜치: `review/open-prs-20260826-r1`
- 기준: `upstream/devel@1011a89475c9`
- 원 PR 상태: non-draft, source CI 녹색, comments/reviews 0건

## 검토 판단

**수용 가능**. 강제 새 쪽에서 이미 fresh-page 조건을 만족한 표 각주를 다시 밀어내지 않도록 처리해
쪽 넘김과 각주 배치의 과잉 보정을 방지한다.

## 증적과 검증

- 대표 증적: `mydocs/report/queued-table-footnote-5966/before_debug_panic.txt`,
  `mydocs/report/queued-table-footnote-5966/after_p69.png`,
  `mydocs/report/queued-table-footnote-5966/after_p70.png`
- 직접 visual_sweep 증적:
  - `rhwp info --json`: `mydocs/pr/assets/sample_issue5966_1130000-202100008_franchise_review_report_info.json`
  - MCP Hancom 2020 PDF: `pdf/pr_6088_6144/hancom2020/pr_6088_6144_issue5966_franchise_review_report_1130000-202100008_franchise_review_report-2020.pdf` (`sha256=4cf22800e29baddb9b84023462c75252efd4efb04d22e9e64a8e3d9313076009`)
  - 대표 review PNG: `mydocs/pr/assets/pr_6098_issue5966_mcp2020_visual_review_069.png`, `mydocs/pr/assets/pr_6098_issue5966_mcp2020_visual_review_070.png`
  - metrics: `mydocs/pr/assets/pr_6088_6144_mcp2020_visual_sweep_metrics.tsv` (`pages=69,70`, `flagged_page_count=0`, worst pixel match `89.47135%`)
- 통합 후보 전체 검증: rustfmt, clippy, 전체 nextest 8,399 pass, WASM, native-Skia 통과

## 후속

통합 PR CI 완료 뒤 원 PR에는 panic/렌더 증적과 함께 close comment를 남긴다.
