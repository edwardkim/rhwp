---
kind: pr-review
status: accepted
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-28
---

# PR #6262 review - #6190 TAG_INDENTATION 없는 저장 lineSeg 들여쓰기 억제

## 라우팅

- 원 PR: https://github.com/edwardkim/rhwp/pull/6262
- 작성자: `planet6897`
- 원 PR head: `612d3e78e67`
- 통합 검토 브랜치: `review/open-ci-green-20260828`
- 최신 기준: `upstream/devel@1a43a507c9da`
- 원 PR 상태: non-draft, 실패·진행 check 0건
- 관련 이슈: #6190

## 검토 판단

**수용 권고.** 저장 `lineSeg`에 `TAG_INDENTATION`이 꺼진 경우에도 문단 들여쓰기를 다시 얹어 첫 줄이
불필요하게 밀리는 문제를 보정한다. 저장 lineSeg의 flag 의미를 존중하는 방향이다.

## 증적과 검증

- 원 PR 시각 보고서:
  `mydocs/report/stored-lineseg-indentation-6190/{p3_before,p3_after}.png`
- 통합 head 기준 MCP/visual sweep:
  - `rhwp info --json`: `mydocs/pr/assets/pr_6275_issue6190_info.json`
  - 저장 제품: `hancom-office-2020 11.0.0.2129` -> MCP `engine 2020`
  - 기준 PDF: `pdf/pr_6275/by_saved_version/pr6275_issue6190_center_align_first_line_indent-2020.pdf`
    (1 page)
  - visual sweep: `pr6275-issue6190-p1`, slice p1(원 PR 설명의 p3 축약 샘플), flagged 0,
    pixel match `96.25901%`, visual accuracy proxy `5.20078%`
  - 장기 asset:
    `mydocs/pr/assets/pr_6275_issue6190_visual_review_p1.png`,
    `mydocs/pr/assets/pr_6275_issue6190_visual_sweep_summary.json`,
    `mydocs/pr/assets/pr_6275_issue6190_visual_overlay_metrics.json`
- 검토자가 직접 확인한 대표 after: `TAG_INDENTATION`이 꺼진 줄에 추가 들여쓰기가 얹히지 않는다.
  통합 head visual sweep review PNG에서도 slice의 표와 문단이 본문 영역 밖으로 밀리는 원 결함은
  재현되지 않았다. 자동 일치율은 낮지만, 이는 전체 글꼴/라스터 차이를 포함한 보조값이며 원 PR이
  주장한 들여쓰기 축과 별도로 해석한다.
- focused test: `issue_6190_stored_lineseg_without_indentation_flag_keeps_box` 1 pass
- 통합 head 공통 검증: fmt, unit tier, suite manifest, focused regressions, clippy, 전체 nextest,
  Native Skia 3종, WASM build 통과.

## 코멘트 처리

merge 후 원 PR/issue 코멘트에는 `p3_after.png` 확인과 focused test 통과를 적는다.
`pr_6275_issue6190_visual_review_p1.png`를 merge commit SHA 고정 raw URL로 표시하고,
"내용 픽셀 중심 자동 일치율 보조값 = 약 5.20%"는 낮은 보조값일 뿐 들여쓰기 원 결함 판정과
분리해서 설명한다.

## 후속

추가 보정 필요 없음.
