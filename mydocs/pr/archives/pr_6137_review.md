---
kind: pr-review
status: accepted-pending-integration-pr
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-26
---

# PR #6137 review - #6127 사각 안 숫자 PUA 벡터 합성

## 라우팅

- 원 PR: https://github.com/edwardkim/rhwp/pull/6137
- 작성자: `planet6897`
- 원 PR head: `4672fbe566d6`
- 통합 검토 브랜치: `review/open-prs-20260826-r1`
- 기준: `upstream/devel@1011a89475c9`
- 원 PR 상태: non-draft, source CI 녹색, comments/reviews 0건

## 검토 판단

**수용 가능**. 사각 안 숫자 PUA가 SVG·Skia 평문 경로에서 tofu나 일반 글리프로 빠지지 않도록 벡터
fallback 합성 경로를 추가한다.

## 증적과 검증

- 대표 시각 증적: `mydocs/report/boxed-pua-fallback-6127/before_p1.png`,
  `mydocs/report/boxed-pua-fallback-6127/after_p1.png`
- 통합 후보 전체 검증: rustfmt, clippy, 전체 nextest 8,399 pass, WASM, native-Skia 통과

- 직접 visual_sweep 증적:
  - `rhwp info --json`: `mydocs/pr/assets/sample_issue6127_2599643_vessel_pass_application_info.json`
  - local Hancom 2020 PDF: `pdf/pr_6088_6144/local_hancom2020/pr_6137_issue6127_vessel_pass_application-local2020.pdf`
  - 대표 review PNG: `mydocs/pr/assets/pr_6137_issue6127_visual_review_2599643.png`
  - metrics: `mydocs/pr/assets/pr_6088_6144_local_visual_sweep_metrics.tsv`, `mydocs/pr/assets/pr_6088_6144_local_visual_sweep_flags.tsv` (`flagged_page_count=0`, pixel match `91.58224%`)

## 후속

원 PR comment에는 SVG·Skia 양쪽 평문 경로에서 fallback이 확인됐음을 명시한다.
