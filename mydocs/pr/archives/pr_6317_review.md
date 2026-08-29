---
kind: pr-review
status: accepted
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-29 KST
pr: 6317
issue: 6315
author: kevin9327
---

# PR #6317 review - text-overlap samples ratchet

## 라우팅

- Original PR: https://github.com/edwardkim/rhwp/pull/6317
- Author: `kevin9327`
- Reviewer request: `jangster77` registered by REST API
- Source head: `0b3665a0ff1d1c9fed631449f87ba70b4e5d7af3`
- Review branch: `review/kevin9327-nondocs-20260829`
- Base: `upstream/devel@955abb5268c3a6a93a41328633729fd095b7390a`
- Applied commits: `ee46e2c35`, `259d42086`

## 검토 판단

**수용 권고.** 기존에 있던 text-overlap 판정기를 samples 전수 래칫으로 승격해, 이후 변경에서
겹침 후보 증가가 조용히 들어오지 못하게 한다. 원 PR은 현재 `devel` 기준 `DIRTY`였지만 통합
브랜치에서 충돌을 해소했고 로컬 검증도 통과했다.

## 메인터너 보정

`tests/cases/text_overlap_baseline.rs` had an add/add conflict because current `upstream/devel` already contains the maintainer-side `scan_document()` based gate. I kept that implementation and applied the PR's updated working note and baseline values. This preserves the current analyzer entrypoint while retaining the contributor's ratchet expansion.

## 증적과 검증

- Focused: `text_overlap_baseline` 1 pass, 135 skipped.
- Full local nextest: 8576 passed, 43 skipped.
- Manifest and unit-tier checks passed.
- Evidence ledger: `mydocs/pr/assets/pr_6317_6320_6322_6329_6338_6339_6341_6345_6347_6352_validation_20260829.md`

## 코멘트 처리

merge 후 원 PR에는 `text_overlap_baseline.rs` base drift를 통합 브랜치에서 해소했고, 현재
`scan_document()` 기반 게이트를 보존한 상태로 수용했다는 점을 남긴다.
