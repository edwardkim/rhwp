---
kind: pr-review
status: accepted
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-28
---

# PR #6260 review - #6196 저장 단일 줄 셀의 과도한 자간 압축 억제

## 라우팅

- 원 PR: https://github.com/edwardkim/rhwp/pull/6260
- 작성자: `planet6897`
- 원 PR head: `b3c67ae91cfb`
- 통합 검토 브랜치: `review/open-ci-green-20260828`
- 최신 기준: `upstream/devel@5645e1f5b`
- 원 PR 상태: non-draft, `CLEAN`, 실패·진행 check 0건
- 관련 이슈: #6196

## 검토 판단

**수용 권고.** 저장 `lineSeg`가 단일 줄 셀에 이미 맞춰진 경우에도 다시 과도한 자간 압축을 적용해
글자가 셀 오른쪽 경계를 넘거나 잘리는 문제를 좁힌다. 판단은 #6196 fixture의 저장 단일 줄 셀 보정
범위로 제한한다.

## 증적과 검증

- 원 PR 시각 보고서:
  `mydocs/report/cell-overflow-spacing-6196/{p4_cell_before,p4_cell_after}.png`
- 검토자가 직접 확인한 대표 after: before에서 `우수 내용` 셀 오른쪽 경계를 넘던 내용이 after에서
  셀 내부에 들어온다.
- focused test: `issue_6196_stored_single_line_cell_compresses_to_fit` 1 pass
- 통합 head 공통 검증: fmt, unit tier, suite manifest, focused regressions, clippy, 전체 nextest,
  Native Skia 3종, WASM build 통과.

## 코멘트 처리

merge 후 원 PR/issue 코멘트에는 `p4_cell_before.png`/`p4_cell_after.png` 비교로 fixture-specific
개선임을 명확히 적고, 전체 corpus의 광범위 개선처럼 과장하지 않는다. 추가 visual sweep이 필요하면
대표 PNG와 summary를 `mydocs/pr/assets`에 보존하고 merge SHA raw URL로 댓글에 표시한다.

## 후속

추가 보정 필요 없음.
