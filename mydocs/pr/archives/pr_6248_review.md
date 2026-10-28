---
kind: pr-review
status: accepted
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-28
---

# PR #6248 review - #6179 오른쪽 탭 뒤 TAC 개체 정렬

## 라우팅

- 원 PR: https://github.com/edwardkim/rhwp/pull/6248
- 작성자: `planet6897`
- 원 PR head: `d84f1e8a4fe1`
- 통합 검토 브랜치: `review/open-ci-green-20260828`
- 최신 기준: `upstream/devel@1a43a507c9da`
- 원 PR 상태: non-draft, 실패·진행 check 0건
- 관련 이슈: #6179

## 검토 판단

**수용 권고.** `autoTabRight` 오른쪽 탭 뒤의 자리차지 그림을 텍스트 폭 계산에서 누락해, 그림의
왼쪽 변이 우단에 놓이고 오른쪽 변이 용지 밖으로 잘리던 문제를 고친다. 탭 뒤 TAC 개체 폭을
`right_tab_block_width_override`에 포함시킨 방향이 이슈의 지문과 맞다.

## 증적과 검증

- 원 PR 시각 보고서:
  `mydocs/report/right-tab-tac-object-6179/{p1_footer_before,p1_footer_after}.png`
- 통합 head 기준 MCP/visual sweep:
  - `rhwp info --json`: `mydocs/pr/assets/pr_6275_issue6179_info.json`
  - 저장 제품: `hancom-office-2018 10.0.0.9139` -> MCP `engine 2020`
  - 기준 PDF: `pdf/pr_6275/by_saved_version/pr6275_issue6179_right_tab_footer_logo-2020.pdf`
    (1 page)
  - visual sweep: `pr6275-issue6179-p1`, p1, flagged 0, pixel match `99.18781%`,
    visual accuracy proxy `28.21174%`
  - 장기 asset:
    `mydocs/pr/assets/pr_6275_issue6179_visual_review_p1.png`,
    `mydocs/pr/assets/pr_6275_issue6179_visual_sweep_summary.json`,
    `mydocs/pr/assets/pr_6275_issue6179_visual_overlay_metrics.json`
- 검토자가 직접 확인한 대표 after: 오른쪽 꼬리말 로고 오른쪽 변이 본문 우단 안에 들어오며 잘림이
  보이지 않는다. 통합 head visual sweep review PNG에서도 오른쪽 탭 뒤 TAC 로고가 페이지 밖으로
  밀려 잘리는 원 결함은 재현되지 않았다.
- focused test: `issue_6179_right_tab_tac_object_alignment` 1 pass
- #6259 추가 뒤 재검증: 1 pass
- 통합 head 공통 검증: fmt, unit tier, suite manifest, clippy, 전체 nextest, Native Skia 3종,
  WASM build 통과.

## 코멘트 처리

merge 후 원 PR/issue 코멘트에는 오른쪽 꼬리말 로고가 용지 밖으로 나가지 않는 대표 after 확인
결과를 적는다. `pr_6275_issue6179_visual_review_p1.png`를 merge commit SHA 고정 raw URL로 표시하고,
"내용 픽셀 중심 자동 일치율 보조값 = 약 28.21%" 및 자동값의 한계를 함께 둔다.

## 후속

추가 보정 필요 없음.
