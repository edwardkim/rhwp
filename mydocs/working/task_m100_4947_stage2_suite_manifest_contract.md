# task_m100_4947 stage 2: issue suite manifest 계약

## 배경

Stage 1은 20개 issue 회귀 테스트를 `issue_regression_pilot` 하나로 묶어 자동 생성되는
integration test 바이너리 수를 558개에서 539개로 줄였다. 그러나 harness를 사람이 직접
편집하면 case 누락, 중복 등록, 다시 늘어나는 top-level test target을 지속적으로 막을 수 없다.

## 목표

- suite와 case의 소유 관계를 하나의 manifest로 고정한다.
- manifest에서 Rust harness를 결정적으로 생성한다.
- 개별 case를 기존처럼 짧은 명령으로 실행할 수 있게 한다.
- CI가 누락, 중복, 생성물 drift와 integration target 예산 증가를 컴파일 전에 거부한다.

## 변경

- `tests/suites/manifest.json`
  - pilot의 20개 case와 현재 전체·standalone issue target 예산을 기록했다.
  - 예산은 상한선이므로 이후 suite 통합으로 target 수가 감소하는 것은 허용한다.
- `scripts/rust-test-suite-manifest.mjs`
  - `--generate`로 `tests/<suite>.rs`를 결정적으로 생성한다.
  - `--check`로 manifest, 실제 case 파일, 생성 harness, target 예산을 대조한다.
- `scripts/run-rust-test.mjs`
  - case 이름을 suite로 해석하고 `cargo nextest`의 test expression으로 한 case만 실행한다.
  - shell을 사용하지 않고 case 이름을 Rust module 식별자로 제한한다.
- `.github/workflows/ci.yml`
  - Rust 컴파일 전에 manifest 계약과 개별 실행 해석을 확인한다.

## 사용법

```bash
node scripts/rust-test-suite-manifest.mjs --generate
node scripts/rust-test-suite-manifest.mjs --check
node scripts/run-rust-test.mjs issue_1035_alignment -- --cargo-profile release-test
```

## 검증 계획

이 단계의 편집 직후에는 장시간 Rust 전수 회귀 테스트를 다시 실행하지 않는다. PR 준비 단계에서
Node 계약 테스트, manifest 검사, pilot 개별 case 실행을 먼저 수행하고 변경 범위에 맞는 Rust
검증 게이트를 적용한다. `CARGO_INCREMENTAL=0`은 사용하지 않는다.

## 다음 단계

Stage 3에서는 남은 top-level `issue_*.rs`를 균형 잡힌 복수 suite로 일괄 이전하고,
manifest 예산을 감소 방향으로 갱신한다.
