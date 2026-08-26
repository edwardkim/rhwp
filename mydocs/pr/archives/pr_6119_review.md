---
kind: pr-review
status: accepted-pending-integration-pr
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-26
---

# PR #6119 review - #5820 축1 partial page start vpos base

## 라우팅

- 원 PR: https://github.com/edwardkim/rhwp/pull/6119
- 작성자: `planet6897`
- 원 PR head: `bda255c04a88`
- 통합 검토 브랜치: `review/open-prs-20260826-r1`
- 기준: `upstream/devel@1011a89475c9`
- 원 PR 상태: non-draft, source CI 녹색, comments/reviews 0건

## 검토 판단

**수용 가능**. 연속 partial fragment로 시작하는 쪽에서 vpos 스냅 기준을 저장 줄로 시드해, 페이지 시작
근처의 본문 위치가 비정상적으로 밀리거나 겹치는 문제를 줄인다.

## 증적과 검증

- 대표 시각 증적: `mydocs/report/partial-page-start-base-5820/before_p2.png`,
  `mydocs/report/partial-page-start-base-5820/after_p2.png`
- 통합 후보 전체 검증: rustfmt, clippy, 전체 nextest 8,399 pass, WASM, native-Skia 통과

- 직접 visual_sweep 증적:
  - `rhwp info --json`: `mydocs/pr/assets/sample_issue5820_156560092_ecard_meeting_press_info.json`
  - MCP Hancom 2020 PDF: `pdf/pr_6088_6144/hancom2020/pr_6088_6144_issue5820_ecard_meeting_press_156560092_ecard_meeting_press-2020.pdf` (`sha256=f4f395675f301ae57bcc7bdf918a5aae5cb8b052ff063e68bbadb0406145825b`)
  - 대표 review PNG: `mydocs/pr/assets/pr_6119_6120_6131_issue5820_mcp2020_visual_review_002.png`
  - metrics: `mydocs/pr/assets/pr_6088_6144_mcp2020_visual_sweep_metrics.tsv` (`page=2`, `flagged_page_count=0`, pixel match `96.77153%`)

## 후속

#5820 축1 후속 코멘트에는 partial page start 증적과 함께 통합 SHA를 남긴다.
