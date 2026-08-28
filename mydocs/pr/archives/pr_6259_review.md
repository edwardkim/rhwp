---
kind: pr-review
status: accepted
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-28
---

# PR #6259 review - #6167 TAC 표 자기 줄 leading 제거

## 라우팅

- 원 PR: https://github.com/edwardkim/rhwp/pull/6259
- 작성자: `planet6897`
- 원 PR head: `87447c260737`
- 통합 검토 브랜치: `review/open-ci-green-20260828`
- 최신 기준: `upstream/devel@5645e1f5b`
- 원 PR 상태: non-draft, `CLEAN`, 실패·진행 check 0건
- 관련 이슈: #6167

## 검토 판단

**수용 권고.** 저장 `linesegarray`가 자리차지 표에 자기 줄을 이미 부여한 경우, 앞 줄 공백 폭을 표
좌표 leading에 다시 싣지 않도록 좁힌다. `text_start == ctrl_pos`와 `column_start == 0` 조건으로
종전 leading 보정이 필요한 통제군과 분리한 점이 적절하다.

## 증적과 검증

- 원 PR 시각 보고서:
  `mydocs/report/leading-space-tac-table-6167/{p38_table_before,p38_table_after}.png`
- 검토자가 직접 확인한 대표 after: before에서 오른쪽으로 밀려 용지 밖에 걸리던 표가 after에서 본문
  좌단 기준으로 배치되고 오른쪽 열이 잘리지 않는다.
- focused test: `issue_6167_leading_space_tac_table_own_line` 1 pass
- 통합 head 공통 검증: fmt, unit tier, suite manifest, clippy, 전체 nextest, Native Skia 3종,
  WASM build 통과.

## 코멘트 처리

merge 후 원 PR/issue 코멘트에는 `p38_table_after.png`에서 표 좌단과 우측 열 잘림이 보정된 직접
확인 결과를 적는다. 추가 증적 산출이 필요하면 visual sweep 정본 절차에 따라 대표 `review_*.png`와
summary를 `mydocs/pr/assets`에 보존하고, merge SHA 고정 raw URL을 댓글에 사용한다.

## 후속

추가 보정 필요 없음.
