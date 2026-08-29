---
kind: pr-review
status: accepted
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-29 KST
pr: 6341
issue: 6340
author: kevin9327
---

# PR #6341 review - off-canvas samples ratchet

## 라우팅

- Original PR: https://github.com/edwardkim/rhwp/pull/6341
- Author: `kevin9327`
- Reviewer request: `jangster77` registered by REST API
- Source head: `f7d34e5409ea880ea9c2290b49393cefcae27fa1`
- Review branch: `review/kevin9327-nondocs-20260829`
- Cherry-pick result: `4ea98063e`

## 검토 판단

**수용 권고.** 이미 존재하는 off-canvas diagnostic을 저장소 전수 래칫으로 고정해, 이후 layout
변경에서 용지 밖 그리기 후보가 조용히 증가하지 못하게 한다.

## 증적과 검증

- Focused: `off_canvas_baseline` 1 pass, 129 skipped.
- Full local nextest: 8576 passed, 43 skipped.
- Slow gate was expected for samples-wide scanning and completed successfully.
- Evidence ledger: `mydocs/pr/assets/pr_6317_6320_6322_6329_6338_6339_6341_6345_6347_6352_validation_20260829.md`

## 코멘트 처리

merge 후 원 PR에는 guardrail 성격의 PR이며 별도 메인터너 보정은 필요 없었다고 남긴다.
