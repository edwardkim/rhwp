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
- 원 PR 상태: non-draft, `CLEAN`, 실패·진행 check 0건
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
- 검토자가 직접 확인한 대표 after: 머리 표가 한컴 2020 oracle과 유사한 높이로 내려오고, 아래
  `보도 일시` 표와 겹치지 않는다.
- focused test: `issue_6194_header_row_picture_height` 1 pass
- 메인터너 보정 후 focused 재검증: 1 pass
- 통합 head 공통 검증: fmt, unit tier, suite manifest, focused regressions, clippy, 전체 nextest,
  Native Skia 3종, WASM build 통과. 상세 숫자는 통합 구현 문서에 기록했다.

## 코멘트 처리

merge 후 원 PR/issue 코멘트에는 메인터너 보정 사유와 대표 시각 증적을 함께 적는다. 추가 증적
산출이 필요하면 `mydocs/manual/verification/visual_sweep_guide.md#github-merge-comment` 기준으로
`scripts/visual_sweep.py`를 실행하고, 대표 `review_*.png`와 summary를 `mydocs/pr/assets` 아래에
보존한 뒤 merge commit SHA 고정 raw URL을 Markdown image로 사용한다. 이번 검토에서 직접 확인한
대표 증적은 `after_p1.png`와 `oracle_p1.png`다.

## 후속

통합 PR 본문에는 #6245 자체 수용과 별도로, `any()` 기반 후속 문단 탐색을 첫 후속 lineSeg로 좁힌
메인터너 보정 사유를 명시한다.
