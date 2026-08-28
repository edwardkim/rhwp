---
kind: pr-review
status: accepted
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-28
---

# PR #6261 review - #6206 표 셀 안 쪽번호 재시작 수집

## 라우팅

- 원 PR: https://github.com/edwardkim/rhwp/pull/6261
- 작성자: `planet6897`
- 원 PR head: `91776e11acc3`
- 통합 검토 브랜치: `review/open-ci-green-20260828`
- 최신 기준: `upstream/devel@1a43a507c9da`
- 원 PR 상태: non-draft, `CLEAN`, 실패·진행 check 0건
- 관련 이슈: #6206

## 검토 판단

**수용 권고.** 표 셀 내부의 `새 번호로 시작` 컨트롤을 본문 순회에서 놓쳐, 셀 안 쪽번호가 이전
페이지 절대 번호를 유지하던 문제를 고친다. 컨트롤 수집 범위를 셀 문단까지 확장하는 방향이 이슈와
맞는다.

## 증적과 검증

- 원 PR 증적:
  `mydocs/report/assets/issue_6206/pagenum-113424-p7.png`,
  `mydocs/report/assets/issue_6206/pagenum-156555538-p2.png`
- 검토자가 직접 확인한 대표 증적: before는 `- 7 -` 또는 `- 2 -`로 남고, after/한컴 oracle은
  `- 1 -`로 재시작한다.
- 통합 head 공통 검증: fmt, unit tier, suite manifest, focused regressions, clippy, 전체 nextest,
  Native Skia 3종, WASM build 통과.

## 코멘트 처리

merge 후 원 PR/issue 코멘트에는 두 대표 PNG의 before/after/oracle 관계를 적고, 쪽번호 재시작이 표
셀 내부 컨트롤 수집으로 해결됐다고 설명한다. 추가 visual sweep 산출이 필요하면 대표 `review_*.png`와
summary를 `mydocs/pr/assets`에 보존한 뒤 merge commit SHA raw URL로 실제 이미지가 보이게 처리한다.

## 후속

추가 보정 필요 없음.
