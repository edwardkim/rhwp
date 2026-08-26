---
kind: pr-review
status: accepted-pending-integration-pr
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-26
---

# PR #6094 review - #6087 vpos=0 충돌 판정 제외

## 라우팅

- 원 PR: https://github.com/edwardkim/rhwp/pull/6094
- 작성자: `planet6897`
- 원 PR head: `f6282d81ccc7`
- 통합 검토 브랜치: `review/open-prs-20260826-r1`
- 기준: `upstream/devel@1011a89475c9`
- 원 PR 상태: non-draft, source CI 녹색, comments/reviews 0건

## 검토 판단

**수용 가능**. 쪽을 실제로 점유하지 않는 문단을 vpos=0 충돌 후보에서 제외해 첫 쪽 상단 충돌 오탐을
줄인다. 배치 좌표가 없는 carrier가 실제 페인트 줄과 경쟁하지 않도록 한정된 조건을 추가한다.

## 증적과 검증

- 대표 시각 증적: `mydocs/report/zero-advance-collision-6087/before_p1.png`,
  `mydocs/report/zero-advance-collision-6087/after_p1.png`
- 직접 visual_sweep 증적:
  - `rhwp info --json`: `mydocs/pr/assets/sample_issue6060_30307_local_service_reform_info.json` (`lastSavedWith.product=null`, version `7.5.12.614`)
  - local Hancom 2020 PDF: `pdf/pr_6088_6144/local_hancom2020/pr_6093_6094_issue6060_local_service_reform-local2020.pdf`
  - 대표 review PNG: `mydocs/pr/assets/pr_6093_6094_issue6060_visual_review_001.png`
  - metrics: `mydocs/pr/assets/pr_6088_6144_local_visual_sweep_metrics.tsv`, `mydocs/pr/assets/pr_6088_6144_local_visual_sweep_flags.tsv` (`page=1`, `flagged_page_count=0`, pixel match `95.99198%`)
- 통합 후보 전체 검증: rustfmt, clippy, 전체 nextest 8,399 pass, WASM, native-Skia 통과

## 후속

통합 PR CI 완료 후 #6087 이슈와 원 PR에 수용 근거를 남긴다.
