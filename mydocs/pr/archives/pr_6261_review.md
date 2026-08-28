---
kind: pr-review
status: accepted
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-28
---

# PR #6261 review - #6206 표 셀 안 쪽번호 재시작 수집

## 라우팅

- 원 PR: https://github.com/edwardkim/rhwp/pull/6261
- 작성자: `planet6897`
- 원 PR head: `91776e11acc3`
- 통합 검토 브랜치: `review/open-ci-green-20260828`
- 최신 기준: `upstream/devel@1a43a507c9da`
- 원 PR 상태: non-draft, 실패·진행 check 0건
- 관련 이슈: #6206

## 검토 판단

**수용 권고.** 표 셀 내부의 `새 번호로 시작` 컨트롤을 본문 순회에서 놓쳐, 셀 안 쪽번호가 이전
페이지 절대 번호를 유지하던 문제를 고친다. 컨트롤 수집 범위를 셀 문단까지 확장하는 방향이 이슈와
맞는다.

## 증적과 검증

- 원 PR 증적:
  `mydocs/report/assets/issue_6206/pagenum-113424-p7.png`,
  `mydocs/report/assets/issue_6206/pagenum-156555538-p2.png`
- 통합 head 기준 MCP/visual sweep:
  - securities fixture `rhwp info --json`: `mydocs/pr/assets/pr_6275_issue6206_securities_info.json`
  - securities 저장 제품: `hancom-office-2020 11.0.0.6402` -> MCP `engine 2020`
  - securities 기준 PDF:
    `pdf/pr_6275/by_saved_version/pr6275_issue6206_securities_settlement_review-2020.pdf`
    (17 pages)
  - securities visual sweep: `pr6275-issue6206-securities-p2`, p2, flagged 1
    (`render_tree_frame_tail_overflow`), pixel match `87.91134%`, visual accuracy proxy `29.94235%`
  - securities 장기 asset:
    `mydocs/pr/assets/pr_6275_issue6206_securities_visual_review_p2.png`,
    `mydocs/pr/assets/pr_6275_issue6206_securities_visual_sweep_summary.json`,
    `mydocs/pr/assets/pr_6275_issue6206_securities_visual_overlay_metrics.json`
  - ACRC fixture `rhwp info --json`: `mydocs/pr/assets/pr_6275_issue6206_acrc_info.json`
  - ACRC 저장 제품: `hancom-office-2024 13.0.0.1053` -> MCP `engine 2024`
  - ACRC 기준 PDF:
    `pdf/pr_6275/by_saved_version/pr6275_issue6206_acrc_113424_review-2024.pdf`
    (46 pages)
  - ACRC visual sweep: `pr6275-issue6206-acrc-p7`, p7, flagged 0, pixel match `86.45204%`,
    visual accuracy proxy `15.49589%`
  - ACRC 장기 asset:
    `mydocs/pr/assets/pr_6275_issue6206_acrc_visual_review_p7.png`,
    `mydocs/pr/assets/pr_6275_issue6206_acrc_visual_sweep_summary.json`,
    `mydocs/pr/assets/pr_6275_issue6206_acrc_visual_overlay_metrics.json`
- 검토자가 직접 확인한 대표 증적: before는 `- 7 -` 또는 `- 2 -`로 남고, after/한컴 oracle은
  `- 1 -`로 재시작한다. 통합 head visual sweep review PNG에서도 두 대표 문서 모두 쪽번호가
  `- 1 -`로 재시작한다. securities p2의 자동 후보는 footer `- 1 -` bbox가 frame 아래로 남는
  `render_tree_frame_tail_overflow`이며, 이번 PR의 핵심 주장인 "셀 내부 새 번호 수집과 쪽번호 재시작"은
  한컴 기준 PDF와 일치한다. 해당 tail 후보는 blocker가 아니라 후속 관찰 항목으로 남긴다.
- 통합 head 공통 검증: fmt, unit tier, suite manifest, focused regressions, clippy, 전체 nextest,
  Native Skia 3종, WASM build 통과.

## 코멘트 처리

merge 후 원 PR/issue 코멘트에는 두 대표 PNG의 before/after/oracle 관계를 적고, 쪽번호 재시작이 표
셀 내부 컨트롤 수집으로 해결됐다고 설명한다. `pr_6275_issue6206_acrc_visual_review_p7.png`와
`pr_6275_issue6206_securities_visual_review_p2.png`를 merge commit SHA 고정 raw URL로 표시한다.
수치 문구는 각각 "내용 픽셀 중심 자동 일치율 보조값 = 약 15.50%"와 "약 29.94%"로 적고,
securities p2의 `render_tree_frame_tail_overflow`는 원 결함과 별도인 후속 관찰이라고 명시한다.

## 후속

추가 보정 필요 없음.
