---
kind: pr-review
status: accepted
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-29 KST
pr: 6322
issue: 6318
author: kevin9327
---

# PR #6322 review - outside-body text-overlap diagnostics

## 라우팅

- Original PR: https://github.com/edwardkim/rhwp/pull/6322
- Author: `kevin9327`
- Reviewer request: `jangster77` registered by REST API
- Source head: `f968ae5a90808e80e1050be9ac627e47208ea32e`
- Review branch: `review/kevin9327-nondocs-20260829`
- Applied after #6317: `e33eaeea1`, `a5735cbee`, `4f7a63496`

## 검토 판단

**수용 권고.** master-page, header, footer, footnote text run을 text-overlap 후보에 포함하되,
해당 컨테이너를 일반 본문 flow column처럼 취급하지 않는다. 사이드바나 본문 밖 텍스트가 실제
내용을 덮어도 0건으로 나오던 사각지대를 닫는다.

## 메인터너 보정

The PR was stacked on #6317, so duplicate commits were skipped. A conflict in `tests/cases/text_overlap_baseline.rs` was resolved by keeping the current `scan_document()` path and retaining the PR's improved failure output that prints the full current TSV when the ratchet fails.

## 증적과 검증

- Focused: `issue_6318_outside_body_text_overlap` 3 pass, 122 skipped.
- Focused: `text_overlap_baseline` 1 pass, 135 skipped.
- Full local nextest: 8576 passed, 43 skipped.
- Evidence ledger: `mydocs/pr/assets/pr_6317_6320_6322_6329_6338_6339_6341_6345_6347_6352_validation_20260829.md`

## 코멘트 처리

merge 후 원 PR에는 stacked PR의 중복 commit은 제외했고, 현재 baseline test 구현과 contributor의
outside-body scan 확장을 모두 보존한 통합 head로 수용했다는 점을 남긴다.
