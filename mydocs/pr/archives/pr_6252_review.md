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
- 원 PR 상태: non-draft, `CLEAN`, 실패·진행 check 0건
- 관련 이슈: #6174

## 검토 판단

**수용 권고.** 글상자 clip rect가 텍스트 descender와 하단 획을 과도하게 잘라 한글 하단이 손상되는
문제를 보정한다. clip 계산을 글줄 ink 범위에 맞춰 넓히는 방향이며, 대표 fixture에서 하단 획 잘림이
사라진다.

## 증적과 검증

- 원 PR 시각 보고서:
  `mydocs/report/textbox-clip-descender-6174/{before_p1,after_p1,oracle_p1}.png`
- 검토자가 직접 확인한 대표 after/oracle: `경찰청` 글자 하단 획이 잘리지 않고 oracle과 같은
  세로 범위 안에 남는다.
- focused test: `issue_6174_textbox_clip_contains_its_own_line` 1 pass
- 통합 head 공통 검증: fmt, unit tier, suite manifest, focused regressions, clippy, 전체 nextest,
  Native Skia 3종, WASM build 통과.

## 코멘트 처리

merge 후 원 PR/issue 코멘트에는 `after_p1.png`/`oracle_p1.png` 직접 확인 결과와 focused test 통과를
적는다. 추가 visual sweep 산출이 필요하면 `visual_sweep_guide.md#github-merge-comment` 기준으로
대표 `review_*.png`와 summary를 `mydocs/pr/assets`에 보존하고, merge SHA raw URL로 실제 이미지가
보이게 댓글 처리한다.

## 후속

추가 보정 필요 없음.
