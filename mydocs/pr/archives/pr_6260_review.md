---
kind: pr-review
status: accepted
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-28
---

# PR #6260 review - #6196 저장 단일 줄 셀의 과도한 자간 압축 억제

## 라우팅

- 원 PR: https://github.com/edwardkim/rhwp/pull/6260
- 작성자: `planet6897`
- 원 PR head: `b3c67ae91cfb`
- 통합 검토 브랜치: `review/open-ci-green-20260828`
- 최신 기준: `upstream/devel@1a43a507c9da`
- 원 PR 상태: non-draft, 실패·진행 check 0건
- 관련 이슈: #6196

## 검토 판단

**수용 권고.** 저장 `lineSeg`가 단일 줄 셀에 이미 맞춰진 경우에도 다시 과도한 자간 압축을 적용해
글자가 셀 오른쪽 경계를 넘거나 잘리는 문제를 좁힌다. 판단은 #6196 fixture의 저장 단일 줄 셀 보정
범위로 제한한다.

## 증적과 검증

- 원 PR 시각 보고서:
  `mydocs/report/cell-overflow-spacing-6196/{p4_cell_before,p4_cell_after}.png`
- 통합 head 기준 MCP/visual sweep:
  - `rhwp info --json`: `mydocs/pr/assets/pr_6275_issue6196_info.json`
  - 저장 제품: `hancom-office-2020 11.0.0.4585` -> MCP `engine 2020`
  - 기준 PDF: `pdf/pr_6275/by_saved_version/pr6275_issue6196_cell_char_spacing_fit-2020.pdf`
    (1 page)
  - visual sweep: `pr6275-issue6196-p1`, p1, flagged 0, pixel match `91.90489%`,
    visual accuracy proxy `28.76852%`
  - 장기 asset:
    `mydocs/pr/assets/pr_6275_issue6196_visual_review_p1.png`,
    `mydocs/pr/assets/pr_6275_issue6196_visual_sweep_summary.json`,
    `mydocs/pr/assets/pr_6275_issue6196_visual_overlay_metrics.json`
- 검토자가 직접 확인한 대표 after: before에서 `우수 내용` 셀 오른쪽 경계를 넘던 내용이 after에서
  셀 내부에 들어온다. 통합 head visual sweep review PNG에서도 해당 단일 줄 셀의 우측 경계 침범
  원 결함은 재현되지 않았다.
- focused test: `issue_6196_stored_single_line_cell_compresses_to_fit` 1 pass
- 통합 head 공통 검증: fmt, unit tier, suite manifest, focused regressions, clippy, 전체 nextest,
  Native Skia 3종, WASM build 통과.

## 코멘트 처리

merge 후 원 PR/issue 코멘트에는 `p4_cell_before.png`/`p4_cell_after.png` 비교로 fixture-specific
개선임을 명확히 적고, 전체 corpus의 광범위 개선처럼 과장하지 않는다.
`pr_6275_issue6196_visual_review_p1.png`를 merge commit SHA 고정 raw URL로 표시하고,
"내용 픽셀 중심 자동 일치율 보조값 = 약 28.77%" 및 자동값의 한계를 함께 둔다.

## 후속

추가 보정 필요 없음.
