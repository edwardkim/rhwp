---
kind: pr-review
status: accepted-with-maintainer-correction
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-29 19:15 KST
pr: 6351
issue: 6350
author: lpaiu-cs
---

# PR #6351 review - equation setter raw passthrough

## 라우팅

- 원 PR: https://github.com/edwardkim/rhwp/pull/6351
- 작성자: `lpaiu-cs`
- reviewer request: `jangster77` 등록 확인
- 원 PR head: `c0a00cdd3c59f3ebc1043edebed51845fea58543`
- 통합 검토 브랜치: `review/lpaiu-cs-20260829`
- 최신 기준: `upstream/devel@cf366d2faad63a57fb663ce38b2e02d99b873e22`
- 적용 commit: `2847442ec`
- 메인터너 보정 commit: `33c6dc4cf`
- 원 PR 상태: non-draft, `BLOCKED`
- 원 PR non-green check: `Lint (fmt, clippy, WASM check)`, `Build & Test`

## 검토 판단

**메인터너 보정 포함 수용 후보.** 기능 방향은 맞다. 봉인된 수식 raw CTRL_HEADER는 setter가
무조건 파괴하지 않고, 명시 크기는 자동 크기 파생으로 덮지 않아야 한다. 다만 원 PR은 신규
source-side `#[cfg(test)]` 증가로 tier 정책에 걸려 CI lint가 실패했다.

## 메인터너 보정

source 내부에 추가된 white-box test를 `tests/cases/issue_6350_equation_setter_raw_passthrough.rs`
integration regression으로 이동했다. 기존 source-side 단위 테스트는 봉인 없는 raw clear 계약 1건만
유지해 source-side test 총량 증가를 없앴다.

## 증적과 검증

- `node scripts/rust-unit-test-tiers.mjs --check`: pass, `4221 tests / 299 modules`
- `issue_6350_equation_setter_raw_passthrough`: 4 pass
- `cargo clippy --workspace --all-targets --target-dir target/pr-review -- -D warnings`: pass
- `cargo clippy -p rhwp --lib --target wasm32-unknown-unknown --target-dir target/pr-review -- -D warnings`: pass
- `cargo check --target wasm32-unknown-unknown --lib --target-dir target/pr-review`: pass
- 공통 검증과 head 증적:
  `mydocs/pr/assets/pr_6331_6333_6336_6351_6356_validation_20260829.md`

## 코멘트 처리

merge 후 원 PR에는 기능 변경은 통합했지만, 저장소 정책상 source-side test 증가가 lint를 깨서
동일 회귀를 `tests/cases` integration test로 이동했다는 점을 남긴다.
