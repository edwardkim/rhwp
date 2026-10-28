---
kind: pr-review
status: accepted-pending-integration-pr
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-26
---

# PR #6089 review - #6035 저장 분할 행 고아 가드

## 라우팅

- 원 PR: https://github.com/edwardkim/rhwp/pull/6089
- 작성자: `planet6897`
- 원 PR head: `35b770ddcf7c`
- 통합 검토 브랜치: `review/open-prs-20260826-r1`
- 기준: `upstream/devel@1011a89475c9` (#6142 merge 포함)
- 원 PR 상태: non-draft, source CI 녹색, comments/reviews 0건

## 검토 판단

**수용 가능**. 저장 분할 흔적이 있는 표 행에서 줄 단위 컷이 과도하게 보수적으로 동작하던 지점을
완화해 실제 저장 줄과 배치 결과의 괴리를 줄인다. 변경은 typeset/table layout 경계에 국한되어 있고,
대표 fixture에서 의도한 방향의 시각 개선을 확인했다.

## 증적과 검증

- 대표 시각 증적: `mydocs/report/cell-row-line-split-6035/before_p5.png`,
  `mydocs/report/cell-row-line-split-6035/after_p5.png`,
  `mydocs/report/cell-row-line-split-6035/before_p6.png`,
  `mydocs/report/cell-row-line-split-6035/after_p6.png`
- 직접 visual_sweep 증적:
  - `rhwp info --json`: `mydocs/pr/assets/sample_issue6035_2804253_cosmetics_gmp_checklist_info.json`
  - local Hancom 2020 PDF: `pdf/pr_6088_6144/local_hancom2020/pr_6089_issue6035_cosmetics_gmp_checklist-local2020.pdf`
  - 대표 review PNG: `mydocs/pr/assets/pr_6089_issue6035_visual_review_005.png`, `mydocs/pr/assets/pr_6089_issue6035_visual_review_006.png`
  - metrics: `mydocs/pr/assets/pr_6088_6144_local_visual_sweep_metrics.tsv`, `mydocs/pr/assets/pr_6088_6144_local_visual_sweep_flags.tsv` (`pages=5,6`, `flagged_page_count=0`, worst pixel match `86.65941%`)
- 통합 후보 전체 검증:
  - `cargo fmt --all -- --check` 통과
  - `cargo clippy --locked --all-targets --target-dir target/pr-review -- -D warnings` 통과
  - 전체 nextest 8,399 pass, 43 skip
  - WASM build, native-Skia `--lib`, `issue_2225_missing_picture_placeholder`,
    `render_p37_direct_pdf_export` 통과

## 후속

통합 PR CI 완료 후 원 PR과 관련 이슈에 시각 증적 경로와 전체 회귀 결과를 남긴다.
