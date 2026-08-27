---
kind: pr-review
status: accepted
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-27
---

# PR #6198 review - #6085 mid-paragraph vpos rewind pin

## 라우팅

- 원 PR: https://github.com/edwardkim/rhwp/pull/6198
- 작성자: `planet6897`
- 원 PR head: `82a82d154b899fe4b24efd6a170451daab13ea5b`
- 통합 검토 브랜치: `review/open-prs-6178-6198-20260827`
- 기준: `upstream/devel@529ab90c25d5`
- 적용 문서: `maintainer_general`, `intake_and_review`, `local_validation`,
  `multi_pr_update_branch`

## 검토 판단

**수용 가능**. 문단 중간의 저장 `vpos` 되감김이 쪽 경계로 오인되는 회귀를 고정하는 테스트
핀이다. 코드 변경 없이 regression case를 추가하는 성격이며, 통합 브랜치 전체 회귀에서 통과했다.

## 증적과 검증

- focused: `node scripts/run-rust-test.mjs issue_6085_midpara_vpos_rewind_page_break -- --cargo-profile release-test --target-dir target/pr-review`
  - `1 passed`, `139 skipped`
- 전체 회귀: `8438 passed`, `43 skipped`, `10 slow`
- manifest/tier:
  - `node scripts/rust-unit-test-tiers.mjs --check` pass
  - `node scripts/rust-test-suite-manifest.mjs --prepare && node scripts/rust-test-suite-manifest.mjs --check` pass

## 후속

병합 후 원 PR에는 test-only pin으로 통합됐고 전체 회귀에서 통과했음을 남긴다.
