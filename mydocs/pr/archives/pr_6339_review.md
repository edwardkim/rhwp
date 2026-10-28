---
kind: pr-review
status: accepted
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-29 KST
pr: 6339
issue: 6334
author: kevin9327
---

# PR #6339 review - HWP5 extension master-page replacement

## 라우팅

- Original PR: https://github.com/edwardkim/rhwp/pull/6339
- Author: `kevin9327`
- Reviewer request: `jangster77` registered by REST API
- Source head: `240bf9adb5101a7ea80c265fec78a2e86d7ff4fc`
- Review branch: `review/kevin9327-nondocs-20260829`
- Applied commits: `97d058597`, `b42082e9a`

## 검토 판단

**수용 권고, 잔여 위험 기록.** HWP5는 HWPX의 `pageDuplicate` 같은 명시 discriminator를
노출하지 않지만, 제출된 oracle 증적에서는 실패 샘플의 extension master page가 base master page를
대체한다. 구현도 HWP5 extension master page에만 좁게 적용된다.

## 증적과 검증

- Focused: `issue_6334_hwp5_extension_master_replaces_base` 2 pass, 128 skipped.
- Full local nextest: 8576 passed, 43 skipped.
- The follow-up rustfmt-only commit was included to keep CI formatting clean.
- Evidence ledger: `mydocs/pr/assets/pr_6317_6320_6322_6329_6338_6339_6341_6345_6347_6352_validation_20260829.md`

## 코멘트 처리

merge 후 원 PR에는 현재 HWP5 oracle 동작을 근거로 수용하며, 향후 base+extension 의도적 overlay를
증명하는 HWP5 샘플이 나오면 별도 예외로 다뤄야 한다고 남긴다.
