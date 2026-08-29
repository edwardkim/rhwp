---
kind: pr-review
status: accepted-maintainer-corrected
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-28 21:27 KST
pr: 6073
issue: 6063
author: kevin9327
---

# PR #6073 review - HWPX 문단 중간 vpos 되감김 본문 침범 분리

## 라우팅

- 원 PR: https://github.com/edwardkim/rhwp/pull/6073
- 작성자: `kevin9327`
- base: `devel`
- 원 head: `b90d80484249da3fbedb06355b92fc149d193e95`
- 통합 검토 브랜치: `review/kevin9327-6073-6083-20260828`
- 기준: `upstream/devel@a6c7e7bb3ae09470c225a4c90c0fc1ad88b6b5a6`
- 상태: non-draft, GitHub CI/CodeQL/Adapter inter-diff/Proptest roundtrip 모두 성공
- 원 PR 코멘트: `postmelee`의 재작업 요청 뒤 `jangster77`가 메인터너 보정 commit
  `b90d80484249`를 원 PR head에 push하고 코멘트 처리했다.

## 검토 판단

**수용 권고.** #6063 재현과 회귀 핀은 유효하다. 다만 원 PR 중간 상태는 `issue_1880`
한컴 오라클 핀을 완화하고 `keep_hwpx_internal_page_break_on_body_overflow`에 문서 실측 상수형 가드를
추가해 required check 적색과 일반화 위험을 만들었다. 메인터너 보정은 이 두 위험을 제거하고,
현재 `devel`에 이미 들어온 #6063 본문 바닥/6쪽 소유 계약을 회귀 증적으로 고정하는 형태로 정리했다.

따라서 통합은 원 PR head를 그대로 수용한 것이 아니라, 메인터너가 오라클 완화와 상수형 승격 가드를
제거한 head 기준으로 검토했다.

## 증적과 검증

- 대상 fixture: `samples/issue1880_anchor_stack_sb_convert.hwpx`
- `rhwp info --json`: `mydocs/pr/assets/pr_6073_issue6063_info.json`
  - `format=hwpx`
  - `lastSavedWith=hancom-office-2020 11.0.0.8362`
  - `pageCount=13`
- 원 PR before/after 이미지 보존:
  - `mydocs/pr/assets/pr_6073_issue6063_before.png`
  - `mydocs/pr/assets/pr_6073_issue6063_after.png`
  - `mydocs/pr/assets/pr_6073_issue6063_before_tail.png`
  - `mydocs/pr/assets/pr_6073_issue6063_after_tail.png`
- focused tests:
  - `issue_6063_hwpx_midpara_vpos_rewind_body_overflow`: 2 pass
  - `issue_3931`: 5 pass
- 통합 head 공통 검증:
  - `cargo fmt --all -- --check` 통과
  - `git diff --check` 통과
  - `node scripts/rust-test-suite-manifest.mjs --check` 통과
  - `node scripts/rust-unit-test-tiers.mjs --check` 통과
  - `cargo clippy --locked --all-targets --target-dir target/pr-review -- -D warnings` 통과
  - `cargo test --locked --doc --target-dir target/pr-review`: 8 pass / 3 ignored
  - 전체 `cargo nextest run --locked --cargo-profile release-test --target-dir target/pr-review --tests --test-threads 12 --no-fail-fast`:
    8551 pass / 43 skipped
  - Native Skia lib: 3946 pass / 13 ignored
  - Native Skia `issue_2225_missing_picture_placeholder`: 2 pass
  - Native Skia `render_p37_direct_pdf_export`: 4 pass
  - `CARGO_TARGET_DIR=target/pr-review scripts/wasm-pack-locked.sh --target web --out-dir pkg` 통과

## 코멘트 처리

원 PR에는 이미 메인터너 보정 사유와 검증 결과를
https://github.com/edwardkim/rhwp/pull/6073#issuecomment-5451532578 에 남겼다.
통합 PR 또는 merge 후 원 PR close 코멘트에는 다음을 반복한다.

- 오라클 핀 완화는 제거했고 `issue_1880` exact 계약을 유지했다.
- 문서 실측 상수 기반 승격 경로는 제품 코드에서 제거했다.
- #6063 회귀 증적과 tests만 남긴 head로 통합했다.
- 전체 nextest, clippy, Native Skia, WASM까지 현재 통합 head에서 재검증했다.

## 후속

추가 메인터너 보정 필요 없음.
