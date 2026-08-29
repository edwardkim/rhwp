---
kind: pr-review
status: accepted
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-29 KST
pr: 6352
author: kevin9327
---

# PR #6352 review - layout-anomaly page filter envelope

## 라우팅

- Original PR: https://github.com/edwardkim/rhwp/pull/6352
- Author: `kevin9327`
- Reviewer request: `jangster77` registered by REST API
- Source head: `892a8422ce541157e7d0889187b2db39858dbe76`
- Review branch: `review/kevin9327-nondocs-20260829`
- Cherry-pick result: `e8c883202`

## 검토 판단

**수용 권고.** `layout-anomaly -p` filter가 output count, `hasSignal`, strict-mode exit behavior에
일관되게 적용된다. page-scoped clean 결과가 다른 쪽의 anomaly를 상속하지 않으므로 CLI 자동화에
더 안전하다.

## 증적과 검증

- Focused: `layout_anomaly_contract` 14 pass, 122 skipped.
- Full local nextest: 8576 passed, 43 skipped.
- Evidence ledger: `mydocs/pr/assets/pr_6317_6320_6322_6329_6338_6339_6341_6345_6347_6352_validation_20260829.md`

## 코멘트 처리

merge 후 원 PR에는 page filter가 JSON/count/exit-code contract 수준에서 검증됐다고 남긴다.
