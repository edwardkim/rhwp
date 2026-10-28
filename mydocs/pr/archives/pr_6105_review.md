---
kind: pr-review
status: accepted-pending-integration-pr
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-26
---

# PR #6105 review - #6101 블록 TAC 표와 동거 텍스트 줄 분리

## 라우팅

- 원 PR: https://github.com/edwardkim/rhwp/pull/6105
- 작성자: `planet6897`
- 원 PR head: `b21ee1bc24cb`
- 통합 검토 브랜치: `review/open-prs-20260826-r1`
- 기준: `upstream/devel@1011a89475c9`
- 원 PR 상태: non-draft, source CI 녹색, comments/reviews 0건

## 검토 판단

**수용 가능**. 저장상 한 줄로 붙어 있는 블록 TAC 표와 일반 텍스트를 배치 시 표 줄/텍스트 줄로 분리해
표와 본문이 같은 줄 회계에 섞이는 문제를 줄인다.

## 증적과 검증

- 대표 시각 증적: `mydocs/report/block-tac-line-split-6101/before_36361137_p7.png`,
  `mydocs/report/block-tac-line-split-6101/after_36361137_p7.png`,
  `mydocs/report/block-tac-line-split-6101/before_36501883_p1.png`,
  `mydocs/report/block-tac-line-split-6101/after_36501883_p1.png`
- 통합 후보 전체 검증: rustfmt, clippy, 전체 nextest 8,399 pass, WASM, native-Skia 통과

- 직접 visual_sweep 증적:
  - `rhwp info --json`: `mydocs/pr/assets/sample_issue6101_36361137_firefighter_training_plan_info.json`, `mydocs/pr/assets/sample_issue6101_36501883_approval_doc_body_info.json`
  - local Hancom 2020 PDF: `pdf/pr_6088_6144/local_hancom2020/pr_6105_issue6101_firefighter_training_plan-local2020.pdf`, `pdf/pr_6088_6144/local_hancom2020/pr_6105_issue6101_approval_doc_body-local2020.pdf`
  - 대표 review PNG: `mydocs/pr/assets/pr_6105_issue6101_firefighter_visual_review_007.png`, `mydocs/pr/assets/pr_6105_issue6101_approval_visual_review_001.png`
  - metrics: `mydocs/pr/assets/pr_6088_6144_local_visual_sweep_metrics.tsv`, `mydocs/pr/assets/pr_6088_6144_local_visual_sweep_flags.tsv` (`flagged_page_count=0`, worst pixel match `85.71163%`)

## 후속

통합 PR CI 완료 후 원 PR과 #6101 이슈에 전/후 증적을 포함해 close한다.
