---
kind: pr-review
status: accepted
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-29 KST
pr: 6345
issue: 6344
author: kevin9327
---

# PR #6345 review - empty-page visible-content filter

## 라우팅

- Original PR: https://github.com/edwardkim/rhwp/pull/6345
- Author: `kevin9327`
- Reviewer request: `jangster77` registered by REST API
- Source head: `5d11dbb8d71da8407cd02c005cf1d20084d56e63`
- Review branch: `review/kevin9327-nondocs-20260829`
- Cherry-pick result: `476687ffe`

## 검토 판단

**수용 권고.** empty-page diagnostic이 빈 쪽을 보고하기 전에 page-level visible content를 확인해,
일반 body text는 없지만 표 같은 실제 내용이 있는 쪽을 빈 쪽으로 오판하지 않게 한다.

## 메인터너 보정

`src/diagnostics/layout_anomaly.rs` conflicted with #6322's outside-body text-overlap expansion. The resolution keeps both behaviors: outside-body text is collected for overlap detection, and `page_has_visible_content()` is used only to suppress empty-page false positives.

## 증적과 검증

- Focused: `issue_6344_empty_page_false_positive` 2 pass, 140 skipped.
- Full local nextest: 8576 passed, 43 skipped.
- Evidence ledger: `mydocs/pr/assets/pr_6317_6320_6322_6329_6338_6339_6341_6345_6347_6352_validation_20260829.md`

## 코멘트 처리

merge 후 원 PR에는 통합 과정에서 두 diagnostic 개선을 모두 보존했고 어느 PR의 동작도 누락하지
않았다고 남긴다.
