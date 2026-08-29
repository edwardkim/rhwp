---
kind: pr-review
status: accepted-with-maintainer-correction
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-29 19:15 KST
pr: 6356
issue: 6355
author: lpaiu-cs
---

# PR #6356 review - picture transform raw rendering invalidation

## 라우팅

- 원 PR: https://github.com/edwardkim/rhwp/pull/6356
- 작성자: `lpaiu-cs`
- reviewer request: `jangster77` 등록 확인
- 원 PR head: `f5010c385e5b2e6e0aa1d4f98af0743522c0b0dc`
- 통합 검토 브랜치: `review/lpaiu-cs-20260829`
- 최신 기준: `upstream/devel@cf366d2faad63a57fb663ce38b2e02d99b873e22`
- 적용 commit: `c098dd1ad`
- 메인터너 보정 commit: `33c6dc4cf`
- 원 PR 상태: non-draft, `BLOCKED`
- 원 PR non-green check: `Lint (fmt, clippy, WASM check)`, `Build & Test`

## 검토 판단

**메인터너 보정 포함 수용 후보.** 그림 변환 키가 JSON 문자열에 등장했는지만 보는 방식은
무변경 setter에서도 한컴 원본 `raw_rendering`을 잃게 하므로, 실제 값 변화 지문으로 무효화를
판정하는 방향이 맞다. 다만 원 PR은 source-side `#[cfg(test)]` 증가로 CI lint가 실패했다.

## 메인터너 보정

source 내부에 추가된 그림 setter white-box test를
`tests/cases/issue_6355_picture_transform_raw_rendering.rs` integration regression으로 이동했다.
공개 API를 통해 같은 값 재적용, 문자열 내부 키, 비변환 속성, 실제 변환 변경의 네 경로를 고정한다.

## 증적과 검증

- `node scripts/rust-unit-test-tiers.mjs --check`: pass, `4221 tests / 299 modules`
- `issue_6355_picture_transform_raw_rendering`: 4 pass
- `cargo clippy --workspace --all-targets --target-dir target/pr-review -- -D warnings`: pass
- `cargo clippy -p rhwp --lib --target wasm32-unknown-unknown --target-dir target/pr-review -- -D warnings`: pass
- `cargo check --target wasm32-unknown-unknown --lib --target-dir target/pr-review`: pass
- 공통 검증과 head 증적:
  `mydocs/pr/assets/pr_6331_6333_6336_6351_6356_validation_20260829.md`

## 코멘트 처리

merge 후 원 PR에는 값 변화 기반 invalidation은 수용하되, source-side test 증가 때문에 동일 검증을
`tests/cases`로 옮긴 메인터너 보정이 포함됐다고 설명한다.
