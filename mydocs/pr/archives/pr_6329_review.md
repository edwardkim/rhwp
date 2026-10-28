---
kind: pr-review
status: accepted
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-29 KST
pr: 6329
issue: 6323
author: kevin9327
---

# PR #6329 review - HWPX optional-page replacement symmetry

## 라우팅

- Original PR: https://github.com/edwardkim/rhwp/pull/6329
- Author: `kevin9327`
- Reviewer request: `jangster77` registered by REST API
- Source head: `28d68650f2053715de4fb0a8629f9a3d1bd7a4c8`
- Review branch: `review/kevin9327-nondocs-20260829`
- Applied commits: `2165ef6be`, `67798bd84`

## 검토 판단

**수용 권고.** parser가 `OPTIONAL_PAGE pageDuplicate="0"`을 replacement master page로 해석하고,
serializer도 `LAST_PAGE` 여부가 아니라 동일한 상태를 왕복한다. 관측된 쪽번호 겹침을 제거하면서
parse/serialize 대칭도 맞춘다.

## 증적과 검증

- Focused: `issue_6323_optional_page_master_replaces_base` 2 pass, 133 skipped.
- Full local nextest: 8576 passed, 43 skipped.
- Visual evidence:
  - `mydocs/report/assets/issue_6323/before.png`: page numbers 2 and 4 overlap.
  - `mydocs/report/assets/issue_6323/after.png`: only page number 4 remains.
- Evidence ledger: `mydocs/pr/assets/pr_6317_6320_6322_6329_6338_6339_6341_6345_6347_6352_validation_20260829.md`

## 코멘트 처리

merge 후 원 PR에는 쪽번호 before/after PNG와 parser/serializer focused 회귀를 근거로 남긴다.
