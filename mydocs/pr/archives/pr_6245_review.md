---
kind: pr-review
status: accepted-with-maintainer-correction
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-28
---

# PR #6245 review - #6194 머리 표 행 높이 과대 계상 보정

## 라우팅

- 원 PR: https://github.com/edwardkim/rhwp/pull/6245
- 작성자: `planet6897`
- 원 PR head: `9c53276c37c8`
- 통합 검토 브랜치: `review/open-ci-green-20260828`
- 최신 기준: `upstream/devel@1a43a507c9da`
- 원 PR 상태: non-draft, 실패·진행 check 0건
- 관련 이슈: #6194

## 검토 판단

**수용 권고.** 머리 표 안의 TopAndBottom 그림 높이가 다음 문단 `lineseg.vertpos`에 이미 흡수된
경우를 저장 사다리 신뢰 증거로 인정해, 행 높이를 한컴 기준 선언 높이 근처로 되돌린다.

다만 원 구현의 `ladder_pushed_following_line`은 뒤쪽 모든 문단을 `any()`로 훑어, 여러 문단 뒤 누적
`vpos`를 개체 흡수 증거로 오인할 수 있었다. 통합 브랜치에서 메인터너 보정으로 **바로 뒤의 실제
lineSeg 1개만** 확인하도록 좁혔다.

## 증적과 검증

- 원 PR 시각 보고서:
  `mydocs/report/header-row-picture-height-6194/{before_p1,after_p1,oracle_p1}.png`
- 통합 head 기준 MCP/visual sweep:
  - `rhwp info --json`: `mydocs/pr/assets/pr_6275_issue6194_info.json`
  - 저장 제품: `hancom-office-2018 10.0.0.11529` -> MCP `engine 2020`
  - 기준 PDF: `pdf/pr_6275/by_saved_version/pr6275_issue6194_agri_press_release-2020.pdf`
    (2 pages)
  - visual sweep: `pr6275-issue6194-p1`, p1, flagged 0, pixel match `90.73707%`,
    visual accuracy proxy `14.76016%`
  - 장기 asset:
    `mydocs/pr/assets/pr_6275_issue6194_visual_review_p1.png`,
    `mydocs/pr/assets/pr_6275_issue6194_visual_sweep_summary.json`,
    `mydocs/pr/assets/pr_6275_issue6194_visual_overlay_metrics.json`
- 검토자가 직접 확인한 대표 after: 머리 표가 한컴 2020 oracle과 유사한 높이로 내려오고, 아래
  `보도 일시` 표와 겹치지 않는다. 통합 head visual sweep review PNG에서도 머리 표와 다음 표의
  관통 겹침은 재현되지 않았다.
- focused test: `issue_6194_header_row_picture_height` 1 pass
- 메인터너 보정 후 focused 재검증: 1 pass
- 통합 head 공통 검증: fmt, unit tier, suite manifest, focused regressions, clippy, 전체 nextest,
  Native Skia 3종, WASM build 통과. 상세 숫자는 통합 구현 문서에 기록했다.

## 코멘트 처리

merge 후 원 PR/issue 코멘트에는 메인터너 보정 사유와 대표 시각 증적을 함께 적는다. comment에는
`mydocs/manual/verification/visual_sweep_guide.md#github-merge-comment`를 정본으로 링크하고,
`pr_6275_issue6194_visual_review_p1.png`를 merge commit SHA 고정 raw URL로 표시한다.
수치 문구는 "내용 픽셀 중심 자동 일치율 보조값 = 약 14.76%"로 적고, 자동값이 사람 판정을
대체하지 않는다는 설명을 함께 둔다.

## 후속

통합 PR 본문에는 #6245 자체 수용과 별도로, `any()` 기반 후속 문단 탐색을 첫 후속 lineSeg로 좁힌
메인터너 보정 사유를 명시한다.
