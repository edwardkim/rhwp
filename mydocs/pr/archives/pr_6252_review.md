---
kind: pr-review
status: accepted
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-28
---

# PR #6252 review - #6174 글상자 clip descender 잘림 보정

## 라우팅

- 원 PR: https://github.com/edwardkim/rhwp/pull/6252
- 작성자: `planet6897`
- 원 PR head: `31b697549bc6`
- 통합 검토 브랜치: `review/open-ci-green-20260828`
- 최신 기준: `upstream/devel@1a43a507c9da`
- 원 PR 상태: non-draft, 실패·진행 check 0건
- 관련 이슈: #6174

## 검토 판단

**수용 권고.** 글상자 clip rect가 텍스트 descender와 하단 획을 과도하게 잘라 한글 하단이 손상되는
문제를 보정한다. clip 계산을 글줄 ink 범위에 맞춰 넓히는 방향이며, 대표 fixture에서 하단 획 잘림이
사라진다.

## 증적과 검증

- 원 PR 시각 보고서:
  `mydocs/report/textbox-clip-descender-6174/{before_p1,after_p1,oracle_p1}.png`
- 통합 head 기준 MCP/visual sweep:
  - `rhwp info --json`: `mydocs/pr/assets/pr_6275_issue6174_info.json`
  - 저장 제품: `hancom-office-2018 10.0.0.13015` -> MCP `engine 2020`
  - 기준 PDF: `pdf/pr_6275/by_saved_version/pr6275_issue6174_police_press_release-2020.pdf`
    (2 pages)
  - visual sweep: `pr6275-issue6174-p1`, p1, flagged 0, pixel match `91.58762%`,
    visual accuracy proxy `18.61601%`
  - 장기 asset:
    `mydocs/pr/assets/pr_6275_issue6174_visual_review_p1.png`,
    `mydocs/pr/assets/pr_6275_issue6174_visual_sweep_summary.json`,
    `mydocs/pr/assets/pr_6275_issue6174_visual_overlay_metrics.json`
- 검토자가 직접 확인한 대표 after/oracle: `경찰청` 글자 하단 획이 잘리지 않고 oracle과 같은
  세로 범위 안에 남는다. 통합 head visual sweep review PNG에서도 글상자 하단 glyph가 clip에
  잘려 사라지는 원 결함은 재현되지 않았다.
- focused test: `issue_6174_textbox_clip_contains_its_own_line` 1 pass
- 통합 head 공통 검증: fmt, unit tier, suite manifest, focused regressions, clippy, 전체 nextest,
  Native Skia 3종, WASM build 통과.

## 코멘트 처리

merge 후 원 PR/issue 코멘트에는 `after_p1.png`/`oracle_p1.png` 직접 확인 결과와 focused test 통과를
적는다. `pr_6275_issue6174_visual_review_p1.png`를 merge commit SHA 고정 raw URL로 표시하고,
"내용 픽셀 중심 자동 일치율 보조값 = 약 18.62%" 및 자동값의 한계를 함께 둔다.

## 후속

추가 보정 필요 없음.
