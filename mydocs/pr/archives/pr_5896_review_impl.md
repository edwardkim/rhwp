---
kind: pr-review-implementation
status: trailing-docs-pending-ci
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-22
---

# PR #5896 메인터너 보정 구현 기록

## 시작점과 대상

- base: `upstream/devel` `72674c5653f09cb78b994dc4cd2dfd0a97ae6c8a`
- contributor source head: `ac7c82d1f8bad2422b29a72bbbf59123ae336249`
- 메인터너 보정 commit: `3368a7336`
- 관련 issue: #5891

원 PR의 `instance_id`는 `0x4400_0000`과 `(section_idx << 20)`을 OR했다. 상수의 bit 26이
이미 켜져 있으므로 section 0과 64의 첫 수식이 모두 `0x44000001`이 되는 차단 충돌을 확인했다.

## 보정 내용

1. `src/document_core/commands/object_ops/equation.rs`에서 문서 전체 기존 수식 수를 세고,
   `0x4400_0000 | sequence` 형식으로 ID를 배정했다. sequence는 겹치지 않는 하위 26비트이며
   1부터 `0x03ff_ffff`까지만 허용한다.
2. 같은 파일에서 구역 내 수식 순서로 계산하던 `z_order`는 변경하지 않았다.
3. `tests/cases/insert_equation_contract.rs`에 65개 구역을 사용하는 공개 API 회귀시험을 추가해
   구역 0/64 ID 충돌과 구역별 z-order를 함께 고정했다. source `#[cfg(test)]` 수를 늘리지 않는
   unit-tier 정책을 따른 위치다.

## 로컬 검증

- focused integration: `5 passed, 119 skipped`
- full nextest: `8161 passed, 4 slow, 39 skipped` (190.204s)
- `cargo clippy --locked --all-targets --target-dir target/pr-review -- -D warnings`
- `cargo fmt --all -- --check`
- `git diff --check`
- test manifest prepare/check 및 unit-tier check

모든 Cargo 명령은 `--locked`를 사용했다. review 중 최초 one-off Cargo 호출이 만든
`Cargo.lock` 순서-only 변경은 되돌려 최종 diff에 남기지 않았다.

## 다음 절차와 복구

code/test 보정과 첫 기록 commit은 source PR head에 fast-forward push했고, code candidate의 Full CI와
CodeQL을 확인했다. 현재 trailing docs-only commit은 그 실측 결과를 고정하는 단계이며, fast-pass와
최신 `CLEAN` 상태를 확인한 뒤 merge한다. 보정이 수용되지 않으면 maintainer가 추가한 commit만 revert하고
contributor source commit은 그대로 보존한다.
