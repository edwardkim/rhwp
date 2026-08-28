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
- 검토자가 직접 확인한 대표 after: 오른쪽 꼬리말 로고 오른쪽 변이 본문 우단 안에 들어오며 잘림이
  보이지 않는다.
- focused test: `issue_6179_right_tab_tac_object_alignment` 1 pass
- #6259 추가 뒤 재검증: 1 pass
- 통합 head 공통 검증: fmt, unit tier, suite manifest, clippy, 전체 nextest, Native Skia 3종,
  WASM build 통과.

## 코멘트 처리

merge 후 원 PR/issue 코멘트에는 오른쪽 꼬리말 로고가 용지 밖으로 나가지 않는 대표 after 확인
결과를 적는다. 추가 증적 산출이 필요하면 `visual_sweep_guide.md#github-merge-comment` 기준으로
visual sweep을 실행하고, 대표 `review_*.png`와 summary를 `mydocs/pr/assets`에 보존한 뒤 merge SHA
고정 raw URL로 댓글에 표시한다.

## 후속

추가 보정 필요 없음.
