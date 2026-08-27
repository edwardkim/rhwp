---
kind: pr-review
status: accepted
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-28
---

# PR #6254 review - #6173 오른쪽 정렬 말미 공백 판정

## 라우팅

- 원 PR: https://github.com/edwardkim/rhwp/pull/6254
- 작성자: `planet6897`
- 원 PR head: `00cac13820e3`
- 통합 검토 브랜치: `review/open-ci-green-20260828`
- 최신 기준: `upstream/devel@5645e1f5b`
- 원 PR 상태: non-draft, `CLEAN`, 실패·진행 check 0건
- 관련 이슈: #6173

## 검토 판단

**수용 권고.** 오른쪽 정렬 문단에서 마지막 인라인 개체 앞의 공백까지 말미 공백으로 제거해 로고가
우측으로 밀리던 문제를, 마지막 인라인 개체 뒤 공백만 제외하도록 고친다. `paragraph_layout`과
`shape_layout` 양쪽에 같은 계약을 적용한 점이 중요하다.

## 증적과 검증

- 원 PR 시각 보고서:
  `mydocs/report/right-align-inline-object-space-6173/{p2_textbox_before,p2_textbox_after}.png`
- 검토자가 직접 확인한 대표 after: 글상자 안 두 로고가 우단 안에 들어오며 잘림이 보이지 않는다.
- focused test: `issue_6173_right_align_space_before_inline_object` 1 pass
- #6259 추가 뒤 재검증: 1 pass
- 통합 head 공통 검증: fmt, unit tier, suite manifest, clippy, 전체 nextest, Native Skia 3종,
  WASM build 통과.

## 후속

추가 보정 필요 없음.
