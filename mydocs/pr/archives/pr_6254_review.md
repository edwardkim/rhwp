---
kind: pr-review
status: accepted
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-28
---

# PR #6254 review - #6173 오른쪽 정렬 말미 공백 판정

## 라우팅

- 원 PR: https://github.com/edwardkim/rhwp/pull/6254
- 작성자: `planet6897`
- 원 PR head: `00cac13820e3`
- 통합 검토 브랜치: `review/open-ci-green-20260828`
- 최신 기준: `upstream/devel@1a43a507c9da`
- 원 PR 상태: non-draft, 실패·진행 check 0건
- 관련 이슈: #6173

## 검토 판단

**수용 권고.** 오른쪽 정렬 문단에서 마지막 인라인 개체 앞의 공백까지 말미 공백으로 제거해 로고가
우측으로 밀리던 문제를, 마지막 인라인 개체 뒤 공백만 제외하도록 고친다. `paragraph_layout`과
`shape_layout` 양쪽에 같은 계약을 적용한 점이 중요하다.

## 증적과 검증

- 원 PR 시각 보고서:
  `mydocs/report/right-align-inline-object-space-6173/{p2_textbox_before,p2_textbox_after}.png`
- 통합 head 기준 MCP/visual sweep:
  - `rhwp info --json`: `mydocs/pr/assets/pr_6275_issue6173_info.json`
  - 저장 제품: `hancom-office-2020 11.0.0.8969` -> MCP `engine 2020`
  - 기준 PDF: `pdf/pr_6275/by_saved_version/pr6275_issue6173_textbox_right_align_logos-2020.pdf`
    (2 pages)
  - visual sweep: `pr6275-issue6173-p2`, p2, flagged 0, pixel match `99.40067%`,
    visual accuracy proxy `45.36346%`
  - 장기 asset:
    `mydocs/pr/assets/pr_6275_issue6173_visual_review_p2.png`,
    `mydocs/pr/assets/pr_6275_issue6173_visual_sweep_summary.json`,
    `mydocs/pr/assets/pr_6275_issue6173_visual_overlay_metrics.json`
- 검토자가 직접 확인한 대표 after: 글상자 안 두 로고가 우단 안에 들어오며 잘림이 보이지 않는다.
  통합 head visual sweep review PNG에서도 오른쪽 정렬 말미 공백 때문에 로고가 밖으로 밀리는 원 결함은
  재현되지 않았다.
- focused test: `issue_6173_right_align_space_before_inline_object` 1 pass
- #6259 추가 뒤 재검증: 1 pass
- 통합 head 공통 검증: fmt, unit tier, suite manifest, clippy, 전체 nextest, Native Skia 3종,
  WASM build 통과.

## 코멘트 처리

merge 후 원 PR/issue 코멘트에는 `p2_textbox_after.png` 기준으로 글상자 안 로고가 우단 안에
배치된다는 직접 확인 결과와 focused test 통과를 적는다. `pr_6275_issue6173_visual_review_p2.png`를
merge commit SHA 고정 raw URL로 표시하고, "내용 픽셀 중심 자동 일치율 보조값 = 약 45.36%" 및
자동값의 한계를 함께 둔다.

## 후속

추가 보정 필요 없음.
