---
kind: pr-review
status: accepted-pending-integration-pr
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-26
---

# PR #6100 review - #6095 중첩 표 아래 host 줄 앵커

## 라우팅

- 원 PR: https://github.com/edwardkim/rhwp/pull/6100
- 작성자: `planet6897`
- 원 PR head: `957d49fc4321`
- 통합 검토 브랜치: `review/open-prs-20260826-r1`
- 기준: `upstream/devel@1011a89475c9`
- 원 PR 상태: non-draft, source CI 녹색, comments/reviews 0건

## 검토 판단

**수용 가능**. 중첩 표 뒤 host 줄을 표 상단으로 오독하지 않도록 하여 후속 본문 앵커가 위로 당겨지는
문제를 막는다.

## 증적과 검증

- 대표 시각 증적: `mydocs/report/post-table-host-anchor-6095/before_p1.png`,
  `mydocs/report/post-table-host-anchor-6095/after_p1.png`,
  `mydocs/report/post-table-host-anchor-6095/before_p2.png`,
  `mydocs/report/post-table-host-anchor-6095/after_p2.png`
- 직접 visual_sweep 증적:
  - `rhwp info --json`: `mydocs/pr/assets/sample_issue6095_3090867_icepack_levy_criteria_info.json`
  - local Hancom 2020 PDF: `pdf/pr_6088_6144/local_hancom2020/pr_6100_issue6095_icepack_levy_criteria-local2020.pdf`
  - 대표 review PNG: `mydocs/pr/assets/pr_6100_issue6095_visual_review_001.png`, `mydocs/pr/assets/pr_6100_issue6095_visual_review_002.png`
  - metrics: `mydocs/pr/assets/pr_6088_6144_local_visual_sweep_metrics.tsv`, `mydocs/pr/assets/pr_6088_6144_local_visual_sweep_flags.tsv` (`pages=1,2`, `flagged_page_count=0`, worst pixel match `87.58375%`)
- 통합 후보 전체 검증: rustfmt, clippy, 전체 nextest 8,399 pass, WASM, native-Skia 통과

## 후속

원 PR comment에는 중첩 표 뒤 host 줄 위치 증적을 포함한다.
