---
kind: pr-review
status: accepted
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-28
---

# PR #6265 review - #6192 셀 안 앞/뒤 그림의 host 문단 앵커 보정

## 라우팅

- 원 PR: https://github.com/edwardkim/rhwp/pull/6265
- 작성자: `planet6897`
- 원 PR head: `17fa0f782d0`
- 통합 검토 브랜치: `review/open-ci-green-20260828`
- 최신 기준: `upstream/devel@1a43a507c9da`
- 원 PR 상태: non-draft, 실패·진행 check 0건
- 관련 이슈: #6192

## 검토 판단

**수용 권고.** 표 셀 내부의 앞/뒤 그림을 페이지/문단 밖 기준으로 잡아 말풍선·아이콘이 셀 내용과
어긋나던 문제를 host 문단 흐름 기준 앵커로 고친다. 표 셀 내부 overlay 그림이라는 이슈 지문과
수정 방향이 일치한다.

## 증적과 검증

- 원 PR 시각 보고서:
  `mydocs/report/cell-overlay-para-anchor-6192/{p4_before,p4_after}.png`
- 통합 head 기준 MCP/visual sweep:
  - `rhwp info --json`: `mydocs/pr/assets/pr_6275_issue6192_info.json`
  - 저장 제품: `hancom-office-2020 11.0.0.7571` -> MCP `engine 2020`
  - 기준 PDF: `pdf/pr_6275/by_saved_version/pr6275_issue6192_cell_behind_text_para_anchor-2020.pdf`
    (2 pages)
  - visual sweep: `pr6275-issue6192-p2`, slice p2(원 PR 설명의 p4 축약 샘플), flagged 0,
    pixel match `99.31622%`, visual accuracy proxy `35.97606%`
  - 장기 asset:
    `mydocs/pr/assets/pr_6275_issue6192_visual_review_p2.png`,
    `mydocs/pr/assets/pr_6275_issue6192_visual_sweep_summary.json`,
    `mydocs/pr/assets/pr_6275_issue6192_visual_overlay_metrics.json`
- 검토자가 직접 확인한 대표 after: 셀 내부 말풍선·아이콘이 host 문단 흐름에 붙어 배치된다.
  통합 head visual sweep review PNG에서도 overlay 그림이 설명 줄 위로 올라가 덮는 원 결함은
  재현되지 않았다.
- focused test: `issue_6192_cell_overlay_picture_anchors_to_host_paragraph` 1 pass
- 통합 head 공통 검증: fmt, unit tier, suite manifest, focused regressions, clippy, 전체 nextest,
  Native Skia 3종, WASM build 통과.

## 코멘트 처리

merge 후 원 PR/issue 코멘트에는 `p4_before.png`/`p4_after.png` 비교와 focused test 통과를 적는다.
`pr_6275_issue6192_visual_review_p2.png`를 merge commit SHA 고정 raw URL로 표시하고,
"내용 픽셀 중심 자동 일치율 보조값 = 약 35.98%" 및 자동값의 한계를 함께 둔다.

## 후속

추가 보정 필요 없음.
