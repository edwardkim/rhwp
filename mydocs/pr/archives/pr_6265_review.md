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
- 원 PR 상태: non-draft, `CLEAN`, 실패·진행 check 0건
- 관련 이슈: #6192

## 검토 판단

**수용 권고.** 표 셀 내부의 앞/뒤 그림을 페이지/문단 밖 기준으로 잡아 말풍선·아이콘이 셀 내용과
어긋나던 문제를 host 문단 흐름 기준 앵커로 고친다. 표 셀 내부 overlay 그림이라는 이슈 지문과
수정 방향이 일치한다.

## 증적과 검증

- 원 PR 시각 보고서:
  `mydocs/report/cell-overlay-para-anchor-6192/{p4_before,p4_after}.png`
- 검토자가 직접 확인한 대표 after: 셀 내부 말풍선·아이콘이 host 문단 흐름에 붙어 배치된다.
- focused test: `issue_6192_cell_overlay_picture_anchors_to_host_paragraph` 1 pass
- 통합 head 공통 검증: fmt, unit tier, suite manifest, focused regressions, clippy, 전체 nextest,
  Native Skia 3종, WASM build 통과.

## 코멘트 처리

merge 후 원 PR/issue 코멘트에는 `p4_before.png`/`p4_after.png` 비교와 focused test 통과를 적는다.
추가 visual sweep 산출이 필요하면 대표 review PNG와 summary를 `mydocs/pr/assets`에 보존하고 merge SHA
raw URL을 사용한다.

## 후속

추가 보정 필요 없음.
