---
kind: pr-review
status: accepted
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-29 KST
pr: 6338
issue: 6337
author: kevin9327
---

# PR #6338 review - oracle page-count baseline gate

## 라우팅

- Original PR: https://github.com/edwardkim/rhwp/pull/6338
- Author: `kevin9327`
- Reviewer request: `jangster77` registered by REST API
- Source head: `940f63d94430d5e95cb8bbf8835a93b0da85ea7d`
- Review branch: `review/kevin9327-nondocs-20260829`
- Applied commits: `2cef770e2`, `5c2b9b62d`

## 검토 판단

**수용 권고.** 저장소 PDF oracle 쪽수를 samples 전수 회귀 게이트로 만들고, 같은 stem이 여러
디렉터리에 있을 때 같은 디렉터리 PDF를 우선하는 짝짓기 보정도 포함한다.

## 증적과 검증

- Focused: `oracle_page_count_baseline` 1 pass, 134 skipped.
- Full local nextest: 8576 passed, 43 skipped.
- Pairing helper check: 566 paired documents, 96 narrowed by directory.
- Representative pair results:
  - `samples/basic/sungeo.hwp` -> `pdf/basic/sungeo-2022.pdf`
  - `samples/KTX.hwp` -> `pdf/KTX-2022.pdf`
  - `samples/basic/KTX.hwp` -> `pdf/basic/KTX-2022.pdf`
- Evidence ledger: `mydocs/pr/assets/pr_6317_6320_6322_6329_6338_6339_6341_6345_6347_6352_validation_20260829.md`

## 코멘트 처리

merge 후 원 PR에는 같은 디렉터리 우선 짝짓기 보정을 `KTX` 충돌 예제로 검증했다는 점을 남긴다.
