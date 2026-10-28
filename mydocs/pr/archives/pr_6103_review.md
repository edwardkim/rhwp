---
kind: pr-review
status: accepted-pending-integration-pr
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-26
---

# PR #6103 review - #6099 Studio 90도 회전 그림 DOM frame

## 라우팅

- 원 PR: https://github.com/edwardkim/rhwp/pull/6103
- 작성자: `planet6897`
- 원 PR head: `b815d1e34e64`
- 통합 검토 브랜치: `review/open-prs-20260826-r1`
- 기준: `upstream/devel@1011a89475c9`
- 원 PR 상태: non-draft, source CI 녹색, comments/reviews 0건

## 검토 판단

**수용 가능**. Studio에서 90도 회전 그림의 outer DOM frame과 inner image transform이 이중 회전되는
문제를 분리해 가로 방향으로 정상 표시되도록 한다.

## 증적과 검증

- 대표 시각 증적: `mydocs/report/studio-rotate-6099/before_rotated_sideways.png`,
  `mydocs/report/studio-rotate-6099/after_correct_landscape.png`
- e2e manifest 보정: `rhwp-studio/e2e/issue-6099-probe.mjs`를 진단 `legacy-name` 항목으로 명시했다.
- Studio 검증: unit 100 tests pass, `npm run build` 통과, e2e manifest check 통과
- 통합 후보 전체 Rust/WASM/native-Skia 검증 통과

- 직접 visual_sweep 증적:
  - `rhwp info --json`: `mydocs/pr/assets/sample_issue6099_2197981_scanned_form_info.json`
  - local Hancom 2020 PDF: `pdf/pr_6088_6144/local_hancom2020/pr_6103_issue6099_scanned_form-local2020.pdf`
  - 대표 review PNG: `mydocs/pr/assets/pr_6103_issue6099_visual_review_2197981.png`
  - metrics: `mydocs/pr/assets/pr_6088_6144_local_visual_sweep_metrics.tsv`, `mydocs/pr/assets/pr_6088_6144_local_visual_sweep_flags.tsv` (`flagged_page_count=0`, pixel match `96.48308%`)

## 후속

원 PR에는 Studio 회전 그림 증적과 e2e manifest 보정 사유를 함께 코멘트한다.
