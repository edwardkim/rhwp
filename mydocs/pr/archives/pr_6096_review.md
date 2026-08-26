---
kind: pr-review
status: accepted-pending-integration-pr
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-26
---

# PR #6096 review - #6086 좌/우 분할 저장 세그 계상

## 라우팅

- 원 PR: https://github.com/edwardkim/rhwp/pull/6096
- 작성자: `planet6897`
- 원 PR head: `e388435b193a`
- 통합 검토 브랜치: `review/open-prs-20260826-r1`
- 기준: `upstream/devel@1011a89475c9`
- 원 PR 상태: non-draft, source CI 녹색, comments/reviews 0건

## 검토 판단

**수용 가능**. 좌/우 분할 저장 세그먼트가 한 줄로 계상되어야 하는 경우를 고정해 표 내부 줄 높이와
페이지 넘김의 불필요한 증가를 막는다.

## 증적과 검증

- 대표 시각 증적: `mydocs/report/lr-split-seg-6086/before_p4.png`,
  `mydocs/report/lr-split-seg-6086/after_p4.png`
- 직접 visual_sweep 증적:
  - `rhwp info --json`: `mydocs/pr/assets/sample_issue6086_30098_resident_registration_reform_info.json` (`lastSavedWith.product=null`, version `7.5.12.614`)
  - local Hancom 2020 PDF: `pdf/pr_6088_6144/local_hancom2020/pr_6096_issue6086_resident_registration_reform-local2020.pdf`
  - 대표 review PNG: `mydocs/pr/assets/pr_6096_issue6086_visual_review_004.png`
  - metrics: `mydocs/pr/assets/pr_6088_6144_local_visual_sweep_metrics.tsv`, `mydocs/pr/assets/pr_6088_6144_local_visual_sweep_flags.tsv` (`page=4`, `flagged_page_count=0`, pixel match `88.22256%`)
- 통합 후보 전체 검증: rustfmt, clippy, 전체 nextest 8,399 pass, WASM, native-Skia 통과

## 후속

원 PR close 전 통합 PR CI 녹색과 최신 mergeability를 다시 확인한다.
