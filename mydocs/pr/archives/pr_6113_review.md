---
kind: pr-review
status: accepted-pending-integration-pr
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-26
---

# PR #6113 review - #6102 자리차지 표 host 과밀 저장 줄

## 라우팅

- 원 PR: https://github.com/edwardkim/rhwp/pull/6113
- 작성자: `planet6897`
- 원 PR head: `915cd17a1cb6`
- 통합 검토 브랜치: `review/open-prs-20260826-r1`
- 기준: `upstream/devel@1011a89475c9`
- 원 PR 상태: non-draft, source CI 녹색, comments/reviews 0건

## 검토 판단

**수용 가능**. 자리차지 표 host 문단의 저장 줄이 과밀한 경우 저장 좌표를 그대로 신뢰하지 않고 frame
기준으로 재래핑하게 해 첫 줄 overflow를 줄인다.

## 증적과 검증

- 대표 시각 증적:
  - `mydocs/report/section-first-line-6102/before_36310257_p1.png`
  - `mydocs/report/section-first-line-6102/after_36310257_p1.png`
  - `mydocs/report/section-first-line-6102/before_36360328_p1.png`
  - `mydocs/report/section-first-line-6102/after_36360328_p1.png`
  - `mydocs/report/section-first-line-6102/before_36444579_p1.png`
  - `mydocs/report/section-first-line-6102/after_36444579_p1.png`
- 통합 후보 전체 검증: rustfmt, clippy, 전체 nextest 8,399 pass, WASM, native-Skia 통과

- 직접 visual_sweep 증적:
  - `rhwp info --json`: `mydocs/pr/assets/sample_issue6102_36310257_overtime_report_info.json`, `mydocs/pr/assets/sample_issue6102_36360328_vehicle_inspection_expense_info.json`, `mydocs/pr/assets/sample_issue6102_36444579_traffic_fine_exemption_info.json`
  - local Hancom 2020 PDF: `pdf/pr_6088_6144/local_hancom2020/pr_6113_issue6102_overtime_report-local2020.pdf`, `pdf/pr_6088_6144/local_hancom2020/pr_6113_issue6102_vehicle_inspection_expense-local2020.pdf`, `pdf/pr_6088_6144/local_hancom2020/pr_6113_issue6102_traffic_fine_exemption-local2020.pdf`
  - 대표 review PNG: `mydocs/pr/assets/pr_6113_issue6102_overtime_visual_review_36310257.png`, `mydocs/pr/assets/pr_6113_issue6102_vehicle_visual_review_36360328.png`, `mydocs/pr/assets/pr_6113_issue6102_traffic_visual_review_001.png`
  - metrics: `mydocs/pr/assets/pr_6088_6144_local_visual_sweep_metrics.tsv`, `mydocs/pr/assets/pr_6088_6144_local_visual_sweep_flags.tsv` (`flagged_page_count=0`, worst pixel match `94.23661%`)

## 후속

원 PR comment에는 3개 대표 문서의 첫 쪽 전/후 증적을 근거로 남긴다.
