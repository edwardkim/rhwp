# Stage 16 - Source `cfg(test)` 정책을 공개 계약 테스트로 이관

## 실패 원인

PR #5185의 lint 단계는 `rust-unit-test-tiers.mjs --check --base-ref`에서 source-side
테스트 정책을 위반했다. `explore`, `scaffold`, `schema`에 새 `#[cfg(test)]` 모듈이
추가됐고, `body_text` 내부 테스트 수 역시 PR 기준선보다 증가했다.

`tests/generated/**`와 `tests/suites/manifest.json`의 Git 추적 제외는 이 정책과 별개다.
해당 파일은 integration target 배치 산출물일 뿐 source-side `cfg(test)` 수를 바꾸지 않는다.

## 보정

- 탐색 메뉴와 scaffold의 공개 API 테스트를 `tests/cases/**` 계약 테스트로 옮긴다.
- HWP5 BodyText 벌크 처리의 공개 입력 경계(탭, UTF-16 surrogate, 문단 끝)를
  `tests/cases/body_text_bulkbuild_contract.rs`에서 검증한다.
- `body_text`의 기존 중첩 표 깊이 회귀 테스트는 PR 기준선 구현으로 복원한다.
- 생성 harness와 manifest는 직접 수정하지 않는다. `--prepare`가 CI와 로컬 검증 중에만
  파생한다.

## 기대 결과

새 기능의 회귀 검증은 source module을 늘리지 않고 `tests/cases`에서 자동 suite 배치를
받는다. 따라서 PR base 대비 source-side test 정책을 지키면서도 공개 계약 검증은 유지한다.

## 검증 예정

- `node scripts/rust-test-suite-manifest.mjs --prepare`
- `node scripts/rust-unit-test-tiers.mjs --check --base-ref upstream/devel`
- 이관된 contract suite의 targeted nextest
- `cargo fmt --all -- --check`

## 검증 결과

- `node scripts/rust-unit-test-tiers.mjs --generate && node scripts/rust-unit-test-tiers.mjs --check --base-ref upstream/devel`: `4225 tests / 298 modules` 통과.
- `node scripts/rust-test-suite-manifest.mjs --prepare && node scripts/rust-test-suite-manifest.mjs --check --base-ref upstream/devel`: 통과.
- `--prepare`가 당시 자동 배정한 세 regression suite에서 `scaffold_contract`·`explore_menu_contract`·`body_text_bulkbuild_contract`를 필터링해 실행: 13 passed.
- `cargo fmt --all -- --check`: 통과.
