---
kind: pr-review
status: accepted
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-29 KST
pr: 6320
issue: 6307
author: kevin9327
---

# PR #6320 review - landscape splittable row absorb guard

## 라우팅

- Original PR: https://github.com/edwardkim/rhwp/pull/6320
- Author: `kevin9327`
- Reviewer request: `jangster77` registered by REST API
- Source head: `47e9cf128a59a582bf5fdf84971b78e5f4629f45`
- Review branch: `review/kevin9327-nondocs-20260829`
- Cherry-pick result: `f8621c22a`

## 검토 판단

**수용 권고.** 저장 사다리가 분할을 선언하지 않은 landscape 경계 행에서, 분할 가능한 행을
흡수해 본문 하단 침범을 숨기는 경로를 좁은 조건으로 막는다. focused 회귀가 원 문제를 고정하고,
before/after 이미지도 PR 설명과 일치한다.

## 증적과 검증

- Focused: `issue_6307_landscape_splittable_row_absorb` 1 pass, 132 skipped.
- Full local nextest: 8576 passed, 43 skipped.
- Visual evidence:
  - `mydocs/report/landscape-splittable-absorb-6307/before_p11.png`: bottom row text reaches the Hancom logo/master-page band.
  - `mydocs/report/landscape-splittable-absorb-6307/after_p11.png`: bottom row no longer intrudes into the logo band.
- Evidence ledger: `mydocs/pr/assets/pr_6317_6320_6322_6329_6338_6339_6341_6345_6347_6352_validation_20260829.md`

## 코멘트 처리

merge 후 원 PR에는 p11 before/after PNG와 focused 회귀 통과를 수용 근거로 남긴다.
