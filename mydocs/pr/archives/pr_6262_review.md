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
- 최신 기준: `upstream/devel@5645e1f5b`
- 원 PR 상태: non-draft, `CLEAN`, 실패·진행 check 0건
- 관련 이슈: #6190

## 검토 판단

**수용 권고.** 저장 `lineSeg`에 `TAG_INDENTATION`이 꺼진 경우에도 문단 들여쓰기를 다시 얹어 첫 줄이
불필요하게 밀리는 문제를 보정한다. 저장 lineSeg의 flag 의미를 존중하는 방향이다.

## 증적과 검증

- 원 PR 시각 보고서:
  `mydocs/report/stored-lineseg-indentation-6190/{p3_before,p3_after}.png`
- 검토자가 직접 확인한 대표 after: `TAG_INDENTATION`이 꺼진 줄에 추가 들여쓰기가 얹히지 않는다.
- focused test: `issue_6190_stored_lineseg_without_indentation_flag_keeps_box` 1 pass
- 통합 head 공통 검증: fmt, unit tier, suite manifest, focused regressions, clippy, 전체 nextest,
  Native Skia 3종, WASM build 통과.

## 코멘트 처리

merge 후 원 PR/issue 코멘트에는 `p3_after.png` 확인과 focused test 통과를 적는다. 추가 visual sweep이
필요하면 정본 절차에 따라 대표 PNG와 summary를 `mydocs/pr/assets`에 보존하고 merge SHA raw URL로
댓글에 표시한다.

## 후속

추가 보정 필요 없음.
