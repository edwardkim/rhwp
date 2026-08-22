---
kind: pr-review
status: trailing-docs-pending-ci
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-22
---

# PR #5896 검토 - 삽입 수식 한컴 호환 메타데이터

## 접수 메타데이터

| 항목 | 작성 시점 확인값 |
| --- | --- |
| PR / 작성자 | [#5896](https://github.com/edwardkim/rhwp/pull/5896) / [@Shadungi](https://github.com/Shadungi) |
| 관련 issue | [#5891](https://github.com/edwardkim/rhwp/issues/5891) |
| base / source head | `devel` `72674c5653f09cb78b994dc4cd2dfd0a97ae6c8a` / `ac7c82d1f8bad2422b29a72bbbf59123ae336249` |
| 변경 규모 | 2 files, +65 / -2 |
| 작성 시점 상태 | non-draft, `MERGEABLE`, maintainerCanModify=true, `FIRST_TIME_CONTRIBUTOR` |
| reviewer | `@jangster77` 요청 완료 |
| 로컬 검토 branch | `review/shadungi-20260822`, source head `ac7c82d1f8bad2422b29a72bbbf59123ae336249` 위 maintainer 보정 `3368a7336` |

GitHub 상태값은 작성 시점 참고값이며, merge 전 최신 source head와 required check를 다시 확인한다.

## 범위와 시각 검증 판정

- `insert_equation_native`가 새 `Equation`의 HWP5 공통 개체 속성, 한컴 수식 메타데이터,
  결정적 `instance_id` 및 `z_order`를 설정하도록 변경한다.
- `insert_equation_contract`가 저장 후 재열기 경로에서 해당 메타데이터를 확인한다.
- renderer, typeset, layout, visual fixture, 기준 PDF는 바뀌지 않는다. 본 PR은 저장 메타데이터
  계약만 변경하므로 이번 검토에서는 visual sweep을 요구하지 않았다.

## 원 PR 검증

- 최신 `upstream/devel` 위 source head를 기준으로 검토했고, maintainer 보정 commit을 source head
  위에 replay했다. `git merge-tree --write-tree upstream/devel HEAD`와
  `git diff --check upstream/devel...HEAD`도 충돌·공백 오류 없이 통과했다.
- `cargo fmt --all -- --check`를 통과했다.
- `node scripts/rust-unit-test-tiers.mjs --check`를 통과했다.
- `node scripts/rust-test-suite-manifest.mjs --prepare` 및 `--check`를 통과했다.
- source head `ac7c82d1f8bad2422b29a72bbbf59123ae336249`의 GitHub Build & Test, Lint,
  archive build/shard, CodeQL Rust/JavaScript/Python, Adapter inter-diff, Proptest roundtrip은
  모두 성공했다. Native Skia, WASM Build, frontend gates는 변경 범위에 따른 skip이었다.

## 발견한 차단 결함

`src/document_core/commands/object_ops/equation.rs`의 `instance_id` 식은 구역 번호의 비트 6을
상수 `0x4400_0000`에 이미 켜진 비트 26으로 OR한다.

```rust
0x4400_0000 | (((section_idx as u32) & 0xff) << 20) | equation_number
```

따라서 첫 수식의 ID가 구역 0과 구역 64에서 모두 `0x4400_0001`이고, 구역 128과 192에서도
동일하다. 함수는 `section_idx < self.document.sections.len()`만 검사하고 구역 수 상한을 두지
않으므로, 다구역 문서에서 이 충돌이 실제로 가능하다. 이는 PR과 #5891이 요구하는 0이 아닌
결정적 고유 개체 ID 계약을 위반한다.

현재 회귀시험은 ID가 0이 아니며 최상위 비트 조건을 만족하는지만 검사하므로 이 충돌을 잡지
못한다. 보정 시에는 비트 영역이 겹치지 않는 ID 배정 규칙으로 바꾸고, 적어도 구역 0/64 및
두 개 이상 삽입한 수식의 ID·`z_order`가 서로 다른지를 검증하는 회귀시험을 추가해야 한다.

## 메인터너 보정

승인된 maintainer 보정은 구역 번호를 ID 비트에 OR하지 않고, 문서 전체의 기존 `Equation` 수에
기반한 1부터 `0x03ff_ffff`까지의 연속 순서를 `0x4400_0000`의 비중첩 하위 26비트에 배정한다.
따라서 한컴 계열 `0x44` 접두는 유지하면서 구역 0과 64의 충돌을 제거하고, 수식별 `z_order`는
기존처럼 각 구역의 수식 순서를 유지한다. 한도를 넘으면 wraparound 대신 명시적 오류를 반환한다.

회귀시험은 source `#[cfg(test)]` 모듈이 아니라 `tests/cases/insert_equation_contract.rs`에 추가했다.
이는 `local_validation.md`의 unit-tier 기준선을 늘리지 않고 공개 삽입 API 계약으로 검증하기 위함이다.
65개 구역 문서에서 구역 0, 구역 64, 다시 구역 0에 삽입해 ID `0x44000001`, `0x44000002`,
`0x44000003`과 구역별 `z_order` 0, 0, 1을 확인한다.

## 보정 로컬 검증

- `CARGO_TARGET_DIR=target/pr-review node scripts/run-rust-test.mjs insert_equation_contract -- --cargo-profile release-test --no-fail-fast`:
  `5 passed, 119 skipped`.
- `cargo nextest run --locked --cargo-profile release-test --target-dir target/pr-review --tests --test-threads 12 --no-fail-fast`:
  `8161 passed, 4 slow, 39 skipped` (190.204s).
- `cargo clippy --locked --all-targets --target-dir target/pr-review -- -D warnings`,
  `cargo fmt --all -- --check`, `git diff --check`를 통과했다.
- `node scripts/rust-test-suite-manifest.mjs --prepare` 및 `--check`:
  874 sources, 4113 static test attrs, 41/48 integration targets 확인.
- `node scripts/rust-unit-test-tiers.mjs --check`:
  4225 tests, 299 modules, ready 0 확인.

모든 Cargo 검증은 `--locked`로 실행했다. 초기 검토 중 `--locked` 없이 실행해 발생했던
`Cargo.lock`의 순서-only 변경은 즉시 되돌렸으며 현재 diff에 포함되지 않는다.

## GitHub CI 실측 결과

code candidate `4ba01d160555e65029b7ceaa36be1f65d0cd3833`의 첫 실행은 작성자의
`FIRST_TIME_CONTRIBUTOR` 보호 정책으로 `action_required`가 되었고, reviewer `@jangster77`이
현재 head에 대해 workflow를 승인했다. 동일 head의 두 번째 attempt에서 다음이 성공했다.

- Build & Test, archive A/B/C build 및 shard A1/A2/B/C, Lint
- CodeQL Rust, Python, JavaScript/TypeScript
- Adapter inter-diff와 Proptest roundtrip
- CI·CodeQL·Adapter·Proptest preflight

Native Skia, WASM Build, frontend package/unit gates는 변경 범위 정책에 따른 skip이며 실패 검사는 없다.
작성 시점 PR은 `MERGEABLE`, `CLEAN`이다. 이 trailing docs-only commit의 최신 head와 fast-pass는
merge 직전에 다시 확인한다.

## 최종 판정

**수용 권고, trailing CI 대기.** 원 PR head 자체의 ID 충돌은 maintainer 보정과 공개 회귀시험으로
해소됐고, 보정 code candidate의 로컬 검증과 GitHub Full CI·CodeQL이 모두 성공했다. 이 기록 commit이
허용된 review-only fast-pass를 통과하고 최신 head가 `CLEAN`이면 merge한다.
